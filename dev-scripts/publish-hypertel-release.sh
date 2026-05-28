#!/bin/sh
# Package Hypertel release zips from CI build artifacts and upload a GitHub release.
set -eu

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version e.g. v1.0.0>" >&2
    exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/dev-scripts"

./prepare-release.sh --prepare-files

prefix="Hypertel_${VERSION}"

./prepare-release.sh --create-zip-macos "$ROOT/artifacts/macos/touchHLE.dmg" \
    -o "$ROOT/${prefix}_macOS_x86_64.zip"
./prepare-release.sh --create-zip-android "$ROOT/artifacts/android/touchHLE.apk" \
    -o "$ROOT/${prefix}_Android_AArch64.zip"
./prepare-release.sh --create-zip-windows \
    "$ROOT/artifacts/windows/touchHLE_windows_bundle/touchHLE.exe" \
    -o "$ROOT/${prefix}_Windows_x86_64.zip"

gh release create "$VERSION" \
    --repo "$GITHUB_REPOSITORY" \
    --title "Hypertel" \
    --notes "Hypertel ${VERSION} (automated release every 5 commits)." \
    "${prefix}_macOS_x86_64.zip" \
    "${prefix}_Android_AArch64.zip" \
    "${prefix}_Windows_x86_64.zip"
