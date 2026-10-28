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

/// Local (per device) setting: where an unfinished lesson stands, so it
/// reopens at the same step with the same score.
fn pos_key(id: &str) -> String {
    format!("lessonpos.{id}")
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Pos {
    at: usize,
    /// First response per step: Some(right). Reveals and Try steps record
    /// too (they gate Continue), but only question steps are scored.
    answers: Vec<Option<bool>>,
    skipped: Vec<bool>,
}

fn save_pos(id: &str, pos: &Pos) {
    if let Err(e) = crate::store::shared().put_local_setting(&pos_key(id), pos) {
        crate::diag::report!("saving lesson position {id} failed: {e}");
    }
}

#[component]
pub fn Player(lesson: Lesson, lecture_title: String) -> Element {
    let ui = use_context::<Ui>();
    let n = lesson.steps.len();
    let id = lesson.id.clone();
    let start = use_hook(|| {
        // BATNA_LESSON_STEP=4 opens at that step (1-based), for screenshots.
        let preview = std::env::var("BATNA_LESSON_STEP").ok().and_then(|s| s.parse::<usize>().ok());
        if let Some(s) = preview {
            return Pos { at: s.saturating_sub(1).min(n), ..Pos::default() };
        }
        let saved: Option<Pos> = crate::store::shared().local_setting(&pos_key(&id));
        match saved {
            Some(p) if p.at < n && p.answers.len() == n && p.skipped.len() == n && mark(&id).is_none() => p,
            _ => Pos::default(),
        }
    });
    let mut at = use_signal(|| start.at);
    let mut answers =
        use_signal(|| if start.answers.len() == n { start.answers.clone() } else { vec![None::<bool>; n] });
    let mut skipped =
        use_signal(|| if start.skipped.len() == n { start.skipped.clone() } else { vec![false; n] });
    let questions: Vec<bool> = lesson.steps.iter().map(|s| s.body.is_question()).collect();

    let go = {
        let id = id.clone();
        let questions = questions.clone();
        move |to: usize| {
            at.set(to);
            if to == n {
                let (right, asked) = score(&answers.read(), &skipped.read(), &questions);
                save_mark(&id, Mark { done_ms: crate::store::now_ms(), right, asked });
                save_pos(&id, &Pos::default());
                crate::sounds::play(crate::sounds::Cue::SessionEnd);
            } else {
                save_pos(
                    &id,
                    &Pos { at: to, answers: answers.read().clone(), skipped: skipped.read().clone() },
                );
            }
            document::eval(
                "document.querySelector('.content')?.scrollTo({top: 0}); window.scrollTo({top: 0});",
            );
        }
    };

    if at() >= n {
        let (right, asked) = score(&answers.read(), &skipped.read(), &questions);
        return rsx! { Summary { lesson: lesson.clone(), right, asked } };
    }

    let i = at();
    let step = lesson.steps[i].clone();
    let is_try = matches!(step.body, StepBody::Try { .. });
    let is_q = step.body.is_question() || is_try;
    // A Try step already played isn't played again: its grades are reviews.
    let try_done = is_try && (answers.read()[i].is_some() || skipped.read()[i]);
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
                if try_done {
                    div { class: "ls-try-done",
                        p { class: "ls-kicker", {tx::TRY} }
                        p { {tx::TRY_DONE} }
                        p { class: "muted small", {tx::TRY_DONE_HINT} }
                    }
                } else {
                    StepView {
                        body: step.body.clone(),
                        seed,
                        on_answer: move |right: bool| {
                            let mut a = answers.write();
                            if a[i].is_none() {
                                a[i] = Some(right);
                                // Answered after all: it counts.
                                skipped.write()[i] = false;
                            }
                        },
                    }
                }
                if let Some(page) = step.page {
                    footer { class: "ls-src",
                        crate::hover::BookPage { page, quote: step.quote.clone() }
                    }
                }
            }
            footer { class: "ls-nav",
                button {
                    class: "btn btn-secondary",
                    disabled: i == 0,
                    onclick: move |_| go_back(i - 1),
                    {tx::BACK}
                }
                if is_q && !can_go {
                    button {
                        class: "btn-link",
                        onclick: move |_| {
                            skipped.write()[i] = true;
                            go_skip(i + 1);
                        },
                        {tx::SKIP}
                    }
                }
                button {
                    class: "btn btn-primary ls-next",
                    "aria-keyshortcuts": "Enter",
                    disabled: !can_go,
                    onclick: move |_| go_next(i + 1),
                    {if i + 1 == n { tx::FINISH } else { tx::NEXT }}
                }
            }
        }
    }
}

/// `(right, asked)` over the question steps he answered: reveals and Try
/// steps don't score, and skips don't count against him.
fn score(answers: &[Option<bool>], skipped: &[bool], questions: &[bool]) -> (u32, u32) {
    let counted = || {
        answers
            .iter()
            .zip(skipped)
            .zip(questions)
            .filter(|((_, s), q)| **q && !**s)
            .filter_map(|((a, _), _)| *a)
    };
    (counted().filter(|r| *r).count() as u32, counted().count() as u32)
}

#[component]
fn Summary(lesson: Lesson, right: u32, asked: u32) -> Element {
    let mut ui = use_context::<Ui>();
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
                        class: "btn btn-primary",
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
                        class: "btn btn-secondary",
                        onclick: move |_| super::open(ui, &l.id),
                        "{tx::NEXT_LESSON}: "
                        Line { text: l.title.clone() }
                    }
                }
                button { class: "btn-link", onclick: move |_| close(ui), {tx::BACK_TO_LIST} }
            }
            p { class: "muted small", {tx::PRACTICE_HINT} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn only_answered_questions_score() {
        // check right, reveal, check wrong, try, skipped check.
        let answers = [Some(true), Some(true), Some(false), Some(true), None];
        let skipped = [false, false, false, false, true];
        let questions = [true, false, true, false, true];
        assert_eq!(score(&answers, &skipped, &questions), (1, 2));
    }
}
