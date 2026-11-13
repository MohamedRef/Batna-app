# Batna handoff: everything a new agent needs

This file lets any new agent, account or chat continue Batna from where the
last one stopped, with no other history. Read it top to bottom once, then
`CLAUDE.md` (working conventions) and the doc for the area you touch.

**Keep it current.** Every PR that changes behaviour, process, commands,
secrets, deploys or the state of the work updates this file in the same PR:
the section it affects (feature notes go in § 10), § 11 Current state if it
changed, and one line in § 13 Log. See `AGENTS.md`.

Last full refresh: 2026-11-09, main at `d6214ce` (= release `v2026.11.2`).

**Contents:** 1 What Batna is · 2 Architecture · 3 Build, test, run · 4 CI ·
5 Merge routine · 6 Releases, secrets, URLs · 7 Content · 8 Sources outside
git · 9 Testing · 10 Feature notes · 11 Current state and open work ·
12 Resume message · 13 Log

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
- New releases only when he says «طلّع ألفا» (§ 6).

## 2. Architecture

Rust stable (edition 2024, `rust-version` 1.90), one Cargo workspace,
Dioxus 0.7 (HTML/CSS UI in a WebView), SQLite, plus minimal Kotlin for Android.

| Crate | Package | What |
|---|---|---|
| `crates/core` | `batna-core` | Pure logic: content schema types, FSRS (`srs`), review store traits + HLC (`review`), session composer, exam/MCQ shuffling, calculators (`calc`: all in `REGISTRY`; README's count is checked by a test, `calc::normals`, `calc::guide`), lab ranges (`labs`), glossary finder, update feed checks, content pack format (`pack`). No UI, no IO; builds for wasm32. |
| `crates/protocol` | `batna-protocol` | Every type that crosses a process boundary (sync ops, `sync::TABLES`, HLC, auth, update feed). Versioned serde. |
| `crates/app` | `batna-app` | The Dioxus app for desktop, Android and web (`--no-default-features --features web`). One module dir per feature: `today`, `study`, `bank`, `case`, `oral`, `learn` (lessons), `lecture`, `calc`, `stats`, `plan`, `own` (own cards), `abbrdict`/`abbrsheet`, `labs`, `hover` (glossary peek), `pagesheet` (book page chips), `settings`, `update`, `cloud`, `store`, `platform`, `i18n/` (all Arabic UI text, one file per feature). |
| `crates/anki` | `batna-anki` | `.apkg` deck builder shared by tools and the app's «صدّر لـ Anki». |
| `crates/server` | `batna-server` | **Batna Cloud**: Cloudflare Worker (workers-rs) + one Durable Object per account (devices, sync hub) + `Login` DO per username. No D1. |
| `crates/tools` | `batna-tools` | `validate`, `packs`, `anki`, `refs` (book page refs), `ids`, `content` (next-id/new-item/diff/preflight), `verdicts`, `update` (keygen/manifest/sign/verify/channels/changes). |

Other top-level paths: `content/` (all study content, § 7), `android/`
(Kotlin glue: WebViewGuard, BackKey, UpdateInstaller, notifications, Share),
`web/` (index.html, sw.js service worker), `assets/`, `scripts/`, `fixtures/`
(recorded replies, test feeds), `changes/` (What's new fragments), `vendor/`
(Dioxus 0.7.10 `dioxus-desktop` + `dioxus-interpreter-js` patched so the
screen survives Android destroying its edit socket; see `vendor/README.md`,
and re-check it before any Dioxus upgrade).

Key design rules (also in `CLAUDE.md`):
- Synced tables carry `account_id`, `device_id`, HLC, tombstone. Reviews are
  append-only events; statuses and settings are LWW by HLC.
- **A new synced table must be added to `batna_protocol::sync::TABLES`** and
  written through the store so it is queued in `sync_outbox`; otherwise the
  hub rejects it. The Worker that knows the table must be deployed: check the
  "Deploy Batna Cloud" run of that PR and re-run it the same day if it fails
  (#780's deploy failed on a 504 and the hub rejected `notes` for two days).
  A hub that doesn't know a table answers `UnknownTable`; the client keeps
  those ops in the outbox, skips that table until the next connection, and
  sends them again then. `store/sync.rs` `RESEND` queues a table's own rows
  again once, for ops an older build dropped.
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
| Full gate before every commit | `just check` = `scripts/check-tracked-files.sh`, `scripts/check-file-sizes.sh` (warns only), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `batna-tools validate`, `batna-tools packs build --check` |
| Content only | `cargo run -q -p batna-tools -- validate`; `just content-preflight` |
| Lesson coverage (#833) | `batna-tools lessons --summary` (course table) · `--report <course>` (TSV per lesson) · `--warnings [<course>]` |
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
| `ci.yml` | PR + push to main (newer run cancels older) | PR: `check` job (fmt, clippy, tests, content, ids, What's new file) + Android arm64 clippy. Main only: the Study perf diagnostics into the job summary (§ 10), web wasm build, Worker build + cloud e2e. |
| `coverage.yml` | every 4 h on main (skips if unchanged) + manual | line-coverage floor (94%). Red run → fix in its own PR. |
| `batna-android-preview.yml` | every 4 h (skips if main unchanged) + manual | signed preview APK → `batna-preview` release + feed |
| `settings-browser.yml`, `screens-browser.yml` | every 4 h + manual; PR only when the test file changes | Playwright checks of Settings / screens on the web build |
| `android-webview-regression.yml` | `android/**` changes + manual | Kotlin/JVM tests |
| `mac.yml` | nightly 02:17 UTC + manual (+ once when `mac.yml` itself changes on main), **never on PRs** | macOS build; manual runs on main also upload a signed `Batna-Mac-preview-<sha>` artifact (§ 10). Don't wait on it to merge. |
| `cloud-e2e-regression.yml` | PR only when `crates/server/tests/e2e.mjs` or this file changes + manual | full local Worker e2e (fake Google, two devices) |
| `web-deploy.yml` | push to main touching app/content/web | builds wasm, deploys Worker `batna-web` |
| `cloud-deploy.yml` | push to main touching server/protocol/Cargo + manual | deploys Worker `batna-cloud` |
| `release.yml` | manual only | alpha/beta/stable release (§ 6) |
| `audit.yml` | weekly (Mon 05:17 UTC) + manual, **never on PRs** | `cargo audit` against RustSec (audit F23). A finding → fix in its own PR. |

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

- **Numbering (Mohamed, 2026-11-08):** calendar versions `YEAR.MONTH.N`
  (`2026.11.1` = first release of November 2026) and one build number for
  every build, preview or release (commits on `main`). Android versionCode
  = 20000000 + build; Mac CFBundleVersion = build. Release tags `v<version>`
  (the old ones are `batna-v1.0.0-alpha.1…3`). `scripts/version.sh` works it
  out; `Cargo.toml` keeps `2026.11.0` and is never bumped by hand. Details:
  `docs/RELEASE.md` §1.
- Channels: `batna-preview` (CI, every 4 h), `batna-alpha`, `batna-beta`,
  `batna-stable` (feed tags: never rename, installed apps read them).
- **Alpha only when Mohamed says so** («طلّع ألفا»). Steps: Actions ›
  Release › Run workflow (channel alpha); no bump PR.
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
- A check shows the app's answer as soon as the feed is read; a newer
  content pack downloads after it (Settings shows «بيجيب الدروس…»), then an
  automatic build download. Every update request fails after 20 s without
  progress (lookup, connect, each read), so a connection Android cut while
  the phone was locked doesn't hold the check for minutes.

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
| `res` | Basics of Research | slide decks (21 lectures 3 Oct–12 Dec 2026) | 20 (L01–L20) |

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
   A lesson step that teaches a list with a تحشيشة lists it in `remind:` so it
   shows again there (from another lesson too, with a link back).
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
14. **Lesson count follows the lecture** (Mohamed, 2026-11-13, #828): about
    one lesson per four book pages with cards, at most about 25 cards and two
    book topics per lesson, each still 6–10 minutes and 8–16 steps
    ([LESSONS.md](LESSONS.md) rule 8). Lesson order stays as it is. A page
    with nothing to teach goes in `content/lesson-skips.yaml` with a reason
    (default until he says otherwise), not a filler step. Hepatology is the
    depth bar (#827); additions are labelled as not from the book.

## 8. Sources outside git (important)

The book PDFs, their extracted page text and page images are copyrighted and
**must never be committed**. They lived in the old project's shared folder
`/mnt/project-files/medstudy/` (about 880 MB):

| Path (under `medstudy/`) | What |
|---|---|
| `book/internal-medicine-pg-part1.pdf` … `part5-nephrology.pdf` | the five textbook PDFs |
| `book/LNN.txt` | hepatology text by printed page (printed p = PDF p + 4) |
| `courses/<id>/text/LNN.txt` | other courses: `=== book page N (PDF page M) ===` blocks + tesseract OCR of image pages |
| `courses/<id>/img/pNNN.jpg` | page images at 200 dpi |
| `courses/res/slides/*.pdf` | research slide decks (`src.page` = slide number). A lecture taught as several decks is one merged PDF (`res-L09.pdf` = budget 1–14 + ethics 15–85 + plagiarism 86–141; `res-L13.pdf` = writing a paper 1–53 + Vancouver 54–120); the original files are in `slides/originals/` |
| `../Lecture N ….pdf`, `../Dr Sahar Revision.pdf`, `../Revision Dr Eman Anwar.pdf`, `../Biostatistics Revision.pdf`, `../Revision Dr Heba.pdf` (project-folder root) | enriched lecture copies: frames of the recorded lecture (video slide numbering, not the deck's), then Arabic study notes of what the lecturer said (text layer: `pdftotext` + NFKC). Facts only said in the talk cite the slide being discussed and carry the tag `lecturer`; revision questions carry `revision` |
| `anki/batna.apkg` | the latest Anki export |
| `book-issues.md` | errors found in the book |

**Backup: the private repo `MohamedRef/Batna-sources`** (since 2026-11-05)
holds this whole folder at its root (PDFs, extracted text, page images,
briefs, `book-issues.md`), plus **every other file of the old project folder**
under `project-files/` (screenshots, art, audits, tahshishat exports, uploads;
files over 100 MB split in `project-files/big/`) and the project's memory
notes under `project-memory/`. About 2.5 GB; see its README. **Never copy anything from it into Batna-app**
(public). Clone it and set `BATNA_SOURCES=<clone>` for the content scripts.
When new books, slides or extracted text arrive, commit them there too. If it
is ever gone, rebuild the text from the PDFs: `pdftotext -layout -f M -l M` per page
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
- Journey test (audit F23): `store/backend/journey_tests.rs` runs first
  launch → grade → restart → restore → two-device sync on one review state.
  What only a device can show (process death, TalkBack, 200% text…) is the
  manual matrix in `docs/NATIVE_QA.md`; record runs there against their
  exact commit.
- Shared plumbing with Kabar: `docs/INFRA_PARITY.md` (audit F24, same file
  in both repos). A fix there says in its PR whether Kabar needs it too.

## 10. Feature notes (what exists, where, and why)

Durable notes on recent features, distilled from the PRs of 2026-11-07/08.
Add a bullet here when a PR adds something a later agent must know.

### Lessons

- **Focus layout** (#767, replaces #739's two-pane grid): from 840px up the
  player is one centred column (40rem; 44rem with 1.125rem text from
  1200px, ≈ 60–75 characters a line), its top at the same height on every
  step, the lesson name a muted header above the card, Back/Next following
  the card (sticky while a long step scrolls). Phones keep the bottom nav.
  `crates/app/src/learn/learn.css`; `screens-browser.cjs` asserts it at
  840/1280/1600px. Research basis: [LESSONS.md](LESSONS.md) § Reading layout.
- **Book map on lesson rows** (#830): each Lessons-screen row has a third
  line with the page range (`BookPages`, «سلايد» for slide courses) and the
  lecture's `topics` titles for the lesson's topics (`learn::section_titles`),
  one clipped line in list view, hidden in compact view; the player header
  shows the same range beside the lecture title.
- **Book page → lesson step** (#831): the page sheet lists the lesson
  steps citing the page in that book («الشرح في الدروس», rows «<lesson> ·
  خطوة N», `pagesheet::explain::explains`) or says none does; a row calls
  `learn::open_at(ui, lesson, step)`, which opens the player at that step
  without saving (the saved place and first results change only once he
  answers or moves; Back reaches earlier steps). «اشرحهولي» opens at the first
  step citing the card's `src.page` (`learn::step_on_page`). The lecture page
  has a collapsible «بترتيب الكتاب» outline (`lecture/outline.rs`): one row per
  page with the section (majority topic of its cards), the steps and the card
  count; a row opens the page sheet.
- **«صفحة» is the book's word** (#832): the lesson's flip view is «تقليب»
  and counts «خطوة 3 من 12» like the steps view; «صفحة»/«ص» only ever mean a
  book page.
- **Reading settings** («Aa» in the lesson bar, and Settings › Look; adapted
  from Steps-app main: `crates/app/src/learn/reading.rs`): text size
  (×0.9/1/1.15/1.3 on step text only, the bar and buttons stay), line spacing
  (×0.86/1/1.2) and line width (Narrow 34/37rem, Book = the focus column
  40/44rem, Wide 52/60rem; 840px and up only, phones are always full width).
  Device-local in `device-prefs.txt` (`read_size`, `read_spacing`,
  `read_width`; older files load the defaults), never synced. The popover sits
  over the step so the text never jumps; while open, «Aa» is the Escape/Back
  target. `data-read-*` on `.ls-player`, rules in `learn/reading.css`; tests
  `reading_choices_restyle_the_lesson_and_close_before_the_lesson`.
- **Text by importance**: `==key==` is a pill in `--mark`/`--mark-ink`, true
  yellow `#ffd43b` on Paper/Night/Black/Night shift (Mohamed's pick; he
  rejected a gold highlighter band), teal on Study themes; test
  `the_key_term_pill_stands_out`. Headings and `**terms**` use full `--text`;
  body text is `--text` softened 16% toward the card; meta stays muted.
- **Study themes** (`batna_core::theme`): `study` (warm off-white, charcoal
  text), `study-dim` (warm graphite, parchment text), `system-study`.
  AA-tested; `study_themes_are_calm` keeps contrast 7–15:1, no pure
  white/black.
- **Own highlights** (#770): select words in the open step → a bar
  (`learn/highlight.js`) with 5 colours, fill/underline, pill/rectangle; tap a
  highlight to edit or remove it; style is remembered per device
  (localStorage). Each highlight is one synced setting
  `hl.<lesson>.<id>` = `batna_core::highlight::Saved` (or `{"gone":true}`),
  so no new sync table. Anchored by quote + 32 chars of context + position
  (`highlight::locate`, mirrored in JS). Shapes are drawn in a `.uhl-layer`
  under the text (`[data-uhl-host]`), never by editing Dioxus nodes; fill text
  darkens via `::highlight(uhl-ink)`. Tests: `learn::highlight`,
  `batna_core::highlight`, `crates/app/tests/highlight-browser.cjs`.
- **Repeated topics remind their تحشيشة** (#771, Mohamed's ask from the ACLF
  card): steps take `remind: [<item id> | <lesson id>:<WORD>]`; the player
  (`learn/remind.rs`) shows each in a dotted box, with «📖 من درس …» opening
  the teaching lesson. 791 reminders on 689 steps in 126 lesson files (script
  match ≥80% of an enumerate's points or ≥70% of a mnemonic's letters, every
  off-topic match dropped by hand, plus 13 cross-lecture twins). Question,
  reveal and hidden-cell steps never get one. `pack::SCHEMA` = 3, so this
  needs an app build, not a content-only update. Checks:
  `lesson::remind_problems` and `batna-tools validate`
  (`lessons::remind_targets`). Rule: [LESSONS.md](LESSONS.md) rule 13.

### Study sessions

- **Keyboard** (#740, Anki-style, `crates/app/src/shortcuts/`): Space reveals,
  1–4 grade, Space/Enter = Good after reveal, `Z` = Undo, RTL arrows in
  lessons (Right = back, Left = next when allowed), A–E, Cmd/Ctrl+1–7,
  Cmd/Ctrl+, and `?` help. A modal owns shortcuts; typing fields,
  contenteditable, auto-repeat and IME composition are ignored.
- **Focus continuity** (#745): after keyboard/screen-reader reveal, grading,
  Undo or lesson step change, focus is restored; never on pointer clicks, in a
  modal or while typing. `keyboard-focus-browser.cjs`.
- **Resume**: `SESSION`/`PHASE` are app-lifetime signals, so switching tabs
  keeps the card (#754 tests a same-process remount and foreground scroll).
  Today shows **Continue** for a live unfinished session and returns to the
  same queue without re-planning or touching Undo (#755).
- **Durable checkpoint** (#756, #757, #763): a device-local versioned
  `SessionCheckpoint` in `meta.local.study.session.v1` (not synced) keeps
  queue order, progress, Again retries, outcomes and the review ids Undo
  needs; it is saved after start, grade, Undo and append, cleared on End or
  finish, and read before Today renders. It is trusted only when its review
  marker matches the committed review log; otherwise (crash between grade and
  save, unknown cards, altered events) it is dropped and the app starts
  normally, never re-recording grades. Marker `v2` is an O(1) random revision
  in `meta`, rotated by migration 4's INSERT/UPDATE/DELETE triggers on review
  events in the same transaction; `v1` checkpoints are checked once against
  the old full-log hash, then upgraded. A borrowed-device wipe keeps and
  rotates the revision row.
- **Background time** (#753): returning from background
  (`visibilitychange`/`pageshow`) resets only the running item's answer clock.
  `study-visibility.test.cjs`.
- **A save that fails is never silent** (engineering audit F02, 2026-11-11):
  every grading entry point logs through `StudyStore::record_logged` and acts
  on its result, never through the fire-and-forget `record`. Study, a lesson's
  Try step (`learn/practice.rs`) and Oral (`oral/mod.rs`) keep the item and
  the answer, say so and let him rate again; the question bank keeps refused
  answers in `bank/unsaved.rs` (`UNSAVED`) with a «احفظها تاني» button and
  hides «sent to review» until they are in the log. Tests use
  `Store::set_read_only` (SQLite `query_only`) to refuse and then accept
  writes.
- **Finishing a lesson** (audit F09, 2026-11-11): the done mark and the
  cleared place are written in one transaction (`Store::finish_place`,
  `store/places.rs`, on `put_lww_in`). A refusal keeps him on the last step
  with «ما عرفتش أسجّل إن الدرس خلص» and Retry; a refused place save
  mid-lesson says so too (`WriteFailed` in `learn/player.rs`).
- **Reads that fail** (audit F10): `ReviewBackend::reload` keeps the last
  good review log and verdicts when a read fails, and `Store::setting`
  reports a database error instead of reading it as "never set"
  (`store/health.rs`: `read_failed`/`read_error`); Today shows it with
  «اقرأ تاني».
- **Storage that didn't open** (audit F11): the in-memory session is said
  above every screen (`persistence::VolatileNotice` in `main.rs`) with what
  will be lost and a button to Settings › Data to export.
- **Sync ops that can't go up** (audit F12, F13, 2026-11-11): an op Batna
  Cloud refuses for good, or an outbox row this build can't read, moves to
  the local `sync_held` table (migration 7, never synced or exported) with
  its reason instead of vanishing (`store/held.rs`; the engine settles a
  push with `Store::settle_outbox`). Settings › Sync shows a «تغييرات ما
  اترفعتش» card (`cloud/held_ui.rs`) with «ابعتهم تاني» (back to the
  outbox) and «سيبهم» (asks first). Newest 1000 kept.
- **One answer, one item** (audit F15, 2026-11-11): a local answer newer
  than everything in the log updates only its item (`Scheduler::step`,
  `core/src/replay.rs`); anything out of order, a reload, sync, import or
  retraction still replays the whole log. Tests assert step = full replay
  after every answer (random grades/origins/gaps; and a debug check in every
  app test). Release timings (hosted Linux, `replay_timings`): 10k answers
  3.6 ms full vs 0.05 ms step; 50k 20 ms vs 0.25 ms; 100k 43 ms vs 0.6 ms.
- **Live content switches are bounded** (audit F17): old libraries are
  leaked (screens hold `&'static`), so a run switches packs live at most
  `MAX_LIVE_RELOADS` (2) times; a later pack opens at the next start.
- **Two connections, bounded waits** (audit F16): the UI's connection
  waits at most `UI_BUSY` (750 ms) for the sync engine's write lock, then
  the write fails into the screen's own saving/retry state; the sync
  connection waits up to `SYNC_BUSY` (5 s) (`store/contention.rs`). Before,
  SQLite's default failed a UI write at once while sync wrote. Every
  transaction begins IMMEDIATE (the connection's default behaviour), since
  a read that turns into a write mid-transaction fails without waiting;
  the export's snapshot stays a deferred read.
- **Export is one moment** (audit F14): `Store::export` reads every table
  in one read transaction, so a sync merge on the other connection can't
  tear it.
- **Not yet verified natively**: Android process death and OS kills, macOS
  relaunch, S24 Ultra lock/unlock, keyboard and Back with a running session.

### Study signals from Steps-app (2026-11-09)

Ported from Mohamed's `MohamedRef/Steps-app` (a Batna fork), credited in each file's header.

- **Confidence and error reason** (`batna_core::review::{Confidence, ErrorReason}`, `crates/app/src/metacog/`): an optional sure / unsure / guess tap before an answer shows (study and bank), and an optional reason chip after a miss (knowledge, misread, second-guessed, reasoning, timing). In study a reason chip also rates Again; in tutor mode it replaces the logged answer (retract, then log again at its first time). Both are optional fields on the append-only `ReviewEvent` and two nullable columns (store migration 5); events from older builds read without them, and an older build keeps the answer and drops the tags. The Worker stores payloads as JSON, so it needs no migration.
- **Mistakes log**: misses he was sure of come first and are marked «كنت متأكد»; chips count and filter by reason (`stats::mistake_reasons`, `mistakes_with_reason`).
- **Retest sure misses** (`stats::retest_due`): misses he was sure of, 2–7 days old and not answered right since, come back from the mistakes view (cards as a study session, MCQs as a tutor set).
- **Catch-up plan** (`crates/app/src/catchup/`): when far more reviews are due than he usually does in a day, Today offers a plan that spreads them over 2–7 days in minutes, most overdue first, optionally pausing new items. A synced setting; `study/planning.rs` cuts the session's due list to today's share.

### Info sheet (book page, abbreviations)

- **Floating card on tablet and Mac** (#773, Mohamed 2026-11-09): docked at the window's foot, a drag up made the «ص N» / abbreviations sheet a very tall strip. From 600dp wide and 480dp tall, `crate::sheet::Sheet` is a floating card (`.bsheet.float`, end of `assets/sheet.css`), placed by `assets/sheet.js` under the chip that opened it, else above, else on the side with more room (content scrolls), else centred; height capped at min(36rem, 76% of the window) until sized by hand.
- He moves it by its heading row and sizes it from any of its four edges or four corners (`.bsheet-rz`, from `sheet.rs` `RESIZE_EDGES`; a grip mark in the foot's end corner). The size is kept in localStorage `batna.sheet.float`; a double-click on an edge forgets it.
- Phones (and phone landscape under 480dp tall) keep the resizable bottom sheet. Crossing the breakpoint switches between the two.

### Notes, weak-fact marks, go to page (from Steps-app)

Ported from `MohamedRef/Steps-app` (its PR text credits each source path).
Batna's books number pages per course (Gastroenterology starts again at 1)
and `res` numbers slides per lecture, so everything page-shaped carries a
**book**: a course id (`hb`) or a slide lecture id (`res.L02`), the same key
as `pagesheet::Scope::book()` / `Scope::of_book()`.

- **Notes** (`batna_core::note`, `crate::notes`, `store/notes.rs`): synced LWW
  table `notes` (migration 6, in `sync::TABLES`), one row per anchor
  (`item:<id>`, `lesson:<lesson>#<step>`, `page:<book>:<n>`), JSON `Note`
  with the page it cited when saved. «＋ ملاحظة» chip on study items (text
  held back until the answer shows), MCQ explanations and lesson steps; the
  page sheet lists a page's notes (`PageNotes { book, page }`); the list
  «ملاحظاتك» is a lecture-stack page (`notes::PAGE`), sorted by the
  program's course order then page, filtered by course, searchable. Max 2000
  characters; empty text deletes. Notes on items go to Anki in a sixth
  field, `Notes` (both note types; «My note» last on the back), from the
  app's export and `batna-tools anki --own <backup>`; the first import after
  2026-11-11 asks Anki to update the note types (docs/ANKI.md).
- **Weak-fact marks** (`crate::weak`): computed from the review log (last
  answer Again = weak; a right answer clears), nothing new to sync. A lesson
  step whose page (in its own book) holds a weak item shows «راجعها تاني»;
  the page sheet lists the weak cards with «ذاكر … دلوقتي» and counts weak
  questions. Synced setting `weak_marks` (default on), Settings › Study.
- **Go to page** (`crate::gopage`): More / the rail «روح لصفحة» or Ctrl/⌘+G.
  Course chips (courses with lectures), a lecture picker for slide courses;
  starts on the course in view, else the last book used (device-only
  `gopage.book`). A page outside every lecture offers the nearest one. Quick
  search (`shortcuts/palette.rs`) turns «30» / «ص 30» into one hit per
  course book that has that page.

### Desktop (Mac) layout

- Full-window canvas, wider labelled sidebar, 1280×820 default window (#739).
- Resizable question/explanation panes (`desk::Panes`, keyboard-operable
  divider, reset, width saved per device) and **Focus Mode** for running Study
  and lessons, hiding the sidebar but keeping Back/End, progress and grades
  (#741).
- Today is task-first: from 1200px the task column takes 3/5 (#742).
- **⌘K / Ctrl+K palette** (#743, #744): offline, searches screens, lectures,
  lessons and calculators (Arabic-aware: the calculator search's `fold` for
  hamza, ة/ه, ى/ي, tashkeel and Arabic digits; text in `i18n/palette.rs`), opens Settings sections, and
  `questions <lecture>` starts lecture-filtered practice. Max 12 results; no
  per-question deep links.

### Phone layout (S24 Ultra, ~384dp)

- Top-bar search button (`shortcuts::MobileSearchButton`) opens the same ⌘K
  palette, results scroll above the keyboard (#746).
- 48dp touch targets: bottom tabs and More Close (#747), list sort/layout,
  section actions, calculator Reset and breakdowns (#748), bank scope and
  history (#750), medical table controls (#758). `mobile-touch-browser.cjs`
  measures them at 320/384px with normal and larger text.
- Running Study toolbar: progress on its own row, Undo and End on a 48dp row,
  grades above the safe area (#749); checked at 384×520 (keyboard open, #751).

### Performance baselines (hosted Linux CI, not phone frame times)

Ignored diagnostics run in main CI only (not on PRs) and print to the job summary; no timing gate.

| What | 100 / 1,000 / 3,000 reviews (p50) |
|---|---|
| Study grade → next card, TestDom (#752) | 2.4 ms p50, 65.6 ms p95 (16 cards) |
| `ReviewBackend::replay` (#761) | 0.22 / 1.39 / 4.03 ms |
| Full grade commit + replay (#761) | 0.35 / 1.51 / 4.57 ms |
| Checkpoint marker, `v1` full hash (#759) | 0.90 / 8.87 / 26.98 ms |
| Checkpoint marker, `v2` revision (#763) | 9 / 11 / 9 µs |

Next: profile real interaction latency on the S24 Ultra before optimizing
further; don't change FSRS or Undo speculatively.

### CI and build notes

- The Cloud Worker e2e uses a throwaway `wrangler.e2e.toml` with
  `worker-build --release --no-opt` (Binaryen downloads kept failing with
  HTTP 504, #760) and a pinned, cached `wasm-bindgen-cli` 0.2.129 built from
  crates.io (`WASM_BINDGEN_BIN`, #762). Production `wrangler.toml` and deploys
  are unchanged.
- Signed Mac preview without a release (#765): Actions › Mac › Run workflow ›
  main, then download the `Batna-Mac-preview-<sha>` artifact (30 days). It is
  a build of that commit, not a version or a feed. Unzip twice, move Batna.app
  to Applications, `xattr -dr com.apple.quarantine /Applications/Batna.app`.
  Fails rather than ship unsigned if `BATNA_MAC_CERT_*` is missing.
- Linux apt installs in CI and Coverage go through `scripts/ci-apt-install.sh`, and the
  Settings/Screens browser runs call it with no packages before `playwright install --with-deps`:
  it walks a mirror list (Azure, archive.ubuntu.com, kernel.org, MIT), skips
  names that don't resolve, rewrites every file in `/etc/apt` (including
  `apt-mirrors.txt`) to the chosen mirror and time-boxes each try (on
  2026-11-11 both ubuntu.com names failed DNS and jobs hung for an hour).

## 11. Current state (2026-11-09)

| Area | State |
|---|---|
| App | Feature-complete for alpha: Today (with a catch-up plan), study (confidence and miss reasons), bank, cases, oral, lessons (with «Aa» reading settings), 102 calculators, stats/planner, own cards + Share, own lesson highlights, own notes, weak-fact marks, go to page, Anki export, abbreviations, lab chips, themes (Paper, Night, Night shift, true black, Study, Study dim, three System themes), sync, password login, OTA app + content updates, web |
| Releases | `v2026.11.2` (2026-11-08, alpha; = main `d6214ce`, adds the تحشيشة reminders). `v2026.11.1` earlier that day (lesson focus layout, own highlights, brand B0). Last old-style: `batna-v1.0.0-alpha.3` (2026-11-06); the alpha.4 bump (#766) was never released. Previews every 4 h |
| Content | 9 courses, 141 lectures, 141 lesson files; `res` L01–L20 written; L02–L11 checked against the enriched lecture copies, and res.L02 against its video recording (2026-11-11). Approved by Mohamed: none yet (`content/approved.txt` holds no ids) |
| Desktop / phone UX | Mac batch #739–#745 and phone batch #746–#751, #758 merged (§ 10); native Mac and S24 Ultra QA is with Mohamed |
| Study resume | Same-process resume, Today Continue and durable checkpoint merged (§ 10); process death not yet tested on a device |
| Info sheet | Bottom sheet on phones; on tablet and Mac a floating card by the tapped chip, movable and resizable (§ 10 Info sheet) |
| Brand | B0 since 2026-11-08: petrol icon with the redrawn liver; the same mark beside the name in the top bar and sidebar (`brand.rs`) |
| Critic loop | Final round (pass 10) finished 2026-11-04; loop closed. Don't start another unless Mohamed asks |
| Open PRs / issues | None on 2026-11-09 |
| Coverage | 95.05% when last measured (2026-11-04); floor 94 |

### Open work and known gaps

| Area | What |
|---|---|
| Content | Courses with no material yet; `res` L21 is the revision session (practice sets, no new lecture); L01–L20 done; everything stays `unverified` until he approves in-app |
| Glossary / audit | Split remaining ICP/SBP stop-gap senses into `course:` entries; 2026-11-04 audit leftovers (ambiguous abbreviations, quote cuts) only if Mohamed asks |
| Mac | Mohamed (2026-11-07) kept being asked for the login Keychain password ("app.batna.cloud") after updates and wants it gone for good. Main builds keep the session in `cloud-session.json` (since #612), and the prompt was traced to local builds of an older checkout; confirm on his Mac after the next update. If it returns, the cause is each build being signed differently (Keychain ACL): fix with a stable signing identity or storage not tied to the build |
| UX backlog | Weak-topic insight on Today; configurable review shortcuts with conflict checks; a focused phone Study view (keep exit, progress, reveal, grades; don't collide with Mac Focus Mode); medical reading polish (mixed Arabic/English, zoomable figures, optional haptics); a warning before a new session replaces a live one |
| Device QA (Mohamed) | Native Mac: WKWebView shortcuts, pane dragging, window resizing, focus, IME. S24 Ultra: Back-key routing, update notifications, lock/unlock freeze fix (`vendor/`, tested on desktop sockets only), Study across background, lock and process death. Browser screenshots are SSR previews, not device checks |
| Low, deliberately not fixed | IndexedDB handle never closed per save; Argon2 on the web main thread (~0.5 s); a second tab on a borrowed computer wipes the first tab's data |
| Test debt | Merge the duplicate click-test helpers (`testdom.rs`, `test_dom.rs`, `tap_dom.rs`, `testing.rs`); more calculator tests against original-paper examples and cut-offs |
| Big files | 14 Rust files over 800 lines (`scripts/check-file-sizes.sh`); split each when a PR already changes it a lot (AGENTS.md § Small files) |

## 12. Resume message

Mohamed can paste this into any new chat or account to continue:

> Continue work on my apps Batna (`MohamedRef/Batna-app`) and Kabar
> (`MohamedRef/Kabar-app`). Before anything else, read `AGENTS.md` and
> `docs/HANDOFF.md` in each repo you touch and follow them. Reply to me in
> Egyptian Arabic. Keep `docs/HANDOFF.md` updated in every PR that changes
> behaviour, process or state.

## 13. Log

Newest first. One line per PR (or batch) that changes behaviour, process or
state; details live in the PR and in § 10.

- 2026-11-13: dm.L16 to the Hepatology bar: 4 → 5 lessons (lesson2 split → lesson5, Case 3 HHS), every case read with its reasons labelled «التفسير المقبول (مش في الكتاب)», three new figures, all pages 196–213 cited, words per page 71 → 255 on the `lessons --report` count; listed in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: neph.L01 to the Hepatology bar: 6 → 9 lessons (lesson1 split → lesson2 takes Henderson, lesson2 split → lesson7, lesson3 split → lesson8, lesson5 split → lesson9), reasons labelled «التفسير المقبول (مش في الكتاب)», 4 new figures, words per page 142 → 278 on the `lessons --report` count; glossary gains ENaC and NH3 (#827–#829).
- 2026-11-13: git.L07 to the Hepatology bar: 3 → 4 lessons (lesson1 split → lesson4; Table 2 moved to lesson3), reasons labelled «التفسير المقبول (مش في الكتاب)», Treitz figure, words per page 196 → 354; p233 (review questions) skipped (#827–#829).
- 2026-11-13: geri.L10 to the Hepatology bar: 4 → 7 lessons (lesson1 → lesson2 takes the classification, lesson2 split → lesson5, lesson3 split → lesson6, lesson4 split → lesson7), reasons labelled «التفسير المقبول (مش في الكتاب)», alveolar wall and nintedanib figures, p185 now taught, words per page 113 → 271 on the `lessons --report` count; in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: dm.L01 to the Hepatology bar: 6 → 9 lessons (lesson1 split → lesson7 classification, lesson4 → lesson8 A1C, lesson5 → lesson9 glycosuria/polyuria), reasons labelled «التفسير المقبول (مش في الكتاب)», three new drawings; words per page 154 → 317 on the `lessons --report` count; in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: hem.L02 to the Hepatology bar: 4 → 5 lessons (lesson3 split → lesson5: MCV table + aplastic anaemia, p15), onset step moved to lesson2, new RI and haemolysis reveals, MCV ruler figure; words per page 204 → 280; listed in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: geri.L06 to the Hepatology bar: 5 → 6 lessons (lesson5 split → lesson6), reasons labelled «التفسير المقبول (مش في الكتاب)», 3 new SVG figures, words per page 129 → 302 on the `lessons --report` count; in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: geri.L01 to the Hepatology bar: 6 → 7 lessons (lesson5 split → lesson7), reasons labelled «التفسير المقبول (مش في الكتاب)», systolic/diastolic SVG, PS (5) case on p24, words per page 144 → 243 on the `lessons --report` count; added to `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: web deploy fixed after Rust 1.100: `scripts/web/build.sh` finds the content pack by package id, since cargo now writes build output to `build/batna-app/<hash>/` (deploys had failed since #821).
- 2026-11-13: res.L01 lessons cover every slide that carries a card (4 → 6 lessons; course and definition split, priority criteria split at criterion 3); the other slides are listed in `content/lesson-skips.yaml` (part of #829).
- 2026-11-13: neph.L12 to the Hepatology bar: 4 → 8 lessons (lesson1 → lesson5, lesson2 → lesson6, lesson3 → lesson7, lesson4 → lesson8), reasons labelled «التفسير المقبول (مش في الكتاب)», new AKI-schema SVG, words per page 153 → 372 on the `lessons --report` count; added to `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: hem.L05 to the Hepatology bar: 4 → 7 lessons (lesson1 → lesson5, lesson2 → lesson6, lesson3 → lesson7), reasons labelled «التفسير المقبول (مش في الكتاب)», book haemarthrosis photo p75, three new SVGs, words per page 137 → 277 on the `lessons --report` count; added to `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: rheum.L06 to the Hepatology bar: 5 → 8 lessons (lesson1 split → lesson6, lesson3 → lesson7, lesson5 → lesson8), every list given its reason, book lymphoid-organ figure p83, words per page 110 → 247 on the `lessons --report` count; p83 and p103 now taught; in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: hem.L01 to the Hepatology bar: 3 → 5 lessons (lesson1 split → lesson4, lesson3 split → lesson5), reasons labelled «التفسير المقبول (مش في الكتاب)», book marrow figure p5, words per page 186 → 323 on the `lessons --report` count; first lecture in `content/lesson-skips.yaml` (#827–#829).
- 2026-11-13: `validate` checks lesson coverage against the book (`coverage.rs`, `content/lesson-skips.yaml`, `batna-tools lessons`); 204 lessons' `pages` narrowed to the pages their steps cite; LESSONS.md rule 8 rewritten for Mohamed's lesson-count rule (#833, part of #828/#829).
- 2026-11-13: lesson flip view renamed «تقليب» and counts steps, not «صفحة» (#832).
- 2026-11-13: a book page lists and opens the lesson steps that explain it; «اشرحهولي» opens at the card's page step; lecture page «بترتيب الكتاب» outline (#831).
- 2026-11-13: lesson rows show their book pages and book sections; the lesson header shows the page range (#830).
- 2026-11-13: 52 cut quotes completed from the book pages (sentence continues on the next page or in the list below); four trailing fragments where the book itself breaks off were dropped and logged in `medstudy/book-issues.md` (#804, #805, #807–#812).
- 2026-11-13: res.L15.113, res.L15.117, res.L17.001 and their lesson steps get `update:` notes (n vs n − 1, 152.16 unrounded, p-value meaning); slide answers unchanged (#802).
- 2026-11-13: stray root file from a mistyped command removed; `check-tracked-files.sh` now fails on tracked paths with shell characters or root names with spaces (#817).
- 2026-11-13: README content table (141 lectures, courses not written yet) and calculator count (102) corrected; `batna-tools` tests `readme_tests.rs` now fail when README's lecture or calculator numbers drift (#803, #818).
- 2026-11-13: `no-key` book questions whose key comes from outside the book now carry `update:` with a named reference (12 items); geri.L01.010/.011 tagged `no-key` (#815).
- 2026-11-13: catch-up plan and weak-fact lines count by `i18n::count` (#801); `grammar_tests.rs` covers `catchup::*` and `weak::*`.
- 2026-11-11: res.L07.121 covers the slide 72 table (Drug A/B × insulin, 523 patients) the lecturer showed without explaining; open L03–L11 questions (L11 quiz 92–94 keys, L09 video-only plagiarism slides) kept as they are, unverified.
- 2026-11-11: Settings and Screens browser runs point apt at a resolving mirror (`scripts/ci-apt-install.sh`) before `playwright install --with-deps`.
- 2026-11-11: journey test (launch → grade → restart → restore → two-device sync), native QA matrix `docs/NATIVE_QA.md`, weekly `cargo audit`, and the shared `docs/INFRA_PARITY.md` checklist (engineering audit F23, F24, § 9).
- 2026-11-11: a local answer updates only its own item instead of replaying the whole log (43 ms → 0.6 ms at 100k answers); live content switches are bounded to 2 a run; UI writes wait up to 750 ms for the sync lock instead of failing at once (engineering audit F15–F17, § 10 Study sessions).
- 2026-11-11: res.L18 (non-parametric tests, correlation, regression; 37 slides), res.L19 (introduction to EBM; 125 slides) and res.L20 (critical appraisal; 69 slides) from the enriched lecture PDFs (video frames + Arabic notes): slide text retyped from the images in `courses/res/text/L18–L20.txt`, Arabic notes kept beside them as `LNN-notes-ar.txt`; 45 + 115 + 100 items, 4 + 7 + 6 lessons; glossary gains ANOVA, MLogRA, CME, LBP, EBHC, GP, EMBASE, DARE, PIER, ARI, NNH, ES, PRISMA, STARD.
- 2026-11-11: res.L17 (hypothesis testing, 33 slides from the enriched lecture PDF): 52 items, 5 lessons; calculators `chi_square` (2×2) and `t_test` (one sample, paired, two groups) reproduce the slides' coffee X² and haemoglobin t examples.
- 2026-11-11: CI apt installs go through `scripts/ci-apt-install.sh` (first resolving mirror of Azure, archive.ubuntu.com, kernel.org, MIT; time-boxed) after ubuntu.com DNS failures hung CI for an hour.
- 2026-11-11: res.L16 (normal distribution curve and Z-score): 47 items, 4 lessons; the `z_score` calculator now uses the lecture's bands (ordinary within ±2, unusual beyond ±2, extreme beyond ±3) instead of ±1.96. Slide 34's 13-answer key has no questions in the deck, so it was left out.
- 2026-11-11: res.L03–L11 checked against the enriched lecture copies and the four revisions: 63 new items (tags `lecturer`, `revision`), lecturer's explanations in lessons, res.L08.020 second correct option fixed.
- 2026-11-11: res.L02 gap-fill from the lecture's video recording (same 70 slides plus the lecturer's spoken points and the 8 pages of Arabic notes): 16 items, lesson 7; calculators `disease_duration`, `death_rate`, `perinatal_mr`, `mmr` (`calc/frequency.rs`); new slide verdicts in `courses/res/verdicts/L02.tsv`.
- 2026-11-11: sync ops the hub refuses (or the device can't read) are held with their reason and shown in Settings › Sync with send-again / let-go; the export is one read snapshot (engineering audit F12–F14, § 10 Study sessions).
- 2026-11-11: the Batna Cloud deploy for #780 had failed (504), so the hub rejected `notes`; deploy re-run, ops on a table the hub doesn't know now wait in the outbox, and notes are queued again once.
- 2026-11-11: finishing a lesson is one transaction with a Retry; a failed read keeps the last good copy and says so on Today; an in-memory session is announced on every screen (engineering audit F09–F11, § 10 Study sessions).
- 2026-11-11: answers the store refuses outside the main Study session (lesson Try, Oral, question bank) are no longer lost silently: the item stays or the answer is kept with a retry (engineering audit F02, § 10 Study sessions).
- 2026-11-11: his notes on items ride in the Anki deck (`Notes` field on Batna Basic and Batna Cloze; docs/ANKI.md), from Steps-app.
- 2026-11-11: res.L15 (introduction to biostatistics: variables, tables, graphs, mean/median/mode, range and SD): 118 items, 7 lessons. Source is the video-frame PDF (142 slides + 7 pages of Arabic notes); slide text typed from the frames into `courses/res/text/L15.txt`, generators in `courses/res/gen-L15/`.
- 2026-11-11: res.L12 (bias and confounders, 61 image-only slides from the lecture video, hand-transcribed in `courses/res/text/L12.txt`): 83 items with the 7-question quiz keyed from the lecturer's notes, 6 lessons with 7 drawn figures; glossary gains STD.
- 2026-11-11: res.L14 (screening and diagnostic accuracy, 31 slides) written: 65 items, 5 lessons; calculators `predictive_values` (slide 25 formula) and `post_test_probability` (likelihood ratios; Deeks 2004, Fagan 1975, Jaeschke 1994); `diagnostic_accuracy` guides now cite the L14 slides; glossary gains AUC, ROC, TP/FP/FN/TN, SpPin, SnNout.
- 2026-11-11: res.L13 (writing a scientific paper + Vancouver referencing, two decks merged as 120 slides): 101 items, 9 lessons; slide text hand-transcribed from the video-frame slides.
- 2026-11-09: lesson reading settings («Aa»: text size, line spacing, line width per device), ported from Steps-app.
- 2026-11-09: notes (synced `notes` table), weak-fact marks and go to page (Ctrl/⌘+G, per-course books) ported from Steps-app (§ 10 Notes, weak-fact marks, go to page).
- 2026-11-09: from Steps-app: confidence and error reason on answers (migration 5), sure misses first in the mistakes log with a 2–7 day retest, and a catch-up plan for a review backlog (§ 10 Study signals).
- 2026-11-09: a release with no new `changes/` lines says «تصليحات صغيرة وتنضيف.» on its page instead of repeating the previous release's lines (`scripts/release-notes.sh` reads the feed's newest history entry only when it is this version's).
- 2026-11-09: the GitHub release page is written in Egyptian Arabic only (`scripts/release-notes.sh`); a `changes/` line without Arabic text is left out (Mohamed).
- 2026-11-09: the book-page / abbreviations sheet floats as a movable, resizable card by the tapped chip on tablet and Mac; phones keep the bottom sheet (#773).
- 2026-11-09: review of the Mac/phone/Study batch (#739–#765): quick-search text moved to `i18n/palette.rs`, the search folds Arabic spelling like the calculator search, the Study perf diagnostics run on main only (PRs stay fast).
- 2026-11-09: handoff reorganised (per-PR journal folded into § 10, open work as a table) and checked against main; README brought up to date.
- 2026-11-08: `v2026.11.2` released: repeated topics remind their تحشيشة (#771, `remind:` on lesson steps, 791 reminders, «من درس …» link; `pack::SCHEMA` 3).
- 2026-11-08: `v2026.11.1` released, the first calendar version.
- 2026-11-08: rebrand B0 (#769): liver redrawn on petrol `#245f7c` with a green bile drop (`assets/src/icon/*.svg`); the same mark heads the top bar, rail and sidebar (`brand.rs`, a test keeps it equal to the icon).
- 2026-11-08: own highlights in lessons (#770): select → colour, fill/underline, pill/rect; one synced setting per highlight.
- 2026-11-08: lesson focus layout (#767): one centred reading column ≥840px, text by importance, key-term pill, Study / Study dim / System study themes.
- 2026-11-08: calendar versions `2026.11.N` and one build counter for previews and releases (#768, Mohamed's ask); `scripts/version.sh`, versionCode 20000000 + build, tags `v<version>`, no bump PR. The `1.0.0-alpha.4` bump (#766) merged just before and was never released.
- 2026-11-08: signed Mac preview as an Actions artifact (#765).
- 2026-11-08: Study checkpoint marker O(1) via review-event triggers, migration 4 (#763, #764); perf diagnostics for the marker, replay and grade path (#759, #761).
- 2026-11-08: Cloud e2e CI made reliable: `--no-opt` test build (#760), pinned `wasm-bindgen-cli` (#762).
- 2026-11-08: durable Study checkpoint (#756) with crash-safety tests (#757); Today Continue resumes a live session (#755); same-process remount and foreground scroll tests (#754).
- 2026-11-08: answer clock ignores background time (#753); grade-to-next-card CI baseline (#752).
- 2026-11-08: phone batch: top-bar search opens the ⌘K palette (#746); 48dp targets for navigation, lists, calculators, bank and tables (#747, #748, #750, #758); Study toolbar rows and 384×520 check (#749, #751).
- 2026-11-07/08: Mac batch: full-window layout (#739), Anki-style keys (#740), resizable panes + Focus Mode (#741), task-first Today (#742), ⌘K palette and its destinations (#743, #744), keyboard focus continuity (#745).
- 2026-11-08: the Keychain prompt ("app.batna.cloud") was traced to local builds of a checkout older than #612; still reported after updates, so it stays open work (§ 11).
- 2026-11-06: `1.0.0-alpha.3` released on Mohamed's ask, so the Mac app (alpha feed only) gets the lock/unlock and update-check fixes.
- 2026-11-06: Android screen froze after lock/unlock: Dioxus's edit socket patched in `vendor/` (edits resent until acknowledged); update check shows the app result before the content pack downloads, requests time out after 20 s of no progress.
- 2026-11-06: res.L11 lesson 4 sampling tree labels wrapped (they failed the nightly lesson browser check).
- 2026-11-05: res.L11 (sampling and sample size): 82 items, 6 lessons; two sample-size calculators reproduce the slides' Epi Info numbers.
- 2026-11-05: res.L10 (data collection, questionnaires, searching the literature): 107 items, 8 lessons.
- 2026-11-05: res.L09 (budget, ethics, plagiarism): 113 items, 6 lessons; glossary gains REC, CIOMS, WAME, WMA, MLA and `res` senses of PI, CV, APA.
- 2026-11-05: Mac nightly green again (tests expect the Mac's own answers; menu test skipped on macOS).
- 2026-11-05: big files split (i18n per feature; core `lesson`, `calc`, `labs`, `session`); AGENTS.md § Small files; `check-file-sizes.sh` warns in CI.
- 2026-11-05: sources and the whole project folder + memory notes backed up to the private repo `MohamedRef/Batna-sources` (§ 8).
- 2026-11-05: handoff docs added (`AGENTS.md`, this file, course briefs, content scripts).
- 2026-11-04: `1.0.0-alpha.2` released (#726); final critic round closed.
