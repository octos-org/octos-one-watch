# Building this variant

This repo builds exactly like upstream octos-one — clone layout, cargo-makepad
toolchain, `liboctos.so`, and APK build are unchanged; follow
[docs/BUILDING-ANDROID.md](docs/BUILDING-ANDROID.md). The differences are:

## 1. Package name / label

```bash
cd app
RUSTFLAGS="-Cprofile-use=$PWD/../aichat/libs/box3d/box3d.profdata" \
MAKEPAD_ANDROID_EXTRA_LIBS="liboctos.so=/ABS/octos/target/armv7-linux-androideabi/release/octos" \
cargo makepad android --abi=armv7 \
  --package-name=dev.makepad.PACKAGE --app-label="LABEL" \
  build -p octos-app --release
```

Substitute this variant's PACKAGE/LABEL from the README and select the ABI from
`adb shell getprop ro.product.cpu.abi`; use the mapping table in the Android
build guide. The Rust client hardcodes its octos-home under
`/data/user/0/<package>/…`, so the `--package-name` MUST match the package id
this repo's `app/src/main.rs` was written for — don't mix flags.

## 2. makepad build-tool patch (native composer theme)

The floating composer on Android is a **native** view drawn by
cargo-makepad's Java (`MakepadActivity`), not by Makepad's GL canvas. To let
variants theme it, apply the bundled patch to the `octos-one-buildtool` clone
after step 1 of the upstream clone layout:

```bash
WATCH_ROOT=$(git rev-parse --show-toplevel)
PATCH="$WATCH_ROOT/patches/0001-composer-mono-theme.patch"
git -C "$WATCH_ROOT/makepad" apply "$PATCH"
RUSTFLAGS="-Cprofile-use=$WATCH_ROOT/aichat/libs/box3d/box3d.profdata" \
  cargo install --path "$WATCH_ROOT/makepad/tools/cargo_makepad" --force
```

The patch is additive and backward-compatible: it reads
`<meta-data android:name="makepad.COMPOSER_THEME">` from the app manifest and
falls back to the stock teal theme when absent. The watch variant keeps the
stock (dark, OLED-friendly) theme, so its manifest does NOT set the meta-data —
applying the patch is optional here and only matters if you also build the
e-ink variant from the same toolchain.

## 3. Launcher + system component

Both are wired through `app/app/resources/android/AndroidManifest.xml.template`
(HOME + DEFAULT categories, `android:persistent`) — see
[docs/SYSTEM-APP.md](docs/SYSTEM-APP.md) for the verified launcher-role and
`/system/priv-app` install recipe (incl. the staged-kernel layout this app's
`stdio_spawn()` looks for).
