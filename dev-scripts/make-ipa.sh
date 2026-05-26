#!/bin/sh
# Package an iOS .app bundle into an unsigned IPA (Payload/<name>.app + zip).
# No codesign step — suitable for jailbroken device sideloading or CI artifacts.
#
# Usage: make-ipa.sh <AppName> [path/to/parent/dir]
#   AppName: bundle name without the .app suffix (e.g. TestApp).
#   path/to/parent/dir: directory containing <AppName>.app (defaults to .).
set -xeu

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: $0 <AppName> [path/to/parent/dir]" >&2
    exit 1
fi

APP_NAME="$1"
PARENT_DIR="${2:-.}"

cd "$PARENT_DIR"
IPA_NAME="${APP_NAME}.ipa"

if [ ! -d "${APP_NAME}.app" ]; then
    echo "Error: ${APP_NAME}.app not found in $(pwd)" >&2
    exit 1
fi

rm -rf "$IPA_NAME" Payload/
mkdir Payload
# Copy the bundle; do not symlink. zip(1) records symlinks as tiny
# placeholder entries unless asked to dereference, which would produce an
# installable IPA only a few hundred bytes in size.
cp -R "${APP_NAME}.app" "Payload/"
zip -r "$IPA_NAME" Payload/
rm -rf Payload/

IPA_SIZE=$(wc -c < "$IPA_NAME" | tr -d ' ')
if [ "$IPA_SIZE" -lt 100000 ]; then
    echo "Error: ${IPA_NAME} is only ${IPA_SIZE} bytes; expected a full .app bundle inside Payload/." >&2
    exit 1
fi

echo "Created unsigned IPA: $(pwd)/$IPA_NAME (${IPA_SIZE} bytes)"
