//! The lesson player: one step at a time, a segmented progress bar, Back and
//! Continue at the foot. Questions must be answered (or skipped) before
//! Continue; only first tries count. The end screen recaps the lesson and
//! hands its practice items to a study session. The layout follows Kabar's
//! lesson player (adapted from kabar-app main: crates/app/src/lesson.rs).

use batna_core::lesson::{Lesson, StepBody};
use dioxus::prelude::*;

use super::steps::StepView;
use super::text::Line;
use super::{Mark, close, mark, next_lesson, save_mark};
use crate::i18n::learn as tx;
use crate::{Screen, Ui};

/// Local (per device) setting: the step to reopen an unfinished lesson at.
fn pos_key(id: &str) -> String {
    format!("lessonpos.{id}")
}

#[component]
pub fn Player(lesson: Lesson, lecture_title: String) -> Element {
    let ui = use_context::<Ui>();
    let n = lesson.steps.len();
    let id = lesson.id.clone();
    let mut at = use_signal(|| {
        // BATNA_LESSON_STEP=4 opens at that step (1-based), for screenshots.
        let preview = std::env::var("BATNA_LESSON_STEP").ok().and_then(|s| s.parse::<usize>().ok());
        if let Some(s) = preview {
            return s.saturating_sub(1).min(n);
        }
        let saved: usize = crate::store::shared().local_setting(&pos_key(&id)).unwrap_or(0);
        if saved < n && mark(&id).is_none() { saved } else { 0 }
    });
    // First answer per step: Some(right).
    let mut answers = use_signal(|| vec![None::<bool>; n]);
    let mut skipped = use_signal(|| vec![false; n]);

    let go = {
        let id = id.clone();
        move |to: usize| {
            at.set(to);
            let _ = crate::store::shared().put_local_setting(&pos_key(&id), &to.min(n.saturating_sub(1)));
            if to == n {
                let a = answers.read();
                let asked = lesson_questions(&a, &skipped.read());
                let right = a.iter().filter(|x| **x == Some(true)).count() as u32;
                save_mark(&id, Mark { done_ms: crate::store::now_ms(), right, asked });
                let _ = crate::store::shared().put_local_setting(&pos_key(&id), &0usize);
                crate::sounds::play(crate::sounds::Cue::SessionEnd);
            }
            document::eval(
                "document.querySelector('.content')?.scrollTo({top: 0}); window.scrollTo({top: 0});",
            );
        }
    };

    if at() >= n {
        return rsx! { Summary { lesson: lesson.clone(), answers: answers.read().clone(), skipped: skipped.read().clone() } };
    }

    let i = at();
    let step = lesson.steps[i].clone();
    let is_q = step.body.is_question() || matches!(step.body, StepBody::Try { .. });
    let can_go = !is_q || answers.read()[i].is_some() || skipped.read()[i];
    let seed = format!("{}#{}", lesson.id, i);
    let mut go_next = go.clone();
    let mut go_back = go.clone();
    let mut go_skip = go.clone();

    rsx! {
        div { class: "ls-player",
            header { class: "ls-bar",
                button {
                    class: "ls-close",
                    "data-back": "true",
                    "aria-keyshortcuts": "Escape",
                    aria_label: tx::CLOSE,
                    title: tx::CLOSE,
                    onclick: move |_| close(ui),
                    "×"
                }
                div { class: "ls-segs", role: "progressbar", aria_valuemin: "0", aria_valuemax: "{n}", aria_valuenow: "{i + 1}",
                    for (k , s) in lesson.steps.iter().enumerate() {
                        span {
                            key: "{k}",
                            class: "ls-seg",
                            "data-q": if s.body.is_question() { "yes" } else { "no" },
                            "data-state": if k < i { "done" } else if k == i { "now" } else { "todo" },
                        }
                    }
                }
                span { class: "muted small ls-count", "{i + 1}/{n}" }
            }
            p { class: "ls-where muted small",
                Line { text: lesson.title.clone() }
                " · "
                bdi { "{lecture_title}" }
            }
            article { key: "{i}", class: "card ls-stage", "data-kind": step.body.kind(),
                StepView {
                    body: step.body.clone(),
                    seed,
                    on_answer: move |right: bool| {
                        let mut a = answers.write();
                        if a[i].is_none() {
                            a[i] = Some(right);
                        }
                    },
                }
                if let Some(page) = step.page {
                    footer { class: "ls-src",
                        crate::hover::BookPage { label: tx::BOOK_PAGE, page, quote: step.quote.clone() }
                    }
                }
            }
            footer { class: "ls-nav",
                button {
                    class: "ls-btn",
                    disabled: i == 0,
                    onclick: move |_| go_back(i - 1),
                    {tx::BACK}
                }
                if is_q && !can_go {
                    button {
                        class: "ls-link",
                        onclick: move |_| {
                            skipped.write()[i] = true;
                            go_skip(i + 1);
                        },
                        {tx::SKIP}
                    }
                }
                button {
                    class: "ls-btn primary ls-next",
                    "aria-keyshortcuts": "Enter",
                    disabled: !can_go,
                    onclick: move |_| go_next(i + 1),
                    {if i + 1 == n { tx::FINISH } else { tx::NEXT }}
                }
            }
        }
    }
}

