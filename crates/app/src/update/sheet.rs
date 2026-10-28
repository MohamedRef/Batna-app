//! The "what's new" sheet: what an update brings before it's installed,
//! what it brought right after (once, by itself), and the whole changelog
//! from Settings › Updates. One sheet for all three, laid out the
//! way Morphe Manager's update dialog is (ideas only, none of its code):
//!
//! - a head with the state ("فيه نسخة جديدة", "باطنة اتحدّثت") and
//!   `باطنة · v0.1.0 (412)`;
//! - a summary card: the installed build → the new one, how many builds
//!   that is, and a pill per kind (new, improved, fixed);
//! - a timeline: a rail with a dot per build (newest open), each build a
//!   row with its badge, date and counts, folding open into one card per
//!   kind that shows five lines and "Show all (n)" beyond six;
//! - "Earlier builds" under it, a few at a time;
//! - the download, install or restart button in the foot, so the whole
//!   update happens here.
//!
//! Adapted from kabar-app main: `crates/app/src/update/sheet.rs` (one
//! language, Batna's colours and buttons).

use batna_core::update::{BuildChanges, ChangeKind};
use dioxus::prelude::*;

use super::Phase;
use super::changelog::{self, between, civil, day_of, label, line, of_kind, tally};
use super::view::{UPDATE_CSS, megabytes, primary_action, use_update_status};
use crate::i18n::update as tx;

const SHEET_CSS: &str = r#"
.us-backdrop { position: fixed; inset: 0; z-index: 40; background: rgb(0 0 0 / 0.35); }
.us-sheet { position: fixed; z-index: 41; inset-inline: 0; inset-block-end: 0; max-block-size: 90dvh;
  display: flex; flex-direction: column; background: var(--surface); box-shadow: 0 -4px 24px rgb(0 0 0 / 0.18);
  border-start-start-radius: var(--radius-lg); border-start-end-radius: var(--radius-lg); }
@media (min-width: 600px) { .us-sheet { inset: 0; margin: auto; inline-size: min(36rem, 92vw);
  block-size: fit-content; max-block-size: 86dvh; border-radius: var(--radius-lg); } }
.us-head { display: grid; grid-template-columns: auto 1fr auto; gap: var(--sp-3); align-items: center;
  padding: var(--sp-4) var(--sp-4) var(--sp-2); }
.us-logo { inline-size: 2.6rem; block-size: 2.6rem; border-radius: 12px; display: grid; place-items: center;
  background: color-mix(in srgb, var(--good) 14%, var(--surface)); color: var(--good); }
.us-logo svg { inline-size: 1.5rem; block-size: 1.5rem; }
.us-head h2 { margin: 0; font-size: var(--fs-lg); line-height: 1.25; }
.us-head p { margin: 0; font-size: var(--fs-sm); color: var(--muted); }
.us-body { overflow-y: auto; padding: var(--sp-2) var(--sp-4) var(--sp-4); overscroll-behavior: contain; }
.us-foot { display: flex; flex-direction: column; gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-4); padding-block-end: max(var(--sp-3), env(safe-area-inset-bottom)); }
.us-foot button { inline-size: 100%; min-block-size: 48px; border-radius: var(--radius-pill); font-weight: 600; }
.us-summary { display: grid; justify-items: center; gap: var(--sp-3); text-align: center;
  padding: var(--sp-4) var(--sp-4); margin-block-end: var(--sp-4); border-radius: var(--radius-lg);
  background: var(--bg); border: 1px solid var(--line); }
.us-jump { display: flex; align-items: center; gap: var(--sp-2); flex-wrap: wrap; justify-content: center; }
.us-ver-badge { padding: 0.2rem 0.65rem; border-radius: var(--radius-sm); font-size: var(--fs-sm);
  background: color-mix(in srgb, var(--muted) 22%, var(--surface)); }
.us-ver-badge.to { background: var(--accent); color: var(--surface); font-weight: 600; }
.us-arrow { color: var(--muted); }
[dir="rtl"] .us-arrow, .app[dir="rtl"] .us-arrow { transform: scaleX(-1); }
.us-count { margin: 0; font-weight: 700; font-size: var(--fs-md); }
.us-pills { display: flex; flex-wrap: wrap; gap: var(--sp-2); justify-content: center; }
.us-pill { display: inline-flex; align-items: center; gap: 0.4rem; padding: 0.45rem 1rem;
  border-radius: var(--radius-pill); font-size: var(--fs-sm); font-weight: 600; }
