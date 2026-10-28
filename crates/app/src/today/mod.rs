//! Today: the screen the app opens on. With an exam day it leads with the
//! plan's target for the day (#99). Then what's due, what's new, one button
//! for a short session, the lectures in progress, and the library.

mod settings;

use dioxus::prelude::*;

pub use settings::StudySection;

use batna_core::plan::{Focus, PlanStatus};
use batna_core::review::StudyStore;

use crate::content::Library;
use crate::i18n::today as tx;
use crate::plan::DailyTarget;
use crate::stats::StudyData;
use crate::store::{self, LectureProgress, Overview, use_review};
use crate::{Screen, Ui};

pub(crate) const CSS: &str = include_str!("today.css");

#[component]
pub fn Today() -> Element {
    let lib = Library::get();
    let review = use_review();
    let o = review.backend.borrow().overview(lib, store::now_ms());
    // With an exam day, the plan sets today's new items (#99).
    let target = crate::plan::target_now(&*review.backend.borrow());
    let new_today = target.as_ref().map_or(o.new_today, |t| (t.new_left() as usize).min(o.unseen));
    let mut ui = use_context::<Ui>();
    let ready = o.due + new_today > 0;
    let minutes = crate::onboarding::session_minutes();

    rsx! {
        style { {CSS} }
        // Tablet landscape and the Mac: today's plan stays in view on one
        // side while the lecture library scrolls on the other.
        div { class: "tab-cols today-cols", "data-wide": "aside",
            div { class: "tab-main tab-sticky",
                section { class: "card today-hero",
                    h1 { {tx::TITLE} }
                    if let Some(t) = target.clone() {
                        PlanHero { t, due: o.due, due_minutes: o.due_minutes }
                    }
                    div { class: "tiles",
                        Tile { value: o.due, label: tx::DUE, accent: o.due > 0 }
                        Tile { value: new_today, label: tx::NEW }
                        Tile { value: o.checked, label: tx::CHECKED }
                    }
                    if ready {
                        if o.due > 0 {
                            p { class: "muted small", "{tx::DUE_TIME_1} {o.due_minutes} {tx::DUE_TIME_2}" }
                        }
                        button {
                            class: "primary start",
                            onclick: move |_| {
                                *crate::study::START.write() = Some(crate::study::Start { lectures: None, items: None, minutes });
                                ui.screen.set(Screen::Study);
                            },
                            {tx::start(minutes)}
                        }
                    } else if o.seen == 0 && o.unseen == 0 {
                        p { class: "muted", {tx::NOTHING_WRITTEN} }
                    } else if o.unseen > 0 {
                        p { {tx::ALL_DONE} }
                        p { class: "muted small", "{tx::NEW_CAP_REACHED} · {tx::TOMORROW} {o.due_tomorrow}" }
                        if target.is_some() {
                            crate::stats::SessionButton { label: tx::AHEAD, ids: ahead(&*review.backend.borrow(), &o) }
                        }
                    } else {
                        p { {tx::ALL_DONE} }
                        p { class: "muted small", "{tx::TOMORROW} {o.due_tomorrow}" }
                    }
                    if let Some(e) = store::open_error() {
                        p { class: "note", "{tx::STORE_FAILED} " bdi { dir: "ltr", "{e}" } }
                    }
                }
                crate::recap::TodayRecap { due_left: o.due, due_tomorrow: o.due_tomorrow }
                if !o.lectures.is_empty() {
                    section { class: "in-progress",
                        h2 { {tx::IN_PROGRESS} }
                        ul { class: "progress-list",
                            for l in o.lectures.iter() {
                                Progress { key: "{l.id}", lecture: l.clone() }
                            }
                        }
                    }
                }
            }
            div { class: "tab-aside", LibraryList {} }
        }
    }
}

/// Rough time for a new item: a first look takes longer than a review.
const NEW_ITEM_SECS: u32 = 40;
/// Items in a "get ahead" session once today's plan is done.
const AHEAD_ITEMS: usize = 10;

/// The exam plan's day on Today: days to the exam, today's target with a
/// ring for how much of it is done, and a calm note if the plan is tight.
#[component]
fn PlanHero(t: DailyTarget, due: usize, due_minutes: u32) -> Element {
    let Some(day) = t.today().cloned() else { return rsx! {} };
    let goal = t.new_goal();
    let left = t.new_left();
    // Reviews: what he has answered today plus what is still due.
    let target_total = goal + t.done.reviews + due as u32;
    let done_total = t.done.new.min(goal) + t.done.reviews;
    let pct = (done_total * 100).checked_div(target_total).unwrap_or(100);
    let minutes = due_minutes + (left * NEW_ITEM_SECS).div_ceil(60);
    let tight = match t.plan.status {
        PlanStatus::Tight { extra_days: Some(n), .. } => Some(tx::tight_days(n)),
        PlanStatus::Tight { extra_days: None, .. } => Some(tx::TIGHT_CAP.to_string()),
        _ => None,
    };
    let mut ui = use_context::<Ui>();
    rsx! {
        p { class: "plan-countdown", {tx::exam_in(t.days_left)} }
        div { class: "plan-line",
            div {
                class: "ring",
                role: "progressbar",
                aria_label: tx::RING_LABEL,
                aria_valuemin: "0",
                aria_valuemax: "100",
                aria_valuenow: "{pct}",
                style: "--p: {pct}%",
                span { class: "ring-hole", bdi { "{pct}%" } }
            }
            div { class: "plan-text",
                match day.focus {
                    Focus::Learn | Focus::Revisit => rsx! {
                        p { class: "plan-target",
                            "{tx::PLAN_TODAY} "
                            if goal > 0 && due > 0 {
                                "{goal} {tx::PLAN_NEW} + {due} {tx::PLAN_REVIEWS}"
                            } else if goal > 0 {
                                "{goal} {tx::PLAN_NEW}"
                            } else {
                                "{due} {tx::PLAN_REVIEWS_ONLY}"
                            }
                            if minutes > 0 {
                                " ({tx::ABOUT} {minutes} {tx::MINUTES})"
                            }
                        }
                        if !day.revisit.is_empty() {
                            p { class: "muted small",
                                "{tx::REVISIT} "
                                bdi { {crate::plan::lecture_list(&day.revisit)} }
                            }
                        }
                        if goal > 0 && left == 0 && due > 0 {
                            p { class: "muted small", {tx::NEW_DONE} }
                        } else if t.done.new > 0 && left > 0 {
                            p { class: "muted small", "{tx::NEW_LEFT_1} {left} {tx::NEW_LEFT_2}" }
                        }
                    },
                    Focus::Mock => rsx! {
                        p { class: "plan-target", {tx::MOCK} }
                        button { class: "link", onclick: move |_| ui.screen.set(Screen::Bank), {tx::OPEN_BANK} }
                    },
                    Focus::WeakTopics => rsx! {
                        p { class: "plan-target", {tx::WEAK} }
                        WeakStart {}
                    },
                    Focus::Rest => rsx! {
                        p { class: "plan-target", {tx::REST} }
                        if due > 0 {
                            p { class: "muted small", {tx::REST_DUE} }
                        }
                    },
                }
            }
        }
        if let Some(note) = tight {
            p { class: "note small", {note} }
        }
    }
}

