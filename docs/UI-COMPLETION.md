# Shared UI completion: Batna

Updated: 2026-10-30. Branch: `improve/shared-ui-completion` (PR #523).

## Implementation checkpoint

- [x] Added scanner regression coverage for shared Card/DivCard classes on the next source line, so migrated cards remain checked for dead-end interactions.
- [x] Incorporated merged preference-recovery and Android Back changes from `main`.
- [x] Added semantic shared content surfaces: `ui::Card` retains `section`, `ui::DivCard` retains `div`. Existing `ui::ActionRow` keeps its native accessible button.
- [x] Migrated a first set of static cards in Study, Plan, Oral, Cases, Stats, Learn, and related views without changing the original child controls. Special-case clickable, keyed, labelled and alertdialog surfaces remain native.
- [x] Extracted clinical-case choices to `case/choices.rs`, lesson-step decisions to `learn/steps/decision.rs`, and session selection/composition into `study/planning.rs` while retaining existing caller behavior.
- [x] Added VirtualDom smoke tests exercising shared components with a real button, label, text input, and action row.
- [x] Added production CSS contract assertions for compact/medium breakpoints, appearance, reduced motion/transparency, RTL/LTR isolation and shared card tokens.
- [ ] Continue safe migration of noninteractive base cards without changing events, keys, IDs, ARIA or HTML semantics.
- [ ] Continue splitting large independent workflows (`update/mod.rs`, `learn/steps.rs`, `study/mod.rs`, `store/backend.rs`) in separately validated changes.
- [ ] Add rendered browser/WebView responsive, focus and keyboard checks. VirtualDom and CSS-selector assertions are not visual QA.
- [ ] Verify latest-head CI and actual Mac/S24 Ultra/Tab S11 behavior before marking ready.

## Quality and rollout

Automated gates run strict Rust formatting, Clippy, tests, content validation, Android arm64 compile and Android renderer-loss regression. Check GitHub Actions for the latest commit; an earlier successful check does not certify a subsequent commit.

Native matrix still needed: 384/600/840dp and wide windows, Paper/Night/True black, Calm/Glossy, largest interface size, Egyptian-Arabic RTL/Latin medical snippets, Tab/Escape, screen readers, nested sheets and Android Back. No actual-device test has been claimed.

Batna owns all adapted code. Do not add a dependency on Kabar crates or couple app releases. No medical content, approval, scheduling data or patient information should change in this UI PR.

## Next change boundaries

Keep draft until tests, target coverage and native verification are complete. Do not merge or deploy as part of this checkpoint.
