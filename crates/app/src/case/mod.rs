//! Cases: key-feature clinical cases, the `calc` number vignettes, and the
//! standalone calculators, as three tabs of one screen.
//!
//! A case shows its stem, then each step reveals new data and asks one
//! decision, with feedback (`why` and the book page) right after. The data
//! revealed so far stays on screen; from 840dp it sits beside the question.
//! A finished case is one review: its grade comes from the steps right.

use std::sync::Mutex;

use batna_core::content::{Body, Item, Status};
use batna_core::exam::{self, Rng};
use batna_core::review::{Grade, MemorySink, Origin, ReviewEvent, ReviewSink, StudyStoreHandle};
use dioxus::prelude::*;

use crate::calc::{Calculators, item::CalcItem};
use crate::content::Library;
use crate::hover::BookPage;
use crate::i18n::case as tx;
use crate::look::{Css, css};

const CSS: Css = css!("src/case/case.css");

/// The tab the screen opens on next, when Today sends him to due items.
static START_TAB: GlobalSignal<Option<Tab>> = Signal::global(|| None);

/// Open the Cases screen on its due items (#242): the cases tab, or the calc
/// items tab when only calc items are due. Due items sit first in each list.
pub fn open_due(mut ui: crate::Ui, calc_only: bool) {
    *START_TAB.write() = Some(if calc_only { Tab::CalcItems } else { Tab::Cases });
    ui.screen.set(crate::Screen::Cases);
}

