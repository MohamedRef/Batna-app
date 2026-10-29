# Batna design system

The 2026-10 redesign (Calm / Glossy · Material You). Every screen builds
from the tokens and shared classes below; feature stylesheets add layout,
never their own colours or their own copy of a button, chip or sheet.

## Ownership

The foundation (this file, `crates/core/src/theme.rs`, `crates/app/src/prefs.rs`,
`crates/app/src/look.rs`, `assets/main.css` root tokens, `assets/parts.css`,
`assets/appearance.css`, `assets/shape.css`, `assets/nav.css`,
`assets/desk.css`, `more.rs`, the shell in `main.rs`, Settings › Look) is
changed by one owner at a time. A screen that needs a new token or shared
part asks for it, or adds a feature-local class in its own stylesheet.

## Colour tokens

Set on `:root` by `batna_core::theme::Palette::css`, per theme. Never write
a hex value in a feature stylesheet.

| Token | Role | Paper | Night |
|---|---|---|---|
| `--bg` | page | `#f3f6f9` | `#11202b` |
| `--surface` | cards | `#ffffff` | `#192d3b` |
| `--surface-2` | raised: grouped rows, the bar, chips | `#eaf0f5` | `#213c4d` |
| `--text` | main text | `#193348` | `#eef4f8` |
| `--muted` | helper text | `#5a6f7f` | `#adc3d2` |
| `--line` | hairlines (`--line-soft` is fainter) | `#dce5ec` | `#435e70` |
| `--accent` | brand: main action, links, current tab (design: primary) | `#245f7c` | `#9bcde3` |
| `--on-accent` | text on a solid accent fill | `#ffffff` | `#132d3b` |
| `--accent-soft` | accent container: selected, active pill, tonal (design: accent) | `#d8eaf2` | `#234656` |
| `--accent-ink` | text on accent-soft when accent is too loud | `#27576b` | `#c0e0ee` |
| `--soft` | quiet container: notes, flat cards | `#e7f0f5` | `#20394a` |
| `--good`, `--good-soft` | right, done | `#277654` / `#e8f4ec` | `#a0d7b3` / `#1c3c31` |
| `--bad`, `--bad-soft` | wrong, needs work | `#ad493c` / `#fbece7` | `#f1b2a3` / `#4a2e30` |
| `--gold`, `--gold-soft` | supporting notes (guideline update, mnemonic), never a grade | `#8a641d` / `#fbf1d9` | `#e3c581` / `#3d3624` |

Aliases in `parts.css` for code ported from the design reference:
`--primary` = `--accent`, `--on-primary` = `--on-accent`, `--good-bg`,
`--bad-bg`, `--gold-bg` = the `-soft` tokens. Careful: the reference's
`--accent` is our `--accent-soft`.

Themes: Paper and Night carry the redesign's batna-light / batna-dark
(Paper's muted is one step darker than the reference's `#5b7080` to pass AA
on `--surface-2`). True black keeps `#000000` page and cards with Night's
brand blue. Night shift keeps its amber accent: it must have no colour where
blue outweighs red. Theme ids never change. `theme.rs` tests every pairing
above against WCAG AA (4.5:1).

## Appearance: Calm and Glossy

A device preference (`prefs::Appearance`, `appearance=` in
`device-prefs.txt`, default Calm, never synced), picked in
Settings › Look › "نمط الواجهة". It sets `data-appearance="calm|glossy"`
on `.app`; only CSS reads it, so switching keeps every screen's state.

- Calm (default): solid surfaces, hairlines, 18px cards, 13px controls.
- Glossy (`assets/appearance.css`): tonal gradients from the theme's own
  tokens, a faint top highlight, pill buttons, 22px cards, a still sheen on
  `.hero`. Blur (18px) only on small floating surfaces (`.nav`,
  `.more-sheet`, `.sheet`); long lists get opaque gradients. Nothing
  animates on its own. Selected/right/wrong/disabled stay solid.
  `prefers-reduced-transparency` and `prefers-contrast: more` fall back to
  solid surfaces. On True black, `--glass-white` is black so pixels stay
  off. It does not take colours from the wallpaper; don't claim it does.

