//! Learn (الدروس): short interactive lessons that teach each lecture before
//! its cards are reviewed. Explanations in Egyptian Arabic, drawn figures,
//! mnemonics and small checks between them (`batna_core::lesson`).
//!
//! The list shows every lecture's lessons; a lesson opens in the player
//! (`player.rs`). Finishing one is saved as a synced setting
//! `lesson.<id>` ([`Mark`]), and its practice items start a study session.

mod player;
mod practice;
mod steps;
mod text;

use batna_core::lesson::Lesson;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::Library;
use crate::i18n::learn as tx;
use crate::{Screen, Ui};

const CSS: &str = include_str!("learn.css");

/// The lesson open in the player; set it, then switch to [`Screen::Learn`].
/// `BATNA_OPEN_LESSON=hb.L01.lesson1` opens one at launch (previews, screenshots).
pub static OPEN: GlobalSignal<Option<String>> = Signal::global(|| std::env::var("BATNA_OPEN_LESSON").ok());

/// Where the open lesson was opened from: closing it goes back there.
static FROM: GlobalSignal<Screen> = Signal::global(|| Screen::Learn);

/// Opens a lesson from anywhere (Today, a study item, the planner).
pub fn open(mut ui: Ui, id: &str) {
    *FROM.write() = *ui.screen.peek();
    *OPEN.write() = Some(id.to_string());
    ui.screen.set(Screen::Learn);
}

/// Closes the player, back to where the lesson was opened from.
pub fn close(mut ui: Ui) {
    *OPEN.write() = None;
    let from = std::mem::replace(&mut *FROM.write(), Screen::Learn);
    if from != Screen::Learn {
        ui.screen.set(from);
    }
}

/// A finished lesson, saved as the synced setting `lesson.<id>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    pub done_ms: i64,
    /// Questions right on the first try, out of `asked`.
    pub right: u32,
    pub asked: u32,
}

const PREFIX: &str = "lesson.";

/// Bumped when a lesson is finished, so lists re-read the marks.
static MARKS_REV: GlobalSignal<u64> = Signal::global(|| 0);

pub fn mark(id: &str) -> Option<Mark> {
    let _ = MARKS_REV();
    crate::store::shared().setting(&format!("{PREFIX}{id}"))
}

fn save_mark(id: &str, m: Mark) {
    let _ = crate::store::shared().put_setting(&format!("{PREFIX}{id}"), &m);
    *MARKS_REV.write() += 1;
}

/// The lesson to do next: the first unfinished one of the earliest lecture
/// in `prefer` that has one left, else the first unfinished in book order.
pub fn next_lesson(prefer: &[String]) -> Option<&'static Lesson> {
    let lib = Library::get();
    let left = |l: &&Lesson| mark(&l.id).is_none();
    prefer
        .iter()
        .filter_map(|lec| lib.lessons.get(lec))
        .find_map(|f| f.lessons.iter().find(left))
        .or_else(|| lib.all_lessons().map(|(_, l)| l).find(left))
}

/// The lesson that teaches an item: one that practises it, else the first
/// lesson of its lecture that covers its topic.
pub fn lesson_for_item(item_id: &str) -> Option<&'static Lesson> {
    let lib = Library::get();
    let lecture = item_id.rsplit_once('.').map(|(l, _)| l)?;
    let file = lib.lessons.get(lecture)?;
    let listed = file.lessons.iter().find(|l| {
        l.practice.iter().any(|p| p == item_id)
            || l.steps.iter().any(|s| matches!(&s.body, batna_core::lesson::StepBody::Try { items } if items.iter().any(|i| i == item_id)))
    });
    listed.or_else(|| {
        let topic = practice::find(item_id)?.1.topic.as_deref()?;
        file.lessons.iter().find(|l| l.topics.iter().any(|t| t == topic))
    })
}

#[component]
pub fn Learn() -> Element {
    let open = OPEN();
    rsx! {
        style { {crate::study::STUDY_CSS} }
        style { {CSS} }
        match open.and_then(|id| Library::get().lesson(&id).map(|(lec, l)| (lec, l.clone()))) {
            Some((lecture, lesson)) => rsx! {
                player::Player { key: "{lesson.id}", lesson, lecture_title: lecture.title.clone() }
            },
            None => rsx! { LessonList {} },
        }
    }
}

