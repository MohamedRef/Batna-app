# Batna on the web

Batna's web app is the same Rust code as the Mac and Android apps, compiled to
WebAssembly. It follows the shape of the ArtCraft "Crafting Apps" (PdfCraft,
WordCraft…): one Rust codebase for every platform, a thin web shell, a static
site any host can serve, and CI that keeps the wasm build compiling. Batna keeps
its Dioxus screens (HTML/CSS, right-to-left) instead of their egui toolkit
(Mohamed, 2026-11-02).

## Shape

| Piece | Where | What it does |
|---|---|---|
| The app | `crates/app` with `--no-default-features --features web` | Every screen, the content pack and the fonts, built in |
| Browser stand-ins | `crates/app/src/platform/web.rs`, `platform/time.rs` | Storage, downloads, prefs, timers |
| Web shell | `web/index.html`, `web/manifest.webmanifest`, `web/_headers` | Loads `batna.js`, installable as an app |
| Build | `scripts/web/build.sh` (`just web`, `just web-release`) | cargo → wasm-bindgen → `target/web/batna` |
| Host | `web/wrangler.toml`, `.github/workflows/web-deploy.yml` | A static-assets Worker on the free plan, same account as Batna Cloud |
| CI gate | `ci.yml` › "Web app compiles (wasm32)" | clippy for wasm32 on every PR |

## What differs in the browser

| Area | Mac / Android | Web |
|---|---|---|
| Database | SQLite file | The same SQLite, compiled to wasm, in memory; its bytes are saved in IndexedDB 1.5 s after a change and when the tab is hidden or closed |
| Two tabs | n/a | The newest tab owns the data; an older one stops saving and says so on Today |
| Device prefs | `device-prefs.txt` | `localStorage` |
| Exports (backup, checks, Anki) | A file in Downloads / Save to | A browser download |
| Restore | File picker / found files | File picker or paste |
| Read aloud | System voice | The browser's voice (Web Speech) |
| Updates | Over the air | Each deploy is the newest build; reload to get it |
| Sync (Batna Cloud) | Live | Not yet: Settings › المزامنة says so. Next phase below |
| Reminders | System notifications | Not on the web; the phone or Mac rings |

Nothing is dropped: everything else on the Mac and Android also runs in the browser.

## Phase 2: sync from the browser

1. Move `cloud/api.rs` and `cloud/engine.rs` from blocking `ureq` and
   `tokio-tungstenite` to a small transport trait with a `fetch` / `WebSocket`
   implementation for wasm (the engine already runs as one async loop).
2. Google sign-in by redirect (the Web OAuth client), session in IndexedDB.
3. CORS on the Worker for the web app's origin only.

## Run it

```sh
cargo install wasm-bindgen-cli@<version in Cargo.lock> --locked   # once
just web          # dev build, http://localhost:8080
just web-release  # what Cloudflare serves (wasm-opt runs if installed)
```
