//! Study: a short session of mixed items from the lectures Mohamed picks,
//! with feedback right after each answer (SPLIT.md chunk 2).
//!
//! Setup → items one by one → summary. The order comes from
//! `batna_core::session::compose`; answers go to the review store.

mod backend;
mod card;
mod cloze;
mod enumerate;
mod essay;
pub mod grade;
mod item;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Instant;

use batna_core::content::{Item, Lecture};
use batna_core::review::{Grade, Origin, ReviewEvent};
use batna_core::session::{Inputs, Planned, Session, compose};
use dioxus::prelude::*;

pub(crate) use backend::use_store;
pub use backend::{StudyStore, StudyStoreHandle};
pub use item::{ItemView, PLAYABLE};

use crate::content::{Derived, Library};
use crate::i18n::count::{DAY, HOUR, ITEM, MINUTE, REVIEW, count};
use crate::i18n::study as tx;
use crate::look::{Css, css};
use backend::now_ms;
use item::short_text;

pub(crate) const STUDY_CSS: Css = css!("assets/study.css");
const MINUTES: [u32; 4] = [5, 10, 15, 20];

/// A request to start a session straight away (Today's "start" button sets
/// it, then switches to the Study screen).
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    /// Lecture ids (`hb.L01`); `None` means every ready lecture.
    pub lectures: Option<Vec<String>>,
    /// Exactly these item ids, in this order ("study these", a session from
    /// mistakes). Overrides `lectures` and the time budget; ids the player
    /// can't show or that are hidden are skipped.
    pub items: Option<Vec<String>>,
    pub minutes: u32,
}

pub static START: GlobalSignal<Option<Start>> = Signal::global(|| None);

/// A START that came while a session was running, waiting for him to choose
/// between adding it to the session, starting it instead, or dropping it.
static PENDING: GlobalSignal<Option<Start>> = Signal::global(|| None);

/// Every item by id, with its lecture.
fn items() -> &'static HashMap<String, (&'static Lecture, &'static Item)> {
    static MAP: Derived<HashMap<String, (&'static Lecture, &'static Item)>> = Derived::new();
    MAP.get(|lib| {
        lib.lectures.values().flat_map(|l| l.items.iter().map(move |i| (i.id.clone(), (l, i)))).collect()
    })
}

/// The name a lecture shows under: the book's title, or "your cards".
fn lecture_title(l: &Lecture) -> String {
    if l.lecture == batna_core::own::LECTURE {
        crate::i18n::own::TITLE.to_string()
    } else {
        Library::get().title(&l.lecture).to_string()
    }
}

/// Whether the study player can show this item (MCQs go to the bank, cases
/// to their own screen).
pub fn is_playable(id: &str) -> bool {
    find(id).is_some_and(|(_, i)| PLAYABLE.contains(&i.body.kind()))
}

/// A book item or one of his own cards, by id.
fn find(id: &str) -> Option<(&'static Lecture, &'static Item)> {
    items().get(id).copied().or_else(|| crate::own::find(id))
}

/// Lectures that have at least one item this player can show, then his own
/// cards' deck when he has any.
fn ready_lectures() -> Vec<&'static Lecture> {
    Library::get()
        .lectures
        .values()
        .filter(|l| l.items.iter().any(|i| PLAYABLE.contains(&i.body.kind())))
        .chain(crate::own::lecture())
        .collect()
}

fn plan(
    store: &StudyStoreHandle,
    lectures: &[&Lecture],
    topics: &HashSet<String>,
    minutes: u32,
) -> Vec<Planned> {
    let now = now_ms();
    let s = store.0.borrow();
    let all = lectures.iter().flat_map(|l| l.items.iter());
    let seen: HashSet<String> = all.clone().filter(|i| !s.is_new(&i.id)).map(|i| i.id.clone()).collect();
    let hidden: HashSet<String> = all.filter(|i| s.is_hidden(&i.id)).map(|i| i.id.clone()).collect();
    compose(&Inputs {
        lectures,
        topics,
        kinds: PLAYABLE,
        due: &s.due(now),
        seen: &seen,
        hidden: &hidden,
        max_new: crate::plan::new_left_today(&*s, now),
        budget_secs: minutes * 60,
    })
}