A feature that needs a Glossy variant of its own class writes
`.app[data-appearance="glossy"] .my-class { … }` in its own stylesheet,
from tokens only.

## Type, space, shape

Fonts are bundled (`fonts.rs`): IBM Plex Sans Arabic for the UI,
Plus Jakarta Sans for English (`bdi`, `--font-latin`). Sizes and spacing are
the existing rem tokens (`--fs-sm/md/lg/xl`, `--sp-1..6`, `--tap`,
`--tap-lg`, `--row`), so Settings › Look › interface size scales them all.
Radii: `--radius-control` 13px, `--radius-card` 18px (22px Glossy),
`--radius-sheet` 24px, `--radius-pill`. Touch targets: `--target` 48px.

## Shared classes

| Part | Markup | Notes |
|---|---|---|
| Buttons | `button.btn.btn-primary` / `.btn-tonal` / `.btn-secondary` / `.btn-ghost` / `.btn-good` / `.btn-danger`, `.btn-lg`, `.btn-block`, `.btn-link` | One `.btn-primary` per screen. |
| Icon button | `button.icon-button` with `aria-label` | 48dp touch area even when drawn smaller. |
| Page heading | `header.page-head > .page-eyebrow? + h1 + p.page-sub?` | Title ~22px, weight 600. |
| Section heading | `div.section-head > h2 + button.btn-ghost?` | The button opens the full list. |
| Status chip | `span.chip[data-tone=accent\|good\|bad\|gold]` | The word carries the state. |
| Card | `.card`, `.card-flat` | A card with one job is `card card-action` with its own `onclick`. |
| Hero | `.card.hero.card-action` (with onclick) or `button.hero`; its button `.btn-on-hero` | The one recommended next step. Test: `heroes_open_something`. |
| Stat tiles | `div.stat-tiles > button.tap.stat-tile > span.stat-num + span.stat-label` | Each opens what it counts. |
| Activity rows | `ul.tap-list > li > button.tap.tap-row > span.row-icon? + .tap-row-main + span.chev` | |
| Note | `.note[data-tone=gold\|good\|bad]?` | |
| Empty state | `div.empty > span.empty-icon + h2 + p + button.btn` | Say why and give the way forward. |
| Answer choice | `button.answer[data-state=selected\|correct\|wrong\|missed] > span.answer-key + span.answer-text + span.answer-mark` | Disabled after checking; `.answer-mark` says ✓/✗ in words or icon. |
| Counter | `span.num-ltr` | Any "3 / 12", %, count in Arabic text. |
| Progress | `div.progress[role=progressbar] > div.progress-fill` | `data-tone="good"` for done. |
| Rating row | `div.rating-row > button.rating[data-grade=again\|hard\|good\|easy] > span + span.rating-when` | Real intervals only. Content checks never sit here. |
| Sheet | `div.sheet-backdrop` + `div.sheet[role=dialog] > div.sheet-head` | Bottom sheet on phones, ≤430px box from 600px; scrolls inside. |
| Field | `label.field > span + input + span.field-hint? + span.field-error[role=alert]?`, `[data-invalid=true]` | Keep the typed text on errors. |
| Search | `label.search > svg + input[type=search]` | |
| Segmented | `div.segmented[role=radiogroup] > button[role=radio]` | |
| Switch | `label > … + input[type=checkbox][role=switch]` | |
| Choice list | `div.choice-list[role=radiogroup] > button.choice[role=radio]` | Settings-style pickers. |

No dead UI: anything that looks tappable opens what it shows.
`crates/app/src/dead_ends.rs` lists the tappable-looking classes; add a new
one there when you make it.
