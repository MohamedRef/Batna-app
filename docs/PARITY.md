# Batna ↔ Kabar app parity

Mohamed (2026-10-27): Batna must have every core *app* feature Kabar has, not
only the study screens. This list compares Batna (`MohamedRef/Batna-app`) with Kabar
(`MohamedRef/Kabar-app` `main` at 21b21db) feature by feature, says how each one is adapted for a
medical study app, and names the owner. Kabar is a reference to copy from
(credit the source path); Batna never depends on it and never changes Kabar-app.

Language-learning features (Journey Map, Reader, Bridge, personas, Sinetron,
games) have no medical twin and are listed at the end with what Batna does
instead.

Statuses checked against Batna-app `main` at b4b676a on 2026-10-28.

Status: ✅ in Batna · 🟡 partly · ⏳ planned (owner named) · ➖ not applicable.

## 1. Shell, look and settings (chunk 1)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 1.1 | Settings sheet over the current screen (never loses a session), sections that follow the device, "syncs" vs "this device only" labels | Same sheet. Sections: Study, Look, Reminders, Android, Tablet, Mac, Your data, About | 1 | ✅ PR #354 |
| 1.2 | Themes: Paper (light), Merapi (dark), Malam (night), WCAG AA contrast tests | Three themes: Paper, Night (dark), and Ward-night (very dark, no blue, for late study). Follow system by default. Contrast tested | 1 | ✅ PR #354 |
| 1.3 | Text size, 4 steps, per device | Same, per device | 1 | ✅ PR #354 |
| 1.4 | Bundled fonts (IBM Plex Sans Arabic, Latin face), OFL in ATTRIBUTION | Copy the Plex Arabic + Latin woff2 files and the Linux tashkeel workaround | 1 | ✅ PR #357 |
| 1.5 | Stroke SVG icons following `currentColor` | Replace the nav glyphs with drawn icons (book, cards, stethoscope, mic, chart, gear) | 1 | ✅ PR #357 |
| 1.6 | Android Back gesture moves back inside the app | Copy `back.rs` (data-back controls, screen history) | 1 | ✅ PR #357 |
| 1.7 | Diagnostics ring + panic hook, copyable from Settings | Copy `diag.rs` and the clippy `eprintln!` ban | 1 | ✅ on main (Batna-app #100: all report!(), clippy bans eprintln!) |
| 1.8 | UI in Egyptian Arabic, MSA and English | Egyptian Arabic now; English UI as phase 2 once the feature screens settle (i18n moves from consts to a lookup) | 1 | 🟡 Egyptian only |
| 1.9 | About: version, build, data folder | Same | 1 | ✅ PR #354 |
| 1.10 | Width layout: bottom bar <600, rail 600–840, sidebar ≥840 | Bottom bar and rail done; labelled sidebar ≥840 goes with §6 | 1 / 23 | ✅ (sidebar from chunk 23) |

## 2. Feedback: sounds and vibration (chunk 20)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 2.1 | Right-answer chime and lesson-end gong, 3 variations each, preview in Settings, sounds off | A soft chime for right, a calm end-of-session tone, 3 variations each, synthesized in code (no samples), WAV | 20 | ✅ on main |
| 2.2 | Vibration tap (Kotlin `Haptics.kt`), follows the phone's haptics setting, shown only on devices with a motor | Copy as is; wrong answers get a barely-there tap, never a buzz | 20 | ✅ on main |

## 3. Reminders and onboarding (chunk 20)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 3.1 | Local reminders (Kotlin `Reminders.kt`, AlarmManager), offline, re-planned on activity, wall-clock times across DST | Daily study reminder at Mohamed's times, skipped once today's target is met; says how many reviews are due | 20 | ✅ on main |
| 3.2 | Only one device rings (phone by default, tablet/Mac opt in) | Same rule | 20 | ✅ on main |
| 3.3 | Notification permission flow and "open app settings" links | Same | 20 | ✅ on main |
| 3.4 | Onboarding (coach: goal, minutes a day, target, study times, reminders) | First run: exam date, study days per week, minutes a day, lectures already studied, reminder times, theme. Writes chunk 8's planner settings | 20 (+8 API) | ✅ on main |
| 3.5 | Daily recap and weekly report | Calm end-of-day line on Today and a weekly report: items learned, retention, weak topics, days to exam. No streak guilt | 20 (+8 data) | ✅ on main |
| 3.6 | Mac notifications (bundled app, opt-in) | Same, from the bundled Batna.app | 20 | 🟡 Android only: Mac notifications not built yet |

## 4. Your data: backup and sync (chunk 21)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 4.1 | Export everything to one JSON file, Android "Save to" dialog (`ExportSaveActivity.kt`) | Same, plus the ✎/✗ verdicts export from chunk 3 in the same place | 21 | ✅ on main (Android Save-to dialog, Batna-app #17) |
| 4.2 | — (Kabar relies on Cloud) | **Import / restore** from that file, merging by HLC, so a backup or a device move needs no cloud | 21 | ✅ on main |
| 4.3 | Kabar Cloud: Google sign-in, live sync, Devices screen, E2E check | **Batna Cloud**: its own free Cloudflare Worker copied from Kabar's `crates/server`, separate database, same sync design | 21 | 🟡 code on main (crates/server, app cloud/), not deployed: needs the Mac (just cloud-deploy) |
| 4.4 | Synced-table rules (account_id, device_id, HLC, tombstone) | Already in chunk 3's store | 3 | ✅ |

## 5. Updates and release builds (chunk 22)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 5.1 | Preview APK built in CI on every merge, throwaway cached key | Same workflow on pushes to `main`, its own preview key | 22 | ✅ on main |
| 5.2 | Over-the-air updates: signed `update.json` (Ed25519) on GitHub Releases, SHA-256 check, automatic / tell me first / off | Same, separate feed and key, channel `batna-preview` now, `batna-alpha` later | 22 | ✅ on main |
| 5.3 | Android install via PackageInstaller (`UpdateInstaller.kt`) | Copy | 22 | ✅ on main |
| 5.4 | Mac: swap the .app in place | Copy | 22 | ✅ on main, compiled, tested and bundled on macos-15 in CI (Batna-app #44) |
| 5.5 | Waits on metered networks / Data Saver | Copy | 22 | ✅ on main |
| 5.6 | Content packs refresh with an update | Packs from chunk 9 ride the same feed | 22 (+9) | ✅ on main |
| 5.7 | Release signing (`~/.kabar` keystore), `just` recipes for APK, install, Mac bundle | `~/.batna/batna-release.jks`, `just apk`, `just install-android`, `just bundle-mac` | 22 | ✅ on main (just apk, bundle-mac) |
| 5.8 | Launcher and Mac icons (adaptive) | A simple drawn liver-and-bile-drop mark in SVG, exported to Android and macOS sizes | 22 | ✅ on main |
| 5.9 | Kotlin glue module `android/` built into the APK, `android-check-apk.sh` | Created by chunk 20 (first user), checked by chunk 22's workflow | 20 → 22 | ✅ on main |

## 6. Tablet, pen, keyboard and Mac (chunk 23)

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 6.1 | Labelled sidebar on wide windows, icon rail on medium; nav style setting | Same | 23 | ✅ on main |
| 6.2 | Study Desk panes on the tablet | Two panes at ≥840: the question on one side, the explanation, `why` lines and book page on the other; bank list + question | 23 | ✅ on main |
| 6.3 | S Pen hover peek (word meaning) | Hover peeks: over an abbreviation or drug name shows its expansion (a glossary from content tags), over "Book p. 14" shows the short quote, over an MCQ option after answering shows its `why`. Setting per device | 23 | ✅ on main (content/glossary.yaml) |
| 6.4 | Mouse hover peek on the Mac | Same peeks with the mouse | 23 | ✅ on main |
| 6.5 | Keyboard shortcuts (screens, reveal, rate 1–4, help list) | Same, plus A–E for MCQ options, Space to reveal and M for a lesson picture's marks | 23 | ✅ on main |
| 6.6 | Mac menu (Batna › Settings… ⌘,) | Copy `settings/menu.rs` | 23 | ✅ on main, compiled in the Mac CI job (Batna-app #44) |

## 7. Study features with a Kabar twin

| # | Kabar feature | Batna adaptation | Owner | Status |
|---|---|---|---|---|
| 7.1 | Review (FSRS 6), typed answers | FSRS, verify-while-study | 2, 3 | ✅ on main |
| 7.2 | Stats: skill radar, pace and forecast charts | Per-lecture table and weak topics; **add** an SVG forecast of due load for the next 14 days and a pace line toward the exam date, drawn like Kabar's `stats/charts.rs` | 8 | ✅ on main (stats/charts.rs) |
| 7.3 | Placement and level tests, monthly level check | Mock exams and the question bank | 4 | ✅ on main |
| 7.4 | Background audio (`PlaybackService.kt`) | Oral mode keeps reading with the screen off, for walking | 7 | ✅ on main (OralService.kt, screen-off mode) |
| 7.5 | Offline packs | Content packs built in | 9 | ✅ |
| 7.6 | Share-to-Kabar (text, screenshots) | Phase 2: share a passage from any app → a draft card of Mohamed's own, flagged as his | 21 | ⏳ phase 2 |
| 7.7 | Teach Kabar (learner adds an entry) | Phase 2: "my own card", same table as 7.6 | 21 | ⏳ phase 2 |

## 8. Kabar features with no medical twin

| Kabar | Why not in Batna | Batna instead |
|---|---|---|
| Journey Map, Lesson Player, Lanjut | Language path | Lecture list + planner |
| Reader, Bridge, speech levels, pronunciation, OCR decoder | Language reading | Book page links and hover peeks (6.3) |
| Personas, Sinetron, Radio | Language listening | Oral mode (7) |
| Games | Language games | Cases and calculators (5), tables and order (6) |
| Kak Nur persona | Mohamed asked for no extra complexity | Plain reminders and recap (3) |
| Runtime AI (router, prompts) | Accuracy: the book is ground truth, no runtime AI | Chunk 19's checks and Mohamed's ✓/✎/✗ |
| Pen writing, aksara | Paused in Kabar too | — |

## 9. After parity: critic rounds

Once Kabar and Batna are both feature-complete, critic threads review Batna
the same way they review Kabar (code, UX, platform, content accuracy), open
GitHub issues labelled `critic:*` with the `batna` label, each fix PR says
"Fixes #N" against base `main` of MohamedRef/Batna-app, and the critics review again until clean.
The coordinator starts those rounds.

## Still open (2026-10-28)

- 1.8: English UI (phase 2, chunk 1).
- 3.6: Mac notifications from the bundled app (chunk 20).
- 4.3: deploy Batna Cloud from the Mac (Mohamed, with docs/CLOUD.md).
- 7.6 and 7.7: Share-to-Batna and "my own card" (phase 2, chunk 21).

## Order

1. Chunk 1: 1.1–1.9 (shell) now.
2. Chunk 20 lands the `android/` module skeleton first (one small PR), then
   sounds, haptics, reminders, onboarding.
3. Chunks 21, 22, 23 start in parallel; 22's workflow waits for 20's skeleton.
4. Phase 2 items (1.8 English UI, 7.6, 7.7) after the January-critical study
   screens are in.