/// A session on exactly the given items, in order, skipping duplicates and
/// anything this player can't show or the learner hid.
fn plan_items(store: &StudyStoreHandle, ids: &[String]) -> Vec<Planned> {
    let s = store.0.borrow();
    let mut seen = HashSet::new();
    ids.iter()
        .filter(|id| seen.insert(id.as_str()))
        .filter_map(|id| find(id))
        .filter(|(_, i)| PLAYABLE.contains(&i.body.kind()) && !s.is_hidden(&i.id))
        .map(|(l, i)| Planned {
            item_id: i.id.clone(),
            lecture: l.lecture.clone(),
            topic: i.topic.as_ref().map_or_else(|| l.lecture.clone(), |t| format!("{}/{t}", l.lecture)),
            est_secs: batna_core::session::estimate_secs(&i.body),
            fresh: s.is_new(&i.id),
        })
        .collect()
}

static SESSION: GlobalSignal<Session> = Signal::global(Session::default);
static PHASE: GlobalSignal<Phase> = Signal::global(|| Phase::Setup);
static STARTED: GlobalSignal<Instant> = Signal::global(Instant::now);

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Setup,
    Running,
    Summary,
}

#[component]
pub fn Study() -> Element {
    let store = use_store();
    // His own cards' deck changes with the review store (a save, an import, a sync).
    let _ = crate::store::use_review().revision.read();
    let ready = ready_lectures();
    let chosen = use_signal(|| ready.iter().map(|l| l.lecture.clone()).collect::<BTreeSet<String>>());
    let topics = use_signal(HashSet::<String>::new);
    let mut minutes = use_signal(crate::onboarding::session_minutes);
    // Global, so a session survives switching tabs (like Bank and Oral).
    let mut session = SESSION.signal();
    let mut phase = PHASE.signal();
    let mut started = STARTED.signal();
    // Coming back to a running session: don't count the time away.
    let mut quiet = use_hook(|| {
        if *phase.peek() == Phase::Running {
            started.set(Instant::now());
        }
        Signal::new(*phase.peek() == Phase::Summary)
    });
    // The calm end-of-session tone, however the session ended (not again when
    // coming back to a summary already shown).
    use_effect(move || {
        let ended = phase() == Phase::Summary;
        if *quiet.peek() {
            quiet.set(false);
            return;
        }
        if ended && session.peek().summary().items > 0 {
            crate::sounds::play(crate::sounds::Cue::SessionEnd);
        }
    });

    let mut begin = {
        let store = store.clone();
        move |lectures: Vec<&'static Lecture>, topics: HashSet<String>, mins: u32| {
            let planned = plan(&store, &lectures, &topics, mins);
            let empty = planned.is_empty();
            session.set(Session::new(planned));
            started.set(Instant::now());
            phase.set(if empty { Phase::Summary } else { Phase::Running });
        }
    };

    // Starts what a START request asks for, replacing whatever is shown.
    let apply = {
        let store = store.clone();
        let begin = begin.clone();
        move |req: Start| {
            if let Some(ids) = &req.items {
                let planned = plan_items(&store, ids);
                let empty = planned.is_empty();
                session.set(Session::new(planned));
                started.set(Instant::now());
                phase.set(if empty { Phase::Summary } else { Phase::Running });
                return;
            }
            let picked: Vec<&Lecture> = ready_lectures()
                .into_iter()
                .filter(|l| req.lectures.as_ref().is_none_or(|ids| ids.contains(&l.lecture)))
                .collect();
            minutes.set(req.minutes);
            let mut begin = begin.clone();
            begin(picked, HashSet::new(), req.minutes);
        }
    };
    let mut pending = PENDING.signal();
    // The question only makes sense while that session runs.
    use_effect(move || {
        if phase() != Phase::Running && pending.peek().is_some() {
            pending.set(None);
        }
    });

    // Today's "start a session" button, and any "study these" asked for
    // while Study is already open (a book page's sheet, a lecture page over
    // it): an effect, so a request made here is not left waiting. A session
    // in progress is never dropped without asking (#319): the request waits
    // in PENDING for his answer.
    use_effect({
        let mut apply = apply.clone();
        move || {
            if START.read().is_none() {
                return;
            }
            if let Some(req) = START.write().take() {
                let live = *phase.peek() == Phase::Running && !session.peek().is_done();
                if live {
                    pending.set(Some(req));
                } else {
                    apply(req);
                }
            }
        }
    });

    let body = match phase() {
        Phase::Setup if ready.is_empty() => rsx! {
            section { class: "card",
                h1 { {tx::TITLE} }
                p { {tx::EMPTY_LIBRARY} }
            }
            crate::own::OwnCardsCard { on_study: move |_| {} }
        },
        Phase::Setup => {
            let picked: Vec<&Lecture> =
                ready.iter().copied().filter(|l| chosen.read().contains(&l.lecture)).collect();
            let preview = plan(&store, &picked, &topics.read(), minutes());
            let mut begin_own = begin.clone();
            rsx! {
                Setup {
                    ready: ready.iter().map(|l| (l.lecture.clone(), lecture_title(l))).collect::<Vec<_>>(),
                    chosen,
                    topics,
                    minutes,
                    preview_count: preview.len(),
                    preview_new: preview.iter().filter(|p| p.fresh).count(),
                    preview_mins: preview.iter().map(|p| p.est_secs).sum::<u32>().div_ceil(60),
                    on_start: move |_| begin(picked.clone(), topics.read().clone(), minutes()),
                }
                crate::own::OwnCardsCard {
                    on_study: move |_| begin_own(crate::own::lecture().into_iter().collect(), HashSet::new(), minutes()),
                }
            }
        }
        Phase::Running => {
            let s = session.read();
            let (pos, len) = s.progress();
            let current = s.current().cloned();
            let retry = s.is_retry();
            drop(s);
            match current.and_then(|p| find(&p.item_id).map(|(l, i)| (p, l, i))) {
                // Only when content changed under a running session; "end" still works.
                None => rsx! {
                    button { class: "btn btn-primary btn-lg", onclick: move |_| phase.set(Phase::Summary), {tx::END} }
                },
                Some((p, lecture, item)) => {
                    let item = store.0.borrow().effective(item);
                    let topic_title = item
                        .topic
                        .as_ref()
                        .and_then(|t| lecture.topics.iter().find(|x| &x.id == t))
                        .map(|t| t.title.clone());
                    let store_for_add = store.clone();
                    let store = store.clone();
                    let item_id = p.item_id.clone();
                    let on_grade = move |grade: Grade| {
                        let took = started().elapsed().as_millis().min(u32::MAX as u128) as u32;
                        store.0.borrow_mut().record(ReviewEvent {
                            item_id: item_id.clone(),
                            grade,
                            origin: Origin::Study,
                            at_ms: now_ms(),
                            took_ms: took,
                        });
                        let hidden = |id: &str| store.0.borrow().is_hidden(id);
                        session.write().grade_unless(grade, hidden);
                        started.set(Instant::now());
                        if session.read().is_done() {
                            phase.set(Phase::Summary);
                        } else {
                            crate::sounds::play(if grade == Grade::Again {
                                crate::sounds::Cue::Wrong
                            } else {
                                crate::sounds::Cue::Right
                            });
                        }
                    };
                    rsx! {
                        div { class: "st-progress",
                            div { class: "st-bar", role: "progressbar",
                                aria_valuemin: "0", aria_valuemax: "{len}", aria_valuenow: "{pos}",
                                div { class: "st-bar-fill", style: "inline-size: {pos * 100 / len.max(1)}%" }
                            }
                            span { class: "muted small", "{pos + 1} {tx::OF} {len}" }
                            button {
                                class: "btn-link",
                                "aria-keyshortcuts": "Escape",
                                onclick: move |_| phase.set(Phase::Summary),
                                {tx::END}
                            }
                        }
                        if let Some(req) = pending() {
                            PendingCard {
                                req,
                                on_add: {
                                    move |ids: Vec<String>| {
                                        let more = plan_items(&store_for_add, &ids);
                                        session.write().append(more);
                                        pending.set(None);
                                    }
                                },
                                on_replace: {
                                    let mut apply = apply.clone();
                                    move |req: Start| {
                                        pending.set(None);
                                        apply(req);
                                    }
                                },
                                on_keep: move |_| pending.set(None),
                            }
                        }
                        // Keyed so every item (and a retry of the same one) starts fresh.
                        for (item , topic_title , on_grade) in std::iter::once((item, topic_title, on_grade)) {
                            ItemView {
                                key: "{pos}-{p.item_id}",
                                item,
                                lecture_title: lecture_title(lecture),
                                topic_title,
                                fresh: p.fresh,
                                retry,
                                on_grade,
                            }
                        }
                    }
                }
            }
        }
        Phase::Summary => {
            let summary = session.read().summary();
            let next = store.0.borrow().next_due_ms(now_ms());
            let shaky: Vec<(String, String)> = summary
                .shaky
                .iter()
                .filter_map(|id| find(id).map(|(_, i)| (id.clone(), short_text(i))))
                .collect();
            rsx! {
                SummaryView {
                    items: summary.items,
                    right: summary.right,
                    fresh: summary.fresh,
                    shaky,
                    next: next.map(|t| when_label(now_ms(), t, &crate::store::zone())),
                    on_again: move |_| phase.set(Phase::Setup),
                }
            }
        }
    };

    rsx! {
        style { {STUDY_CSS.text()} }
        div { class: "st", {body} }
    }
}