/// A session on the weakest topics, as on the progress screen.
#[component]
fn WeakStart() -> Element {
    let review = use_review();
    let data = StudyData::load(&*review.backend.borrow());
    let weak = batna_core::stats::weakest_topics(&data.stats(), 3);
    let ids = batna_core::stats::topics_session(&data.lectures, &weak, &data.events, &data.marks);
    rsx! {
        crate::stats::SessionButton { label: tx::WEAK_START, ids }
    }
}

/// The next few unseen items once today's plan is done: lectures already
/// started first, then book order. The session button keeps only what the
/// player can show.
fn ahead(store: &dyn StudyStore, o: &Overview) -> Vec<String> {
    let lib = Library::get();
    let mut order: Vec<&str> = o.lectures.iter().map(|l| l.id.as_str()).collect();
    for e in &lib.index.lectures {
        if !order.contains(&e.id.as_str()) {
            order.push(&e.id);
        }
    }
    order
        .into_iter()
        .filter_map(|id| lib.lectures.get(id))
        .flat_map(|l| &l.items)
        .filter(|i| crate::study::PLAYABLE.contains(&i.body.kind()))
        .filter(|i| store.is_new(&i.id) && !store.is_hidden(&i.id))
        .take(AHEAD_ITEMS)
        .map(|i| i.id.clone())
        .collect()
}

#[component]
fn Tile(value: usize, label: &'static str, #[props(default)] accent: bool) -> Element {
    rsx! {
        div { class: "tile", "data-accent": accent,
            span { class: "tile-value", "{value}" }
            span { class: "tile-label", {label} }
        }
    }
}

#[component]
fn Progress(lecture: LectureProgress) -> Element {
    let pct = lecture.seen * 100 / lecture.total.max(1);
    let number = lecture.id.rsplit('.').next().unwrap_or(&lecture.id).trim_start_matches('L').to_string();
    rsx! {
        li { class: "progress-row",
            span { class: "num", "{number}" }
            div { class: "lecture-main",
                bdi { class: "lecture-title", "{lecture.title}" }
                div {
                    class: "bar",
                    role: "progressbar",
                    aria_valuemin: "0",
                    aria_valuemax: "{lecture.total}",
                    aria_valuenow: "{lecture.seen}",
                    div { class: "bar-fill", style: "inline-size: {pct}%" }
                }
                span { class: "muted small", "{lecture.seen} {tx::OF} {lecture.total} {tx::STARTED}" }
            }
            if lecture.due > 0 {
                span { class: "chip due", "{lecture.due} {tx::DUE_CHIP}" }
            }
        }
    }
}

/// The chapter's lectures and how many items each has.
#[component]
fn LibraryList() -> Element {
    let lib = Library::get();
    let rows = lib.index.lectures.iter().map(|entry| {
        let items = lib.lectures.get(&entry.id).map(|l| l.items.len());
        let number = entry.id.rsplit('.').next().unwrap_or(&entry.id).trim_start_matches('L').to_string();
        let [first, last] = entry.pages;
        (entry.id.clone(), number, entry.title.clone(), items, format!("{first}–{last}"))
    });
    rsx! {
        section { class: "card",
            h2 { {tx::LIBRARY} }
            p { class: "muted", bdi { {tx::CHAPTER} } }
            p { class: "note", {tx::UNVERIFIED_NOTE} }
        }
        ol { class: "lectures",
            for (id, number, title, items, pages) in rows {
                li { key: "{id}", class: "lecture", "data-ready": if items.is_some() { "yes" } else { "no" },
                    span { class: "num", "{number}" }
                    div { class: "lecture-main",
                        bdi { class: "lecture-title", "{title}" }
                        span { class: "muted small", "{tx::PAGES} " bdi { dir: "ltr", "{pages}" } }
                    }
                    match items {
                        Some(n) => rsx! { span { class: "chip ready", "{tx::READY} · {n} {tx::ITEMS}" } },
                        None => rsx! { span { class: "chip", {tx::WRITING} } },
                    }
                }
            }
        }
    }
}
