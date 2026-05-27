#!/bin/sh
set -xeu
cd "$(dirname "$0")"
../dev-scripts/make-ipa.sh TestApp .