/// Questions that count: answered ones (skips don't count against him).
fn lesson_questions(answers: &[Option<bool>], skipped: &[bool]) -> u32 {
    answers.iter().zip(skipped).filter(|(a, s)| a.is_some() && !**s).count() as u32
}

#[component]
fn Summary(lesson: Lesson, answers: Vec<Option<bool>>, skipped: Vec<bool>) -> Element {
    let mut ui = use_context::<Ui>();
    let asked = lesson_questions(&answers, &skipped);
    let right = answers.iter().filter(|a| **a == Some(true)).count() as u32;
    let lecture = lesson.id.rsplit_once('.').map(|(l, _)| l.to_string()).unwrap_or_default();
    let next = next_lesson(std::slice::from_ref(&lecture)).filter(|l| l.id != lesson.id);
    let practice = lesson.practice.clone();
    let n_practice = practice.len();
    rsx! {
        section { class: "card ls-summary",
            p { class: "ls-confetti", aria_hidden: "true", "🎉" }
            h1 { {tx::DONE_TITLE} }
            p { class: "muted", Line { text: lesson.title.clone() } }
            if asked > 0 {
                p { class: "ls-score",
                    strong { "{right}/{asked}" }
                    " {tx::FIRST_TRY}"
                }
                p { class: "small", {tx::score_line(right, asked)} }
            }
            h2 { {tx::RECAP} }
            ul { class: "ls-recap",
                for (k , r) in lesson.recap.iter().enumerate() {
                    li { key: "{k}", Line { text: r.clone() } }
                }
            }
            div { class: "ls-summary-actions",
                if n_practice > 0 {
                    button {
                        class: "ls-btn primary",
                        onclick: move |_| {
                            *crate::study::START.write() = Some(crate::study::Start {
                                lectures: None,
                                items: Some(practice.clone()),
                                minutes: crate::onboarding::session_minutes(),
                            });
                            *super::OPEN.write() = None;
                            ui.screen.set(Screen::Study);
                        },
                        "{tx::PRACTICE} ({n_practice})"
                    }
                }
                if let Some(l) = next {
                    button {
                        class: "ls-btn",
                        onclick: move |_| super::open(ui, &l.id),
                        "{tx::NEXT_LESSON}: "
                        Line { text: l.title.clone() }
                    }
                }
                button { class: "ls-link", onclick: move |_| close(ui), {tx::BACK_TO_LIST} }
            }
            p { class: "muted small", {tx::PRACTICE_HINT} }
        }
    }
}
