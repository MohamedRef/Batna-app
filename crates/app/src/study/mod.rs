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
use std::sync::OnceLock;
use std::time::Instant;

use batna_core::content::{Item, Lecture};
use batna_core::review::{Grade, Origin, ReviewEvent};
use batna_core::session::{Inputs, Planned, Session, compose};
use dioxus::prelude::*;

pub(crate) use backend::use_store;
pub use backend::{StudyStore, StudyStoreHandle};
pub use item::{ItemView, PLAYABLE};

use crate::content::Library;
use crate::i18n::study as tx;
use backend::now_ms;
use item::short_text;

pub(crate) const STUDY_CSS: &str = include_str!("../../assets/study.css");
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

/// Every item by id, with its lecture.
fn items() -> &'static HashMap<String, (&'static Lecture, &'static Item)> {
    static MAP: OnceLock<HashMap<String, (&'static Lecture, &'static Item)>> = OnceLock::new();
    MAP.get_or_init(|| {
        Library::get()
            .lectures
            .values()
            .flat_map(|l| l.items.iter().map(move |i| (i.id.clone(), (l, i))))
            .collect()
    })
}

/// The name a lecture shows under: the book's title, or "your cards".
fn lecture_title(l: &Lecture) -> String {
    if l.lecture == batna_core::own::LECTURE { crate::i18n::own::TITLE.to_string() } else { l.title.clone() }
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
        max_new: s.new_left_today(now),
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

    // Today's "start a session" button.
    use_hook(|| {
        if let Some(req) = START.write().take() {
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
            begin(picked, HashSet::new(), req.minutes);
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
                    button { class: "st-primary", onclick: move |_| phase.set(Phase::Summary), {tx::END} }
                },
                Some((p, lecture, item)) => {
                    let item = store.0.borrow().effective(item);
                    let topic_title = item
                        .topic
                        .as_ref()
                        .and_then(|t| lecture.topics.iter().find(|x| &x.id == t))
                        .map(|t| t.title.clone());
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
                        session.write().grade(grade);
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
                                class: "st-link",
                                "aria-keyshortcuts": "Escape",
                                onclick: move |_| phase.set(Phase::Summary),
                                {tx::END}
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
                    next: next.map(|t| when_label(t - now_ms())),
                    on_again: move |_| phase.set(Phase::Setup),
                }
            }
        }
    };

    rsx! {
        style { {STUDY_CSS} }
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
            h1 { {tx::TITLE} }
            p { class: "muted", {tx::SETUP_INTRO} }

            h2 { {tx::TIME} }
            div { class: "st-segmented", role: "radiogroup", aria_label: tx::TIME,
                for m in MINUTES {
                    button {
                        key: "{m}",
                        role: "radio",
                        "aria-checked": if minutes() == m { "true" } else { "false" },
                        onclick: move |_| minutes.set(m),
                        "{m} {tx::MINUTES}"
                    }
                }
            }

            if preview_count == 0 {
                p { class: "note", {tx::NOTHING_NOW} }
            } else {
                p { class: "st-preview",
                    "{preview_count} {tx::ITEMS} · {reviews} {tx::REVIEWS} · {preview_new} {tx::NEW} · {tx::ABOUT} {preview_mins} {tx::MINUTES}"
                }
                button { class: "st-primary st-start", onclick: move |_| on_start.call(()), {tx::START} }
            }

            div { class: "st-section-head",
                h2 { {tx::LECTURES} }
                button {
                    class: "st-link",
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
                button { class: "st-primary", onclick: move |_| on_again.call(()), {tx::ANOTHER} }
            }
        }
    }
}

/// "بعد ٢٠ دقيقة", "بكرة", "بعد ٣ أيام".
fn when_label(ms: i64) -> String {
    let mins = (ms.max(0) as f64 / 60_000.0).round() as i64;
    match mins {
        0..=1 => tx::NOW.to_string(),
        2..=59 => format!("{} {mins} {}", tx::AFTER, tx::MINUTES),
        60..=1439 => format!("{} {} {}", tx::AFTER, (mins as f64 / 60.0).round(), tx::HOURS),
        1440..=2879 => tx::TOMORROW.to_string(),
        _ => format!("{} {} {}", tx::AFTER, mins / 1440, tx::DAYS),
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
        assert_eq!(when_label(0), tx::NOW);
        assert_eq!(when_label(20 * 60_000), format!("{} 20 {}", tx::AFTER, tx::MINUTES));
        assert_eq!(when_label(26 * 3_600_000), tx::TOMORROW);
        assert_eq!(when_label(3 * 86_400_000), format!("{} 3 {}", tx::AFTER, tx::DAYS));
    }
}
