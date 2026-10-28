# Batna task runner. Install `just` first: `brew install just`.

set shell := ["bash", "-euo", "pipefail", "-c"]
set dotenv-load := true

server := "crates/server"

default:
    @just --list

# fmt + clippy (zero warnings) + tests + content checks: run before every commit
check:
    scripts/check-tracked-files.sh
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -q -p batna-tools -- validate
    cargo run -q -p batna-tools -- packs build --check

fmt:
    cargo fmt --all

# Check the content files (SPLIT.md §4–5)
validate:
    cargo run -q -p batna-tools -- validate

# One Anki file with every written lecture: dist/anki/batna.apkg (docs/ANKI.md)
anki:
    cargo run -q -p batna-tools -- anki

# Run the desktop app (BATNA_WINDOW_SIZE=412x900 previews the phone layout)
run:
    cargo run -p batna-app

# Android debug APK on a connected device (needs dx 0.7.10, Android SDK + NDK)
android:
    cd crates/app && dx serve --platform android --no-default-features --features mobile

# --- Batna Cloud (docs/CLOUD.md) ----------------------------------------------

# Install the Worker tools once: wrangler 4 + worker-build
cloud-tools:
    npm install -g wrangler@4
    cargo install worker-build --version ^0.8 --locked

# One-time, after `wrangler login`: token signing key and your Gmail on the allowlist
cloud-setup:
    scripts/cloud-setup.sh

# Run Batna Cloud locally on http://127.0.0.1:8787 (local workerd)
cloud-dev:
    cd {{server}} && wrangler dev

# Deploy to batna-cloud.<your-subdomain>.workers.dev
cloud-deploy:
    cd {{server}} && wrangler deploy

# Stream live logs from the deployed Worker
cloud-logs:
    cd {{server}} && wrangler tail
