# Shared UI completion: Batna

Updated: 2026-10-30. Current branch: `test/settings-browser-20261030`.
Previous checkpoint: merged PR #526; subsequent merged UX changes are retained.

## Item 1: interactive and dynamic cards (2026-10-30)

- [x] Migrated 44 remaining native base-card surfaces, including dynamic
  classes, clickable buttons, status/alert regions, labelled sections and keyed
  list items. No native `.card` markup remains outside the shared factory.
- [x] Added `CardElement` for the original native tags. Global HTML/ARIA/data
  attributes pass through; typed click and drag handlers retain their callbacks.
- [x] `DivCard` delegates to the same Card implementation. `CardButton` retains
  native keyboard, disabled and form-button behavior with one explicit type.
- [x] Added rendered-production-component tests for semantic roots, attributes,
  dynamic status, event registration, keyed input retention and form type.
- [x] Kept shared components visible to the dead-end scanner and added a guard
  against copying base card markup back into individual features.
- [x] Local strict workspace/all-target Clippy, full workspace tests, content
  validation and production-script Node tests passed without desktop features.
  Batna packs also build and round-trip. Full native/target CI runs on the PR.

The existing physical-device and browser-layout gaps below remain separate
numbered items. Passing these component tests does not certify pixel layout.

## Item 2: workflow ownership (2026-10-30)

- [x] Split rendering stages, update state and orchestration, persistence
  responsibilities and existing tests into focused sibling modules.
- [x] Preserve the public feature facades and original function bodies. Child
  modules keep parent-only visibility for implementation helpers; no new runtime
  dependency, persistence format, scheduler or content approval change is involved.
- [x] Retain each original behavior test once, rebasing physical fixture paths.
  A function inventory confirms that no function was lost or duplicated.
- Validation results and full native/target CI are recorded on this item's PR.
  Local checks omit desktop features because this runner has no WebView libraries.

| Feature facade | Before (lines) | After (lines) |
|---|---:|---:|
| `learn/steps.rs` | 976 | 208 |
| `study/mod.rs` | 740 | 500 |
| `case/mod.rs` | 725 | 355 |
| `store/backend.rs` | 773 | 423 |
| `update/mod.rs` | 2,234 | 492 |

The remaining facade code owns integration and shared state. Tests live in
`tests.rs`; they are not replaced with new tests of the extraction itself.

## Item 3: rendered Settings browser checks (2026-10-30)

- [x] Export actual Settings/Look components and production styles through the
  test-only SSR renderer, using synthetic preferences and an in-memory store.
- [x] Add a pinned Chromium job for computed layout and the production Settings
  focus boundary across compact/medium/expanded widths, all themes, both surface
  appearances and largest text, including RTL/LTR where the app supports it.
- Browser execution and screenshot evidence belong to this item's CI head; see
  [SETTINGS-BROWSER-QA.md](SETTINGS-BROWSER-QA.md) for commands and scope. The
  lesson player and study screen have the same kind of job:
  [SCREENS-BROWSER-QA.md](SCREENS-BROWSER-QA.md).
- [ ] Complete broader screen integration and the physical-device matrix. SSR
  callbacks are not hydrated; no application-close or native-device pass is claimed.

## Implementation checkpoint

- [x] Added scanner regression coverage for shared Card/DivCard classes on the next source line, so migrated cards remain checked for dead-end interactions.
- [x] Incorporated merged preference-recovery and Android Back changes from `main`.
- [x] Added semantic shared content surfaces: `ui::Card` retains `section`, `ui::DivCard` retains `div`. Existing `ui::ActionRow` keeps its native accessible button.
- [x] Migrated a first set of static cards in Study, Plan, Oral, Cases, Stats, Learn, and related views without changing the original child controls. Special-case clickable, keyed, labelled and alertdialog surfaces remain native.
- [x] Extracted clinical-case choices to `case/choices.rs`, lesson-step decisions to `learn/steps/decision.rs`, and session selection/composition into `study/planning.rs` while retaining existing caller behavior.
- [x] Added VirtualDom smoke tests exercising shared components with a real button, label, text input, and action row.
- [x] Added production CSS contract assertions for compact/medium breakpoints, appearance, reduced motion/transparency, RTL/LTR isolation and shared card tokens.
- [x] Completed the remaining base-card migration, including interactive and dynamic variants, with events, keys, IDs and ARIA preserved.
- [x] Split the targeted large workflows behind their existing feature APIs (see item 2).
- [ ] Add rendered browser/WebView responsive, focus and keyboard checks. VirtualDom and CSS-selector assertions are not visual QA.
- [ ] Complete the separate Mac/S24 Ultra/Tab S11 device matrix. Current-item CI status is recorded on its PR.

## Quality and rollout

Automated gates run strict Rust formatting, Clippy, tests, content validation, Android arm64 compile and Android renderer-loss regression. Check GitHub Actions for the latest commit; an earlier successful check does not certify a subsequent commit.

Native matrix still needed: 384/600/840dp and wide windows, Paper/Night/True black, Calm/Glossy, largest interface size, Egyptian-Arabic RTL/Latin medical snippets, Tab/Escape, screen readers, nested sheets and Android Back. No actual-device test has been claimed.

Batna owns all adapted code. Do not add a dependency on Kabar crates or couple app releases. No medical content, approval, scheduling data or patient information should change in this UI PR.

## Next change boundaries

Each numbered item is checked, committed and uploaded separately. The current
item covers the shared card contract; later workflow, browser, device and write
feedback work remains explicitly tracked. CI status belongs to the current PR
head. No merge, release or deployment is implied by uploading a checkpoint.
