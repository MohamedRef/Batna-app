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
| Card markup | `crates/app/src/ui.rs::Card`, `DivCard`, `CardButton` | All base-card surfaces; explicit native tags and attributes are retained |
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

## Workflow module ownership (2026-10-30)

| Responsibility | Module |
|---|---|
| Lesson dispatch, shared gate and response context | `learn/steps.rs` |
| Figures, reveals and instructional blocks | `learn/steps/presentation.rs` |
| Check, order, match, sort, mini-case and table interactions | `learn/steps/{check,order,matching,sorting,mini_case,table}.rs` |
| Study setup and completion | `study/{setup,outcome}.rs` |
| Clinical case stages and recap | `case/player.rs` |
| Review backend's StudyStore implementation | `store/backend/study.rs` |
| Update state and execution | `update/{model,service}.rs` |

Keep callers on the existing feature facade. Change the owning stage rather
than copying its state into another screen. Public names are re-exported with
their existing visibility; internal helpers use parent-only visibility. Shared
state stays with the parent, and persistence operations keep their transactions.
Existing tests live beside the owning facade in a child `tests.rs` module.

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
3. **Guard shared component reuse.** Base cards now use `ui::Card`, the
   compatibility `DivCard` delegate or native `CardButton`. Use `CardElement`
   for articles, list items, paragraphs and disclosure roots; pass existing ARIA,
   data, IDs and typed event props. The source guard rejects new copied `.card`
   markup. Specialized overlays keep their own component and styling category.
4. **Native visual coverage.** Verify 384/600/840dp and wider windows,
   maximum interface size, Paper/Night/True black, keyboard, Android Back and
   nested sheets on actual WebViews. A browser fixture is a narrower check.
5. **Repository independence.** Port useful Kabar changes deliberately with
   source credit. Do not depend on Kabar crates or couple releases merely to
   avoid maintaining a small adapted component.

No medical content, approvals, scheduling data, runtime service or dependency
changes are needed for this UI pass.

## Today dashboard hierarchy (2026-10-30)

Today keeps the existing recommended study action, due/new sessions, exam
plan, continuation and lecture progress. The visible hero CTA now calls the
same action as its full-card tap directly, stopping event bubbling to avoid
double navigation. Two stat tiles still open the exact due/new sessions;
estimated minutes without an exam plan are a quiet link to the forecast.
The plan's own workload estimate is not repeated beneath it.

After the hero and counts come Continue Learning, in-progress lectures, and
the existing recap. Bank and content-verification shortcuts remain under a
native disclosure, with their original destinations and no new state.
Below 840dp the complete lecture catalogue is browsed on the Lessons screen
via Today's existing "All Lessons" action, not repeated at the foot of Today.
From 840dp the existing lecture aside and sticky main column are retained.
No course focus, saved views, scheduling, study records or content data change.

## Lesson player focus and accessible navigation (2026-10-30)

The focused lesson player retains its saved Attempt, first-try grading,
step guard, Skip, Steps/Pages choice, citations and return navigation.
The lesson is now a page heading, its English lecture title is on a separate
LTR-isolated line, and a semantic progress meter describes the *current
step*, not a completion score. Question steps stay visibly distinguished.

When Continue is disabled without a Check action, the footer explains how
to complete the current interaction or skip it; the disabled Next action
references the same explanation for accessibility. Close and view-mode
controls use the shared 48dp touch target. On the final summary, Next
Lesson becomes the primary action only when there are no practice cards;
otherwise, Return to Lessons becomes primary if no follow-up exists. Practice
guidance is only shown when there are practice cards. The next-lesson chain
keeps the screen that originally opened the player as its exit destination.
The page arrow uses the shared on-accent contrast token.
No medical content, scoring, saved drafts, FSRS or Back stack changes.

The compact footer (Back, Continue, Check and Skip) and summary actions
use 48dp minimum touch heights independently of the compact button token.
The summary's return-to-list button also participates in the shared Android
Back controller, so Back exits the finished lesson instead of skipping past
its origin. Actual gesture and screen-reader behavior still need device QA.

### Follow-up: clean page and in-step targets (2026-10-30)

The "complete this step or skip it" explanation is read to screen readers
only (`.ls-sr-only`, referenced by the disabled Continue). The visible page
stays free of helper notes, as Mohamed asked on 2026-10-29; the visible
Skip button is the way on. In-step controls (tiles, sort bins and chips,
hidden table cells, order Undo, retry, image zoom) use the shared 48dp
`--target`. Text on solid accent fills uses `--on-accent`. The finished
summary takes focus on its heading so screen readers announce the result.

## Study review: suggested grade in words (2026-10-30)

An optional suggested rating is now explicitly marked with the word
"مقترح" inside its existing rating button, in addition to the current
focus-independent ring. It remains only a suggestion; all four grades,
their real FSRS intervals, review storage and retry/Undo semantics are
unchanged. The label is absent when there is no recommendation.

## Study review: touch targets and announcements (2026-10-30)

Every `.btn`, `.btn-link` and `.btn-lg` inside the study screen (`.st`), the
lecture chips, the time choice, cloze blanks and recall points use at least
the shared 48dp `--target`; compact `--tap` alone is 40px. The progress bar
gives `aria-valuetext` ("N من M") so it no longer reads the 0-based count.
The session summary focuses its heading on mount so the result is announced
after the last grade button disappears. Ratings, FSRS, Undo and review
storage are unchanged.
