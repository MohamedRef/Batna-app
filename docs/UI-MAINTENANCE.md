# UI maintenance and review

Reviewed against `cac31ff` (2026-10-30). See [DESIGN.md](DESIGN.md) for the
visual vocabulary. This pass simplifies presentation without removing tools.

## Architecture

The app uses Rust structs/enums with `impl` methods and Dioxus component
composition. It does not need an inheritance hierarchy to share card behavior.
`crates/core` owns pure logic, `protocol` owns boundary types, and `app` owns
rendering, persistence and platform adapters. Batna remains independent of
Kabar; adapted primitives live in this repository.

`Ui` in `state.rs` supplies reactive device preferences. Study data already has
a central `store::Review` context: `store/backend.rs` bumps its revision after
writes and after sync reloads. Use that write path and subscribe to its
revision rather than keeping another editable copy in each screen. A UI Card
and a study content card are distinct: the former shares presentation; the
latter must be updated through its storage operation.

## Where a shared change belongs

| Change | Owner | Effect |
|---|---|---|
| Card markup | `crates/app/src/ui.rs::Card` | Settings content surfaces using the component |
| Navigation row markup and semantics | `ui.rs::ActionRow` | All Settings section links |
| Card radius, padding, border, shadow | `assets/main.css`: `--radius-card`, `--card-pad`, `--card-border`, `--card-shadow` | Normal cards and collection surfaces; Glossy remains an explicit variant |
| Row layout, icon, title and detail | `assets/parts.css` | Every `.activity-row` |
| Default list presentation | `parts.rs::Layout::DEFAULT` | New collection choices; saved per-screen choices keep precedence |
| Settings sections and search terms | `settings/mod.rs`: `Section`, `GROUPS`, `SectionView` | Index, search and detail pane |
| Settings layout and controls | `assets/settings.css` | Phone and wide layouts, also available to look updates |
| Settings keyboard boundary | `assets/settings-dialog.js` | Focus containment, Escape/Back selection and focus restoration |

Keep feature CSS about layout. Reuse the tokens and components; add a modifier
for a real variant instead of copying a card skin. Interactive cards still
need real buttons; `Card` itself is a content surface.

## Settings behavior

The index groups Study/Reminders, Look/Device, and Account/Data. Search finds
sections by titles, summaries and field-related Arabic/English keywords. It
tolerates Arabic marks, alef variants and repeated spaces. It filters the
index, not the controls inside a section. Empty results offer Clear search.
Reopening clears the query; returning to the index keeps it. The Updates row
retains its live status and About shows the build version.

Phones show the index or one section. Wide windows show both with independent
scrolling and a section description. Invalid platform-only preview sections
fall back to Study. While Settings is open, background controls are inert;
Tab stays inside the active dialog and Escape uses the visible Back/Close
control. Closing restores focus while the study screen remains mounted.

## Preference recovery

`state.rs` owns app-wide state; root re-exports preserve existing imports.
`persistence/queue.rs` stores only the newest failed action per stable key.
A successful unrelated write does not hide another failure. Retrying snapshots
checks each generation before executing, performs IO without a live signal
borrow, and removes only successful current actions. Pending choices live in
memory; closing the app can lose them, as the notice explicitly states.

The notice appears in Settings while open and on the main surface otherwise.
Its live region announces failure and the retry action stays keyboard accessible.
Capture owned desired values and durable storage handles; do not capture
screen-local signals that could be dropped when a section closes.

`Ui::update_prefs` is the shared device update path. `prefs/storage.rs` writes
a uniquely owned temporary file, flushes it, then atomically replaces the
existing file; a failed publication keeps the previous preference file intact.
Device snapshots use one stable key so the newest complete choice wins.

Synced settings and collection/reminder choices use stable separate keys.
Review retention and daily-goal retries apply one command to current backend
settings, keeping unrelated updates and the existing fitted-parameter merge.
Recovery clears only the corresponding legacy write diagnostic. A successful
retry reloads synced preference globals so screens observe the recovered data.

Tests cover a real blocked directory, retained prior files where applicable,
latest-choice replacement, unrelated failures, repeated failed retry and a real
Dioxus reactive observer. Native WebView layout still requires device QA.

## Remaining work, in priority order

1. **Feature-specific write recovery.** Shared preference writes now use
   `persistence/`, but domain writes such as reviews, content edits and exports
   retain their existing error handling. Give those workflows an appropriate
   recovery UI; never blindly replay append-only events through the preference
   queue.
2. **Smaller workflow modules.** `update/mod.rs`, `learn/steps.rs`,
   `study/mod.rs`, `case/mod.rs` and `store/backend.rs` have large workflow
   units. Extract pure decisions and focused stage components before
   replacing their APIs. Keep review scheduling and content checks separate.
3. **Gradual primitive adoption.** This pass migrates the direct Settings
   content and index. Other features still have hand-written `.card` markup:
   shared CSS changes reach them, markup changes require migration.
4. **Native visual coverage.** Verify 384/600/840dp and wider windows,
   maximum interface size, Paper/Night/True black, keyboard, Android Back and
   nested sheets on actual WebViews. A browser fixture is a narrower check.
5. **Repository independence.** Port useful Kabar changes deliberately with
   source credit. Do not depend on Kabar crates or couple releases merely to
   avoid maintaining a small adapted component.

No medical content, approvals, scheduling data, runtime service or dependency
changes are needed for this UI pass.
