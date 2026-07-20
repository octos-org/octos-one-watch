# Building for Android

How to build the `octoswatch.apk` from a fresh clone, deploy it, and run it on a
device. Do not assume the ABI: phones are commonly arm64, while some watches
still report 32-bit ARM.

```bash
adb devices -l
adb shell getprop ro.product.cpu.abi
```

Use the following mapping consistently for the Rust kernel and Makepad APK:

| Android reports | Rust target | `cargo makepad --abi` |
|---|---|---|
| `armeabi-v7a` | `armv7-linux-androideabi` | `armv7` |
| `arm64-v8a` | `aarch64-linux-android` | `aarch64` |
| `x86_64` | `x86_64-linux-android` | `x86_64` |

The OWW212 watch was verified with the first row (`armv7`).

## 0. Prerequisites

- Rust (stable) + the Android target selected above, for example
  `rustup target add armv7-linux-androideabi`
- `git`, `python3`, and `adb` (on WSL, use the Windows `adb.exe` to reach
  USB-connected devices — see the WSL note at the bottom)
- **The Android SDK/NDK for `cargo-makepad`.** Installed once with
  `cargo makepad android install-toolchain` (downloads the NDK under
  `makepad/tools/cargo_makepad/android_33_linux_x64/`). ⚠️ **This download hits
  `dl.google.com`, which is blocked on some networks (e.g. WSL behind a proxy).**
  If you already have a populated `android_33_linux_x64/`, reuse it — the build
  does not need to re-download.

## 1. Clone layout

This variant intentionally tracks three symlinks:

```text
aichat  -> ../octos-one/aichat
makepad -> ../octos-one/makepad
octos   -> ../octos-one/octos
```

Keep that layout and place the referenced dependency checkouts in a sibling
`octos-one/` directory:

```bash
git clone https://github.com/octos-org/octos-one-watch.git
mkdir -p octos-one

# framework fork (Splash engine + sys.* helpers)
git clone -b octos-one-framework https://github.com/octos-org/makepad.git octos-one/aichat

# build tool (cargo-makepad + native composer Java)
git clone -b octos-one-buildtool https://github.com/octos-org/makepad.git octos-one/makepad

# complete octos workspace; app/ path-depends on octos-core inside it
git clone https://github.com/octos-org/octos.git octos-one/octos

cd octos-one-watch
```

On Linux/WSL, `readlink aichat makepad octos` should print the three targets
above. On Windows, Git can check symlinks out as ordinary text files when
`core.symlinks=false` (the default in many installations). Check before building:

```powershell
git config --get core.symlinks
Get-Item aichat,makepad,octos | Select-Object Name,LinkType,Target
```

If `LinkType` is empty, use WSL, or enable Windows Developer Mode and symlink
support and then clone the watch repository again:

```powershell
git config --global core.symlinks true
git clone https://github.com/octos-org/octos-one-watch.git
```

Do not commit locally retargeted links. The repository's relative targets are
shared by Linux, WSL, and symlink-enabled Windows checkouts.

These repositories are branch dependencies, so their moving heads can add enum
variants that older clients do not compile against. The following revisions are
the reproducible set verified on the OWW212 (Android 11, API 30) on 2026-07-19:

| Dependency | Revision |
|---|---|
| `aichat` / `octos-one-framework` | `1afd9532fe511163846e9db14de1daa41c4be232` |
| `makepad` / `octos-one-buildtool` | `030fe180fdc636fc7a8cadec234275498675e2e4` |
| `octos` | `81ca39e900f49f777d54a9b109c406b8a3641431` |

```bash
git -C aichat checkout 1afd9532fe511163846e9db14de1daa41c4be232
git -C makepad checkout 030fe180fdc636fc7a8cadec234275498675e2e4
git -C octos checkout 81ca39e900f49f777d54a9b109c406b8a3641431
```

## 2. Install `cargo-makepad`

```bash
# The watch composer lives in cargo-makepad's Android Java activity. Apply the
# repository patch before installing the build tool; otherwise the APK retains
# the phone-width [new][switch][QR][input][send] row.
WATCH_ROOT="$PWD"
git -C makepad apply "$WATCH_ROOT/patches/0001-composer-mono-theme.patch"

# The build tool. The PGO profdata rustflag ships as a RELATIVE path and breaks
# from another CWD, so override it with an absolute one for the install:
RUSTFLAGS="-Cprofile-use=$PWD/aichat/libs/box3d/box3d.profdata" \
  cargo install --path makepad/tools/cargo_makepad --force
```

