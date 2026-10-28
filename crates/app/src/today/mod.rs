//! Today: the screen the app opens on. What's due, what's new, one button
//! for a 10-minute session, the lectures in progress, then the library.

mod settings;

use dioxus::prelude::*;

pub use settings::StudySection;

use crate::content::Library;
use crate::i18n::today as tx;
use crate::store::{self, LectureProgress, use_review};
use crate::{Screen, Ui};

pub(crate) const CSS: &str = include_str!("today.css");

#[component]
pub fn Today() -> Element {
    let lib = Library::get();
    let review = use_review();
    let o = review.backend.borrow().overview(lib, store::now_ms());
    let mut ui = use_context::<Ui>();
    let ready = o.due + o.new_today > 0;

    rsx! {
        style { {CSS} }
        // Tablet landscape and the Mac: today's plan stays in view on one
        // side while the lecture library scrolls on the other.
        div { class: "tab-cols today-cols", "data-wide": "aside",
            div { class: "tab-main tab-sticky",
                section { class: "card today-hero",
                    h1 { {tx::TITLE} }
                    div { class: "tiles",
                        Tile { value: o.due, label: tx::DUE, accent: o.due > 0 }
                        Tile { value: o.new_today, label: tx::NEW }
                        Tile { value: o.checked, label: tx::CHECKED }
                    }
                    if ready {
                        if o.due > 0 {
                            p { class: "muted small", "{tx::DUE_TIME_1} {o.due_minutes} {tx::DUE_TIME_2}" }
                        }
                        button {
                            class: "primary start",
                            onclick: move |_| {
                                *crate::study::START.write() = Some(crate::study::Start { lectures: None, items: None, minutes: crate::onboarding::session_minutes() });
                                ui.screen.set(Screen::Study);
                            },
                            {tx::START}
                        }
                    } else if o.seen == 0 && o.unseen == 0 {
                        p { class: "muted", {tx::NOTHING_WRITTEN} }
                    } else if o.unseen > 0 {
                        p { {tx::ALL_DONE} }
                        p { class: "muted small", "{tx::NEW_CAP_REACHED} · {tx::TOMORROW} {o.due_tomorrow}" }
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
