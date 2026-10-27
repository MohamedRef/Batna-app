# Batna: working conventions

Batna (باطنة) is a personal internal-medicine study app in Rust, for one learner: Mohamed, who is studying for an Egyptian MSc in General Internal Medicine. His exam is in January 2027. Batna is private and is never published.

It lives on the orphan branch `batna` of `mohamedref/kabar-app`. **Kabar (on `main`) is a separate app: never change `main`, never open PRs against it, and never depend on Kabar crates.** Copy a Kabar module when it helps, adapt it, and credit the source path in a comment.

The plan and the per-thread briefs are in `/mnt/project-files/medstudy/SPLIT.md` and `chunks/`. Where this file and SPLIT.md differ, SPLIT.md wins.

## Non-negotiables
- $0 a month. Ask before adding anything that could cost money.
- No secrets in the repo, ever.
- **The textbook never goes in git**: no PDF and no extracted text. Items paraphrase it, cite `src.page`, and quote at most 25 words. CI enforces this.
- Every content item ships `status: unverified`. Only Mohamed approves (`content/approved.txt`, from his in-app export).
- The book's answer is the exam answer. Guideline differences go in `update:` with the guideline name and year, never as silent changes.
- No real patient data to any AI engine. Tests never call live APIs.
- No generated images and no image assets.

## Language
- Reply to Mohamed in Egyptian Arabic, and keep it short. Code, comments, commit messages and docs are in English.
- UI text is Egyptian Arabic (`crates/app/src/i18n.rs`, one module per feature). Medical content is in English, wrapped in `<bdi>` inside Arabic UI.

## Workflow
- Branch from `origin/batna` and open PRs with base `batna`. Before merging, merge the latest `origin/batna`, run `just check`, wait for green CI, then squash-merge it yourself.
- `just check` = check-tracked-files, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `batna-tools validate`.
- Keep `docs/RUN.md` current.

## Code
- Rust stable, edition 2024. Workspace: `crates/core` (pure logic, no UI, no IO, builds for wasm32), `crates/app` (Dioxus 0.7 + HTML/CSS), and `crates/tools`.
- UI: logical CSS properties only (RTL). Layout by window width (compact under 600dp, medium under 840dp, expanded 840dp and up), never by device name.
- Review events are append-only. Statuses and settings are last-writer-wins. Synced tables carry `account_id`, `device_id`, an HLC timestamp and a tombstone.

## Tone
Warm, specific and encouraging. No guilt, no fear, no fake urgency, no dark patterns.
