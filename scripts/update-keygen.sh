#!/usr/bin/env bash
# Creates Batna's update signing key (Ed25519) once, on the Mac. The public
# half goes into crates/app/update-key.pub (commit it: builds trust it); the
# secret half goes straight into the macOS Keychain (service "batna-update")
# and never touches a file. docs/RELEASE.md §5.
# Adapted from kabar-app main: scripts/update-keygen.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

if security find-generic-password -s batna-update >/dev/null 2>&1; then
  echo "The Keychain already holds a batna-update key. Keep using it: installed builds trust it." >&2
  exit 0
fi
secret="$(cargo run -q --release -p batna-tools -- update keygen)"
# Through security's own command prompt (stdin), so the secret never shows
# up in another process's argument list.
printf 'add-generic-password -U -s batna-update -a %s -w %s\n' "$USER" "$secret" | security -i >/dev/null
unset secret
security find-generic-password -s batna-update >/dev/null || { echo "storing the key in the Keychain failed" >&2; exit 1; }
echo "Stored the secret in the Keychain (batna-update). Commit crates/app/update-key.pub."
echo "Back the secret up with the Android key: security find-generic-password -s batna-update -w"
