#!/usr/bin/env bash
# Builds the Batna web app (docs/WEB.md) into target/web/batna: a static
# site (index.html, batna.js, batna_bg.wasm) any static host can serve.
#   scripts/web/build.sh            release build
#   scripts/web/build.sh --dev      faster, unoptimised
# Needs the wasm32 target (rust-toolchain.toml) and wasm-bindgen-cli at the
# same version as Cargo.lock's wasm-bindgen; wasm-opt (binaryen) is used when
# it is installed.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
profile=release
[[ "${1:-}" == "--dev" ]] && profile=dev
out="$root/target/web/batna"
want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/, ""); print; exit}' Cargo.lock)
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')
if [[ "$have" != "$want" ]]; then
  echo "wasm-bindgen-cli $want is needed (found '${have:-none}'): cargo install wasm-bindgen-cli@$want --locked" >&2
  exit 1
fi
# `strip` must stay off for wasm-bindgen (its custom sections), as for the Worker.
CARGO_PROFILE_RELEASE_STRIP=false cargo build -p batna-app --bin batna --target wasm32-unknown-unknown \
  --no-default-features --features web --profile "$profile"
dir=$([[ $profile == dev ]] && echo debug || echo release)
rm -rf "$out" && mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" --out-name batna \
  "target/wasm32-unknown-unknown/$dir/batna.wasm"
if [[ $profile == release ]] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-reference-types \
    "$out/batna_bg.wasm" -o "$out/batna_bg.wasm"
fi
cp web/index.html web/manifest.webmanifest web/_headers "$out/"
cp assets/export/desktop/256x256.png "$out/icon-256.png"
cp assets/export/desktop/512x512.png "$out/icon-512.png"
echo "Batna web: $out ($(du -sh "$out" | cut -f1))"
