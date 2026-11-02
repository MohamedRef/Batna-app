#!/usr/bin/env bash
# Once, on the Mac, before the first release from CI (docs/RELEASE.md §4):
# makes Batna's release keys where the other scripts keep them (or reuses
# them), then copies them into this repository's Actions secrets so
# .github/workflows/release.yml can build, sign and publish. Run it again
# any time: it reuses every key that exists and only refreshes the secrets.
#
# | Secret                          | From                                            |
# |---------------------------------|-------------------------------------------------|
# | BATNA_UPDATE_SECRET             | Keychain batna-update (just update-keygen)      |
# | BATNA_ANDROID_KEYSTORE          | ~/.batna/batna-release.jks, base64 (just android-keystore) |
# | BATNA_ANDROID_KEYSTORE_PASSWORD | Keychain batna-android                          |
# | BATNA_MAC_CERT_P12              | ~/.batna/batna-codesign.p12, base64 (just mac-codesign-cert) |
# | BATNA_MAC_CERT_PASSWORD         | Keychain batna-codesign                         |
#
# The values go from the Keychain straight into `gh secret set` on stdin:
# never into a file in the repo, an argument list or the terminal.
# Needs gh (logged in with access to the repo), a JDK (keytool) and cargo.
set -euo pipefail
cd "$(dirname "$0")/.."
repo="${BATNA_REPO:-MohamedRef/Batna-app}"

command -v gh >/dev/null || { echo "install the GitHub CLI: brew install gh && gh auth login" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "log in first: gh auth login" >&2; exit 1; }

if ! security find-generic-password -s batna-update >/dev/null 2>&1; then
  scripts/update-keygen.sh
fi
grep -q '^[A-Za-z0-9+/]\{43\}=$' crates/app/update-key.pub \
  || { echo "crates/app/update-key.pub holds no key, but the Keychain already has batna-update: put its public half back first" >&2; exit 1; }
scripts/android-keystore.sh "$HOME/.batna/batna-release.jks" batna
if [ ! -f "$HOME/.batna/batna-codesign.p12" ]; then
  if security find-identity -v -p codesigning | grep -q '"Batna Code Signing"'; then
    echo "\"Batna Code Signing\" is in the Keychain but ~/.batna/batna-codesign.p12 is missing: export it there (Keychain Access › Export) and store its password as batna-codesign" >&2
    exit 1
  fi
  scripts/mac-codesign-cert.sh
fi

put() { gh secret set "$1" --repo "$repo" >/dev/null && echo "set $1"; }
printf %s "$(security find-generic-password -s batna-update -w)" | put BATNA_UPDATE_SECRET
base64 < "$HOME/.batna/batna-release.jks" | tr -d '\n' | put BATNA_ANDROID_KEYSTORE
printf %s "$(security find-generic-password -s batna-android -w)" | put BATNA_ANDROID_KEYSTORE_PASSWORD
base64 < "$HOME/.batna/batna-codesign.p12" | tr -d '\n' | put BATNA_MAC_CERT_P12
printf %s "$(security find-generic-password -s batna-codesign -w)" | put BATNA_MAC_CERT_PASSWORD
gh secret list --repo "$repo"

echo
echo "Done. Commit crates/app/update-key.pub if it changed:"
grep -v '^#' crates/app/update-key.pub
echo "Back up ~/.batna/ and the Keychain items batna-update, batna-android, batna-codesign together."