/// Open the Cases screen on its Calculators tab (More › الحاسبات).
// Wired by the More sheet (more.rs, redesign shell PR); unused until then.
#[allow(dead_code)]
pub fn open_calculators(mut ui: crate::Ui) {
    *START_TAB.write() = Some(Tab::Calculators);
    ui.screen.set(crate::Screen::Cases);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tab {
    Cases,
    CalcItems,
    Calculators,
}

#[component]
pub fn CasesScreen() -> Element {
    let mut tab = use_signal(|| START_TAB.write().take().unwrap_or(Tab::Cases));
    let mut open = use_signal(|| None::<String>);
    let back = move |_| open.set(None);
    rsx! {
        style { {CSS.text()} }
        if let Some(id) = open() {
            if tab() == Tab::Cases {
                CasePlayer { key: "{id}", item_id: id, on_back: back }
            } else {
                CalcItem { key: "{id}", item_id: id, on_back: back }
            }
        } else {
            div { class: "tabs", role: "tablist",
                for (t, label) in [(Tab::Cases, tx::TAB_CASES), (Tab::CalcItems, tx::TAB_CALC_ITEMS), (Tab::Calculators, crate::i18n::case::TAB_CALCULATORS)] {
                    button {
                        key: "{t:?}",
                        role: "tab",
                        "aria-selected": if tab() == t { "true" } else { "false" },
                        onclick: move |_| tab.set(t),
                        {label}
                    }
                }
            }
            match tab() {
                Tab::Cases => rsx! { ItemList { kind: "case", on_open: move |id| open.set(Some(id)) } },
                Tab::CalcItems => rsx! { ItemList { kind: "calc", on_open: move |id| open.set(Some(id)) } },
                Tab::Calculators => rsx! { Calculators {} },
            }
        }
    }
}

/// The review store from `App`'s context. `None` outside the app (tests,
/// previews): answers then stay in memory.
pub fn use_case_store() -> Option<StudyStoreHandle> {
    use_hook(try_consume_context::<StudyStoreHandle>)
}

/// Items of one kind across every written lecture, in book order, without
/// the ones Mohamed marked ✗ and with his ✎ edits applied.
fn items_of(kind: &str, store: Option<&StudyStoreHandle>) -> Vec<(&'static str, Item)> {
    items_in_library(kind)
        .into_iter()
        .filter(|(_, i)| !store.is_some_and(|s| s.0.borrow().is_hidden(&i.id)))
        .map(|(l, i)| (l, effective(store, i)))
        .collect()
}

/// The item as Mohamed should see it: with his ✎ edit, if he made one.
pub fn effective(store: Option<&StudyStoreHandle>, item: &Item) -> Item {
    store.map_or_else(|| item.clone(), |s| s.0.borrow().effective(item))
}

fn items_in_library(kind: &str) -> Vec<(&'static str, &'static Item)> {
    let lib = Library::get();
    lib.index
        .lectures
        .iter()
        .filter_map(|e| lib.lectures.get(&e.id))
        .flat_map(|l| l.items.iter().map(move |i| (lib.title(&l.lecture), i)))
        .filter(|(_, i)| i.body.kind() == kind && i.status != Status::Rejected)
        .collect()
}

/// The lecture title and the item, by item id.
pub fn find_item(id: &str) -> Option<(&'static str, &'static Item)> {
    let lib = Library::get();
    let lecture = id.rsplit_once('.').map(|(l, _)| l)?;
    let lec = lib.lectures.get(lecture)?;
    lec.items.iter().find(|i| i.id == id).map(|i| (lib.title(&lec.lecture), i))
}

#[component]
fn ItemList(kind: &'static str, on_open: EventHandler<String>) -> Element {
    let store = use_case_store();
    let mut items = items_of(kind, store.as_ref());
    // Due ones first (#242); the sort is stable, so each part keeps book order.
    let due: std::collections::HashSet<String> =
        store.as_ref().map(|s| s.0.borrow().due(now_ms()).into_iter().collect()).unwrap_or_default();
    items.sort_by_key(|(_, i)| !due.contains(&i.id));
    let empty = if kind == "case" { tx::EMPTY } else { tx::EMPTY_CALC };
    rsx! {
        if kind == "case" { p { class: "muted", {tx::INTRO} } }
        if items.is_empty() {
            p { class: "card muted", {empty} }
        }
        ul { class: "case-list",
            for (lecture, item) in items {
                li { key: "{item.id}",
                    button { class: "case-row", dir: "auto", onclick: { let id = item.id.clone(); move |_| on_open.call(id.clone()) },
                        if due.contains(&item.id) {
                            span { class: "chip due case-due", {tx::DUE} }
                        }
                        bdi { class: "case-row-stem", {first_line(&item)} }
                        span { class: "muted small", dir: "rtl",
                            match &item.body {
                                Body::Case { steps, .. } => rsx! { "{steps.len()} {tx::STEPS} · " },
                                Body::Calc { calculator, .. } => rsx! { bdi { {batna_core::calc::spec(calculator).map_or("", |s| s.name)} } " · " },
                                _ => rsx! {},
                            }
                            bdi { "{lecture}" }
                        }
                    }
                }
            }
        }
    }
}

fn first_line(item: &Item) -> String {
    let text = match &item.body {
        Body::Case { stem, .. } | Body::Calc { stem, .. } => stem.as_str(),
        _ => "",
    };
    let mut out: String = text.chars().take(120).collect();
    if text.chars().count() > 120 {
        out.push('…');
    }
    out
}

/// The line above every player: back, lecture, and the review status.
#[component]
pub fn ItemMeta(lecture: String, item_id: String, on_back: EventHandler) -> Element {
    let unverified = find_item(&item_id).is_some_and(|(_, i)| i.status == Status::Unverified);
    rsx! {
        div { class: "player-meta",
            button { class: "btn-link", "aria-keyshortcuts": "Escape", onclick: move |_| on_back.call(()),
                span { aria_hidden: "true", "→ " }
                {tx::BACK}
            }
            bdi { class: "muted small", "{lecture}" }
            if unverified { span { class: "chip", {tx::UNVERIFIED} } }
        }
    }
}

#[component]
pub fn CasePlayer(item_id: String, on_back: EventHandler) -> Element {
    // `step` runs past the last index once the case is finished.
    let mut step = use_signal(|| 0usize);
    let mut picked = use_signal(|| None::<usize>);
    let mut shown = use_signal(|| false);
    // One entry per finished step: right or not, and the option picked
    // (written index; `None` when he graded himself).
    let mut results = use_signal(Vec::<(bool, Option<usize>)>::new);
    let mut started = use_signal(now_ms);
    let store = use_case_store();

    let Some((lecture, item)) = find_item(&item_id) else { return rsx! {} };
    let item = effective(store.as_ref(), item);
    let Body::Case { stem, steps } = &item.body else { return rsx! {} };
    let total = steps.len();
    let k = step();
    let done = k >= total;
    let answered = results.read().len() > k;
    let page = item.src.page;
    let record_id = item.id.clone();
    let step_answer = steps.get(k).and_then(|s| s.answer);
    // A fresh order on every attempt, steady while the step is open.
    let order = steps.get(k).map_or_else(Vec::new, |s| option_order(&s.options, started() as u64, k));

    let mut answer = move |right: bool, pick: Option<usize>| {
        if results.read().len() == step() {
            results.write().push((right, pick));
        }
    };

    rsx! {
        article { class: "player case",
            ItemMeta { lecture: lecture.to_owned(), item_id: item.id.clone(), on_back }
            div { class: "case-grid",
                // What is known so far: the stem, every reveal up to this
                // step, and each finished step's question and answer.
                section { class: "card case-data",
                    bdi { class: "case-stem", "{stem}" }
                    for (i, s) in steps.iter().enumerate().take(k + 1) {
                        div { key: "{i}", class: "case-history",
                            if let Some(reveal) = &s.reveal {
                                div { class: if i == k { "case-reveal new" } else { "case-reveal" },
                                    span { class: "small muted", "{tx::STEP} {i + 1}" if i == k { " · {tx::NEW_DATA}" } }
                                    bdi { "{reveal}" }
                                }
                            }
                            if let Some((ok, pick)) = results.read().get(i).copied().filter(|_| i < k) {
                                div { class: if ok { "case-done ok" } else { "case-done miss" },
                                    bdi { class: "small case-done-q", dir: "ltr", "{s.question}" }
                                    match pick.and_then(|p| s.options.get(p)) {
                                        Some(o) => rsx! {
                                            p { class: "small", dir: "ltr",
                                                span { aria_hidden: "true", if ok { "✓ " } else { "✗ " } }
                                                "{o}"
                                            }
                                        },
                                        None => rsx! {
                                            p { class: "small",
                                                span { aria_hidden: "true", if ok { "✓ " } else { "✗ " } }
                                                if ok { {tx::GOT_IT} } else { {tx::MISSED} }
                                            }
                                        },
                                    }
                                    if let Some(right) = s.answer.filter(|_| !ok).and_then(|a| s.options.get(a)) {
                                        p { class: "small", "{tx::CORRECT_WAS} " bdi { "{right}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                if !done {
                    section { class: "card case-step", key: "{k}",
                        p { class: "small muted", "{tx::STEP} {k + 1} {tx::OF} {total}" }
                        bdi { class: "case-question", "{steps[k].question}" }
                        if !steps[k].options.is_empty() {
                            div { class: "options", role: "list",
                                for (pos, i) in order.iter().copied().enumerate() {
                                    button {
                                        key: "{i}",
                                        role: "listitem",
                                        dir: "auto",
                                        "aria-keyshortcuts": option_key(pos),
                                        class: option_class(i, picked(), steps[k].answer),
                                        disabled: picked().is_some(),
                                        onclick: move |_| {
                                            picked.set(Some(i));
                                            answer(step_answer == Some(i), Some(i));
                                        },
                                        bdi { "{steps[k].options[i]}" }
                                    }
                                }
                            }
                        } else if !shown() {
                            button { class: "btn btn-primary", "aria-keyshortcuts": "Space", onclick: move |_| shown.set(true), {tx::SHOW_ANSWER} }
                        }
                        if picked().is_some() || shown() {
                            // Brought into view on the phone, where it would
                            // otherwise open below the fold.
                            div {
                                class: "case-feedback",
                                onmounted: move |e: MountedEvent| async move {
                                    let _ = e
                                        .scroll_to_with_options(ScrollToOptions {
                                            behavior: ScrollBehavior::Smooth,
                                            vertical: ScrollLogicalPosition::Nearest,
                                            horizontal: ScrollLogicalPosition::Nearest,
                                        })
                                        .await;
                                },
                                if let Some(p) = picked() {
                                    p { class: if steps[k].answer == Some(p) { "verdict good" } else { "verdict bad" },
                                        if steps[k].answer == Some(p) { {tx::RIGHT} } else { {tx::WRONG} }
                                    }
                                }
                                div { class: "case-why",
                                    bdi { "{steps[k].why}" }
                                    BookPage { page, quote: item.src.quote.clone() }
                                }
                                if shown() && !answered {
                                    div { class: "row",
                                        button { class: "btn btn-good", "aria-keyshortcuts": "3", onclick: move |_| answer(true, None), {tx::GOT_IT} }
                                        button { class: "btn btn-secondary", "aria-keyshortcuts": "1", onclick: move |_| answer(false, None), {tx::MISSED} }
                                    }
                                }
                                if answered {
                                    button {
                                        class: "btn btn-primary",
                                        "aria-keyshortcuts": "Enter",
                                        onclick: {
                                            let record_id = record_id.clone();
                                            move |_| {
                                                if k + 1 == total {
                                                    let right = results.read().iter().filter(|r| r.0).count() as u32;
                                                    record(store.as_ref(), &record_id, right, total as u32, started());
                                                }
                                                step.set(k + 1);
                                                picked.set(None);
                                                shown.set(false);
                                            }
                                        },
                                        if k + 1 == total { {tx::FINISH} } else { {tx::NEXT} }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    Summary {
                        results: results().iter().map(|r| r.0).collect::<Vec<_>>(),
                        steps: steps.iter().map(|s| (s.question.clone(), s.why.clone())).collect::<Vec<_>>(),
                        update: item.update.as_ref().map(|u| format!("{} ({})", u.text, u.reference)),
                        on_again: move |_| {
                            step.set(0);
                            picked.set(None);
                            shown.set(false);
                            results.set(Vec::new());
                            started.set(now_ms());
                        },
                        on_back,
                    }
                }
            }
        }
    }
}

/// The order a step's options are shown in, as written indices. Shuffled
/// per attempt (`seed`) so the key isn't always "A"; steps whose options
/// refer to each other ("All of these", "A and C") keep their order.
fn option_order(options: &[String], seed: u64, step: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..options.len()).collect();
    if exam::shufflable(options) {
        Rng::new(seed ^ (step as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)).shuffle(&mut order);
    }
    order
}

/// "A"–"E": the key that picks the option shown at `pos`.
fn option_key(pos: usize) -> &'static str {
    ["A", "B", "C", "D", "E", "F"].get(pos).copied().unwrap_or("")
}

fn option_class(i: usize, picked: Option<usize>, answer: Option<usize>) -> &'static str {
    match picked {
        None => "option",
        Some(_) if answer == Some(i) => "option right",
        Some(p) if p == i => "option wrong",
        Some(_) => "option dim",
    }
}

#[component]
fn Summary(
    results: Vec<bool>,
    /// Each step's question and explanation.
    steps: Vec<(String, String)>,
    update: Option<String>,
    on_again: EventHandler,
    on_back: EventHandler,
) -> Element {
    let right = results.iter().filter(|r| **r).count();
    let total = results.len();
    let message = match grade_for(right as u32, total as u32) {
        Grade::Good | Grade::Easy => tx::ALL_RIGHT,
        Grade::Hard => tx::SOME_RIGHT,
        Grade::Again => tx::FEW_RIGHT,
    };
    rsx! {
        section { class: "card case-summary",
            p { class: "case-score", bdi { dir: "ltr", "{right} / {total}" } " " {tx::SCORE} }
            p { {message} }
            ol { class: "case-steps-done", dir: "ltr",
                for (i, (ok, (question, why))) in results.iter().zip(steps.iter()).enumerate() {
                    li { key: "{i}", class: if *ok { "ok" } else { "miss" },
                        p { class: "case-done-q",
                            span { aria_hidden: "true", if *ok { "✓ " } else { "✗ " } }
                            "{question}"
                        }
                        p { class: "small muted", "{why}" }
                    }
                }
            }
            if let Some(u) = update { p { class: "small update", "{tx::UPDATE}: " bdi { "{u}" } } }
            div { class: "row",
                button { class: "btn btn-secondary", "aria-keyshortcuts": "Space", onclick: move |_| on_again.call(()), {tx::AGAIN} }
                button { class: "btn btn-primary", "aria-keyshortcuts": "Enter", onclick: move |_| on_back.call(()), {tx::BACK} }
            }
        }
    }
}

/// One review per case: every step right is Good, at least half is Hard,
/// fewer is Again. Easy is left to the self-rating screens.
pub fn grade_for(right: u32, total: u32) -> Grade {
    if total == 0 || right * 2 < total {
        Grade::Again
    } else if right < total {
        Grade::Hard
    } else {
        Grade::Good
    }
}

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// Answers given outside the app (no store in context) wait here.
static EVENTS: Mutex<MemorySink> = Mutex::new(MemorySink { events: Vec::new() });

/// One finished case or calc item as one review event in the store.
pub fn record(store: Option<&StudyStoreHandle>, item_id: &str, right: u32, total: u32, started_ms: i64) {
    let at_ms = now_ms();
    let event = ReviewEvent {
        item_id: item_id.to_owned(),
        grade: grade_for(right, total),
        origin: Origin::Case,
        at_ms,
        took_ms: u32::try_from(at_ms.saturating_sub(started_ms)).unwrap_or(u32::MAX),
    };
    match store {
        Some(s) => s.0.borrow_mut().record(event),
        None => {
            if let Ok(mut sink) = EVENTS.lock() {
                sink.record(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_from_steps_right() {
        assert_eq!(grade_for(3, 3), Grade::Good);
        assert_eq!(grade_for(2, 3), Grade::Hard);
        assert_eq!(grade_for(1, 2), Grade::Hard);
        assert_eq!(grade_for(1, 3), Grade::Again);
        assert_eq!(grade_for(0, 0), Grade::Again);
    }

    #[test]
    fn options_are_marked_after_a_pick() {
        assert_eq!(option_class(0, None, Some(1)), "option");
        assert_eq!(option_class(1, Some(0), Some(1)), "option right");
        assert_eq!(option_class(0, Some(0), Some(1)), "option wrong");
        assert_eq!(option_class(2, Some(0), Some(1)), "option dim");
    }

    fn opts(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| (*x).to_owned()).collect()
    }

    // Issue #86: the key was "A" in 61% of steps because options were
    // never shuffled.
    #[test]
    fn options_are_shuffled_per_attempt() {
        let o = opts(&["Propranolol", "Carvedilol", "Band ligation", "TIPS", "Octreotide"]);
        let a = option_order(&o, 1, 0);
        assert_eq!(a, option_order(&o, 1, 0), "steady while the step is open");
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, [0, 1, 2, 3, 4]);
        // Across attempts the written first option lands in every position.
        let firsts: std::collections::BTreeSet<usize> =
            (0..200u64).map(|seed| option_order(&o, seed, 0).iter().position(|&i| i == 0).unwrap()).collect();
        assert_eq!(firsts.len(), 5);
        // "All of the above" steps keep their written order.
        let fixed = opts(&["a", "b", "c", "All of the above"]);
        assert_eq!(option_order(&fixed, 7, 0), [0, 1, 2, 3]);
    }

    #[test]
    fn record_keeps_the_case_as_one_review() {
        record(None, "hb.L03.050", 2, 3, now_ms() - 1500);
        let sink = EVENTS.lock().unwrap();
        let e = sink.events.iter().find(|e| e.item_id == "hb.L03.050").unwrap();
        assert_eq!(e.grade, Grade::Hard);
        assert_eq!(e.origin, Origin::Case);
        assert!(e.took_ms >= 1500);
    }
}
