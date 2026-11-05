# Batna handoff: everything a new agent needs

This file lets any new agent, account or chat continue Batna from where the
last one stopped, with no other history. Read it top to bottom once, then
`CLAUDE.md` (working conventions) and the doc for the area you touch.

**Keep it current.** Every PR that changes behaviour, process, commands,
secrets, deploys or the state of the work updates this file in the same PR
(usually § Current state and § Log). See `AGENTS.md`.

Last full refresh: 2026-11-05, main at `c35ffec` (1.0.0-alpha.2).

---

## 1. What Batna is

- **Batna (باطنة)**: a personal internal-medicine study app for one learner,
  **Mohamed Refaat** (GitHub `MohamedRef`), who is studying an Egyptian
  **MSc in General Internal Medicine**. Exam: **January 2027**.
- Study model: short retrieval sessions, FSRS spaced repetition, MCQ bank with
  explanations, key-feature cases, interactive Egyptian-Arabic lessons,
  calculators, oral-exam mode, stats and planner, Anki export.
- Devices: **Galaxy S24 Ultra** (phone, ~384dp), **Galaxy Tab S11** (753dp
  portrait / 1205×680dp landscape), **MacBook Air M2**, plus a **web** build.
- Personal only: never published to a store. Releases are sideloaded APKs, a
  self-signed Mac app, and a web site on Cloudflare.
- Repo `MohamedRef/Batna-app` is **public on purpose** while testing (free
  Actions minutes). Mohamed makes it private later. Don't flag it.