The manifest sets `makepad.COMPOSER_LAYOUT=watch`. The patched build tool reads
that key and emits `[menu][input][send]`; rebuilding the app with an older or
unpatched `cargo-makepad` silently restores the unusable phone composer.

Then, if you don't already have the NDK, `cargo makepad android install-toolchain`
(see the ⚠️ above).

## 3. Provide `liboctos.so` (the bundled kernel)

The APK bundles the octos kernel as `liboctos.so`. Build it for Android from
[`octos-org/octos`](https://github.com/octos-org/octos) with features
`api,git,ast`, using the Rust target selected above. For the OWW212:

```bash
# Makepad ships a compact NDK. Point Cargo and crates that compile C/C++
# directly at its LLVM tools; installing the Rust target alone is not enough.
export ANDROID_API=24
export NDK_TOOLCHAIN="$PWD/makepad/tools/cargo_makepad/android_33_linux_x64/ndk/28.2.13676358/toolchains/llvm/prebuilt/linux-x86_64"
export ANDROID_CC="$NDK_TOOLCHAIN/bin/armv7a-linux-androideabi${ANDROID_API}-clang"
export ANDROID_CXX="$NDK_TOOLCHAIN/bin/armv7a-linux-androideabi${ANDROID_API}-clang++"
export ANDROID_AR="$NDK_TOOLCHAIN/bin/llvm-ar"
export ANDROID_RANLIB="$NDK_TOOLCHAIN/bin/llvm-ranlib"

test -x "$ANDROID_CC" || { echo "Android clang not found: $ANDROID_CC" >&2; exit 1; }

export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$ANDROID_CC"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_AR="$ANDROID_AR"
export CC_armv7_linux_androideabi="$ANDROID_CC"
export CXX_armv7_linux_androideabi="$ANDROID_CXX"
export AR_armv7_linux_androideabi="$ANDROID_AR"
export RANLIB_armv7_linux_androideabi="$ANDROID_RANLIB"

cd octos
cargo build --target armv7-linux-androideabi --release \
  -p octos-cli --features "api,git,ast"
cd ..
```

This produces `octos/target/armv7-linux-androideabi/release/octos`. The compiler
wrapper uses Android API 24, below the verified watch's API 30. For another ABI,
change both the Rust target and the compiler/env-variable prefixes consistently.

## 4. Build the APK

```bash
cd app
export MAKEPAD_ANDROID_EXTRA_LIBS="liboctos.so=/ABS/PATH/TO/octos/target/armv7-linux-androideabi/release/octos"
cargo makepad android --abi=armv7 \
  --package-name=dev.makepad.octos_watch --app-label="Octos Watch" \
  build -p octos-app --release
```

- Look for `Bundled extra native lib: liboctos.so` and `APK Build completed`.
- Output: `app/target/android/makepad-android-apk/octos_app/apk/octoswatch.apk`
  (~74 MB). Verify: `unzip -l …/octoswatch.apk | grep -E 'liboctos|libmakepad'`.
- Fast type-check without building the whole APK:
  `RUSTFLAGS="-Cprofile-use=$PWD/../aichat/libs/box3d/box3d.profdata" \
   cargo check --target armv7-linux-androideabi -p makepad-widgets`
  (a host `cargo check` fails on the Linux desktop backend's `wayland-client`; the
  Android target skips it).

## 5. Install + run

```bash
cd ..   # repository root
ADB=adb   # or /mnt/c/.../adb.exe on WSL
$ADB install -r app/target/android/makepad-android-apk/octos_app/apk/octoswatch.apk

# launch. Extras (all optional):
#   makepad.OCTOS_PROXY  http proxy for the LLM + data fetches (phones w/o direct net)
#   makepad.APP_CONFIG   'base_url|profile|token'  (headless auth provisioning)
#   makepad.AUTO_PROMPT  auto-submit one prompt on boot (for testing)
$ADB shell am start -S -n dev.makepad.octos_watch/.MakepadApp \
    --es makepad.OCTOS_PROXY 'http://127.0.0.1:8899' \
    --es makepad.AUTO_PROMPT 'TSLA'
```

Package `dev.makepad.octos_watch`, launch activity `.MakepadApp`. On install Android
extracts `liboctos.so` into the app's `nativeLibraryDir`; the app execs it as
`liboctos.so serve --stdio`.

## Deploy the app-card memory

The app agents only generate good cards if the `a2app/` tree is in the app's octos
profile. **octos assembles it itself** — the kernel looks for an `app-cards/` tree
under the profile's memory dir and, when present, concatenates it (framework →
widget helpers → each app's `app.md` + exemplars) into the injected long-term
memory at inject time. No build step, no generated `MEMORY.md` artifact: the
`a2app/` tree IS the source of truth on disk (see
`octos/crates/octos-memory/src/memory_store.rs` → `assemble_app_cards`).

```bash
# Build the tree on the host. PROVISION_DIR copies it into the app-private
# octos-home before the embedded kernel starts; root/su is not required.
PROVISION=$(mktemp -d)
mkdir -p "$PROVISION/.octos/profiles/_main/data/memory"
cp -R a2app "$PROVISION/.octos/profiles/_main/data/memory/app-cards"

REMOTE=/data/local/tmp/octos-watch-provision
$ADB shell rm -rf "$REMOTE"
$ADB push "$PROVISION/." "$REMOTE"
$ADB shell chmod -R a+rX "$REMOTE"

# If you also put a complete _main.json in the tree, pass both extras in this
# SAME cold start so provisioning finishes before session/open.
$ADB shell am start -W -S -n dev.makepad.octos_watch/.MakepadApp \
  --es makepad.PROVISION_DIR "$REMOTE" \
  --es makepad.APP_CONFIG 'http://127.0.0.1:50080|_main|local-stdio-bypass'

# The staging directory may contain an API key. Remove it after a successful
# cold start; the copied app-private data persists.
$ADB shell rm -rf "$REMOTE"
```

Place a complete profile at
`$PROVISION/.octos/profiles/_main.json` before `adb push` when provisioning an
LLM non-interactively. Keep real keys out of shell history and this repository.
On Android 11, `/data/local/tmp` plus `chmod -R a+rX` is more reliable for this
one-time import than `/sdcard/Android/data/...`, which may be blocked by scoped
storage.

**The injection token budget is handled by the app.** octos's built-in
`memory.max_inject_tokens` default is 2500 — far below the ~23k-token 3-app
tree — and an over-budget tree is truncated **silently** at inject time (the
agent then never sees the exemplars and cards come out with empty values). The
budget knob lives in the KERNEL config, `$APPHOME/.config/octos/config.json`
(NOT in `_main.json` — the current octos profile schema has no
`max_inject_tokens` key, so the old sed recipe silently no-ops). On every boot
the app writes `"memory": {"max_inject_tokens": 40000}` into that file when
the key is absent (`ensure_kernel_memory_budget` in `app/src/main.rs`); an
explicit value you set there yourself is respected.

⚠️ If you tune the value manually, keep it above the assembled-tree token
estimate (`wc -c` over `a2app/` ÷ 4, ~23k tokens for the 3-app tree), or octos
truncates the tail (the last app) on injection. Adding an app is a drop-in:
create `a2app/apps/<id>/app.md` (+ exemplars) and re-push — no code edit.

The per-profile LLM provider + key live in `_main.json` (`config.llm` +
`config.env_vars`) — provision that on the device; **never commit keys**.

## WSL / networking notes

- Use the Windows `adb.exe` (e.g. `/mnt/c/Users/<you>/…/platform-tools/adb.exe`)
  to talk to USB phones; the SDK's Linux `adb` can't see them.
- Phones without a direct internet route reach out through a host proxy: run an
  HTTP CONNECT proxy on the host, map it with
  `adb reverse tcp:8899 tcp:<host-proxy-port>`, and pass
  `--es makepad.OCTOS_PROXY 'http://127.0.0.1:8899'`. This routes both the LLM
  (`api.z.ai` / provider) and the card data fetches (open-meteo, Yahoo, HN).
