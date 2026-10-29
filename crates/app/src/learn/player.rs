//! The lesson player: one step at a time, a segmented progress bar, Back and
//! Continue at the foot. Questions must be answered (or skipped) before
//! Continue; only first tries count. The end screen recaps the lesson and
//! hands its practice items to a study session. The layout follows Kabar's
//! lesson player (adapted from kabar-app main: crates/app/src/lesson.rs).

use batna_core::lesson::{Lesson, Step, StepBody};
use dioxus::prelude::*;

use super::practice::{Tries, TryLog};
use super::steps::{CheckBar, StepAt, StepView};
use super::text::Line;
use super::{Mark, VIEW, View, close, mark, next_lesson, save_mark, set_view};
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
    /// Try steps part-way through: the cards already graded, so leaving
    /// and coming back never grades a card twice (#318).
    #[serde(default)]
    tries: Vec<TryLog>,
}

fn save_pos(id: &str, pos: &Pos) {
    if let Err(e) = crate::store::shared().put_local_setting(&pos_key(id), pos) {
        crate::diag::report!("saving lesson position {id} failed: {e}");
    }
}

/// Swiping between steps, with the page following the finger (swipe.js).
const SWIPE_JS: &str = include_str!("swipe.js");

#[component]
pub fn Player(lesson: Lesson, lecture_title: String) -> Element {
    let ui = use_context::<Ui>();
    // The lesson gets the whole phone screen; × (data-back) is the way out.
    crate::parts::use_focus_mode(true);
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
    let tries = use_signal(|| {
        let mut t = start.tries.clone();
        t.resize(n, TryLog::default());
        t
    });
    // Saves where he is now: after every answer and every graded card, not
    // only on Back/Continue, since the lesson can close mid-step (the page
    // chip's lecture, the rail).
    let save_now = {
        let id = id.clone();
        move || {
            if *at.peek() < n {
                save_pos(
                    &id,
                    &Pos {
                        at: *at.peek(),
                        answers: answers.peek().clone(),
                        skipped: skipped.peek().clone(),
                        tries: tries.peek().clone(),
                    },
                );
            }
        }
    };
    use_context_provider({
        let save_now = save_now.clone();
        move || Tries { log: tries, save: Callback::new(move |_| save_now()) }
    });
    let check = use_signal(|| vec![None::<Callback<()>>; n]);
    use_context_provider(|| CheckBar(check));
    // Steps stay mounted once seen (hidden when not current), so Back shows
    // an answered step as he left it, read-only, and never grades it again.
    let mut seen = use_signal(|| (0..n).map(|k| k == start.at).collect::<Vec<_>>());
    // Try steps answered in an earlier session: their grades are reviews
    // already, so they show a done note instead of the cards.
    let tried_before: Vec<bool> = (0..n)
        .map(|k| {
            start.answers.get(k).is_some_and(Option::is_some)
                || start.skipped.get(k).copied().unwrap_or(false)
        })
        .collect();
    let questions: Vec<bool> = lesson.steps.iter().map(|s| s.body.is_question()).collect();
    // Which way the last move went, for the enter animation.
    let mut dir = use_signal(|| "fwd");
    use_hook(|| {
        document::eval(SWIPE_JS);
    });
    let view = VIEW();
    let pages = view == View::Pages;

    let go = {
        let id = id.clone();
        let questions = questions.clone();
        move |to: usize| {
            dir.set(if to < *at.peek() { "back" } else { "fwd" });
            at.set(to);
            if to < n {
                seen.write()[to] = true;
            }
            if to == n {
                let (right, asked) = score(&answers.read(), &skipped.read(), &questions);
                save_mark(&id, Mark { done_ms: crate::store::now_ms(), right, asked });
                save_pos(&id, &Pos::default());
                crate::sounds::play(crate::sounds::Cue::SessionEnd);
            } else {
                save_pos(
                    &id,
                    &Pos {
                        at: to,
                        answers: answers.read().clone(),
                        skipped: skipped.read().clone(),
                        tries: tries.read().clone(),
                    },
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
    let can_go = !is_q || answers.read()[i].is_some() || skipped.read()[i];
    let ready = check.read()[i];
    let mut go_next = go.clone();
    let mut go_back = go.clone();
    let mut go_skip = go.clone();

    rsx! {
        div {
            class: "ls-player",
            "data-view": if pages { "pages" } else { "steps" },
            "data-dir": dir(),
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
                button {
                    class: "ls-view",
                    aria_label: if pages { tx::TO_STEPS } else { tx::TO_PAGES },
                    title: if pages { tx::TO_STEPS } else { tx::TO_PAGES },
                    onclick: move |_| set_view(if pages { View::Steps } else { View::Pages }),
                    {if pages { crate::icons::steps() } else { crate::icons::book() }}
                }
            }
            p { class: "ls-where muted small",
                Line { text: lesson.title.clone() }
                " · "
                bdi { "{lecture_title}" }
            }
            for (k , st) in lesson.steps.iter().enumerate().filter(|(k, _)| seen.read()[*k]) {
                Stage {
                    key: "{k}",
                    k,
                    current: k == i,
                    step: st.clone(),
                    seed: format!("{}#{}", lesson.id, k),
                    try_done: tried_before[k] && matches!(st.body, StepBody::Try { .. }),
                    on_answer: {
                        let save_now = save_now.clone();
                        move |right: bool| {
                            let fresh = answers.peek()[k].is_none();
                            if fresh {
                                answers.write()[k] = Some(right);
                                // Answered after all: it counts.
                                skipped.write()[k] = false;
                                save_now();
                            }
                        }
                    },
                }
            }
            footer { class: "ls-nav",
                button {
                    class: if pages { "ls-arrow" } else { "btn btn-secondary" },
                    "data-nav": "back",
                    aria_label: tx::BACK,
                    title: tx::BACK,
                    disabled: i == 0,
                    onclick: move |_| go_back(i - 1),
                    if pages {
                        {crate::icons::chevron(false)}
                    } else {
                        {tx::BACK}
                    }
                }
                if is_q && !can_go && ready.is_none() {
                    button {
                        class: "btn-link ls-skip",
                        onclick: move |_| {
                            skipped.write()[i] = true;
                            go_skip(i + 1);
                        },
                        {tx::SKIP}
                    }
                } else if pages {
                    span { class: "ls-page-of muted small", "{tx::PAGE_OF} {i + 1} {tx::OF_N} {n}" }
                }
                if let Some(run) = ready.filter(|_| !can_go) {
                    button {
                        class: "btn btn-primary ls-next",
                        "data-nav": "check",
                        "aria-keyshortcuts": "Enter",
                        onclick: move |_| run.call(()),
                        {tx::CHECK_IT}
                    }
                } else if pages && i + 1 < n {
                    button {
                        class: "ls-arrow ls-arrow-next",
                        "data-nav": "next",
                        "aria-keyshortcuts": "Enter",
                        aria_label: tx::NEXT,
                        title: tx::NEXT,
                        disabled: !can_go,
                        onclick: move |_| go_next(i + 1),
                        {crate::icons::chevron(true)}
                    }
                } else {
                    button {
                        class: "btn btn-primary ls-next",
                        "data-nav": "next",
                        "aria-keyshortcuts": "Enter",
                        disabled: !can_go,
                        onclick: move |_| go_next(i + 1),
                        {if i + 1 == n { tx::FINISH } else { tx::NEXT }}
                    }
                }
            }
        }
    }
}

/// One step's card. It stays mounted after he leaves it, hidden.
#[component]
fn Stage(
    k: usize,
    current: bool,
    step: Step,
    seed: String,
    try_done: bool,
    on_answer: EventHandler<bool>,
) -> Element {
    use_context_provider(|| StepAt(k));
    rsx! {
        article { class: "card ls-stage", hidden: !current, "data-kind": step.body.kind(), "data-n": k + 1,
            if try_done {
                div { class: "ls-try-done",
                    p { class: "ls-kicker", {tx::TRY} }
                    p { {tx::TRY_DONE} }
                    p { class: "muted small", {tx::TRY_DONE_HINT} }
                }
            } else {
                StepView { body: step.body.clone(), seed, on_answer }
            }
            if let Some(page) = step.page {
                footer { class: "ls-src",
                    crate::hover::BookPage { page, quote: step.quote.clone() }
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
                        span { class: "ls-btn-label",
                            "{tx::NEXT_LESSON}: "
                            Line { text: l.title.clone() }
                        }
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