- **Kabar** (`MohamedRef/Kabar-app`) is his separate Indonesian/Javanese app
  with the same architecture. Never push Batna work there and never depend on
  Kabar crates; copy a module and credit it ("adapted from kabar-app main:
  <path>").

### Talking to Mohamed

- Reply in **Egyptian Arabic**, short. Code, comments, commits, docs: English.
- He wants **status and statistics as Markdown tables**.
- "Do everything yourself": open PRs and squash-merge them yourself once CI
  is green. Still **ask before anything that costs money** or changes the
  stack, and before anything that can't be undone outside the repo.
- Treat his asks as seeds and expand them sensibly; he dislikes being a
  bottleneck. He prefers the agent does console/browser setup itself and only
  asks him to sign in.
- A bare "." or "?" from him is not an approval.
- PRs he opens from ChatGPT/Codex under his login ("No merge requested"):
  review the whole diff critically, fix and improve on the branch, then merge
  (his standing rule, 2026-10-30).
- New alpha releases only when he says «طلّع ألفا» (§ 6).

## 2. Architecture

Rust stable (edition 2024, `rust-version` 1.90), one Cargo workspace,
Dioxus 0.7 (HTML/CSS UI in a WebView), SQLite, plus minimal Kotlin for Android.

| Crate | Package | What |
|---|---|---|
| `crates/core` | `batna-core` | Pure logic: content schema types, FSRS (`srs`), review store traits + HLC (`review`), session composer, exam/MCQ shuffling, calculators (`calc`: 92 in `REGISTRY`, `calc::normals`, `calc::guide`), lab ranges (`labs`), glossary finder, update feed checks, content pack format (`pack`). No UI, no IO; builds for wasm32. |
| `crates/protocol` | `batna-protocol` | Every type that crosses a process boundary (sync ops, `sync::TABLES`, HLC, auth, update feed). Versioned serde. |
| `crates/app` | `batna-app` | The Dioxus app for desktop, Android and web (`--no-default-features --features web`). One module dir per feature: `today`, `study`, `bank`, `case`, `oral`, `learn` (lessons), `lecture`, `calc`, `stats`, `plan`, `own` (own cards), `abbrdict`/`abbrsheet`, `labs`, `hover` (glossary peek), `pagesheet` (book page chips), `settings`, `update`, `cloud`, `store`, `platform`, `i18n.rs` (all Arabic UI text). |
| `crates/anki` | `batna-anki` | `.apkg` deck builder shared by tools and the app's «صدّر لـ Anki». |
| `crates/server` | `batna-server` | **Batna Cloud**: Cloudflare Worker (workers-rs) + one Durable Object per account (devices, sync hub) + `Login` DO per username. No D1. |
| `crates/tools` | `batna-tools` | `validate`, `packs`, `anki`, `refs` (book page refs), `ids`, `content` (next-id/new-item/diff/preflight), `verdicts`, `update` (keygen/manifest/sign/verify/channels/changes). |

Other top-level paths: `content/` (all study content, § 7), `android/`
(Kotlin glue: WebViewGuard, BackKey, UpdateInstaller, notifications, Share),
`web/` (index.html, sw.js service worker), `assets/`, `scripts/`, `fixtures/`
(recorded replies, test feeds), `changes/` (What's new fragments).

Key design rules (also in `CLAUDE.md`):
- Synced tables carry `account_id`, `device_id`, HLC, tombstone. Reviews are
  append-only events; statuses and settings are LWW by HLC.
- **A new synced table must be added to `batna_protocol::sync::TABLES`** and
  written through the store so it is queued in `sync_outbox`; otherwise the
  hub rejects it.
- Layout by window width (compact <600dp, medium <840dp, expanded ≥840dp),
  never by device. Logical CSS properties only (RTL). English medical text
  inside Arabic UI is wrapped in `<bdi>`.
- App code must also compile for wasm: no `std::time::Instant/SystemTime`, no
  `std::thread::spawn` (use `web_time`, `platform::time`).

## 3. Build, test, run

`just` drives everything (`brew install just`; in a cloud container it is
usually missing, so run the recipe lines by hand). `.env` (git-ignored, from
`.env.example`) holds `BATNA_CLOUD_URL`, `BATNA_GOOGLE_CLIENT_ID`,
`BATNA_GOOGLE_CLIENT_SECRET` for local builds.

| Task | Command |
|---|---|
| Full gate before every commit | `just check` = `scripts/check-tracked-files.sh`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `batna-tools validate`, `batna-tools packs build --check` |
| Content only | `cargo run -q -p batna-tools -- validate`; `just content-preflight` |
| Mac / Linux desktop | `just run` (`BATNA_DATA_DIR=<dir>` for a throwaway profile) |
| Android | `just android-keystore` (once), `just apk`, `just devices`, install with adb ([RUN.md](RUN.md)) |
| Mac app bundle | `just bundle-mac` (self-signed; first install `xattr -dr com.apple.quarantine /Applications/Batna.app`) |
| Web | `just web` (dev), `just web-release` → `target/web/batna` |
| Cloud local | `just cloud-dev`; end-to-end test `node crates/server/tests/e2e.mjs` (fake Google, two devices, local workerd) |
| Coverage | `just coverage` (floor in `scripts/coverage-floor`, now **94**%) |
| Anki file | `cargo run -q -p batna-tools -- anki --out <dir>` → one `batna.apkg`, a subdeck per lecture |

Container gotchas:
- Linux desktop builds need `apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev`.
  Without them use `cargo test --workspace --exclude batna-app` and rely on CI.
- Two target dirs (Batna + Kabar) fill a 40 GB allowance; delete one before building the other.
- `*.workers.dev` is blocked by the cloud proxy: verify deploys through Actions runs.
- `gh` works for REST (`gh api repos/...`) but not GraphQL; secrets and job-log endpoints return 403.
- Screenshots: Xvfb + `BATNA_WINDOW_SIZE=412x915` (phone) / `753x1130` / `1205x680` (tablet), or
  the web build + Playwright (preinstalled Chromium).

More: [RUN.md](RUN.md), [TESTING.md](TESTING.md), [WEB.md](WEB.md), [CLOUD.md](CLOUD.md).

## 4. CI and workflows (what runs when)

Mohamed (2026-11-04, firm): **pull requests run only the fast checks**; heavy
jobs are periodic or manual. Never re-add PR/push triggers to heavy workflows.

| Workflow | Runs on | What |
|---|---|---|
| `ci.yml` | PR + push to main (newer run cancels older) | PR: `check` job (fmt, clippy, tests, content, ids, What's new file) + Android arm64 clippy. Main only: web wasm build, Worker build + cloud e2e. |
| `coverage.yml` | every 4 h on main (skips if unchanged) + manual | line-coverage floor (94%). Red run → fix in its own PR. |
| `batna-android-preview.yml` | every 4 h (skips if main unchanged) + manual | signed preview APK → `batna-preview` release + feed |
| `settings-browser.yml`, `screens-browser.yml` | every 4 h + manual; PR only when the test file changes | Playwright checks of Settings / screens on the web build |
| `android-webview-regression.yml` | `android/**` changes + manual | Kotlin/JVM tests |
| `mac.yml` | nightly 02:17 UTC + manual, **never on PRs** | macOS build. Don't wait on it to merge. |
| `web-deploy.yml` | push to main touching app/content/web | builds wasm, deploys Worker `batna-web` |
| `cloud-deploy.yml` | push to main touching server/protocol/Cargo + manual | deploys Worker `batna-cloud` |
| `release.yml` | manual only | alpha/beta/stable release (§ 6) |

Gotcha: `cloud-deploy.yml` finishes **green but skipped** when
`CLOUDFLARE_API_TOKEN` is missing. Check that the step "Deploy" succeeded.

## 5. Merge routine

1. Branch from `origin/main`; PR with base `main`.
2. Add `changes/<topic>.yaml` (every PR; CI fails without one):
   `- {kind: new|improved|fixed, ar_eg: "...", en: "..."}`, one line per
   change Mohamed can see, max 160 chars per text, quote lines containing
   `:`. Housekeeping PRs add a file holding `[]`.
3. Update this file if behaviour, process, commands or state changed.
4. Before merging: merge latest `origin/main` into the branch, run the checks,
   wait for green CI on that commit, then **squash-merge yourself**.
   It's fine to merge when CI is green and only slightly behind main with no
   overlap. Never force-push `main`. If main is red, look for an open fix PR first.
5. A session may only push its own designated branch; deleting other
   branches returns 403.

## 6. Releases and updates

- Version in workspace `Cargo.toml` (now **1.0.0-alpha.2**).
- Channels: `batna-preview` (CI, every 4 h), `batna-alpha`, `batna-beta`,
  `batna-stable`. Human release tags: `batna-v<version>`.
- **Alpha only when Mohamed says so** («طلّع ألفا»). Steps: PR bumping
  `version` (+ `changes/` line) → merge → Actions › Release › Run workflow.
  It builds the content pack, release APK (Linux), Batna.app (macOS), signs
  the feed and publishes. Then verify `update.json` with
  `batna-tools update verify` against `crates/app/update-key.pub`.
- Preview builds are signed with the release keys too, so any build can switch
  channel in Settings › Updates.
- Over-the-air: signed `update.json` (+ `.sig`) per channel; separate signed
  **content pack** so content/CSS changes reach devices without reinstalling
  (CI skips the APK when only content/changes/docs/CSS changed). Details:
  [RELEASE.md](RELEASE.md), [COMPATIBILITY.md](COMPATIBILITY.md).
- What's new comes from `changes/*.yaml` (Morphe-style sheet before/after an update).

### Secrets (names only, values never in git)

| Where | Name | What |
|---|---|---|
| GitHub Actions | `BATNA_UPDATE_SECRET` | Ed25519 feed signing key |
| | `BATNA_ANDROID_KEYSTORE`, `BATNA_ANDROID_KEYSTORE_PASSWORD` | APK release keystore |
| | `BATNA_MAC_CERT_P12`, `BATNA_MAC_CERT_PASSWORD` | self-signed "Batna Code Signing" identity |
| | `BATNA_GOOGLE_CLIENT_SECRET` | Google Desktop client secret (one shared value with Kabar) |
| | `CLOUDFLARE_API_TOKEN` | deploys batna-cloud and batna-web |
| | `BATNA_ALLOWED_EMAILS` | who may sign in to Batna Cloud |
| Mac Keychain | `batna-update`, `batna-android`, `batna-codesign` | originals, made by `scripts/release-secrets.sh` (also `~/.batna/`) |
| Mac `.env` | `BATNA_CLOUD_URL`, `BATNA_GOOGLE_CLIENT_ID`, `BATNA_GOOGLE_CLIENT_SECRET` | local builds |
| Worker | `ALLOWED_EMAILS`, `TOKEN_SIGNING_KEY` | set by the deploy workflow |

Public (not secret): Google client ids in `crates/server/wrangler.toml`
(`GOOGLE_CLIENT_IDS`, Desktop + "Batna web"), `WEB_ORIGINS`.

### Live URLs

| What | URL |
|---|---|
| Batna Cloud (sync) | https://batna-cloud.hamorefaat85.workers.dev |
| Batna web | https://batna-web.hamorefaat85.workers.dev |
| Releases | https://github.com/MohamedRef/Batna-app/releases |

Sign-in: Google (Desktop client, PKCE loopback; web redirect client) **or
username + password** (Argon2id on device, `Login` DO; changing
`PASSWORD_PARAMS` or the salt prefix invalidates every password). "Not my
device" sign-in wipes the study DB on sign-out. On macOS the session lives in
`<data dir>/cloud-session.json`, not the Keychain (self-signed apps re-prompt).
[CLOUD.md](CLOUD.md) § 4.1.

## 7. Content

### Courses

The MSc has 20 courses ([courses/COURSES.md](courses/COURSES.md),
`content/program.yaml`). Content exists for 9, from the Alexandria Faculty of
Medicine "Internal Medicine Postgraduate Lectures" (2025) and research slides:

| Course id | Course | Source | Lectures in `content/<id>/` |
|---|---|---|--:|
| `hb` | Hepatology (book ch. 1) | part 1 | 20 |
| `git` | Gastroenterology (ch. 2) | part 1 | 11 |
| `rheum` | Rheumatology & Immunology (ch. 3) | part 2 | 21 |
| `hem` | Hematology (ch. 4) | part 2 | 12 |
| `endo` | Endocrinology (ch. 5) | part 3 | 7 |
| `dm` | Diabetes, Metabolism, Lipids (ch. 6) | part 3 | 23 |
| `geri` | Geriatrics (ch. 7) | part 4 | 13 |
| `neph` | Nephrology (ch. 8) | part 5 | 14 |
| `res` | Basics of Research | slide decks (21 lectures 3 Oct–12 Dec 2026) | 8 |

Semester 1 = `res`, `hem`, `hb`. Not started (no material yet): `cvs`,
`chest`, `neuro`, `trop`, `img`, `bioch`, `phys`, `path`, `clinpath`,
`pharm`, `micro`. Never rename `hb.*` ids (they are synced review keys);
ids are never reused or renumbered (retired ids in `content/ids-retired.txt`).
No bare "L03" codes in the UI: always course + number/title.

Files: `content/<course>/LNN.yaml` (`batna.content/v1` items: card, cloze,
mcq, enumerate, case, table, order, calc, essay),
`content/lessons/<course>/LNN.yaml` (`batna.lesson/v1` interactive Arabic
lessons), `content/book-refs/<course>/` (page chips), `content/glossary.yaml`
(abbreviations), `content/calc-quotes.yaml`, `content/approved.txt`.
How to write: [CONTENT.md](CONTENT.md), [LESSONS.md](LESSONS.md),
[AUTHORING.md](AUTHORING.md), [SPLIT.md](SPLIT.md) (original thread rules;
quote cap there is outdated), course briefs in [courses/](courses/).

### Content rules (Mohamed's decisions; all still in force)

1. **The book's answer is the exam answer.** Guideline differences go in
   `update:` with guideline name + year, never as silent changes.
2. **Every item ships `status: unverified`.** Only Mohamed approves, through
   his in-app export → `content/approved.txt`.
3. **The textbook never goes in git**: no PDF, no extracted text
   (`check-tracked-files.sh` blocks `*.pdf` and any `book/` path). Items
   paraphrase and cite `src.page` (printed page of that chapter).
4. **Quotes: no length cap.** `src.quote` is the page's own words: the full
   paragraph/bullet that proves the fact. Tables are drawn as real tables;
   any mnemonic gets its breakdown in a small table under it; abbreviations
   are spelled in full the first time with the short form in brackets. No
   «من الكتاب بالنص» label.
5. **Abbreviations**: `batna-tools validate` fails on any lesson word with
   ≥2 capitals not in `content/glossary.yaml`. Add a `kind: abbr` entry
   (likeliest meaning first), a `course:` sense when the meaning differs per
   course, or put it in `plain:` if it is not an abbreviation. Per-course
   «قاموس الاختصارات» page + «اختصارات N» chip on lesson steps.
6. **تحشيشة (enumerate hooks)**: every `enumerate` item has a `hook`, an
   Egyptian-Arabic mnemonic, plain text, no extra facts; write ranges in
   Arabic words ("من 2 لـ 3") so RTL doesn't flip them. All 1,927 lists in 9
   courses are done; new enumerates need one. Combined file for Mohamed:
   `python3 -I scripts/content-tools/tahshishat-gen.py <repo> <out-prefix>`.
7. **Calculators** must rest on cited original papers/guidelines. Every
   result band states what it means, the **next step** and the **DD**
   (`GUIDES` per module; app test `every_result_has_a_next_step` blocks a band
   without a guide). Normal ranges in `calc::normals` (book first, then NBME
   lab values April 2026). Every cited page has its book quote
   (`content/calc-quotes.yaml`). Each calculator needs an Arabic blurb in
   `i18n::calc`. Always call `spec.run`, never `(spec.compute)`. Regenerate
   [CALCULATORS.md](CALCULATORS.md) after adding one.
8. **Lab values** in lessons/cards show a normal-range chip (`batna_core::labs::LABS`).
9. **Images**: no generated images. Crops of the book's own figures
   (`book: true`, page cited) or open-licence images with credit, in
   `content/lessons/img/`, small WebP. Code-drawn SVG where clearer.
10. **Arabic number agreement** in UI: 1 singular, 2 dual, 3–10 plural,
    0 and 11+ singular. Use `i18n::count::count(n, &NOUN)` / `count_text` /
    `word`; never `format!("{n} درس")`.
11. **No dead UI**: anything that looks like a card/tile/row/chip opens what
    it shows (`.tap` classes; `dead_ends.rs` test lists tappable classes).
12. YAML gotchas: unquoted flow lists split on commas (quote options); quote
    titles containing `: `; deleting an item also removes it from lesson
    `practice` lists.
13. No real patient data to any AI engine; tests never call live APIs.

## 8. Sources outside git (important)

The book PDFs, their extracted page text and page images are copyrighted and
**must never be committed**. They lived in the Claude project's shared folder
`/mnt/project-files/medstudy/` (about 880 MB):

| Path (under `medstudy/`) | What |
|---|---|
| `book/internal-medicine-pg-part1.pdf` … `part5-nephrology.pdf` | the five textbook PDFs |
| `book/LNN.txt` | hepatology text by printed page (printed p = PDF p + 4) |
| `courses/<id>/text/LNN.txt` | other courses: `=== book page N (PDF page M) ===` blocks + tesseract OCR of image pages |
| `courses/<id>/img/pNNN.jpg` | page images at 200 dpi |
| `courses/res/slides/*.pdf` | research slide decks (`src.page` = slide number) |
| `anki/batna.apkg` | the latest Anki export |
| `book-issues.md` | errors found in the book |

**Mohamed must keep his own copy of this folder** (Mac or Drive). If it is
gone, rebuild the text from the PDFs: `pdftotext -layout -f M -l M` per page
into the format above, `pdftoppm -r 200 -jpeg` for images, `tesseract` for
image-only pages. Content scripts take the folder from `BATNA_SOURCES`
(default `/mnt/project-files/medstudy`): see
[scripts/content-tools/README.md](../scripts/content-tools/README.md).

## 9. Testing

- ~95% line coverage (floor 94, ratchet = one point under main). Run
  `just coverage` before merging a PR that adds code.
- Harnesses: FakeCloud sync harness + workerd e2e; click-through DOM tests
  (`testdom.rs`, `test_dom.rs`, `tap_dom.rs`, `testing.rs`; merging them is
  open work); Playwright browser checks in `crates/app/tests/*.cjs`.
- Android Back: `system-back.js` keeps one history trap; `BackKey.kt` routes
  the Back key through the page («every Back = one step; exit only from
  Today»). Tests must use `history.system()`, not `history.back()`.
- Android crash reports start from Settings › Your data › diagnostics
  (WebViewGuard records renderer loss and exit reasons).

## 10. Current state (2026-11-05)

| Area | State |
|---|---|
| App | Feature-complete for alpha: Today, study, bank, cases, oral, lessons, 92 calculators, stats/planner, own cards + Share, Anki export, abbreviations, lab chips, themes (Paper, Night, Night shift, true black, two System themes), sync, password login, OTA app + content updates, web |
| Releases | `batna-v1.0.0-alpha.2` published 2026-11-04 (feed `batna-alpha`); previews every 4 h |
| Critic loop | Final round (pass 10) finished 2026-11-04; loop closed. Don't start another unless Mohamed asks. |
| Open PRs / issues | none on 2026-11-05 |
| Coverage | 95.05% (floor 94) |

### Open work and known gaps

- Content for courses with no material yet; `res` L09–L21 as slides arrive.
- Content still `unverified` until Mohamed approves in-app.
- Glossary: split remaining ICP/SBP stop-gap senses into `course:` entries.
- Review leftovers from the 2026-11-04 content audit (ambiguous abbreviations,
  quote cuts) if Mohamed asks; lists were in `/mnt/project-files/content-audit/`.
- Low, deliberately not fixed: IndexedDB handle never closed per save;
  Argon2 on the web main thread (~0.5 s); a second tab on a borrowed computer
  wipes the first tab's data.
- Test debt: merge duplicate click-test helpers; more calculator tests against
  original-paper examples and cut-offs.
- Not yet verified on a real device: Back-key routing, update notifications.

## 11. Resume message

Mohamed can paste this into any new chat or account to continue:

> Continue work on my apps Batna (`MohamedRef/Batna-app`) and Kabar
> (`MohamedRef/Kabar-app`). Before anything else, read `AGENTS.md` and
> `docs/HANDOFF.md` in each repo you touch and follow them. Reply to me in
> Egyptian Arabic. Keep `docs/HANDOFF.md` updated in every PR that changes
> behaviour, process or state.

## 12. Log

Newest first. One line per PR that changes behaviour, process or state.

- 2026-11-05: handoff docs added (`AGENTS.md`, this file, course briefs, content scripts).
- 2026-11-04: 1.0.0-alpha.2 released (#726); final critic round closed.
