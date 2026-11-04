# Horizon on iOS (experimental)

The iOS app is the Android port's launcher and touch client running on UIKit
through winit, rendering with Metal through wgpu. It is built by GitHub Actions
on macOS and shipped as an **unsigned IPA** (ad-hoc signature only).

## Build

Push to GitHub or run **Actions > iOS unsigned IPA > Run workflow**. The job:

1. downloads the pinned Minecraft 26.3 files from Mojang and prepares the engine
   tables (`tools/android_prepare.py`, cached afterwards);
2. builds `crates/horizon-ios` for `aarch64-apple-ios`;
3. packages `Horizon.app` with `packaging/ios/Info.plist` and the engine data
   (`packaging/ios/build_ipa.sh`) and uploads `Horizon-iOS-unsigned-N.ipa`.

Locally on a Mac with Xcode:

```sh
rustup target add aarch64-apple-ios
python3 tools/android_prepare.py --java-home "$JAVA_HOME"
python3 -c "import sys; sys.path.insert(0, 'tools'); import android_build; android_build.engine_data(True)"
HORIZON_ANDROID_PUBLIC=1 packaging/ios/build_ipa.sh .local/android/engine-data-generated.zip dist/Horizon.ipa
```

## Install

An unsigned IPA must be signed for your device:

- **Sideloadly** or **AltStore/SideStore** (Windows or macOS): sign with a free
  Apple ID. Free signatures expire after 7 days; reinstall to renew.
- **TrollStore** (supported iOS versions only): installs the IPA as is.

On first launch choose **Download Minecraft 26.3** (about 500 MiB from Mojang).

## Files

Worlds, packs, screenshots and logs are in the app's Documents folder, shown in
the Files app under **On My iPhone > Horizon**. Account tokens are kept in the
app's private Library folder.

## Status

Works the same as Android: launcher, touch controls, single player, LAN and Java
multiplayer, sound (Core Audio), clipboard, the system keyboard, haptics and
Microsoft sign-in through Safari (device code copied to the clipboard).

Not yet on iOS: the document picker (import through the Files app instead),
game controllers (GameController.framework), gyro aiming (CoreMotion), render
scale below 100 %, Keychain storage and safe-area insets (use the HUD margins
option to keep controls clear of the notch).

Platform code: `crates/horizon-client/src/ios_app.rs` and `src/ios/`; the
winit stand-ins for the Android viewport helpers are in
`third_party/winit/src/platform/android_ios_shim.rs`.
