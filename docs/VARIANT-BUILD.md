# Building this variant

This repo builds exactly like upstream octos-one — clone layout, cargo-makepad
toolchain, `liboctos.so`, and APK build are unchanged; follow
[docs/BUILDING-ANDROID.md](docs/BUILDING-ANDROID.md). The differences are:

## 1. Package name / label

```bash
cd app
RUSTFLAGS="-Cprofile-use=$PWD/../aichat/libs/box3d/box3d.profdata" \
MAKEPAD_ANDROID_EXTRA_LIBS="liboctos.so=/ABS/octos/target/x86_64-linux-android/release/octos" \
cargo makepad android --abi=x86_64 \
  --package-name=dev.makepad.PACKAGE --app-label="LABEL" \
  build -p octos-app --release
```

(substitute this variant's PACKAGE/LABEL from the README; for a phone use
`--abi=aarch64`). The Rust client hardcodes its octos-home under
`/data/user/0/<package>/…`, so the `--package-name` MUST match the package id
this repo's `app/src/main.rs` was written for — don't mix flags.

## 2. makepad build-tool patch (native composer theme)

The floating composer on Android is a **native** view drawn by
cargo-makepad's Java (`MakepadActivity`), not by Makepad's GL canvas. To let
variants theme it, apply the bundled patch to the `octos-one-buildtool` clone
after step 1 of the upstream clone layout:

```bash
cd octos-one/makepad            # the octos-one-buildtool clone
git apply ../patches/0001-composer-mono-theme.patch
cargo install --path tools/cargo_makepad --force   # reinstall the tool
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
