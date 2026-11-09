# Batna handoff: everything a new agent needs

This file lets any new agent, account or chat continue Batna from where the
last one stopped, with no other history. Read it top to bottom once, then
`CLAUDE.md` (working conventions) and the doc for the area you touch.

**Keep it current.** Every PR that changes behaviour, process, commands,
secrets, deploys or the state of the work updates this file in the same PR
(usually § Current state and § Log). See `AGENTS.md`.

Last full refresh: 2026-11-05. Mac desktop continuation updated 2026-11-08, base `main` at `57775359` (1.0.0-alpha.3, no new release in this batch).

---

## Floating book-page and abbreviation card (2026-11-09)

- Mohamed (2026-11-09): on the Mac and the tablet the sheet a «ص N» chip or the abbreviations chip opens sat at the window's foot, and dragged up it became a very tall strip. From 600dp wide and 480dp tall, `crate::sheet::Sheet` is now a floating card (`.bsheet.float`, `assets/sheet.css`): every corner rounded, 27rem wide (30rem from 1100px), placed by `assets/sheet.js` right under the chip that opened it, else above it, else on the side with more room (content scrolls inside), else centred. Its height is capped at min(36rem, 76% of the window). Dragging its heading row moves it; Mohamed then asked (same day) to size it freely: its four edges and four corners (`.bsheet-rz`, drawn by `sheet.rs`, a small grip mark in the foot's end corner) resize it, the size is kept in localStorage `batna.sheet.float` for the next card, and a double-click on an edge forgets it. Esc, Back, ✕ and a tap outside still close it.
- Phones (and phone landscape, under 480dp tall) keep the resizable bottom sheet unchanged. Crossing the breakpoint (window resize) switches between the two.

## Repeated topics remind their تحشيشة (2026-11-08)

