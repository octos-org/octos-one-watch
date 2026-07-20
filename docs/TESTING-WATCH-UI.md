# Testing the watch UI without hardware

The useful setup has two layers. The desktop preview is fast, while an Android
Virtual Device (AVD) is the closest check for the native composer and keyboard.
Neither replaces the final pass on the OPPO Watch 3 because its square display,
system bars, IME, density, and ColorOS behavior are OEM-specific.

## 1. Fast desktop shell preview

Run the app normally on the desktop. This variant starts at `360 x 480` and
uses the same compact-shell threshold as Android: sidebar, divider, and desktop
glass controls are removed, leaving the card surface and bottom composer.

This validates Makepad layout and generated-card overflow. It does **not** show
the Android-native `EditText` overlay.

## 2. Android emulator

Install Android Studio, then install these SDK components from SDK Manager:

- Android Emulator
- Android SDK Platform-Tools
- an x86_64 Android system image (API 30 is closest to the verified watch)

In **Device Manager -> Add device -> Create Virtual Device**, use either:

1. a Wear OS AVD for watch lifecycle, system bars, permissions, and IME checks;
2. a custom Phone/Tablet hardware profile with a square `480 x 480` display and
   about `240 dpi` (roughly `320 x 320 dp`) for the OPPO-like layout check.

The generic square AVD is usually the more useful visual target for this app;
the official Wear OS profiles may be round and do not emulate OPPO ColorOS.

## 3. Build the emulator ABI

The emulator cannot run the OWW212 `armeabi-v7a` APK's native Rust libraries.
Build both the octos kernel and the APK for `x86_64`:

```bash
rustup target add x86_64-linux-android

# Build octos with the x86_64 Android NDK clang/linker environment, following
# docs/BUILDING-ANDROID.md section 3 and changing every target/prefix together.
cargo build --target x86_64-linux-android --release \
  -p octos-cli --features "api,git,ast"

cd app
export MAKEPAD_ANDROID_EXTRA_LIBS="liboctos.so=/ABS/PATH/TO/octos/target/x86_64-linux-android/release/octos"
cargo makepad android --abi=x86_64 \
  --package-name=dev.makepad.octos_watch --app-label="Octos Watch" \
  build -p octos-app --release
```

Install and launch:

```bash
adb install -r app/target/android/makepad-android-apk/octos_app/apk/octos_app.apk
adb shell am start -S -n dev.makepad.octos_watch/.MakepadApp
```

## 4. UI regression checklist

- Startup contains no sidebar or 1px divider.
- Expanded bottom bar is `[menu] [usable text field] [send]`; the text field is
  visibly wider than zero before and after the keyboard opens.
- Menu exposes New conversation, Switch app, and Scan config QR.
- IME Send submits the same text as the send button.
- Keyboard does not cover the input bar.
- Collapsing the composer leaves a reachable 48dp `+` button.
- Long generated cards scroll without horizontal clipping.
- Rotate/rescale the AVD once and repeat the keyboard checks.

When the watch is available, repeat the checklist on the OWW212 and record a
screenshot with the keyboard open; that is the acceptance test for density and
OEM insets.
