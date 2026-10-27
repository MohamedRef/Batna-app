//! The bank's start screen: filters, tutor/timed start, the mock exam, and
//! the history of finished papers.

use std::collections::BTreeSet;

use batna_core::exam::{self, Filter, MOCK_SIZE, Mode, Only};
use dioxus::prelude::*;

use super::{HISTORY, date, lecture_number, lecture_title, minutes, now_ms, start, topic_title};
use crate::content::Library;
use crate::i18n::bank as tx;

/// The filters stay put between papers.
static FILTER: GlobalSignal<Filter> = Signal::global(Filter::default);
static COUNT: GlobalSignal<usize> = Signal::global(|| 20);
static MOCK_COUNT: GlobalSignal<usize> = Signal::global(|| MOCK_SIZE);

const COUNTS: [usize; 4] = [10, 20, 40, usize::MAX];
const MOCK_COUNTS: [usize; 4] = [20, 40, 60, 100];
const TAGS: [(&str, &str); 3] =
    [("high-yield", tx::TAG_HIGH_YIELD), ("book-question", tx::TAG_BOOK), ("numbers", tx::TAG_NUMBERS)];

#[component]
pub fn Setup() -> Element {
    let lib = Library::get();
    let pool = exam::mcqs(lib.lectures.values());
    if pool.is_empty() {
        return rsx! {
            section { class: "card",
                h1 { {tx::TITLE} }
                h2 { class: "muted", {tx::EMPTY_TITLE} }
                p { {tx::EMPTY_BODY} }
            }
            History {}
        };
    }

    let filter = FILTER.read().clone();
    let history = HISTORY.read();
    let selected = filter.select(&pool, &history.last);
    drop(history);
    let available = selected.len();
    let count = *COUNT.read();
    let mock_count = *MOCK_COUNT.read();

    // Lectures that have MCQs, with how many.
    let mut lectures: Vec<(String, usize)> = Vec::new();
    for q in &pool {
        match lectures.last_mut() {
            Some((l, n)) if l == q.lecture => *n += 1,
            _ => lectures.push((q.lecture.to_string(), 1)),
        }
    }
    // Topics of the chosen lectures (all lectures when none is chosen).
    let topics: BTreeSet<(String, String)> = pool
        .iter()
        .filter(|q| filter.lectures.is_empty() || filter.lectures.contains(q.lecture))
        .filter_map(|q| Some((q.lecture.to_string(), q.topic?.to_string())))
        .collect();
    let tags_present: BTreeSet<&str> = pool.iter().flat_map(|q| q.tags.iter().map(String::as_str)).collect();
    let tags: Vec<(&str, &str)> = TAGS.into_iter().filter(|(t, _)| tags_present.contains(t)).collect();
    let several_lectures = lectures.len() > 1;

    let start_with = move |mode: Mode| {
        let filter = FILTER.read().clone();
        let selected = filter.select(&pool, &HISTORY.read().last);
        start(mode, exam::practice(&selected, *COUNT.read(), now_ms() as u64));
    };
    let start_tutor = start_with.clone();
    let start_mock = move |_| {
        let pool = exam::mcqs(Library::get().lectures.values());
        start(Mode::Mock, exam::mock(&pool, *MOCK_COUNT.read(), now_ms() as u64));
    };
    let shown = count.min(available);
    let mock_shown = mock_count.min(lectures.iter().map(|l| l.1).sum());

    rsx! {
        section { class: "card",
            h1 { {tx::TITLE} }
            p { class: "muted", {tx::LEAD} }

            if several_lectures {
                h2 { class: "label", {tx::LECTURES} }
                div { class: "chips",
                    for (id, n) in lectures {
                        Chip {
                            key: "{id}",
                            on: filter.lectures.contains(&id),
                            onclick: {
                                let id = id.clone();
                                move |_| {
                                    let mut f = FILTER.write();
                                    if !f.lectures.remove(&id) {
                                        f.lectures.insert(id.clone());
                                    }
                                    let keep = f.lectures.clone();
                                    f.topics.retain(|(l, _)| keep.is_empty() || keep.contains(l));
                                }
                            },
                            bdi { "L{lecture_number(&id)} · {lecture_title(&id)}" }
                            span { class: "count", bdi { "{n}" } }
                        }
                    }
                }
            }

            if !topics.is_empty() {
                h2 { class: "label", {tx::TOPICS} }
                div { class: "chips",
                    for (lecture, topic) in topics {
                        Chip {
                            key: "{lecture}/{topic}",
                            on: filter.topics.contains(&(lecture.clone(), topic.clone())),
                            onclick: {
                                let key = (lecture.clone(), topic.clone());
                                move |_| {
                                    let mut f = FILTER.write();
                                    if !f.topics.remove(&key) {
                                        f.topics.insert(key.clone());
                                    }
                                }
                            },
                            if several_lectures {
                                bdi { "L{lecture_number(&lecture)} · {topic_title(&lecture, &topic)}" }
                            } else {
                                bdi { {topic_title(&lecture, &topic)} }
                            }
                        }
                    }
                }
            }

            h2 { class: "label", {tx::TAGS} }
            div { class: "chips",
                for (tag, label) in tags {
                    Chip {
                        key: "{tag}",
                        on: filter.tags.contains(tag),
                        onclick: move |_| {
                            let mut f = FILTER.write();
                            if !f.tags.remove(tag) {
                                f.tags.insert(tag.to_string());
                            }
                        },
                        {label}
                    }
                }
                Chip {
                    on: filter.only == Only::WrongOrUnseen,
                    onclick: move |_| {
                        let mut f = FILTER.write();
                        f.only = if f.only == Only::All { Only::WrongOrUnseen } else { Only::All };
                    },
                    {tx::ONLY_WRONG}
                }
            }

            h2 { class: "label", {tx::COUNT} }
            div { class: "chips",
                for c in COUNTS {
                    Chip {
                        key: "{c}",
                        on: count == c,
                        onclick: move |_| *COUNT.write() = c,
                        if c == usize::MAX { {tx::ALL} } else { bdi { "{c}" } }
                    }
                }
            }

            if available == 0 {
                p { class: "note", {tx::NONE_MATCH} }
            } else {
                p { class: "muted small", bdi { "{available}" } " {tx::AVAILABLE}" }
                div { class: "start-row",
                    button { class: "button primary start", onclick: move |_| start_tutor(Mode::Tutor),
                        span { {tx::START_TUTOR} " · " bdi { "{shown}" } }
                        span { class: "hint", {tx::TUTOR_HINT} }
                    }
                    button { class: "button start", onclick: move |_| start_with(Mode::Timed),
                        span { {tx::START_TIMED} " · " bdi { "{minutes(exam::time_limit_ms(shown))}" } " {tx::MINUTES}" }
                        span { class: "hint", {tx::TIMED_HINT} }
                    }
                }
            }
        }

        section { class: "card",
            h1 { {tx::MOCK_TITLE} }
            p { class: "muted", {tx::MOCK_LEAD} }
            h2 { class: "label", {tx::COUNT} }
            div { class: "chips",
                for c in MOCK_COUNTS {
                    Chip { key: "{c}", on: mock_count == c, onclick: move |_| *MOCK_COUNT.write() = c, bdi { "{c}" } }
                }
            }
            div { class: "start-row",
                button { class: "button primary start", onclick: start_mock,
                    span { {tx::START_MOCK} " · " bdi { "{mock_shown}" } " {tx::QUESTION}" }
                    span { class: "hint", bdi { "{minutes(exam::time_limit_ms(mock_shown))}" } " {tx::MINUTES}" }
                }
            }
        }

        History {}
    }
}

#[component]
fn Chip(on: bool, onclick: EventHandler<MouseEvent>, children: Element) -> Element {
    rsx! {
        button {
            class: "chip toggle",
            "aria-pressed": if on { "true" } else { "false" },
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}

#[component]
fn History() -> Element {
    let papers = HISTORY.read().papers.clone();
    if papers.is_empty() {
        return rsx! {};
    }
    rsx! {
        section { class: "card",
            h2 { {tx::HISTORY} }
            ol { class: "history", reversed: true,
                for (i, o) in papers.iter().enumerate().rev() {
                    li { key: "{i}",
                        span { class: "chip", if o.mode == Mode::Mock { {tx::MODE_MOCK} } else { {tx::MODE_TIMED} } }
                        bdi { class: "muted small", {date(o.at_ms)} }
                        span { class: "grow" }
                        bdi { class: "score", "{o.total.right}/{o.total.total}" }
                        bdi { class: "pct", "{o.total.percent()}%" }
                    }
                }
            }
            p { class: "muted small", {tx::HISTORY_NOTE} }
        }
    }
}