.us-pill svg, .us-kind-icon svg { inline-size: 1rem; block-size: 1rem; }
[data-kind="new"] { --tone: var(--accent); }
[data-kind="improved"] { --tone: var(--good); }
[data-kind="fixed"] { --tone: color-mix(in srgb, var(--bad) 60%, var(--good)); }
.us-pill { background: color-mix(in srgb, var(--tone) 22%, var(--surface)); color: var(--tone); }
.us-summary progress { inline-size: 100%; accent-color: var(--good); }
.us-timeline { list-style: none; margin: 0; padding: 0; }
.us-release { position: relative; padding-inline-start: 1.6rem; padding-block-end: var(--sp-4); }
.us-release::before { content: ""; position: absolute; inset-inline-start: 0.45rem; inset-block: 1.1rem 0;
  inline-size: 2px; background: var(--line); }
.us-release:last-child::before { display: none; }
/* Nodes: a dashed ring for builds not installed yet, a green ring for the
   installed one, a small dot for the ones before it. */
.us-release::after { content: ""; position: absolute; inset-inline-start: 0.2rem; inset-block-start: 0.6rem;
  inline-size: 0.55rem; block-size: 0.55rem; border-radius: 50%; background: var(--muted); }
.us-release[data-ahead="true"]::after { inset-inline-start: 0.05rem; inset-block-start: 0.45rem;
  inline-size: 0.85rem; block-size: 0.85rem; box-sizing: border-box; background: var(--surface);
  border: 2px dashed var(--muted); }
.us-release[data-badge="installed"]::after { inset-inline-start: 0.05rem; inset-block-start: 0.45rem;
  inline-size: 0.85rem; block-size: 0.85rem; box-sizing: border-box; background: var(--surface);
  border: 3px solid var(--good); }
.us-release[data-badge="latest"], .us-release[data-badge="new"] { --node: color-mix(in srgb, var(--bad) 60%, var(--good)); }
.us-release[data-badge="installed"] { --node: var(--good); }
.us-rel-head { inline-size: 100%; display: grid; grid-template-columns: 1fr auto; gap: 0.15rem var(--sp-2);
  text-align: start; border: 0; background: transparent; padding: 0.2rem 0; color: inherit; }
.us-rel-title { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-2); font-weight: 700; }
.us-rel-meta { grid-column: 1; font-size: var(--fs-sm); color: var(--muted); }
.us-chevron { grid-row: 1 / span 2; grid-column: 2; align-self: center; color: var(--muted);
  transition: transform var(--dur) var(--ease); }
.us-rel-head[aria-expanded="true"] .us-chevron { transform: rotate(180deg); }
.us-badge { font-size: 0.75rem; font-weight: 600; padding: 0.1rem 0.55rem; border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--node, var(--muted)) 22%, var(--surface)); color: var(--node, var(--muted)); }
.us-kinds { display: grid; gap: var(--sp-2); margin-block-start: var(--sp-2); }
.us-kind { border: 1px solid var(--line); border-radius: var(--radius-lg); padding: var(--sp-3) var(--sp-4);
  background: var(--bg); }
.us-kind-head { display: flex; align-items: center; gap: var(--sp-3); margin-block-end: var(--sp-2); }
.us-kind-icon { inline-size: 1.9rem; block-size: 1.9rem; border-radius: var(--radius-sm); display: grid; place-items: center;
  background: color-mix(in srgb, var(--tone) 22%, var(--surface)); color: var(--tone); }
.us-kind-head h3 { margin: 0; font-size: var(--fs-md); font-weight: 600; }
.us-chip { margin-inline-start: auto; min-inline-size: 1.6rem; padding: 0.05rem 0.45rem; border-radius: var(--radius-sm);
  text-align: center; font-size: var(--fs-sm); background: color-mix(in srgb, var(--muted) 18%, var(--surface)); }
.us-items { list-style: none; margin: 0; padding: 0; display: grid; }
.us-items li { position: relative; padding-inline-start: 1.25rem; padding-block: 0.55rem; line-height: var(--lh); }
.us-items li + li { border-block-start: 1px solid var(--line-soft); }
.us-items li::before { content: ""; position: absolute; inset-inline-start: 0.2rem; inset-block-start: calc(0.55rem + 0.6em);
  inline-size: 0.35rem; block-size: 0.35rem; border-radius: 50%; background: var(--tone); }
