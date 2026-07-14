#!/usr/bin/env python3
"""Encode an octos LLM config (and optional server auth) into a QR code that the
app's composer scans to provision itself — so a user brings their own key
without it ever touching the repo, a keyboard, or the network.

The QR payload is a single compact JSON object carrying ALL the info (no URL):

    {"llm_family":"zai","llm_model":"glm-5.2","llm_key":"sk-XXXX"
     [,"base_url":"...","profile":"...","token":"..."]}

The app parses it and writes `llm_family`/`llm_model` into the octos profile
config (`_main.json` → config.llm) and `llm_key` into
config.env_vars.<PROVIDER>_API_KEY; the optional server fields go through the
existing auth-provisioning path.

Usage:
    python3 scripts/llm_qr.py --family zai --model glm-5.2 --key sk-XXXX
    python3 scripts/llm_qr.py --family deepseek --model deepseek-v4-pro --key sk-XXXX \
        --out llm.png              # also write a PNG (needs `pip install qrcode[pil]`)
    python3 scripts/llm_qr.py --json '{"llm_family":"zai",...}'   # encode ready JSON

By default it prints the JSON payload and an ASCII QR to the terminal (scan it
straight off the screen). NOTE: the key is a secret — treat the QR like a password.
"""
import argparse
import json
import sys

# Provider family_id -> the env-var name octos reads the key from. Extend as the
# octos provider registry grows (crates/octos-llm/src/registry/*).
KEY_ENV = {
    "zai": "ZAI_API_KEY",
    "deepseek": "DEEPSEEK_API_KEY",
    "openai": "OPENAI_API_KEY",
    "anthropic": "ANTHROPIC_API_KEY",
    "gemini": "GEMINI_API_KEY",
    "openrouter": "OPENROUTER_API_KEY",
}


def build_payload(a: argparse.Namespace) -> str:
    if a.json:
        json.loads(a.json)  # validate
        return a.json
    if not (a.family and a.key):
        sys.exit("error: --family and --key are required (or pass --json)")
    d: dict[str, str] = {"llm_family": a.family, "llm_key": a.key}
    if a.model:
        d["llm_model"] = a.model
    for k, v in (("base_url", a.base_url), ("profile", a.profile), ("token", a.token)):
        if v:
            d[k] = v
    # compact — QR capacity is limited, so no spaces
    return json.dumps(d, separators=(",", ":"), ensure_ascii=False)


def render_qr(payload: str, out: str | None) -> None:
    try:
        import qrcode  # type: ignore
    except ImportError:
        sys.exit(
            "The `qrcode` package is required for the image/ASCII QR.\n"
            "  pip install 'qrcode[pil]'\n"
            "(The JSON payload was printed above — paste it into any QR generator.)"
        )
    qr = qrcode.QRCode(error_correction=qrcode.constants.ERROR_CORRECT_M, border=2)
    qr.add_data(payload)
    qr.make(fit=True)
    qr.print_ascii(invert=True)  # scan straight off the terminal
    if out:
        qr.make_image(fill_color="black", back_color="white").save(out)
        print(f"\nwrote {out}")


def main() -> None:
    p = argparse.ArgumentParser(description="Encode an octos LLM config as a QR code (JSON payload).")
    p.add_argument("--family", help="provider family_id (zai, deepseek, openai, anthropic, …)")
    p.add_argument("--model", help="model_id (e.g. glm-5.2, deepseek-v4-pro)")
    p.add_argument("--key", help="the provider API key (stays on-device once scanned)")
    p.add_argument("--base-url", dest="base_url", help="optional octos server URL")
    p.add_argument("--profile", help="optional octos profile id")
    p.add_argument("--token", help="optional server bearer token")
    p.add_argument("--json", help="encode a ready-made JSON payload instead")
    p.add_argument("--out", help="also write a PNG to this path (needs qrcode[pil])")
    a = p.parse_args()

    payload = build_payload(a)
    if a.family and a.family not in KEY_ENV:
        print(f"note: unknown family '{a.family}' — the app maps it via octos's "
              f"provider registry (key env may fall back to {a.family.upper()}_API_KEY).",
              file=sys.stderr)
    print("QR payload (treat as a secret):\n  " + payload + "\n")
    render_qr(payload, a.out)


if __name__ == "__main__":
    main()
