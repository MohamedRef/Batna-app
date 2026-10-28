//! One lecture's page: what it covers, how far along it is, and every way in
//! (a session on it, what's due, its lessons, its questions, its cases, a
//! session on one topic). Every lecture row in the app opens it, so a row
//! is never a dead end.
//!
//! It opens over whichever screen it came from and closes with Back, the
//! page's own back button, or any move to another screen.

use batna_core::review::StudyStore;
use batna_core::stats::WeakTopic;
use dioxus::prelude::*;

use crate::content::{Library, lecture_code};
use crate::i18n::lecture as tx;
use crate::stats::{StudyData, pct};
use crate::store::{now_ms, use_review};
use crate::{Screen, Ui};

const CSS: &str = include_str!("lecture.css");

/// The open lecture and the screen it opened over. `BATNA_OPEN_LECTURE=hb.L01`
/// opens one at launch over the first screen (previews, screenshots).
static OPEN: GlobalSignal<Option<(String, Screen)>> = Signal::global(|| None);

/// Opens a lecture's page over the current screen.
pub fn open(ui: Ui, id: &str) {
    *OPEN.write() = Some((id.to_string(), *ui.screen.peek()));
    document::eval(r#"document.querySelector("main.content")?.scrollTo(0, 0);"#);
}

pub fn close() {
    *OPEN.write() = None;
    document::eval(r#"document.querySelector("main.content")?.scrollTo(0, 0);"#);
}

/// The lecture to show over `screen`, if one is open there.
pub fn shown(screen: Screen) -> Option<String> {
    OPEN.read().as_ref().filter(|(_, on)| *on == screen).map(|(id, _)| id.clone())
}

/// Whether a lecture page is open (Back closes it before leaving the screen).
pub fn is_open() -> bool {
    OPEN.read().is_some()
}

/// Wires the page to the shell: the launch preview, and closing it once
/// the screen it opened over is left.
pub fn use_lecture_page(ui: Ui) {
    use_hook(move || {
        if let Ok(id) = std::env::var("BATNA_OPEN_LECTURE") {
            *OPEN.write() = Some((id, *ui.screen.peek()));
        }
    });
    use_effect(move || {
        let now = *ui.screen.read();
        let stale = OPEN.peek().as_ref().is_some_and(|(_, on)| *on != now);
        if stale {
            *OPEN.write() = None;
        }
    });
}

/// Item ids for a session on one topic of a lecture (`None`: its untagged
/// items): what he has seen first, then the rest, rejected items left out.
pub fn topic_session(data: &StudyData, lecture: &str, topic: Option<String>) -> Vec<String> {
    let t = WeakTopic {
        lecture: lecture.to_string(),
        topic,
        title: String::new(),
        score: 0.0,
        retention: Default::default(),
        mcq: Default::default(),
    };
    batna_core::stats::topics_session(&data.lectures, &[t], &data.events, &data.marks)
}

/// Starts a study session: on exactly `items` when given, else on the
/// lecture with the usual time budget.
fn start(mut ui: Ui, lecture: &str, items: Option<Vec<String>>) {
    // Over Study itself the screen doesn't change, so close the page here.
    close();
    *crate::study::START.write() = Some(crate::study::Start {
        lectures: Some(vec![lecture.to_string()]),
        items,
        minutes: crate::onboarding::session_minutes(),
    });
    ui.screen.set(Screen::Study);
}

#[component]
pub fn LecturePage(id: String) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let review = use_review();
    let Some(entry) = lib.index.lectures.iter().find(|e| e.id == id) else {
        return rsx! {};
    };
    let lecture = lib.lectures.get(&id);
    let lessons = lib.lessons.get(&id);
    let data = StudyData::load(&*review.backend.borrow());
    let stats = data.stats().into_iter().find(|l| l.id == id);
    let (due, playable, mcqs, cases) = {
        let b = review.backend.borrow();
        let prefix = format!("{id}.");
        let due: Vec<String> =
            b.session_due(now_ms()).into_iter().filter(|i| i.starts_with(&prefix)).collect();
        let items = lecture.map(|l| l.items.as_slice()).unwrap_or_default();
        let count = |kind: &[&str]| {
            items.iter().filter(|i| kind.contains(&i.body.kind()) && !b.is_hidden(&i.id)).count()
        };
        (due, count(crate::study::PLAYABLE), count(&["mcq"]), count(&["case"]))
    };
    let done_lessons =
        lessons.map_or(0, |f| f.lessons.iter().filter(|l| crate::learn::mark(&l.id).is_some()).count());
    let [first, last] = entry.pages;
    let number = lecture_code(&id);
    let (seen, total) = stats.as_ref().map_or((0, 0), |s| (s.counts.seen, s.counts.total));
    let share = (seen * 100).checked_div(total).unwrap_or(0);
    let due_n = due.len();

    rsx! {
        style { {crate::learn::CSS} }
        style { {CSS} }
        div { class: "lec-page",
            div { class: "lec-top",
                button {
                    class: "icon-button lec-back",
                    "data-back": "true",
                    aria_label: tx::BACK,
                    title: tx::BACK,
                    onclick: move |_| close(),
                    span { aria_hidden: "true", "→" }
                }
                span { class: "muted small", {tx::KICKER} }
            }
            section { class: "card lec-hero",
                div { class: "lec-head",
                    span { class: "num", "{number}" }
                    div { class: "lecture-main",
                        h1 { bdi { class: "lecture-title", "{entry.title}" } }
                        span { class: "muted small", "{tx::PAGES} " bdi { dir: "ltr", "{first}–{last}" } }
                    }
                }
                if lecture.is_some() {
                    div {
                        class: "lec-bar",
                        role: "progressbar",
                        aria_label: tx::PROGRESS,
                        aria_valuemin: "0",
                        aria_valuemax: "{total}",
                        aria_valuenow: "{seen}",
                        div { class: "lec-bar-fill", style: "inline-size: {share}%" }
                    }
                    p { class: "muted small",
                        "{seen} {tx::OF} {total} {tx::STARTED}"
                        if due_n > 0 { " · " span { class: "lec-due", "{due_n} {tx::DUE}" } }
                    }
                    div { class: "lec-actions",
                        if playable > 0 {
                            button {
                                class: "btn btn-primary",
                                onclick: { let id = id.clone(); move |_| start(ui, &id, None) },
                                {tx::STUDY}
                            }
                        }
                        if due_n > 0 {
                            button {
                                class: "btn btn-tonal",
                                onclick: { let id = id.clone(); move |_| start(ui, &id, Some(due.clone())) },
                                "{tx::REVIEW_DUE} ({due_n})"
                            }
                        }
                    }
                } else {
                    p { class: "note", {tx::WRITING} }
                }
            }

            if let Some(file) = lessons {
                section { class: "lec-section",
                    h2 {
                        {tx::LESSONS}
                        span { class: "muted small", " · {done_lessons} {tx::OF} {file.lessons.len()}" }
                    }
                    ol { class: "ls-cards",
                        for (i , l) in file.lessons.iter().enumerate() {
                            crate::learn::LessonCard { key: "{l.id}", n: i + 1, lesson: l.clone() }
                        }
                    }
                }
            }

            if mcqs > 0 || cases > 0 {
                section { class: "lec-section",
                    h2 { {tx::PRACTICE} }
                    div { class: "lec-ways",
                        if mcqs > 0 {
                            button {
                                class: "tap lec-way",
                                onclick: { let id = id.clone(); move |_| crate::bank::practise(ui, &id, None) },
                                span { class: "lec-way-n", "{mcqs}" }
                                span { class: "lec-way-label", {tx::MCQS} }
                                span { class: "chev", aria_hidden: "true" }
                            }
                        }
                        if cases > 0 {
                            button {
                                class: "tap lec-way",
                                onclick: move |_| { let mut ui = ui; ui.screen.set(Screen::Cases) },
                                span { class: "lec-way-n", "{cases}" }
                                span { class: "lec-way-label", {tx::CASES} }
                                span { class: "chev", aria_hidden: "true" }
                            }
                        }
                    }
                }
            }

            if let Some(s) = stats.filter(|s| !s.topics.is_empty()) {
                section { class: "lec-section",
                    h2 { {tx::TOPICS} }
                    p { class: "muted small", {tx::TOPICS_HINT} }
                    ul { class: "tap-list",
                        for t in s.topics.iter() {
                            {
                                // Cards and the like start a session; a topic with only
                                // MCQs opens the bank on it instead.
                                let ids: Vec<String> =
                                    topic_session(&data, &id, t.id.clone()).into_iter().filter(|i| crate::study::is_playable(i)).collect();
                                let topic = t.id.clone();
                                let lecture_id = id.clone();
                                rsx! {
                                    li { key: "{t.title}",
                                        button {
                                            class: "tap tap-row",
                                            onclick: move |_| {
                                                if ids.is_empty() {
                                                    crate::bank::practise(ui, &lecture_id, topic.clone());
                                                } else {
                                                    start(ui, &lecture_id, Some(ids.clone()));
                                                }
                                            },
                                            span { class: "tap-row-main",
                                                if t.id.is_some() {
                                                    bdi { dir: "ltr", class: "lecture-title", "{t.title}" }
                                                } else {
                                                    span { class: "lecture-title", {crate::i18n::stats::UNTAGGED} }
                                                }
                                                span { class: "muted small",
                                                    "{t.counts.seen}/{t.counts.total} {tx::STARTED} · {crate::i18n::stats::COL_RETENTION} "
                                                    bdi { {pct(t.counts.retention)} }
                                                }
                                            }
                                            span { class: "chev", aria_hidden: "true" }
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
