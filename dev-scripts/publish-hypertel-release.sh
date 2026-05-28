#!/bin/sh
# Package Hypertel release zips from CI build artifacts and upload a GitHub release.
set -eu

VERSION="$1"
CHANGELOG_FROM="${2:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version e.g. v1.0.0> [changelog_from_ref]" >&2
    exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

for path in \
    artifacts/macos/touchHLE.dmg \
    artifacts/android/touchHLE.apk \
    artifacts/windows/touchHLE_windows_bundle/touchHLE.exe
do
    if [ ! -e "$path" ]; then
        echo "Missing build artifact (all platform builds must succeed): $path" >&2
        exit 1
    fi
done

if [ -z "$CHANGELOG_FROM" ]; then
    patch="${VERSION#v1.0.}"
    if [ "$patch" = "0" ]; then
        CHANGELOG_FROM="$(git rev-list -n 1 HEAD -- dev-scripts/hypertel-should-release.sh)"
    else
        CHANGELOG_FROM="v1.0.$((patch - 1))"
    fi
fi

notes_file="$(mktemp)"
trap 'rm -f "$notes_file"' EXIT INT TERM

{
    printf '%s\n\n' "Hypertel ${VERSION}"
    printf '%s\n\n' "## Changelog"
    if [ -n "$CHANGELOG_FROM" ]; then
        git log "${CHANGELOG_FROM}..HEAD" --reverse --format='%H%x1f%s%x1f%an' |
            while IFS="$(printf '\037')" read -r hash subject author; do
                [ -z "$hash" ] && continue
                short_hash="$(printf '%.7s' "$hash")"
                if [ -n "${GITHUB_SERVER_URL:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
                    printf -- '- [`%s`](%s/%s/commit/%s) %s (%s)\n' \
                        "$short_hash" "$GITHUB_SERVER_URL" "$GITHUB_REPOSITORY" "$hash" \
                        "$subject" "$author"
                else
                    printf -- '- `%s` %s (%s)\n' "$short_hash" "$subject" "$author"
                fi
            done
    else
        printf '%s\n' "_No commit range available._"
    fi
} >"$notes_file"

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
    --notes-file "$notes_file" \
    "$ROOT/${prefix}_macOS_x86_64.zip" \
    "$ROOT/${prefix}_Android_AArch64.zip" \
    "$ROOT/${prefix}_Windows_x86_64.zip"
