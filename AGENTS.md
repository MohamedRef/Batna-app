# AGENTS.md: start here

You are continuing work on **Batna (باطنة)**, Mohamed Refaat's personal
internal-medicine study app (Rust + Dioxus; Android, Mac, web, Cloudflare
Worker). Earlier agents may be gone, along with their chats and shared
folders. Everything you need is in this repo.

## Read in this order

1. [`docs/HANDOFF.md`](docs/HANDOFF.md): what the app is, architecture,
   commands for every platform, CI, releases, secrets (names only), live URLs,
   content rules, current state and open work.
2. [`CLAUDE.md`](CLAUDE.md): working conventions and non-negotiables.
3. The doc for the area you touch (HANDOFF links them all).

## Rules that matter most

- Reply to Mohamed in **Egyptian Arabic**, short; status as Markdown tables.
  Code, comments, commits and docs in English.
- $0/month: ask before anything that could cost money. No secrets in git.
- **The textbook (PDF or extracted text) never goes in git.**
- Every content item ships `status: unverified`; only Mohamed approves.
- Never push Batna work to `MohamedRef/Kabar-app`; never depend on Kabar crates.
- Branch from `origin/main`, PR to `main`, add `changes/<topic>.yaml`, run
  `just check`, wait for green CI, squash-merge yourself.
- New releases (calendar versions, `2026.11.N`) only when Mohamed asks
  («طلّع ألفا»).

## Small files (fewer merge conflicts)

Many PRs run in parallel, so code is split so that two PRs rarely edit the
same file (Mohamed, 2026-11-05).

- One responsibility per file. Aim for under ~500 lines; a Rust file over
  **800 lines** should be split (`scripts/check-file-sizes.sh` lists them and
  CI shows a warning; it never fails a PR).
- Split a big file when your PR already changes it a lot, in its **own
  commit or PR that only moves code**: no behaviour change, same tests, items
  keep their paths through `pub use` re-exports.
- A feature adds its own files and touches shared files by one line at most:
  UI strings go in `crates/app/src/i18n/<feature>.rs` plus one
  `pub mod <feature>;` line in `i18n/mod.rs`; a calculator is its own
  `calc/<id>.rs` plus one `mod` line and one `REGISTRY` line.
- Long lists that every PR appends to (registries, `mod` lists, tables) stay
  one entry per line, sorted or grouped, so parallel additions merge cleanly.
- Tests of a module live beside it in a child `tests.rs`, not inline at the
  bottom of a big file.

## Keep the handoff current (required)

**Every PR that changes behaviour, process, commands, CI, secrets, deploys,
content rules or the state of the work must update `docs/HANDOFF.md` in the
same PR**: the section it affects, § Current state / Open work if they
changed, and one line in § Log. A new rule from Mohamed goes into HANDOFF (and
`CLAUDE.md` if it is a working convention) in the PR that applies it. A PR
that changes none of these needs no HANDOFF edit.

The goal: if every chat and project file were deleted, a new agent given only
this repo (and the message in HANDOFF § Resume) continues from the same point.