.us-more { border: 0; background: transparent; color: var(--accent); padding: var(--sp-1) 0; font-weight: 600; }
.us-divider { padding-inline-start: 1.6rem; padding-block: 0 var(--sp-4); font-weight: 600; position: relative; }
.us-divider::before { content: ""; position: absolute; inset-inline-start: 0.45rem; inset-block: 0;
  inline-size: 2px; background: var(--line); }
@media (prefers-reduced-motion: reduce) { .us-chevron { transition: none; } }
"#;

/// Which sheet is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetMode {
    /// What the offered update brings (banner, card, notification).
    Offer,
    /// What the update just installed brought: opens once by itself.
    Updated,
    /// Every build so far (Settings › Updates).
    History,
}

/// Opens and closes the sheet; provided once in the shell.
#[derive(Clone, Copy)]
pub struct UpdateSheetNav(pub Signal<Option<SheetMode>>);

impl UpdateSheetNav {
    pub fn new() -> Self {
        Self(Signal::new(None))
    }

    pub fn open(mut self, mode: SheetMode) {
        self.0.set(Some(mode));
    }

    /// What the offer brings, or the changelog when nothing is offered
    /// right now (a notification tapped after Batna restarted).
    pub fn open_offer(self) {
        let offered = super::get().is_some_and(|u| {
            matches!(
                u.status().phase,
                Phase::Available(_)
                    | Phase::Downloading { .. }
                    | Phase::Ready(_)
                    | Phase::Installing(_)
                    | Phase::NeedsPermission(_)
                    | Phase::Installed(_)
            )
        });
        self.open(if offered { SheetMode::Offer } else { SheetMode::History });
    }
}

/// Lines a kind card shows before "Show all"; it folds only past `FOLD_AT`.
const PREVIEW: usize = 5;
const FOLD_AT: usize = 6;
/// Earlier builds shown at a time.
const EARLIER_STEP: usize = 4;
/// Builds the changelog lists before "Earlier builds".
const HISTORY_FIRST: usize = 6;

fn kind_label(k: ChangeKind) -> &'static str {
    match k {
        ChangeKind::New => tx::KIND_NEW,
        ChangeKind::Improved => tx::KIND_IMPROVED,
        ChangeKind::Fixed => tx::KIND_FIXED,
    }
}

fn kind_id(k: ChangeKind) -> &'static str {
    match k {
        ChangeKind::New => "new",
        ChangeKind::Improved => "improved",
        ChangeKind::Fixed => "fixed",
    }
}

/// Our own glyphs: a four-point spark, a rising line, a bug.
fn kind_icon(k: ChangeKind) -> Element {
    let path = match k {
        ChangeKind::New => "M12 3l1.9 5.6L19.5 10l-5.6 1.9L12 17.5l-1.9-5.6L4.5 10l5.6-1.4z",
        ChangeKind::Improved => "M4 16l5-5 4 4 7-7M15 8h5v5",
        ChangeKind::Fixed => {
            "M8 10h8v4a4 4 0 0 1-8 0zM9.5 10a2.5 2.5 0 0 1 5 0M12 10v8M4 13h4M16 13h4M5 8l3 2M19 8l-3 2M5 19l3-2M19 19l-3-2"
        }
    };
    let fill = if k == ChangeKind::New { "currentColor" } else { "none" };
    rsx! {
        svg { view_box: "0 0 24 24", fill: "{fill}", stroke: "currentColor", stroke_width: "2.2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "{path}" }
        }
    }
}

/// "Today", "2 days ago" within a week, then "28 أكتوبر".
fn when(published: &str, today: i64) -> Option<String> {
    let day = day_of(published)?;
    let ago = today - day;
    Some(match ago {
        ..=0 => tx::TODAY.to_owned(),
        1 => tx::YESTERDAY.to_owned(),
        2 => tx::TWO_DAYS_AGO.to_owned(),
        3..=6 => tx::DAYS_AGO.replace("{n}", &ago.to_string()),
        _ => {
            let (y, m, d) = civil(day);
            let (ty, _, _) = civil(today);
            let month = tx::MONTHS[m as usize - 1];
            if y == ty { format!("{d} {month}") } else { format!("{d} {month} {y}") }
        }
    })
}

