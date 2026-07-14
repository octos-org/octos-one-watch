#!/usr/bin/env python3
"""Encode an octos LLM config (and optional server auth) into a QR code that the
app's composer can scan to provision itself — so a user brings their own key
without it ever touching the repo, a keyboard, or the network.

The QR encodes a single provisioning URL:

    octos://provision?llm_family=<id>&llm_model=<id>&llm_key=<key>
                       [&base_url=<url>&profile=<id>&token=<tok>]

The app parses it and writes `llm_family/llm_model/llm_key` into the octos
profile config (`_main.json` → config.llm + config.env_vars.<PROVIDER>_API_KEY);
the optional server fields go through the existing auth-provisioning path.

Usage:
    python3 scripts/llm_qr.py --family zai --model glm-5.2 --key sk-XXXX
    python3 scripts/llm_qr.py --family deepseek --model deepseek-v4-pro --key sk-XXXX \
        --out llm.png              # also write a PNG (needs `pip install qrcode[pil]`)
    python3 scripts/llm_qr.py --url 'octos://provision?...'   # encode a ready URL

By default it prints the URL and an ASCII QR to the terminal (scan it straight
off the screen). NOTE: the key is a secret — treat the QR/URL like a password.
"""
import argparse
import sys
from urllib.parse import quote

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


def build_url(a: argparse.Namespace) -> str:
    if a.url:
        return a.url
    if not (a.family and a.key):
        sys.exit("error: --family and --key are required (or pass --url)")
    parts = [f"llm_family={quote(a.family)}", f"llm_key={quote(a.key)}"]
    if a.model:
        parts.insert(1, f"llm_model={quote(a.model)}")
    for name, val in (("base_url", a.base_url), ("profile", a.profile), ("token", a.token)):
        if val:
            parts.append(f"{name}={quote(val)}")
    return "octos://provision?" + "&".join(parts)


def render_qr(url: str, out: str | None) -> None:
    try:
        import qrcode  # type: ignore
    except ImportError:
        sys.exit(
            "The `qrcode` package is required.  pip install 'qrcode[pil]'\n"
            "(The provisioning URL was still printed above — you can paste it into\n"
            "any QR generator.)"
        )
    qr = qrcode.QRCode(error_correction=qrcode.constants.ERROR_CORRECT_M, border=2)
    qr.add_data(url)
    qr.make(fit=True)
    qr.print_ascii(invert=True)  # scan straight off the terminal
    if out:
        qr.make_image(fill_color="black", back_color="white").save(out)
        print(f"\nwrote {out}")


def main() -> None:
    p = argparse.ArgumentParser(description="Encode an octos LLM config as a QR code.")
    p.add_argument("--family", help="provider family_id (zai, deepseek, openai, anthropic, …)")
    p.add_argument("--model", help="model_id (e.g. glm-5.2, deepseek-v4-pro)")
    p.add_argument("--key", help="the provider API key (stays on-device once scanned)")
    p.add_argument("--base-url", dest="base_url", help="optional octos server URL")
    p.add_argument("--profile", help="optional octos profile id")
    p.add_argument("--token", help="optional server bearer token")
    p.add_argument("--url", help="encode a ready-made octos://provision URL instead")
    p.add_argument("--out", help="also write a PNG to this path (needs qrcode[pil])")
    a = p.parse_args()

    url = build_url(a)
    if a.family and a.family not in KEY_ENV:
        print(f"note: unknown family '{a.family}' — the app maps it via octos's "
              f"provider registry (key env may not be auto-known).", file=sys.stderr)
    print("provisioning URL (treat as a secret):\n  " + url + "\n")
    render_qr(url, a.out)


if __name__ == "__main__":
    main()
