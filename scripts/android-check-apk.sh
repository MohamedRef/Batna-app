#!/usr/bin/env bash
# Fails loudly if an APK lacks Batna's Kotlin glue from android/, i.e. if dx
# did not pick up the `#[manganis::ffi]` module (crates/app/src/platform/background.rs).
# Adapted from kabar-app main: scripts/android-check-apk.sh.
#
# Usage: scripts/android-check-apk.sh path/to/batna.apk
# When a chunk adds a Kotlin class that Rust calls by name, add it to CLASSES.
set -euo pipefail
APK="${1:-}"
CLASSES=(app/batna/Haptics app/batna/OralService app/batna/OralBridge)

# Failures become annotations on GitHub, where job logs are hard to reach.
fail() {
  for line in "$@"; do
    if [ -n "${GITHUB_ACTIONS:-}" ]; then echo "::error::$line"; else echo "$line"; fi
  done
  exit 1
}

[ -f "$APK" ] || fail "No APK at '${APK}'"

# Read everything first: `producer | grep -q` under pipefail fails whenever
# grep stops early and the producer gets SIGPIPE.
dex="$(mktemp)"
trap 'rm -f "$dex"' EXIT
unzip -p "$APK" 'classes*.dex' >"$dex"
missing=()
for cls in "${CLASSES[@]}"; do
  LC_ALL=C grep -aqF "L$cls;" "$dex" || missing+=("$cls")
done
if [ "${#missing[@]}" -ne 0 ]; then
  fail "Missing from the APK: ${missing[*]}" "dx did not build android/ into the APK. See docs/RUN.md, 'Kotlin glue'."
fi
echo "Kotlin glue present: ${CLASSES[*]}"

# Permissions the glue declares in android/src/main/AndroidManifest.xml must
# survive the manifest merger. aapt2 ships with the Android build-tools; skip
# quietly where it isn't on PATH.
if command -v aapt2 >/dev/null; then
  manifest="$(aapt2 dump xmltree --file AndroidManifest.xml "$APK")"
  grep -q "android.permission.VIBRATE" <<<"$manifest" || fail "The merged manifest lost VIBRATE (android/src/main/AndroidManifest.xml)."
  echo "Merged manifest: glue permissions present"
fi

# The S24 Ultra and Tab S11 are arm64.
unzip -Z1 "$APK" | grep -q '^lib/arm64-v8a/' || fail "The APK has no arm64-v8a library."
echo "Native library: arm64-v8a present"
