# Native QA matrix

Engineering audit F23 (2026-10-29): green CI doesn't prove a full native
journey. What CI covers, and what only a real device can, is listed here.
Record each manual run against the **exact commit** it was made on (never as
a pass for a different head). The Mac workflow stays nightly/by hand; no PR
waits for any of this.

## Covered by automated tests

| Journey | Where |
|---|---|
| First launch → grade → restart → restore → two-device sync | `store/backend/journey_tests.rs` |
| Batna Cloud with fake Google and two devices (real Worker in workerd) | `crates/server/tests/e2e.mjs` (CI job "Batna Cloud end-to-end") |
| Refused sync ops held, sent again | `store/held.rs` tests |
| A UI write while sync holds the lock (bounded wait, then retry state) | `store/contention.rs` tests |
| Save failures in Study, Try, Oral, bank; lesson finish; read failures | `learn`, `oral`, `bank`, `store/places.rs`, `store/health.rs` tests |
| Android Back, Settings keyboard, study foreground lifecycle bridge | CI steps of the same names |
| Dependency advisories (RustSec) | `.github/workflows/audit.yml`, weekly |

## Manual, on devices (Mac, S24 Ultra, Tab S11)

Copy the table into a dated section below for each run.

| Check | How | Result |
|---|---|---|
| First launch on a clean install | Uninstall, install the build, open Today | |
| Lesson → grade → kill the app (recents swipe) → reopen | Lands on the same place; the answer is there | |
| Process death in a study session | Developer options › Don't keep activities, leave and return | |
| Background / resume during a session | Home, wait 1 min, return; timer and item unchanged | |
| Offline | Airplane mode, study, back online; the outbox empties | |
| Permission denied | Deny notifications; reminders say so, nothing crashes | |
| Two devices | Grade on the phone, see it on the tablet within a minute | |
| TalkBack / VoiceOver | Today, a study item, Settings › Sync read in order | |
| Keyboard focus (Mac, tablet keyboard) | Tab through Settings and a study item; focus visible | |
| 200% text | Android font size max; nothing clipped on Today and Study | |
| RTL with mixed script | An item with English terms in Arabic UI; `<bdi>` keeps order | |
| Release-build feel | Grade 20 answers quickly; no visible lag | |

## Runs

(none recorded yet)
