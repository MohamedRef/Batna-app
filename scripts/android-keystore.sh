#!/usr/bin/env bash
# Creates the release signing key outside the repo (adapted from kabar-app
# main: scripts/android-keystore.sh). The password lives in the
# macOS Keychain (service "batna-android"), never in a file.
set -euo pipefail
KEYSTORE="$1"
ALIAS="$2"

if [ -f "$KEYSTORE" ]; then
  echo "Keystore already exists: $KEYSTORE (back it up; losing it means uninstalling Batna to update)"
  exit 0
fi

mkdir -p "$(dirname "$KEYSTORE")"
PASS="$(openssl rand -base64 24)"
security add-generic-password -U -s batna-android -a "$USER" -w "$PASS"
keytool -genkeypair -v -keystore "$KEYSTORE" -alias "$ALIAS" -keyalg RSA -keysize 4096 \
  -validity 10000 -storepass "$PASS" -keypass "$PASS" -dname "CN=Batna, O=Personal"
chmod 600 "$KEYSTORE"
echo "Created $KEYSTORE. Back it up somewhere safe (not in the repo)."