/// Today's day number (UTC: close enough for "today" and "yesterday").
fn today() -> i64 {
    (super::now_ms() as i64).div_euclid(86_400_000)
}

/// The sheet; mounted once in the shell. Opens by itself once after an
/// update that brought something to tell.
#[component]
pub fn UpdateSheet() -> Element {
    let UpdateSheetNav(mut mode) = use_context::<UpdateSheetNav>();
    let status = use_update_status();
    let mut opened_after_update = use_signal(|| false);
    let mut open_builds = use_signal(Vec::<String>::new);
    let mut open_kinds = use_signal(Vec::<String>::new);
    let mut earlier_shown = use_signal(|| EARLIER_STEP);

    use_effect(move || {
        let has = status.read().as_ref().is_some_and(|s| !s.whats_new().is_empty());
        if has && !*opened_after_update.peek() && mode.peek().is_none() {
            opened_after_update.set(true);
            mode.set(Some(SheetMode::Updated));
        }
    });

    let Some(m) = *mode.read() else { return rsx! {} };
    let Some(s) = status.read().clone() else { return rsx! {} };
    let close = move |_| {
        if *mode.peek() == Some(SheetMode::Updated)
            && let Some(u) = super::get()
        {
            u.close_whats_new();
        }
        mode.set(None);
        open_builds.set(Vec::new());
        open_kinds.set(Vec::new());
        earlier_shown.set(EARLIER_STEP);
    };

    let offer = match &s.phase {
        Phase::Available(o)
        | Phase::Ready(o)
        | Phase::Installing(o)
        | Phase::NeedsPermission(o)
        | Phase::Installed(o)
        | Phase::Downloading { offer: o, .. } => Some(o.clone()),
        Phase::Failed { offer, .. } => offer.clone(),
        _ => None,
    };
    let current = s.current();
    let this_label = label(&s.version, s.build);
    let latest_key = s.history.first().map(BuildChanges::key);

    // What the timeline shows first, the jump the summary names, and what
    // goes under "Earlier builds".
    let (title, sub, top, from_label, to_label, earlier): (
        String,
        Option<String>,
        Vec<BuildChanges>,
        _,
        _,
        Vec<BuildChanges>,
    ) = match m {
        SheetMode::Offer => {
            let head = match &s.phase {
                Phase::Downloading { .. } => tx::DOWNLOADING,
                Phase::Ready(_) => tx::READY,
                Phase::Installing(_) => tx::INSTALLING,
                Phase::NeedsPermission(_) => tx::NEEDS_PERMISSION,
                Phase::Installed(_) => tx::INSTALLED,
                _ => tx::AVAILABLE,
            };
            let top = offer.as_ref().map(|o| o.changes.clone()).unwrap_or_default();
            let earlier = current
                .map(|c| between(&s.history, None, c).into_iter().cloned().collect())
                .unwrap_or_default();
            let to = offer.as_ref().map(|o| o.label());
            (head.to_owned(), None, top, Some(this_label.clone()), to, earlier)
        }
        SheetMode::Updated => {
            let top = s.whats_new();
            let from = s.prefs.whats_new_from.as_deref().map(|k| {
                let (v, b) = k.split_once('+').unwrap_or((k, "0"));
                label(v, b.parse().unwrap_or(0))
            });
            (
                tx::UPDATED_TITLE.to_owned(),
                Some(tx::UPDATED_SUB.to_owned()),
                top,
                from,
                Some(this_label.clone()),
                Vec::new(),
            )
        }
        SheetMode::History => {
            let (top, rest) = s.history.split_at(s.history.len().min(HISTORY_FIRST));
            (
                tx::HISTORY.to_owned(),
                Some(tx::HISTORY_SUB.to_owned()),
                top.to_vec(),
                None,
                None,
                rest.to_vec(),
            )
        }
    };
    let earlier: Vec<BuildChanges> =
        earlier.into_iter().filter(|e| !top.iter().any(|t| t.key() == e.key())).collect();
    let progress = match &s.phase {
        Phase::Downloading { offer, done } if m == SheetMode::Offer => Some((*done, offer.asset.size.max(1))),
        _ => None,
    };
    let action = if m == SheetMode::Offer { primary_action(&s.phase) } else { None };
    let today = today();
    let badge_of = |b: &BuildChanges| -> Option<(&'static str, &'static str)> {
        let o = changelog::order(&b.version, b.build);
        if Some(b.key()) == Some(format!("{}+{}", s.version, s.build)) {
            Some(("installed", tx::BADGE_INSTALLED))
        } else if latest_key.as_deref() == Some(b.key().as_str()) {
            Some(("latest", tx::BADGE_LATEST))
        } else if o.is_some() && o > current {
            Some(("new", tx::BADGE_NEW))
        } else {
            None
        }
    };
    // The newest build starts open; the learner's taps flip any build.
    let first_open = top.first().map(BuildChanges::key);
    let is_open = move |key: &str| {
        let flipped = open_builds.read().iter().any(|k| k == key);
        (first_open.as_deref() == Some(key)) != flipped
    };
    let summary_builds = if m == SheetMode::History { Vec::new() } else { top.clone() };
    let pills = tally(summary_builds.iter());
    let show_earlier = (*earlier_shown.read()).min(earlier.len());
    let shown: Vec<(bool, BuildChanges)> = top
        .iter()
        .cloned()
        .map(|b| (false, b))
        .chain(earlier.iter().take(show_earlier).cloned().map(|b| (true, b)))
        .collect();
    // As in Morphe, the installed build closes the main list and "Earlier
    // builds" starts after it.
    let this_key = format!("{}+{}", s.version, s.build);
    let first_earlier = shown.iter().position(|(e, b)| *e && b.key() != this_key);
    let more_earlier = earlier.len() > show_earlier;
    let empty = shown.is_empty();
    let fallback_notes = offer
        .as_ref()
        .filter(|o| m == SheetMode::Offer && o.changes.is_empty())
        .map(super::notes_of)
        .filter(|n| !n.is_empty());

    rsx! {
        style { {UPDATE_CSS} }
        style { {SHEET_CSS} }
        div { class: "us-backdrop", onclick: close }
        div { class: "us-sheet", role: "dialog", aria_modal: "true", aria_labelledby: "us-title",
            div { class: "us-head",
                span { class: "us-logo", aria_hidden: "true", {kind_icon(ChangeKind::New)} }
                div {
                    h2 { id: "us-title", "{title}" }
                    p {
                        {tx::SHEET_APP}
                        bdi { dir: "ltr", class: "latin",
                            "v{offer.as_ref().filter(|_| m == SheetMode::Offer).map_or(this_label.clone(), |o| o.label())}"
                        }
                    }
                }
                button { class: "icon-button", "data-back": "true", aria_label: "{tx::CLOSE}", onclick: close,
                    span { aria_hidden: "true", "✕" }
                }
            }
            div { class: "us-body",
                if let Some(sub) = sub {
                    p { class: "muted", "{sub}" }
                }
                if m != SheetMode::History && (from_label.is_some() || !pills.is_empty() || progress.is_some()) {
                    div { class: "us-summary",
                        if let (Some(f), Some(t)) = (from_label.clone(), to_label.clone()) {
                            div { class: "us-jump",
                                span { class: "us-ver-badge", bdi { dir: "ltr", "v{f}" } }
                                span { class: "us-arrow", aria_hidden: "true", "→" }
                                span { class: "us-ver-badge to", bdi { dir: "ltr", "v{t}" } }
                            }
                        }
                        if !top.is_empty() {
                            p { class: "us-count", "{tx::builds_count(top.len())}" }
                        }
                        if !pills.is_empty() {
                            div { class: "us-pills",
                                for (k, n) in pills {
                                    span { key: "{kind_id(k)}", class: "us-pill", "data-kind": "{kind_id(k)}",
                                        {kind_icon(k)}
                                        "{n} {kind_label(k)}"
                                    }
                                }
                            }
                        }
                        if let Some((done, total)) = progress {
                            progress { max: "{total}", value: "{done}" }
                            p { class: "muted", "{megabytes(done)} / {megabytes(total)}" }
                        }
                    }
                }
                if let Some(n) = fallback_notes {
                    p { class: "update-notes", "{n}" }
                }
                if empty {
                    p { class: "muted", {if m == SheetMode::History { tx::HISTORY_EMPTY } else { tx::NO_DETAILS }} }
                }
                ol { class: "us-timeline",
                    for (i, (is_earlier, b)) in shown.into_iter().enumerate() {
                        if Some(i) == first_earlier {
                            li { key: "{i}-earlier", class: "us-divider", role: "presentation", "{tx::EARLIER}" }
                        }
                        {
                            let key = b.key();
                            let open = is_open(&key);
                            let badge = badge_of(&b);
                            let ahead = current.is_some() && changelog::order(&b.version, b.build) > current;
                            let counts = tally([&b]);
                            let mut meta: Vec<String> = when(&b.published_at, today).into_iter().collect();
                            meta.extend(counts.iter().map(|(k, n)| format!("{n} {}", kind_label(*k))));
                            let meta = meta.join(" · ");
                            let toggle_key = key.clone();
                            let _ = is_earlier;
                            rsx! {
                                li { key: "{key}", class: "us-release", "data-badge": badge.map_or("", |b| b.0), "data-ahead": "{ahead}",
                                    button {
                                        class: "us-rel-head",
                                        aria_expanded: if open { "true" } else { "false" },
                                        onclick: move |_| {
                                            let mut v = open_builds.write();
                                            if let Some(i) = v.iter().position(|k| *k == toggle_key) {
                                                v.remove(i);
                                            } else {
                                                v.push(toggle_key.clone());
                                            }
                                        },
                                        span { class: "us-rel-title",
                                            bdi { dir: "ltr", class: "latin", "v{label(&b.version, b.build)}" }
                                            if let Some((_, u)) = badge {
                                                span { class: "us-badge", "{u}" }
                                            }
                                        }
                                        span { class: "us-rel-meta", "{meta}" }
                                        span { class: "us-chevron", aria_hidden: "true", "⌄" }
                                    }
                                    if open {
                                        div { class: "us-kinds",
                                            for k in ChangeKind::ALL {
                                                {
                                                    let items = of_kind(&b, k);
                                                    let kkey = format!("{key}:{}", kind_id(k));
                                                    let all = open_kinds.read().contains(&kkey);
                                                    let folds = items.len() > FOLD_AT;
                                                    let n = items.len();
                                                    let all_label = tx::SHOW_ALL.replace("{n}", &n.to_string());
                                                    let show: Vec<(String, bool)> = items
                                                        .into_iter()
                                                        .take(if folds && !all { PREVIEW } else { usize::MAX })
                                                        .map(line)
                                                        .collect();
                                                    rsx! {
                                                        if n > 0 {
                                                            section { key: "{kkey}", class: "us-kind", "data-kind": "{kind_id(k)}",
                                                                div { class: "us-kind-head",
                                                                    span { class: "us-kind-icon", {kind_icon(k)} }
                                                                    h3 { "{kind_label(k)}" }
                                                                    span { class: "us-chip", "{n}" }
                                                                }
                                                                ul { class: "us-items",
                                                                    for (j, (text, latin)) in show.into_iter().enumerate() {
                                                                        li { key: "{j}",
                                                                            if latin {
                                                                                bdi { dir: "ltr", class: "latin", "{text}" }
                                                                            } else {
                                                                                "{text}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                if folds {
                                                                    button {
                                                                        class: "us-more",
                                                                        onclick: move |_| {
                                                                            let mut v = open_kinds.write();
                                                                            if let Some(i) = v.iter().position(|x| *x == kkey) {
                                                                                v.remove(i);
                                                                            } else {
                                                                                v.push(kkey.clone());
                                                                            }
                                                                        },
                                                                        if all {
                                                                            "{tx::SHOW_LESS}"
                                                                        } else {
                                                                            "{all_label}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if more_earlier {
                    button { class: "us-more", onclick: move |_| earlier_shown += EARLIER_STEP, "{tx::SHOW_OLDER}" }
                }
            }
            div { class: "us-foot",
                if action.is_some() {
                    button { class: "btn btn-secondary", onclick: close, "{tx::LATER}" }
                }
                match action {
                    Some((label, run)) => rsx! {
                        button { class: "btn btn-primary", onclick: move |_| run(), "{label}" }
                    },
                    None => rsx! {
                        button { class: "btn btn-primary", onclick: close,
                            {if m == SheetMode::Updated { tx::GOT_IT } else { tx::CLOSE }}
                        }
                    },
                }
            }
        }
    }
}