#[component]
fn LessonList() -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let total: usize = lib.lessons.values().map(|f| f.lessons.len()).sum();
    let done = lib.all_lessons().filter(|(_, l)| mark(&l.id).is_some()).count();
    let next = next_lesson(&[]);
    rsx! {
        section { class: "card ls-hero",
            h1 { {tx::TITLE} }
            p { class: "muted", {tx::INTRO} }
            if total == 0 {
                p { class: "note", {tx::NONE_YET} }
            } else {
                div { class: "ls-hero-row",
                    span { class: "chip", "{done} {tx::OF} {total} {tx::LESSONS_DONE}" }
                    if let Some(l) = next {
                        button {
                            class: "ls-btn primary",
                            onclick: move |_| open(ui, &l.id),
                            {if done == 0 { tx::START_FIRST } else { tx::CONTINUE }}
                            " · "
                            text::Line { text: l.title.clone() }
                        }
                    } else {
                        span { class: "ls-verdict good", {tx::ALL_LESSONS_DONE} }
                    }
                }
                p { class: "note small", {tx::UNVERIFIED_NOTE} }
            }
        }
        for entry in lib.index.lectures.iter() {
            {
                let number = entry.id.rsplit('.').next().unwrap_or(&entry.id).trim_start_matches('L').to_string();
                match lib.lessons.get(&entry.id) {
                    Some(file) => rsx! {
                        section { key: "{entry.id}", class: "ls-lecture",
                            h2 { class: "ls-lecture-head",
                                span { class: "num", "{number}" }
                                bdi { "{entry.title}" }
                            }
                            ol { class: "ls-cards",
                                for (i , l) in file.lessons.iter().enumerate() {
                                    LessonCard { key: "{l.id}", n: i + 1, lesson: l.clone() }
                                }
                            }
                        }
                    },
                    None => rsx! {
                        div { key: "{entry.id}", class: "ls-lecture ls-soon",
                            span { class: "num", "{number}" }
                            bdi { "{entry.title}" }
                            span { class: "chip", {tx::WRITING} }
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn LessonCard(n: usize, lesson: Lesson) -> Element {
    let ui = use_context::<Ui>();
    let m = mark(&lesson.id);
    let id = lesson.id.clone();
    let questions = lesson.steps.iter().filter(|s| s.body.is_question()).count();
    rsx! {
        li { class: "ls-card", "data-done": if m.is_some() { "yes" } else { "no" },
            button { class: "ls-card-btn", onclick: move |_| open(ui, &id),
                span { class: "ls-card-n", "{n}" }
                span { class: "ls-card-main",
                    strong { class: "ls-card-title", text::Line { text: lesson.title.clone() } }
                    span { class: "ls-card-hook muted small", text::Line { text: lesson.hook.clone() } }
                    span { class: "ls-card-meta small",
                        "{lesson.minutes} {tx::MINUTES} · {questions} {tx::QUESTIONS} · {tx::PAGES} "
                        bdi { dir: "ltr", "{lesson.pages[0]}–{lesson.pages[1]}" }
                    }
                }
                match m {
                    Some(m) => rsx! { span { class: "chip ls-done", "✓ {m.right}/{m.asked}" } },
                    None => rsx! { span { class: "chip", {tx::NEW} } },
                }
            }
        }
    }
}

/// Today's card: the next lesson, one tap away.
#[component]
pub fn TodayLesson(prefer: Vec<String>) -> Element {
    let ui = use_context::<Ui>();
    let Some(l) = next_lesson(&prefer) else {
        return rsx! {};
    };
    let lecture = l.id.rsplit_once('.').map(|(l, _)| l).unwrap_or(&l.id);
    let title = Library::get().lectures.get(lecture).map(|l| l.title.clone()).unwrap_or_default();
    rsx! {
        style { {CSS} }
        section { class: "card ls-today",
            p { class: "ls-kicker", {tx::TODAY_KICKER} }
            h2 { text::Line { text: l.title.clone() } }
            p { class: "muted small", bdi { "{title}" } " · {l.minutes} {tx::MINUTES}" }
            p { text::Line { text: l.hook.clone() } }
            button { class: "ls-btn primary", onclick: move |_| open(ui, &l.id), {tx::OPEN_LESSON} }
        }
    }
}

/// A small "explain it to me" link under a study item whose lesson exists.
#[component]
pub fn ExplainLink(item_id: String) -> Element {
    let ui = use_context::<Ui>();
    let Some(l) = lesson_for_item(&item_id) else {
        return rsx! {};
    };
    let id = l.id.clone();
    rsx! {
        button { class: "st-link ls-explain", title: tx::EXPLAIN_HINT, onclick: move |_| open(ui, &id),
            "📖 {tx::EXPLAIN} · "
            text::Line { text: l.title.clone() }
        }
    }
}
