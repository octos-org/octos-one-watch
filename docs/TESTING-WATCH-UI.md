# Testing the watch UI

The useful setup has three layers. The desktop preview is fast, an Android
Virtual Device (AVD) exercises the native composer and keyboard, and the OPPO
Watch 3 is the final acceptance target because its display, system bars, IME,
density, and ColorOS behavior are OEM-specific.

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
- an x86_64 Android system image (API 30 is closest to the verified watch, but
  API 34 is a useful fallback when the API 30 renderer exposes only GLES 2)

In **Device Manager -> Add device -> Create Virtual Device**, use either:

1. a Wear OS AVD for watch lifecycle, system bars, permissions, and IME checks;
2. a custom Phone/Tablet hardware profile using the OWW212's measured
   `372 x 430` display and `320 dpi` for the closest layout check. A separate
   `480 x 480` profile remains useful as a larger square-screen stress case.

The generic square AVD is usually the more useful visual target for this app;
the official Wear OS profiles may be round and do not emulate OPPO ColorOS.
Makepad requires an OpenGL ES 3 context. Before treating a black screen as an
app regression, verify the emulator renderer:

```bash
adb shell dumpsys SurfaceFlinger | grep '^GLES:'
```

If an API 30 AVD exposes only GLES 2 (`EGL_BAD_CONFIG` /
`CreateContextFailed` in logcat), select ANGLE or desktop OpenGL with the API
level set to the renderer maximum. If it still cannot expose GLES 3, use an
API 34 AVD with the same display and density; do not change app code to work
around an emulator-only renderer limitation.

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
adb install -r app/target/android/makepad-android-apk/octos_app/apk/octoswatch.apk
adb shell am start -S -n dev.makepad.octos_watch/.MakepadApp
```

## 4. OWW212 physical-device verification

The Watch Shell was verified on 2026-07-20 using the following physical device
and locked source set:

| Item | Verified value |
|---|---|
| Device | OPPO Watch 3 (`OWW212`) |
| Android | 11 / API 30 |
| ABI | `armeabi-v7a` |
| Physical display | `372 x 430`, 320 dpi |
| Watch branch | `codex/watch-shell-ui` at `218ceb4` |
| `aichat` / framework | `1afd9532fe511163846e9db14de1daa41c4be232` |
| `makepad` / build tool | `030fe180fdc636fc7a8cadec234275498675e2e4` plus the bundled Composer patch |
| `octos` kernel | `81ca39e900f49f777d54a9b109c406b8a3641431` |

The armv7 release APK was installed with `adb install -r`, preserving the
existing app data. On the real 372 x 430 display the app showed no sidebar,
divider, or desktop toolbar; the bottom row rendered as
`[menu] [usable text field] [send]`, with the controls fully visible. The
on-device visual pass confirmed the Watch Shell layout; the interaction and
keyboard checks remain explicit checklist items below so future changes do not
claim them from a static screenshot alone.

The existing `_main` provisioning also survived the upgrade. A cold-started
`Weather` prompt loaded `deepseek-chat`, created the AMA plus weather/stock/news
sessions, routed to the weather agent, completed an 827-character response,
and rendered the resulting weather card through Splash. No
`profile _main is not configured` error occurred. The host proxy used during
this check was mapped with `adb reverse tcp:8899 tcp:7897`; no credentials are
stored in this repository.

Several `transport: event channel full, dropping frame` warnings appeared
during generation. The authoritative response and card still completed, but
the warning should remain part of transport stress/regression testing.

## 5. UI regression checklist

- Startup contains no sidebar or 1px divider.
- Expanded bottom bar is `[menu] [usable text field] [send]`; the text field is
  visibly wider than zero before and after the keyboard opens.
- Menu exposes New conversation, Switch app, and Scan config QR.
- IME Send submits the same text as the send button.
- Keyboard does not cover the input bar.
- Collapsing the composer leaves a reachable 48dp `+` button.
- Long generated cards scroll without horizontal clipping.
- Rotate/rescale the AVD once and repeat the keyboard checks.

After every composer or shell-layout change, repeat the checklist on the
OWW212 and record a screenshot with the keyboard open. That is the acceptance
test for density and OEM insets; an emulator pass alone is not sufficient.
