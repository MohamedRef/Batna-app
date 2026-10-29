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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tab {
    Cases,
    CalcItems,
    Calculators,
}

// The tab and the open case or calc item, kept across screens so a case
// left half-way opens where it was.
static TAB: GlobalSignal<Tab> = Signal::global(|| Tab::Cases);
static OPEN: GlobalSignal<Option<String>> = Signal::global(|| None);

#[component]
pub fn CasesScreen() -> Element {
    use_hook(|| {
        if let Some(t) = START_TAB.write().take() {
            *TAB.write() = t;
            *OPEN.write() = None;
        }
    });
    let tab = move || *TAB.read();
    let back = move |_| *OPEN.write() = None;
    rsx! {
        style { {CSS.text()} }
        if let Some(id) = OPEN.read().clone() {
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
                        onclick: move |_| *TAB.write() = t,
                        {label}
                    }
                }
            }
            match tab() {
                Tab::Cases => rsx! { ItemList { kind: "case", on_open: move |id| *OPEN.write() = Some(id) } },
                Tab::CalcItems => rsx! { ItemList { kind: "calc", on_open: move |id| *OPEN.write() = Some(id) } },
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

/// A case in progress. Global, so leaving the screen half-way and coming
/// back finds the same step, selection and answers.
#[derive(Clone, Debug, PartialEq)]
struct CaseRun {
    item_id: String,
    /// Runs past the last index once the case is finished.
    step: usize,
    /// The option selected on this step but not yet checked (written index).
    draft: Option<usize>,
    /// The checked option on this step.
    picked: Option<usize>,
    /// A step without options: its answer is showing.
    shown: bool,
    /// One entry per finished step: right or not, and the option picked
    /// (written index; `None` when he graded himself).
    results: Vec<(bool, Option<usize>)>,
    started: i64,
}

impl CaseRun {
    fn new(item_id: &str) -> Self {
        CaseRun {
            item_id: item_id.to_owned(),
            step: 0,
            draft: None,
            picked: None,
            shown: false,
            results: Vec::new(),
            started: now_ms(),
        }
    }

    /// Select an option; a checked step is locked.
    fn select(&mut self, option: usize) {
        if self.picked.is_none() && self.results.len() == self.step {
            self.draft = Some(option);
        }
    }

    /// Check the selection against the step's answer, once.
    fn check(&mut self, answer: Option<usize>) -> bool {
        if self.picked.is_some() || self.results.len() != self.step {
            return false;
        }
        let Some(d) = self.draft.take() else { return false };
        self.picked = Some(d);
        self.results.push((answer == Some(d), Some(d)));
        true
    }

    /// A step without options, graded by himself.
    fn grade_self(&mut self, right: bool) {
        if self.results.len() == self.step {
            self.results.push((right, None));
        }
    }

    fn next(&mut self) {
        if self.results.len() > self.step {
            self.step += 1;
            self.draft = None;
            self.picked = None;
            self.shown = false;
        }
    }
}

static RUN: GlobalSignal<Option<CaseRun>> = Signal::global(|| None);

/// Brings a step of the case into view (the stepper's buttons).
fn scroll_to(id: &str) {
    document::eval(&format!(
        r#"document.getElementById("{id}")?.scrollIntoView({{behavior: "smooth", block: "start"}});"#
    ));
}

#[component]
pub fn CasePlayer(item_id: String, on_back: EventHandler) -> Element {
    let store = use_case_store();
    if RUN.peek().as_ref().is_none_or(|r| r.item_id != item_id) {
        *RUN.write() = Some(CaseRun::new(&item_id));
    }
    let Some(run) = RUN.read().clone() else { return rsx! {} };

    let Some((lecture, item)) = find_item(&item_id) else { return rsx! {} };
    let item = effective(store.as_ref(), item);
    let Body::Case { stem, steps } = &item.body else { return rsx! {} };
    let total = steps.len();
    let k = run.step;
    let done = k >= total;
    let answered = run.results.len() > k;
    let page = item.src.page;
    let record_id = item.id.clone();
    let step_answer = steps.get(k).and_then(|s| s.answer);
    // A fresh order on every attempt, steady while the step is open.
    let order = steps.get(k).map_or_else(Vec::new, |s| option_order(&s.options, run.started as u64, k));
    let selected = run.picked.or(run.draft);
    let edit = |f: &dyn Fn(&mut CaseRun)| {
        if let Some(r) = RUN.write().as_mut() {
            f(r);
        }
    };
    let leave = move |_| {
        // A finished case starts fresh next time; one half-way is kept.
        if RUN.peek().as_ref().is_some_and(|r| r.step >= total) {
            *RUN.write() = None;
        }
        on_back.call(());
    };

    rsx! {
        article { class: "player case",
            ItemMeta { lecture: lecture.to_owned(), item_id: item.id.clone(), on_back: leave }
            // Where he is in the case: each step done (✓/✗) opens its place
            // in the data, the current one its question.
            ol { class: "case-stepper", "aria-label": tx::STEPS_LABEL,
                for i in 0..total {
                    li { key: "{i}",
                        if i < k || (i == k && !done) {
                            button {
                                class: "case-stepper-step",
                                "data-state": match run.results.get(i) {
                                    _ if i == k => "now",
                                    Some((true, _)) => "ok",
                                    _ => "miss",
                                },
                                "aria-current": if i == k { "step" } else { "false" },
                                onclick: move |_| scroll_to(&if i == k { "case-q".to_string() } else { format!("case-step-{i}") }),
                                span { class: "ltr-count", "{i + 1}" }
                                match run.results.get(i) {
                                    Some((true, _)) if i < k => rsx! { span { aria_hidden: "true", "✓" } },
                                    Some(_) if i < k => rsx! { span { aria_hidden: "true", "✗" } },
                                    _ => rsx! { span { {tx::NOW} } },
                                }
                            }
                        } else {
                            span { class: "case-stepper-later", "data-state": if done { "ok" } else { "later" },
                                span { class: "ltr-count", "{i + 1}" }
                            }
                        }
                    }
                }
            }
            div { class: "case-grid",
                // What is known so far: the stem, every reveal up to this
                // step, and each finished step's question and answer.
                section { class: "card case-data",
                    h2 { class: "label", {tx::KNOWN} }
                    bdi { class: "case-stem", lang: "en", "{stem}" }
                    for (i, s) in steps.iter().enumerate().take(k + 1) {
                        div { key: "{i}", id: "case-step-{i}", class: "case-history",
                            if let Some(reveal) = &s.reveal {
                                div { class: if i == k && !done { "case-reveal new" } else { "case-reveal" },
                                    span { class: "small muted", "{tx::STEP} " span { class: "ltr-count", "{i + 1}" } if i == k && !done { " · {tx::NEW_DATA}" } }
                                    bdi { lang: "en", "{reveal}" }
                                }
                            }
                            if let Some((ok, pick)) = run.results.get(i).copied().filter(|_| i < k) {
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
                    section { class: "card case-step", key: "{k}", id: "case-q",
                        p { class: "small muted", "{tx::STEP} " span { class: "ltr-count", "{k + 1} / {total}" } }
                        bdi { class: "case-question", lang: "en", "{steps[k].question}" }
                        if !steps[k].options.is_empty() {
                            div { class: "options", role: "list",
                                for (pos, i) in order.iter().copied().enumerate() {
                                    button {
                                        key: "{i}",
                                        role: "listitem",
                                        dir: "auto",
                                        "aria-keyshortcuts": option_key(pos),
                                        "aria-pressed": if selected == Some(i) { "true" } else { "false" },
                                        class: option_class(i, run.picked, run.draft, steps[k].answer),
                                        disabled: run.picked.is_some(),
                                        onclick: move |_| edit(&|r| r.select(i)),
                                        span { class: "option-letter", aria_hidden: "true", {option_key(pos)} }
                                        bdi { "{steps[k].options[i]}" }
                                    }
                                }
                            }
                            if run.picked.is_none() {
                                if run.draft.is_none() {
                                    p { class: "muted small case-hint", {tx::PICK_FIRST} }
                                }
                                button {
                                    class: "btn btn-primary btn-lg case-check",
                                    disabled: run.draft.is_none(),
                                    "aria-keyshortcuts": "Enter",
                                    onclick: move |_| edit(&|r| { r.check(step_answer); }),
                                    {tx::CHECK}
                                }
                            }
                        } else if !run.shown {
                            button { class: "btn btn-primary", "aria-keyshortcuts": "Space", onclick: move |_| edit(&|r| r.shown = true), {tx::SHOW_ANSWER} }
                        }
                        if run.picked.is_some() || run.shown {
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
                                if let Some(p) = run.picked {
                                    p { class: if steps[k].answer == Some(p) { "verdict good" } else { "verdict bad" },
                                        if steps[k].answer == Some(p) { {tx::RIGHT} } else { {tx::WRONG} }
                                    }
                                }
                                div { class: "case-why",
                                    bdi { lang: "en", "{steps[k].why}" }
                                    BookPage { page, quote: item.src.quote.clone() }
                                }
                                if run.shown && !answered {
                                    div { class: "row",
                                        button { class: "btn btn-good", "aria-keyshortcuts": "3", onclick: move |_| edit(&|r| r.grade_self(true)), {tx::GOT_IT} }
                                        button { class: "btn btn-secondary", "aria-keyshortcuts": "1", onclick: move |_| edit(&|r| r.grade_self(false)), {tx::MISSED} }
                                    }
                                }
                                if answered {
                                    button {
                                        class: "btn btn-primary btn-lg",
                                        "aria-keyshortcuts": "Enter",
                                        onclick: {
                                            let record_id = record_id.clone();
                                            move |_| {
                                                let Some(r) = RUN.peek().clone() else { return };
                                                if r.step + 1 == total && r.results.len() == total {
                                                    let right = r.results.iter().filter(|x| x.0).count() as u32;
                                                    record(store.as_ref(), &record_id, right, total as u32, r.started);
                                                }
                                                edit(&|r| r.next());
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
                        recap: steps
                            .iter()
                            .zip(&run.results)
                            .map(|(s, &(ok, pick))| Recap {
                                ok,
                                question: s.question.clone(),
                                yours: pick.and_then(|p| s.options.get(p)).cloned(),
                                right: s.answer.filter(|_| !ok).and_then(|a| s.options.get(a)).cloned(),
                                why: s.why.clone(),
                            })
                            .collect::<Vec<_>>(),
                        update: item.update.as_ref().map(|u| format!("{} ({})", u.text, u.reference)),
                        on_again: {
                            let id = item_id.clone();
                            move |_| *RUN.write() = Some(CaseRun::new(&id))
                        },
                        on_back: move |_| leave(()),
                    }
                }
            }
        }
    }
}

/// One finished step in the recap.
#[derive(Clone, Debug, PartialEq)]
struct Recap {
    ok: bool,
    question: String,
    /// The option he picked; `None` when he graded himself.
    yours: Option<String>,
    /// The right option, when his was not.
    right: Option<String>,
    why: String,
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

fn option_class(
    i: usize,
    picked: Option<usize>,
    draft: Option<usize>,
    answer: Option<usize>,
) -> &'static str {
    match picked {
        None if draft == Some(i) => "option selected",
        None => "option",
        Some(_) if answer == Some(i) => "option right",
        Some(p) if p == i => "option wrong",
        Some(_) => "option dim",
    }
}

#[component]
fn Summary(
    recap: Vec<Recap>,
    update: Option<String>,
    on_again: EventHandler,
    on_back: EventHandler,
) -> Element {
    let right = recap.iter().filter(|r| r.ok).count();
    let total = recap.len();
    let message = match grade_for(right as u32, total as u32) {
        Grade::Good | Grade::Easy => tx::ALL_RIGHT,
        Grade::Hard => tx::SOME_RIGHT,
        Grade::Again => tx::FEW_RIGHT,
    };
    rsx! {
        section { class: "card case-summary",
            p { class: "case-score", bdi { class: "ltr-count", "{right} / {total}" } " " {tx::SCORE} }
            p { {message} }
            // The reasoning, step by step: the question, his answer, the
            // right one when his wasn't, and why.
            h2 { class: "label", {tx::RECAP} }
            ol { class: "case-steps-done",
                for (i, r) in recap.into_iter().enumerate() {
                    li { key: "{i}", class: if r.ok { "ok" } else { "miss" },
                        p { class: "case-done-q", dir: "ltr", lang: "en",
                            span { aria_hidden: "true", if r.ok { "✓ " } else { "✗ " } }
                            "{r.question}"
                        }
                        p { class: "small",
                            "{tx::YOURS} "
                            match r.yours {
                                Some(y) => rsx! { bdi { lang: "en", "{y}" } },
                                None => rsx! { if r.ok { {tx::GOT_IT} } else { {tx::MISSED} } },
                            }
                        }
                        if let Some(right) = r.right {
                            p { class: "small", "{tx::CORRECT_WAS} " bdi { lang: "en", "{right}" } }
                        }
                        p { class: "small muted", dir: "ltr", lang: "en", "{r.why}" }
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
        assert_eq!(option_class(0, None, None, Some(1)), "option");
        assert_eq!(option_class(0, None, Some(0), Some(1)), "option selected");
        assert_eq!(option_class(1, Some(0), None, Some(1)), "option right");
        assert_eq!(option_class(0, Some(0), None, Some(1)), "option wrong");
        assert_eq!(option_class(2, Some(0), None, Some(1)), "option dim");
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
    fn a_step_is_selected_then_checked_once() {
        let mut r = CaseRun::new("hb.L03.050");
        // Nothing selected: Check does nothing.
        assert!(!r.check(Some(1)));
        r.select(0);
        r.select(1);
        assert_eq!((r.draft, r.results.len()), (Some(1), 0));
        assert!(r.check(Some(1)));
        assert_eq!((r.picked, r.results.clone()), (Some(1), vec![(true, Some(1))]));
        // Locked: a second Check or a new selection changes nothing.
        assert!(!r.check(Some(1)));
        r.select(2);
        assert_eq!((r.picked, r.draft, r.results.len()), (Some(1), None, 1));
        r.next();
        assert_eq!((r.step, r.picked, r.draft), (1, None, None));
        // Next without an answer stays put.
        r.next();
        assert_eq!(r.step, 1);
        r.grade_self(false);
        r.grade_self(true);
        assert_eq!(r.results, vec![(true, Some(1)), (false, None)]);
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