- Mohamed (2026-11-08, ACLF card in Liver Cirrhosis): a topic that comes back must bring its تحشيشة back, or at least remind him of it. Lesson steps now take `remind: [<item id> | <lesson id>:<WORD>]`; the player (`crates/app/src/learn/remind.rs`) shows each one under the step in a dotted box, with «📖 من درس …» opening the lesson that taught it when that is another lesson.
- Content: 791 reminders on 689 steps in 126 lesson files. Found by script (a step whose text covers at least 80% of an enumerate's points, or 70% of another lesson's mnemonic letters), every off-topic match read and dropped; plus 13 cross-lecture twin lists (e.g. KDIGO in hb.L09/neph.L05, anti-TNF in rheum L01/L16) and SHIBS on the ACLF step by hand. Question, reveal and hidden-cell steps never get one.
- `pack::SCHEMA` is 3 (steps gained `remind`): an older app keeps its own content until it updates, so this ships with an APK/Mac/web build, not as a content-only update.
- Checks: `batna_core::lesson::remind_problems` (shape, step kind, duplicates) and `batna-tools validate` (`lessons::remind_targets`: each entry is an enumerate with a hook, or a lesson's mnemonic word). Rule: docs/CONTENT.md and docs/LESSONS.md rule 13. New lessons should add `remind` where a list repeats.

## Own highlights in lessons (2026-11-08)

- Mohamed asked to highlight or underline lesson text himself, in colours he picks, shaped as a pill or a rectangle. Select words in the open step → a bar (`crates/app/src/learn/highlight.js`) offers 5 colours (yellow `#ffd43b`, green, blue, pink, orange), fill/underline and pill/rect; tapping a highlight edits or removes it. The chosen style/shape is remembered per device (localStorage).
- Storage: one synced setting per highlight, `hl.<lesson>.<id>` = `batna_core::highlight::Saved` (a highlight or `{"gone":true}`), so devices never overwrite each other; no new sync table or migration. Anchoring by quote + 32 chars of context + position (`batna_core::highlight::locate`, mirrored in the script).
- Drawing never edits Dioxus nodes: shapes go in a `.uhl-layer` under the text of the nearest box with a background (`[data-uhl-host]`); fill text turns dark through `::highlight(uhl-ink)` (CSS Custom Highlight API), else the fill is lighter. Swipe ignores drags while text is selected; on touch the bar sits under the selection (the system copy menu is above).
- Tests: `learn::highlight` (messages, load/tombstones), `batna_core::highlight` (locate, cleaning, serde) and `crates/app/tests/highlight-browser.cjs` (select → colour → edit → remove → reload) inside the screens-browser run.

## Lesson focus layout and Study themes (2026-11-08)

- Mohamed asked (from a Mac screenshot: small context card top-right, step card offset left, nav far at the bottom) for a focused lesson screen based on where the eye rests, plus a calm study theme and text colours by importance. This replaces PR #739's ≥1200px two-pane lesson grid.
- From 840px up (`crates/app/src/learn/learn.css`) the player is one centred column (40rem, 44rem with 1.125rem text from 1200px, ≈ 60–75 characters a line). The column's top sits at the same height on every step; the lesson name is a quiet muted header above the card; Back/Next follow the card (still sticky while a long step scrolls). Phones keep the bottom nav for thumb reach. Pages view also gets the centred column.
- Text hierarchy: `==key==` marks are a pill in new palette tokens `--mark`/`--mark-ink`, a hue opposite each theme (true yellow `#ffd43b` on Paper/Night/Black/Night shift, picked by Mohamed from five shades; teal on Study themes); Mohamed rejected a gold highlighter band and asked the pill to stand out from its card (test `the_key_term_pill_stands_out`); headings and `**terms**` use full `--text`; body text is `--text` softened 16% toward the card; meta stays muted.
- New themes in `batna_core::theme`: `study` (warm off-white, charcoal text), `study-dim` (warm graphite, parchment text) and `system-study`; AA-tested plus `study_themes_are_calm` (no pure white/black, text contrast 7–15:1). Research basis and sources: `docs/LESSONS.md` § Reading layout.
- `screens-browser.cjs` now asserts the centred column at 840/1280/1600px instead of side-by-side panes.

## Study checkpoint constant-time marker (2026-11-08, merged PR #763)

- PR #762 merged as `83710ccd` after green PR Rust/content, Android and full Cloud Worker multi-device E2E. The pinned `wasm-bindgen-cli` setup avoids GitHub Releases HTTP 504s in CI. A post-merge main run should be verified separately.
- PR #763 squash-merged as `91a5eb0e` after green Rust/content, Clippy, Android arm64 and legacy recovery tests. `perf/study-review-marker-constant-time-20261108` replaces repeated full-log JSON hashing on **checkpoint saves** with an O(1) SQLite-local `v2` marker stored in `meta`. Migration 4 adds review-events INSERT/UPDATE/DELETE triggers that rotate a random 128-bit token in the same committed transaction, covering local grades, Undo, remote merges, imports and direct SQL mutations. Rolled-back writes also roll back the token. A random revision prevents equally sized divergent DB snapshots from accidentally sharing a counter.
- Old `v1` checkpoints are verified against the original complete review-log hash on first reopen. They can recover safely and the next save upgrades to `v2`; corrupt/stale checkpoints are still rejected. No cloud protocol, review event, grading logic or sync schema change.
- Regression tests cover transaction rollback, insert/update/delete, double Undo, two divergent DB copies, snapshot reopen, borrowed-device `forget_study` with subsequent grading, and legacy Study recovery. The borrowed-device wipe preserves and rotates the revision row instead of deleting it. The existing crash-between-grade-and-checkpoint test must remain green, alongside Rust/Clippy, Android and content checks. The benchmark confirmed O(1) retrieval: Linux p50 at 100/1,000/3,000 reviews = **9 / 11 / 9 µs** versus before = **903 / 8,869 / 26,977 µs**, a ~3,000× improvement at 3,000 events. The result measures marker retrieval only, not a full app frame or cold restart. Next: investigate measured S24 Ultra interaction latency and any remaining UI rendering/sync waits, without changing FSRS or Undo behavior speculatively. No release.

## Cloud main-CI wasm-bindgen dependency (2026-11-08, merged PR #762)

- PR #761 merged green as `5b604038`. Hosted Linux p50: FSRS replay = 0.218 / 1.386 / 4.028 ms; full commit + replay = 0.349 / 1.511 / 4.568 ms; checkpoint marker = 0.903 / 8.869 / 26.977 ms at 100 / 1,000 / 3,000 synthetic events. Marker dominates at larger histories. These are not S24 Ultra UI latencies.
- PR #760's full PR Cloud E2E passed, but the following **main** run failed: worker-build could not download `wasm-bindgen` 0.2.129 from GitHub Releases (HTTP 504, five retries). This is distinct from the earlier `wasm-opt` download failure; the Worker never started, so no sync assertion failed.
- `fix/cloud-ci-pinned-bindgen-20261108` pins `wasm-bindgen-cli` 0.2.129, installs from crates.io source and caches the resulting binary in main CI and the path-scoped PR Cloud E2E workflow; exports `WASM_BINDGEN_BIN` to worker-build. The existing test-only `--no-opt` remains, the app and production Worker deployment configuration are unchanged.
- Require a successful **full** fake-Google + device-sync PR E2E, Rust/content and Android checks, then merge. Recheck main Cloud CI because its cache and workflow context differ. Native performance and release remain deferred. Next performance priority: make checkpoint marker cheaper without weakening crash/Undo/sync integrity.

## FSRS replay latency diagnostic (2026-11-08, merged PR #761)

- Cloud E2E regression PR #760 **merged as `ba319f49`**, with the complete local Worker fake-Google/device sync PR test and Rust/Android checks passing. Production Worker build remains optimized; test-only Wrangler config avoids external Binaryen downloads. Verify the next `main` Cloud E2E run separately.
- New `test/study-review-replay-baseline-20261108` adds an ignored benchmark calling the real `ReviewBackend::replay()` seven times over independent synthetic SQLite histories of 100/1,000/3,000 committed events across 64 items, mixed grades. It also times a full `StudyStore::record_logged` grade, including SQLite + in-memory replay but excluding checkpoint marker and WebView painting.
- Report p50/p95/max in the existing CI job summary. No fixed timing gate and no application state, scheduling, saving or Undo behavior changes. Treat hosted Linux data as directional only; do not infer real phone latency until S24 Ultra profiling. After green CI and merge, compare marker vs FSRS replay cost before a separate optimization PR.

## Cloud Worker CI: Binaryen download failure (2026-11-08, merged PR #760)

- Main CI on #756-#759 reports **Cloud Worker E2E failed**, although Rust, Android and web compile jobs passed. Logs identify five HTTP 504 responses while wasm-pack downloaded Binaryen version 132 for `wasm-opt`; consequently the local Worker never started. This is a build-time dependency failure, **not** evidence of a failed sync assertion.
- `crates/server/tests/e2e.mjs` writes a disposable `wrangler.e2e.toml` next to `wrangler.toml` and changes only its build command to `worker-build --release --no-opt`. The test still performs a Rust release build, launches the real local Worker, runs fake Google + WebSocket/device flows and the Rust app sync E2E. The normal production `wrangler.toml`, deploy and optimization are unchanged. The ephemeral test file is cleaned on exit and ignored by git.
- A path-scoped PR workflow runs these integration tests on E2E launcher changes so a successful compile alone cannot falsely count as verified. The end-to-end job and standard PR CI passed; PR #760 squash-merged as `ba319f49`. No release.
- PR #759 previously merged as `3e7b2458`: hosted Linux marker p50 = 1.118 ms at 100 reviews, 10.898 ms at 1,000 and 32.540 ms at 3,000. Writes of its local meta were 0.019/0.032/0.055 ms. Measure `ReviewBackend::replay` with similar large histories **after fixing the CI**, not concurrently modifying grading or FSRS.

## Study checkpoint performance baseline (2026-11-08, merged PR #759)

- PR #758 merged green as `b8775089` (Android, Rust/content, Chromium); table controls are >=48dp on compact phones. The previous touch-target PR's pending label below is historical.
- Follow-up `test/study-checkpoint-marker-baseline-20261108` measures the existing `Store::study_review_marker` read/hash separately from the local SQLite checkpoint-meta write at 100, 1000, and 3000 committed review events. The ignored diagnostic runs in PR CI and publishes p50/p95/max; it changes no persistence, syncing, review grading or UX.
- Hosted Linux timings are **not** S24 Ultra UI frame times. Do not optimize without considering the full scan's growth and the fail-closed semantics after a crash. Next, review results, then use a separate feature PR for any optimization, preserving cold-process recovery tests and event IDs. Native Android profiling remains deferred. No release.

## Mobile medical table touch targets (2026-11-08, PR #758 merged)

- PR #756 merged as `6d3d885a`, and follow-up SQLite crash/restart tests PR #757 merged as `d05b23a5`; native process-kill QA remains deferred.
- PR #758 widens medical table typing, mark (yes/no) and correction targets to the existing 48dp `--target` while leaving Reveal, scroll, pinned labels and grading unchanged.
- The production-CSS Chromium regression covers all five table controls at 320/384px with normal/enlarged text. Screens-browser CI passed alongside Rust/content and Android; PR #758 merged as `b8775089`. No alpha release.

## Durable Study checkpoint (2026-11-08, PR #756 merged)

- PR #756 merged as `6d3d885a` after green Rust/Android CI. PR #755 had merged as `81cc01b1`. Today already offers Continue for a live, same-process Study queue.
- Follow-up branch `feat/durable-study-checkpoint-20261108`: a **device-local versioned** `SessionCheckpoint` preserves queue order, progress, Again retries, outcomes, and the exact review IDs needed for Undo. A new run reads it before rendering Today, without replaying or recording any answers.
- A deterministic marker covers the committed, non-tombstoned SQLite review log. If the process dies **after** a review commits but **before** its matching checkpoint, a marker mismatch refuses recovery rather than duplicating FSRS grades. Invalid schema, unknown or hidden cards, missing or altered review events similarly fail closed. A stale/invalid checkpoint is cleared and the app starts normally.
- Save after starting, grading, Undo and appending items; clear after End or finishing. Uses `meta.local.study.session.v1`, not synced settings or a new sync table. A borrowed device still clears its local meta when its session ends.
- Automated coverage includes core version/grade/undo/retry validation plus review marker invalidation and SQLite snapshot reopen. Follow-up PR adds application-level SQLite recovery tests: a real persisted Again grade and Undo event ID must restore without replay, and a crash after review commit but before snapshot update must reject and clear the stale checkpoint. **Not yet native-tested:** Android process death, OS background kills and macOS relaunch. The snapshot is not an automatic recovery guarantee when storage is unavailable; a failed snapshot write never blocks an already-saved review.
- CI must pass on the latest PR head before merging; no new Alpha, language change, or release.

## Manual Mac preview build, 2026-11-08

- The Mac workflow (`.github/workflows/mac.yml`) still compiles/tests nightly without publishing an app.
- On `workflow_dispatch` on `main` **or once on the `main` push that changes `mac.yml`**, it additionally signs the newly built `Batna.app` with the existing `Batna Code Signing` identity and uploads `Batna-Mac-preview.zip` as a GitHub Actions artifact (30 days). This is a new build from the selected commit, **not** a new version/tag or an update feed. It does not publish an Alpha release.
- A new Mac preview is triggered automatically when this packaging workflow merges. To rebuild later: Actions > Mac > Run workflow > main; open the successful run and download the `Batna-Mac-preview-<sha>` artifact. Unzip both the Actions artifact and the enclosed app ZIP, move Batna.app to Applications and run `xattr -dr com.apple.quarantine /Applications/Batna.app` when required for the self-signed app.
- Build signing uses the already configured `BATNA_MAC_CERT_P12` and `BATNA_MAC_CERT_PASSWORD` repository secrets; a manual run fails rather than silently shipping an unsigned preview if either is missing.
- Mac native visual/keyboard testing remains for Mohamed. The new workflow changes packaging only. No actual downloadable artifact exists until the automatic push or manual workflow runs successfully.

## Mac desktop workflow plan, refreshed 2026-11-08 (PR #744 merged; mobile UX is next)

User directive: **code work first**. Native Mac interaction testing is deferred to the user; don't block feature implementation on manual macOS verification. Automated CI, layout regression checks, PR review, and conflict checks remain mandatory before merging. Leave interface-language work out of this batch. No alpha unless explicitly requested. Keep the PR-first, sequential merge discipline.

Status on `main`: PR #739 (full-window desktop UI), PR #740 (Anki-like keyboard navigation), and PR #741 (resizable Mac panes + Focus Mode), PR #742 (Today task-first layout), PR #743 (offline Cmd+K command palette), PR #744 (palette destinations), and PR #745 (keyboard focus continuity) merged. Do not rebuild their already-working pieces.

Priority-ordered product backlog (not all implemented):
1. **Merged PR #741:** resizable question/explanation panes on desktop pointer devices with keyboard-operable divider, a reset to default, device-local persistent width and safe small-window fallback; intentional Mac focus mode for running Study and Lesson, hiding the sidebar while keeping Back/end, lesson progress, and grade controls. Use existing layout CSS, `desk::Panes`, shared `Ui::update_prefs`, and the single app scroll container. Preserve study state when toggling.
2. **Merged PR #742:** desktop task-first Today **layout** with the existing hero, due/new metrics, resume lesson, and library. The 1200px+ task column now gets 3/5 of the space instead of 2/5, the long task column follows the main app scroll, and the library stays a scrollable side reference. Do not rebuild its already-working data or review actions. New weak-topic insight stays a later separate item.
3. **Merged PR #743:** `⌘K` / `Ctrl+K` offline command palette, using the real content Library for **screen, lecture and lesson navigation**. Search from existing lecture/lesson ids and titles, rank exact/prefix matches, max 12 results, keyboard arrows/Enter/Escape, Tab focus trap, modal isolation and no network. Do not claim question/calc indexing yet; those should be linked to their existing product search interfaces only when concrete navigation/filters are supported.
4. **Merged PR #744:** extend `⌘K` using the calculator picker's existing Arabic-aware search and direct calculator opening, exact Settings section routing, and lecture-filtered question practice only for explicit `questions <lecture>` searches. Fix the Calculators tool when Cases is already open. Preserve recent calculators and study state. **Next:** add weak-topic insight to Today, configurable review shortcuts with conflict checks, and better review actions after specifying behavior.
5. **Merged PR #745:** restore focus after keyboard/screen-reader reveal, grading, Undo and lesson step navigation; preserve typing, mouse and modal focus. Chromium behavioral regression, Rust CI and Android checks succeeded before merge.
6. **Later UX:** reduced UI jank on item transitions, readable very-wide typography, saved panel preferences and narrow-window behavior, consistent empty/loading states, shortcut discoverability/tooltips, accessible contrast and reduced motion.
7. **Deferred manual checks by user:** actual WKWebView shortcuts, splitter dragging with mouse, both right-to-left panes and Mac window resizing, focus escape/restore and IME. Browser previews are **not** native Mac screenshots.

Keep these labels honest as PRs merge, refresh this section and a `changes/*.yaml` fragment in each PR. Prefer a single PR for related pane/focus work, then move to the next feature only after CI passes and merge succeeds.

## Merged PR #745: keyboard focus continuity (2026-11-08)

- Base: `main` commit `367d9902`. Merged as `57775359`; CI and browser focus regression passed.
- Real keyboard interactions: with the active Study item removed during grading, the browser can drop focus to the document, causing Tab order/assistive-tech context loss. Similarly, a lesson changes the `hidden` Stage rather than replacing the whole player. Scope focus restoration to those transitions.
- Only restore on keyboard / screen-reader (zero-detail click), not on pointer clicks; cancel on pointer activity, modal, contenteditable, or active input. Keep updates in `crates/app/src/shortcuts/` with focused browser regression; no manual macOS QA is required to progress code.
- PR #744 merged after green CI: the palette opens real calculators, Settings sections and lecture-filtered practice, not individual question deep links.

## Same-process Study resume regression (2026-11-08, PR in progress)

- New Rust TestDom regression mounts a two-card running Study session, reveals/rates its first card, unmounts **only the screen**, and remounts it in the same app context. It asserts the second card and 2/2 progress remain active, recorded grades are not duplicated and Undo is available. **This does not claim process-death recovery**.
- Production-JS Chromium regression checks that a retained Study scroll container remains at the same offset and active card after foreground visibility/pageshow events. It does not simulate actual Android lifecycle or restore a killed WebView.
- The existing focus continuity and background-answer timer implementations remain unchanged; tests are intended to prevent regressing them.
- Native macOS and S24 Ultra manual verification remains with the user. Next: isolate a real Android-grade item scroll jump if observed, then investigate cold-process durable snapshots separately with versioned schema and sync/backward-compatibility rules. No alpha or language changes.
- PR-first, merge only after branch CI and relevant browser tests are green.

## Today: resume existing Study session (2026-11-08, PR pending)

- PR #754 merged (`a5b40421`) after successful Rust, Android arm64 and browser checks; same-process Study remount and foreground scroll are regression-covered. No killed-process restoration is claimed.
- Follow-up branch `feat/today-resume-study-20261108`: expose the active Study session's reactive position and length; Today displays a clear Continue action when a running unfinished session exists. Clicking returns to the same in-memory queue **without** pushing `START`, re-planning, re-grading, or replacing active Undo history.
- Keep ordinary daily-plan and new-session actions intact, and hide Continue after summary/end. Ensure phone hit targets and Mac layout remain stable; add Rust behavior checks and a change fragment. Run CI and merge sequentially.
- Future work: consider an explicit new-session warning when a live session is being replaced; versioned process-death snapshots require separate design, invalid-id recovery, event-id/Undo integrity, and consistency with synced review events. No alpha or language changes.

## Study event-to-render and background timing, 2026-11-08 (PRs #752–#753 merged)

- An ignored diagnostic test in `crates/app/src/study/flow.rs` measures 16 real card Reveal → Good → next-card actions through the in-memory Dioxus TestDom and review backend. It prints mount time plus grade-action p50/p95/max in microseconds.
- The normal PR CI check runs this focused diagnostic *after* `cargo test --workspace`, and writes results to the job summary. No fixed timing failure threshold: GitHub-hosted Linux CPUs are noisy and this omits Android WebView layout/paint, Java/Kotlin lifecycle, IO variance and GPU timing.
- Compare multiple runs on similar runners to locate a regression, then profile on S24 Ultra before optimizing; do not treat CI measurements as phone latency. Avoid changing review persistence or study scheduling without correctness coverage.
- Existing Study `SESSION`/`PHASE` are global in-memory signals, so same-process tab switching retains the current card. Cold process death is **not** proven durable. Lock/unlock, scroll offset, Android Back and elapsed-time-after-background are separate native validation items.
- PR #752 merged green. First CI hosted Linux baseline: 16 cards, setup 167948µs, grade p50 2380µs, p95/max 65600µs. These are TestDom event/mutation timings, **not** Android paint and not a pass/fail speed threshold.
- Merged PR #753: the app-lifetime `visibilitychange`/`pageshow` listener resets the active Study answer clock on return from background, without navigation, grading, scroll, or session replacement. Isolated production-JS tests passed on CI (hidden/visible, pageshow, repeat installation). Native Android lifecycle event delivery remains to be verified on S24 Ultra.
- A retained WebView should keep the live Study `SESSION` and current item. A killed process **does not** have a durable session snapshot; do not claim crash/process-death resume. Next: add same-process remount/scroll regressions, verify native S24 Ultra lock/unlock + keyboard + Android Back, then profile `ReviewBackend::replay` against a larger history before optimizing. No release in this batch.
- No new alpha or language changes.

## Mobile Study progress continuity (2026-11-08, PRs #749–#751 merged)

- PRs #748 and #750 merged with successful Rust/Android/Chromium workflows; their compact list, calculator and bank touch-target changes are preserved.
- Merged PR #749 (`fix/mobile-study-toolbar-20261108`): at 320/384dp, the running Study toolbar keeps the progress meter and counter on a dedicated row, with Undo and End on a separate 48dp row. The grade row respects Android's bottom safe-area while preserving the existing four-button order and running session.
- Production SSR/Chromium checks assert meter readability and non-overlap, including larger text. Native S24 Ultra gesture behavior remains for owner QA.
- Merged PR #751: the browser regression at 384×520 verifies End reachability, grade target geometry, and no sideways overflow. It is a static SSR layout test, not a hydrated WebView or native keyboard/lock test. **Next:** establish a baseline for study transition/render latency and preserve scroll and Android Back before changing runtime code.
- No alpha release or English UI changes. Native S24 Ultra lock/keyboard gesture validation remains deferred.

## Mobile study UX continuation, 2026-11-08 (mobile quick search, touch, and short-height QA merged)

The top-bar search is a reusable `shortcuts::MobileSearchButton` opening
`⌘K`'s offline palette, not a second search engine. Dismissal restores focus
to the mobile trigger if still mounted; selecting a destination does not
return to the old screen. Browser fixtures now render the real compact
top-bar actions; Chromium regression checks 320dp/384dp search hit target
(48dp) and a 520px viewport simulating the on-screen keyboard. Retain
safe-area CSS, 48dp controls, Android Back/IME behavior and no alpha release.
Manual device validation is deferred, automated browser and CI are required.



Mohamed directive: **finish and merge the Mac work first**, then improve the
Android phone experience in small, sequential PRs. Primary device: Galaxy S24
Ultra (~384dp). Preserve the existing phone bottom bar, five-row More sheet,
Android Back controller, keyboard safety, 48dp Study targets, and safe-area
insets. Do not duplicate or redesign working features. No alpha without an
explicit request, and English UI stays deferred.

Priority order and scope (PRs #746–#751 merged; next performance/resume review):
1. **Merged PR #746, P0 mobile quick search:** an always-visible, reachable search affordance
   in the compact top bar that opens the existing Cmd/Ctrl+K palette; reuses
   its Library, calculator, Settings-section and lecture-practice destinations
   without another index or router. Support hardware keyboards as before;
   ensure modal close/back, text focus, on-screen keyboard, and unobstructed
   result selection at ~384dp and large font sizes.
2. **P0 one-handed tap audit (#747, #748 and #750 merged):** measure all interactive controls on phone-sized
   pages, prioritize Study / lesson progression, bottom navigation, question
   bank and calculator picker; fix any undersized or crowded controls, respect
   system gesture/nav insets. Study already uses 48dp controls and sticky
   grading, so improve verified gaps only.
3. **P1 focused mobile study view:** keep an explicit exit/Back, progress,
   reveal and grade access while reducing chrome; preserve active session when
   enabling/disabling the view (do not collide with Mac Focus Mode).
4. **P1 speed and resume:** profile question transitions and large lecture
   rendering before optimizing; verify state resumes at the same item and
   scroll position after backgrounding/lock without breaking Android Back.
5. **P2 medical reading:** ensure readable English/Arabic mixing, scrollable
   medical tables, appropriately zoomable real figures, and optional low-key
   haptics. No new generated figures and no unnecessary permissions.

Implementation rules: PR-first and sequential; update this handoff and add a
`changes/*.yaml` fragment each PR. Run the fast PR CI and relevant Android
regression / browser viewport tests; native S24 Ultra usability validation is
deferred to Mohamed. **Do not mark any mobile item delivered until merged.**

---

## Mac desktop UX follow-up (2026-11-07, PR #739)

- Desktop references reviewed: RemNote for spaced-repetition home, AMBOSS for clear clinical question/explanation separation, Obsidian for navigable workspaces, and Apple macOS sidebar/split-view guidance.
- The previous Mac canvas fix removed excess margins but the 1440px lesson breakpoint missed the default 1280x820 Mac window. On desktop-height windows (>=1200px wide, >=740px high) the Steps player used a 2-pane reading workspace (superseded 2026-11-08 by the single centred reading column, see § Lesson focus layout).
- The labelled Mac sidebar is slightly wider and visually separated, with stronger active/hover affordances. Short Tab S11 landscape and phone layouts are unchanged. Pages reading mode remains single column.
- Browser check now asserts side-by-side lesson panes at 1280px as well as at 1600px. New PR Actions must pass before merge. Native Mac visual QA remains pending; Playwright fixtures are not native screenshots.
- Next larger UX phase after this PR: task-first Today dashboard and resizable/collapsible side panels; do not imply those are already implemented.

## Keyboard-first Mac review (2026-11-07, merged PR #740)

- Reviewed Anki's desktop review workflow. Batna already had Space-to-reveal and 1–4 grading; do not duplicate these actions.
- The global keyboard dispatcher in `crates/app/src/shortcuts/mod.rs` now uses Space/Enter to choose Good after revealing the answer in an active Study item, and `Z` triggers Study Undo when available. Lesson steps accept RTL-correct arrows: Right goes back and Left advances only when allowed. The original 1–4, A–E, Cmd/Ctrl+1–7, Cmd/Ctrl+, and ? remain.
- A modal always owns shortcuts over the underlying screen; typing fields, contenteditable and textboxes ignore review shortcuts and ?; auto-repeat/composition are ignored; focused buttons retain native Space/Enter behavior.
- `crates/app/src/study/mod.rs` marks its Undo with `aria-keyshortcuts="Z"`; `crates/app/src/learn/player.rs` assigns arrows on the real navigation buttons; help sheet adds Good and Undo rows.
- Check CI and Mac behavior in the PR before merge. No alpha release unless requested.

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
| Full gate before every commit | `just check` = `scripts/check-tracked-files.sh`, `scripts/check-file-sizes.sh` (warns only), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `batna-tools validate`, `batna-tools packs build --check` |
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
| `res` | Basics of Research | slide decks (21 lectures 3 Oct–12 Dec 2026) | 11 |

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
| `courses/res/slides/*.pdf` | research slide decks (`src.page` = slide number). A lecture taught as several decks is one merged PDF (`res-L09.pdf` = budget 1–14 + ethics 15–85 + plagiarism 86–141); the original files are in `slides/originals/` |
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

## 10. Current state (2026-11-08)

| Area | State |
|---|---|
| App | Feature-complete for alpha: Today, study, bank, cases, oral, lessons, 94 calculators, stats/planner, own cards + Share, own lesson highlights, Anki export, abbreviations, lab chips, themes (Paper, Night, Night shift, true black, Study, Study dim, three System themes), sync, password login, OTA app + content updates, web |
| Releases | Latest `batna-v1.0.0-alpha.3` (2026-11-06); the alpha.4 bump merged but was never released. Next release is the first calendar one (`2026.11.1`), once the lesson screen redesign merges (Mohamed, 2026-11-08). Previews every 4 h |
| Critic loop | Final round (pass 10) finished 2026-11-04; loop closed. Don't start another unless Mohamed asks. |
| Open PRs / issues | None open on 2026-11-08; PRs #752 and #753 merged after green Rust/Android/Node CI |
| Coverage | 95.05% (floor 94) |
| Mac wide layout | PRs #739–745 merged as applicable: expanded window, Anki-style keys, focus continuity and quick palette; 2026-11-08 lesson player is one centred reading column (§ Lesson focus layout); native Mac QA remains with Mohamed |
| Reminders | Lesson steps that repeat a list show its تحشيشة (`remind:`), 791 on 689 steps (§ Repeated topics) |
| Info sheet | Bottom sheet on phones; floating card next to the tapped chip on tablet and Mac, movable and resizable (§ Floating book-page card) |
| Brand | B0 since 2026-11-08: petrol icon with the redrawn liver; the same mark beside the name in the top bar and sidebar (`brand.rs`) |

### Open work and known gaps

- Content for courses with no material yet; `res` L12–L21 as slides arrive.
- Content still `unverified` until Mohamed approves in-app.
- Glossary: split remaining ICP/SBP stop-gap senses into `course:` entries.
- Review leftovers from the 2026-11-04 content audit (ambiguous abbreviations,
  quote cuts) if Mohamed asks; lists were in `/mnt/project-files/content-audit/`.
- Low, deliberately not fixed: IndexedDB handle never closed per save;
  Argon2 on the web main thread (~0.5 s); a second tab on a borrowed computer
  wipes the first tab's data.
- Test debt: merge duplicate click-test helpers; more calculator tests against
  original-paper examples and cut-offs.
- Not yet verified on a real device: Back-key routing, update notifications,
  the lock/unlock freeze fix (`vendor/`, tested on desktop sockets only).
- Wide Mac layout needs visual and interaction QA in the native macOS bundle;
  the 1600px browser screenshots are SSR previews, not native-device checks.
- 14 Rust files are still over 800 lines (`scripts/check-file-sizes.sh`);
  split each when a PR already changes it a lot (AGENTS.md § Small files).

## 11. Resume message

Mohamed can paste this into any new chat or account to continue:

> Continue work on my apps Batna (`MohamedRef/Batna-app`) and Kabar
> (`MohamedRef/Kabar-app`). Before anything else, read `AGENTS.md` and
> `docs/HANDOFF.md` in each repo you touch and follow them. Reply to me in
> Egyptian Arabic. Keep `docs/HANDOFF.md` updated in every PR that changes
> behaviour, process or state.

## 12. Log

Newest first. One line per PR that changes behaviour, process or state.

- 2026-11-08: rebrand B0 (Mohamed's pick from the proposal page https://claude.ai/artifact/Y9h75TKgVP3TBEJNX68pKT): launcher icon is the liver redrawn as one clean shape on petrol `#245f7c` with a green bile drop (`assets/src/icon/*.svg`, re-exported); the same mark now heads the phone top bar, rail and sidebar (`crates/app/src/brand.rs`, theme colours `--accent`/`--good`; a test keeps it equal to the icon paths).
- 2026-11-08: release numbering switched to calendar versions (`2026.11.N`) and one build counter (commits on main) for previews and releases, on Mohamed's ask; `scripts/version.sh`, versionCode 20000000 + build, tags `v<version>`, no bump PR (`docs/RELEASE.md` §1).
- 2026-11-08: PR #753 merged with green Rust/Android/Node CI: foreground return resets only Study's active answer timer; native S24 Ultra event delivery unverified, process-death resume remains out of scope.

- 2026-11-08: Study background time fix PR: on WebView visibility return, reset answer timing only for a running session; isolated JS bridge tests, without session/scroll/navigation changes. PR #752 benchmark merged green (16 cards, p50 2380µs, p95 65600µs on hosted Linux).

- 2026-11-08: Added focused CI-runner baseline for actual Study grade-to-next-card TestDom mutations (mount, p50/p95/max) with no timing gate. Native Android render latency and durable process-death resume remain unverified.

- 2026-11-08: PR #751 merged after green Rust/Android/Chromium CI. Static SSR checks now include running Study at 384×520; verify native WebView keyboard, gesture strip and lock/unlock manually on S24 Ultra. No new alpha. Next code work should measure transition/render latency and resume fidelity before optimizing.

- 2026-11-08: Short-height Android study layout regression PR: run running Study SSR at 384×520, assert End reachability and grade target geometry, and preserve original desktop/phone viewport tests. Native keyboard and background-resume checks remain for device QA.
- 2026-11-08: PR #749 merged after green current-head CI; 320/384 phone running Study progress and Undo/End are separated, with safe-area grading.

- 2026-11-08: PR #750 merged with green Rust, Android and Chromium checks. Bank scope and history disclosures now use 48dp targets; PR #749 follows for the running Study toolbar.

- 2026-11-08: Bank touch-target follow-up PR: guarantee 48dp on lecture/topic scope opener and previous-paper disclosure; extend 320/384px normal/larger text browser geometry regression. Includes native-device QA caveat and avoids rebuilding bank answer controls.
- 2026-11-08: PR #748 merged after Rust, Android and Chromium CI green; compact list and calculator actions now use 48dp controls.

- 2026-11-08: Compact phone touch audit PR: give list sorting, layout choice, section actions, calculator Reset and breakdown disclosures full 48dp controls. Add browser CSS measurements on 320/384px at normal/larger text, plus Rust style guards. This validates CSS hit targets, not hydrated bank/calc interactions or a native phone. Bank MCQ answer buttons already use --target.
- 2026-11-08: PR #747 merged after green Rust, Android and Chromium CI for bottom navigation and More Close.

- 2026-11-08: Phone navigation touch audit PR: enforce 48dp for bottom tabs and More Close; add 320/384px Chromium checks for bottom navigation, absent from the earlier main.content-only audit. Study/lesson/bank/calculator touch reviews remain follow-up work, not claimed complete.
- 2026-11-08: PR #746 merged with Rust, Android and Chromium CI green; the phone search button reuses the offline palette.

- 2026-11-08: Phone UX quick-search PR: add search icon beside Settings in the compact top bar, route to the shared offline command palette, and keep results scrollable above the Android keyboard. Native S24 Ultra and Android Back QA remain deferred.
- 2026-11-08: PR #744 merged with green CI, completing the planned Mac palette destinations before starting Android changes.

- 2026-11-07: Mac expanded-layout pass: eliminate fixed-width canvas padding, adapt lesson Steps into context and active-step panes on ≥1440px, widen study/Settings, and default the Mac window to 1280×820; browser SSR at 1600px includes inert desktop navigation and saves previews. Layout at 840–1099px stacks form panes to prevent larger-text overflow. Native Mac QA remains open.

- 2026-11-08: 1.0.0-alpha.4 bump merged (#766) but the release was held for the lesson screen redesign, then replaced by calendar numbering. The Keychain password prompt ("app.batna.cloud") Mohamed kept seeing came from local builds of a checkout older than #612; main builds keep the session in `cloud-session.json` and launch without it.
- 2026-11-06: 1.0.0-alpha.3 released on Mohamed's ask, so the Mac app (alpha feed only; previews carry no Mac build) gets the lock/unlock and update-check fixes.
- 2026-11-06: Android screen froze after lock/unlock: Dioxus's edit socket patched in `vendor/` (edits resent until acknowledged); update check shows the app result before the content pack downloads, requests time out after 20 s of no progress.
- 2026-11-06: res.L11 lesson 4 sampling tree: purposive labels wrapped onto four lines (they ran past the picture edge and failed the nightly lesson browser regression).
- 2026-11-05: res.L11 (sampling and sample size) written: 82 items, 6 lessons; two sample-size calculators (`sample_size_survey`, `sample_size_two_proportions`) reproduce the slides' Epi Info numbers.
- 2026-11-05: `res.L09` written (budget, ethics, plagiarism: 113 items, 6 lessons); L09–L11 slides, text and images added to Batna-sources; glossary gains REC, CIOMS, WAME, WMA, MLA and `res` senses of PI, CV, APA.
- 2026-11-05: res.L10 (data collection and questionnaire design + searching the literature, 146 slides) added: 107 items, 8 lessons; image-slide quotes and review-MCQ refs in `courses/res/verdicts/L10.tsv`.
- 2026-11-05: Mac nightly green again: three tests now expect the Mac's own answers (installer message, device name; menu test skipped on macOS, muda needs the main thread).
- 2026-11-05: big files split into smaller modules (i18n per feature; core `lesson`, `calc`, `labs`, `session`); AGENTS.md § Small files; `check-file-sizes.sh` warns in CI.
- 2026-11-05: the whole project folder and memory notes archived in `MohamedRef/Batna-sources`.
- 2026-11-05: source material backed up to the private repo `MohamedRef/Batna-sources` (§ 8).
- 2026-11-05: handoff docs added (`AGENTS.md`, this file, course briefs, content scripts).
- 2026-11-04: 1.0.0-alpha.2 released (#726); final critic round closed.
- 2026-11-08: lesson focus layout (one centred reading column ≥840px), text colours by importance, key-term pill, Study/Study dim/System study themes.
- 2026-11-08: own highlights in lessons (select → colour, fill/underline, pill/rect; synced per highlight).
- 2026-11-08: repeated topics remind their تحشيشة (`remind:` on lesson steps, 791 reminders, «من درس …» link).
- 2026-11-09: the book-page / abbreviations sheet floats as a movable, resizable card by the tapped chip on tablet and Mac (phones keep the bottom sheet).
