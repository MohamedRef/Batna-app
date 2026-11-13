#!/usr/bin/env bash
# Builds the Batna web app (docs/WEB.md) into target/web/batna: a static
# site (index.html, batna.js, batna_bg.wasm, content.pack, sw.js) any static
# host can serve.
#   scripts/web/build.sh            release build
#   scripts/web/build.sh --dev      faster, unoptimised
# Needs the wasm32 target (rust-toolchain.toml) and wasm-bindgen-cli at the
# same version as Cargo.lock's wasm-bindgen; wasm-opt (binaryen) is used when
# it is installed.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
profile=web-release
[[ "${1:-}" == "--dev" ]] && profile=dev
out="$root/target/web/batna"
want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/, ""); print; exit}' Cargo.lock)
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')
if [[ "$have" != "$want" ]]; then
  echo "wasm-bindgen-cli $want is needed (found '${have:-none}'): cargo install wasm-bindgen-cli@$want --locked" >&2
  exit 1
fi
# `strip` must stay off for wasm-bindgen (its custom sections), as for the Worker.
# build.rs writes the content pack into batna-app's OUT_DIR, which cargo
# names in its JSON messages; the app fetches it as its own file.
pack=$(CARGO_PROFILE_RELEASE_STRIP=false cargo build -p batna-app --bin batna --target wasm32-unknown-unknown \
  --no-default-features --features web --profile "$profile" --message-format=json-render-diagnostics |
  python3 -c '
import json, sys
for line in sys.stdin:
    m = json.loads(line)
    # Match the package, not the path: cargo 1.100 moved build output from
    # build/batna-app-<hash>/ to build/batna-app/<hash>/.
    if m.get("reason") == "build-script-executed" and "#batna-app@" in m.get("package_id", ""):
        print(m["out_dir"] + "/content.pack")
' | tail -1)
[[ -f "$pack" ]] || { echo "content.pack not found (cargo didn't report batna-app's build script)" >&2; exit 1; }
dir=$([[ $profile == dev ]] && echo debug || echo "$profile")
rm -rf "$out" && mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" --out-name batna \
  "target/wasm32-unknown-unknown/$dir/batna.wasm"
if [[ $profile != dev ]] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-reference-types \
    "$out/batna_bg.wasm" -o "$out/batna_bg.wasm"
fi
cp "$pack" "$out/content.pack"
cp web/index.html web/manifest.webmanifest web/_headers "$out/"
cp assets/export/desktop/256x256.png "$out/icon-256.png"
cp assets/export/desktop/512x512.png "$out/icon-512.png"
# The service worker keeps exactly this build's files: its name is their
# hash, so any change makes a new worker the browser installs in the background.
files=$(cd "$out" && find . -type f ! -name _headers ! -name index.html | sed 's|^\./||' | sort)
build=$(cd "$out" && sha256sum index.html $files | sha256sum | cut -c1-16)
list=$(printf '"./"'; for f in $files; do printf ', "./%s"' "$f"; done)
sed -e "s|__BUILD__|$build|" -e "s|\"__FILES__\"|$list|" web/sw.js > "$out/sw.js"
echo "Batna web: $out ($(du -sh "$out" | cut -f1))"
