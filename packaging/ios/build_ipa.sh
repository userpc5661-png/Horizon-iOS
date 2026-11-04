#!/usr/bin/env bash
# Builds an unsigned Horizon.ipa (ad-hoc code signature only) on macOS.
# Sideloadly, AltStore or a signing service re-signs it for a device.
#
#   packaging/ios/build_ipa.sh <engine-data.zip> <output.ipa> [version] [build]
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
engine="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
output="$2"
version="${3:-0.7.7}"
build="${4:-1}"
target=aarch64-apple-ios
profile="${HORIZON_IOS_PROFILE:-release}"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-15.0}"

cd "$root"
cargo build -p horizon-ios --target "$target" --profile "$profile"
binary="$target_dir/$target/$profile/Horizon"
test -f "$binary"

stage="$(mktemp -d)"
app="$stage/Payload/Horizon.app"
mkdir -p "$app"
cp "$binary" "$app/Horizon"
strip -x "$app/Horizon" || true
sed -e "s/@VERSION@/$version/" -e "s/@BUILD@/$build/" packaging/ios/Info.plist > "$app/Info.plist"
plutil -lint "$app/Info.plist"
cp "$engine" "$app/engine-data.zip"
cp LICENSE "$app/LICENSE.txt"
cp THIRD_PARTY_NOTICES.md "$app/THIRD_PARTY_NOTICES.md"
if [ -f packaging/ios/AppIcon60x60@2x.png ]; then
  cp packaging/ios/AppIcon*.png "$app/"
fi
printf 'APPL????' > "$app/PkgInfo"
# An ad-hoc signature: TrollStore installs it as is; others re-sign it.
codesign --force --sign - --timestamp=none "$app"

mkdir -p "$(dirname "$output")"
output="$(cd "$(dirname "$output")" && pwd)/$(basename "$output")"
rm -f "$output"
(cd "$stage" && zip -qry "$output" Payload)
rm -rf "$stage"
echo "IPA: $output"