#[component]
fn Setup(
    ready: Vec<(String, String)>,
    chosen: Signal<BTreeSet<String>>,
    topics: Signal<HashSet<String>>,
    minutes: Signal<u32>,
    preview_count: usize,
    preview_new: usize,
    preview_mins: u32,
    on_start: EventHandler<()>,
) -> Element {
    // Topic chips only when one lecture is picked: across lectures they'd be a wall.
    let single = {
        let c = chosen.read();
        if c.len() == 1 { c.iter().next().and_then(|id| Library::get().lectures.get(id)) } else { None }
    };
    let reviews = preview_count - preview_new;
    let all_ids: Vec<String> = ready.iter().map(|(id, _)| id.clone()).collect();
    let all_chosen = chosen.read().len() == all_ids.len();
    rsx! {
        section { class: "card st-setup",
            // Tablet landscape and the Mac: the time and Start stay in view
            // beside the lecture and topic picker.
            div { class: "tab-cols", "data-wide": "aside",
                div { class: "tab-main tab-sticky st-setup-go",
                    h1 { {tx::TITLE} }
                    p { class: "muted", {tx::SETUP_INTRO} }

                    h2 { {tx::TIME} }
                    div { class: "segmented", role: "radiogroup", aria_label: tx::TIME,
                        for m in MINUTES {
                            button {
                                key: "{m}",
                                role: "radio",
                                "aria-checked": if minutes() == m { "true" } else { "false" },
                                onclick: move |_| minutes.set(m),
                                {count(m, &MINUTE)}
                            }
                        }
                    }

                    if preview_count == 0 {
                        p { class: "note", {tx::NOTHING_NOW} }
                    } else {
                        p { class: "st-preview",
                            {count(preview_count, &ITEM)} " · " {count(reviews, &REVIEW)} " · {preview_new} {tx::NEW} · {tx::ABOUT} " {count(preview_mins, &MINUTE)}
                        }
                        button { class: "btn btn-primary btn-lg st-start", onclick: move |_| on_start.call(()), {tx::START} }
                    }
                }
                div { class: "tab-aside st-setup-pick",
                    div { class: "st-section-head",
                        h2 { {tx::LECTURES} }
                        button {
                            class: "btn-link",
                            onclick: move |_| {
                                let mut c = chosen.write();
                                if c.len() == all_ids.len() {
                                    c.clear();
                                } else {
                                    *c = all_ids.iter().cloned().collect();
                                }
                                topics.write().clear();
                            },
                            {if all_chosen { tx::CLEAR_ALL } else { tx::PICK_ALL }}
                        }
                    }
                    div { class: "st-chips st-lectures",
                        for (id , title) in ready {
                            button {
                                key: "{id}",
                                class: "st-chip",
                                "aria-pressed": if chosen.read().contains(&id) { "true" } else { "false" },
                                onclick: {
                                    let id = id.clone();
                                    move |_| {
                                        let mut c = chosen.write();
                                        if !c.remove(&id) {
                                            c.insert(id.clone());
                                        }
                                        topics.write().clear();
                                    }
                                },
                                span { class: "st-chip-num",
                                    if id == batna_core::own::LECTURE { "✎" } else { {id.rsplit('.').next().unwrap_or(&id).to_string()} }
                                }
                                bdi { "{title}" }
                            }
                        }
                    }

                    if let Some(lecture) = single {
                        if !lecture.topics.is_empty() {
                            h2 { {tx::TOPICS} }
                            p { class: "muted small", {tx::TOPICS_HINT} }
                            div { class: "st-chips",
                                for t in lecture.topics.iter() {
                                    {
                                        let key = format!("{}/{}", lecture.lecture, t.id);
                                        let on = topics.read().contains(&key);
                                        rsx! {
                                            button {
                                                key: "{key}",
                                                class: "st-chip",
                                                "aria-pressed": if on { "true" } else { "false" },
                                                onclick: move |_| {
                                                    let mut s = topics.write();
                                                    if !s.remove(&key) {
                                                        s.insert(key.clone());
                                                    }
                                                },
                                                bdi { "{t.title}" }
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

#[component]
fn SummaryView(
    items: usize,
    right: usize,
    fresh: usize,
    shaky: Vec<(String, String)>,
    next: Option<String>,
    on_again: EventHandler<()>,
) -> Element {
    rsx! {
        section { class: "card st-summary",
            h1 { {if items == 0 { tx::SUMMARY_EMPTY } else { tx::SUMMARY_TITLE }} }
            if items > 0 {
                dl { class: "st-stats",
                    div { dt { {tx::SUM_ITEMS} } dd { "{items}" } }
                    div { dt { {tx::SUM_RIGHT} } dd { "{right}" } }
                    div { dt { {tx::SUM_NEW} } dd { "{fresh}" } }
                }
                if shaky.is_empty() {
                    p { {tx::SUM_ALL_SOLID} }
                } else {
                    h2 { {tx::SUM_BACK_SOON} }
                    p { class: "muted small", {tx::SUM_BACK_SOON_HINT} }
                    ul { class: "st-shaky", lang: "en", dir: "ltr",
                        for (id , text) in shaky {
                            li { key: "{id}", "{text}" }
                        }
                    }
                }
            }
            p { class: "st-next",
                match next {
                    Some(when) => rsx! { "{tx::NEXT_SESSION} " strong { "{when}" } },
                    None => rsx! { {tx::NEXT_TOMORROW} },
                }
            }
            div { class: "st-actions",
                button { class: "btn btn-primary btn-lg", onclick: move |_| on_again.call(()), {tx::ANOTHER} }
            }
        }
    }
}

/// "بعد 20 دقيقة", "بعد ساعتين", "بكرة", "بعد 3 أيام". Days count study
/// days (from 4 am), the days due dates land on, not 24-hour spans (#245).
fn when_label(now: i64, due: i64, zone: &batna_core::zone::Zone) -> String {
    let days = zone.study_day(due).0 - zone.study_day(now).0;
    let mins = ((due - now).max(0) as f64 / 60_000.0).round() as i64;
    match (days, mins) {
        (..=0, 0..=1) => tx::NOW.to_string(),
        (..=0, 2..=59) => format!("{} {}", tx::AFTER, count(mins, &MINUTE)),
        (..=0, _) => format!("{} {}", tx::AFTER, count((mins as f64 / 60.0).round() as i64, &HOUR)),
        (1, _) => tx::TOMORROW.to_string(),
        _ => format!("{} {}", tx::AFTER, count(days, &DAY)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_items_keep_order_and_skip_unknown_and_duplicates() {
        let mut ids: Vec<String> = items()
            .values()
            .filter(|(_, i)| PLAYABLE.contains(&i.body.kind()))
            .map(|(_, i)| i.id.clone())
            .collect();
        ids.sort();
        let ids: Vec<String> = ids.into_iter().take(2).rev().collect();
        if ids.len() < 2 {
            return; // no content embedded
        }
        let store =
            StudyStoreHandle(std::rc::Rc::new(std::cell::RefCell::new(backend::MemoryStore::default())));
        let asked = [ids[0].clone(), "nope.L99.001".into(), ids[1].clone(), ids[0].clone()];
        let plan = plan_items(&store, &asked);
        let got: Vec<&str> = plan.iter().map(|p| p.item_id.as_str()).collect();
        assert_eq!(got, [ids[0].as_str(), ids[1].as_str()]);
        assert!(plan.iter().all(|p| p.fresh));
    }

    #[test]
    fn when_labels() {
        // UTC, so study days start at 04:00 UTC. `now` is 05:00 on a day.
        let zone = batna_core::zone::Zone::fixed(0);
        let now = 20_000 * 86_400_000 + 5 * 3_600_000;
        let at = |h: i64| now + h * 3_600_000;
        let label = |due: i64| when_label(now, due, &zone);
        assert_eq!(label(now), tx::NOW);
        assert_eq!(label(now + 20 * 60_000), "بعد 20 دقيقة");
        assert_eq!(label(at(5)), "بعد 5 ساعات");
        assert_eq!(label(at(2)), "بعد ساعتين");
        assert_eq!(label(at(23)), tx::TOMORROW, "04:00 tomorrow");
        // A 2-day interval lands at 04:00 two study days on: 47 hours away.
        assert_eq!(label(at(47)), "بعد يومين");
        assert_eq!(label(at(71)), "بعد 3 أيام");
    }
}

/// Asked when "study this" arrives during a session: his session, its
/// retries and its summary stay unless he says otherwise.
#[component]
fn PendingCard(
    req: Start,
    on_add: EventHandler<Vec<String>>,
    on_replace: EventHandler<Start>,
    on_keep: EventHandler<()>,
) -> Element {
    let n = req.items.as_ref().map(Vec::len);
    rsx! {
        section { class: "card st-pending", role: "alertdialog", aria_label: tx::PENDING_TITLE,
            p { strong { {tx::PENDING_TITLE} } }
            p { class: "muted small", {tx::PENDING_BODY} }
            div { class: "st-actions",
                if let (Some(ids), Some(n)) = (req.items.clone(), n) {
                    button { class: "btn btn-primary", onclick: move |_| on_add.call(ids.clone()), "{tx::PENDING_ADD} ({n})" }
                }
                button {
                    class: "btn btn-secondary",
                    onclick: {
                        let req = req.clone();
                        move |_| on_replace.call(req.clone())
                    },
                    {tx::PENDING_REPLACE}
                }
                button { class: "btn-link", onclick: move |_| on_keep.call(()), {tx::PENDING_KEEP} }
            }
        }
    }
}
