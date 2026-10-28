//! Answering a paper. Tutor mode reveals after each pick; timed papers and
//! mocks run against a calm clock with a question navigator and flags.

use batna_core::exam::{self, Mode};
use dioxus::prelude::*;

use super::question::QuestionView;
use super::{PHASE, Phase, choose, finish, go, leave, minutes, now_ms, toggle_flag};
use crate::i18n::bank as tx;

#[component]
pub fn Player() -> Element {
    let Phase::Running(p) = PHASE.read().clone() else {
        return rsx! {};
    };
    let n = p.questions.len();
    let at = p.at;
    let q = p.questions[at].clone();
    let a = p.answers[at].clone();
    let timed = p.mode.timed();
    let reveal = !timed && a.chosen.is_some();
    let answered = p.answers.iter().filter(|a| a.chosen.is_some()).count();
    let right = p.questions.iter().zip(&p.answers).filter(|(q, a)| a.correct(q)).count();
    let left = minutes(exam::remaining_ms(n, p.started_ms, now_ms()));
    let last = at + 1 == n;
    let cells: Vec<(usize, bool, bool)> =
        p.answers.iter().enumerate().map(|(i, a)| (i, a.chosen.is_some(), a.flagged)).collect();
    let flagged = p.answers.iter().filter(|a| a.flagged).count();
    let first_blank = p.answers.iter().position(|a| a.chosen.is_none());
    // A timed paper with blanks or flags asks once before it goes in.
    let mut confirming = use_signal(|| false);
    let submit = move |_| {
        if answered < n || flagged > 0 {
            let mut confirming = confirming;
            confirming.set(true);
        } else {
            finish();
        }
    };

    rsx! {
        div { class: "bank-run", "data-mode": if timed { "timed" } else { "tutor" },
            div { class: "run-main",
                div { class: "run-head",
                    span { class: "run-count", "{tx::QUESTION} " bdi { "{at + 1}" } " {tx::OF} " bdi { "{n}" } }
                    if timed {
                        span { class: "muted small", "{tx::LEFT_ABOUT} " bdi { "{left}" } " {tx::MINUTES}" }
                    } else {
                        span { class: "muted small", "{tx::RIGHT} " bdi { "{right}/{answered}" } }
                    }
                    button {
                        class: "link",
                        title: if timed { tx::LEAVE_TIMED_HINT } else { "" },
                        onclick: move |_| leave(),
                        {tx::LEAVE}
                    }
                }
                if timed {
                    Timer { key: "{p.started_ms}", n, started_ms: p.started_ms }
                }
                QuestionView {
                    key: "{q.id}-{at}",
                    q: q.clone(),
                    chosen: a.chosen,
                    reveal,
                    onchoose: if reveal { None } else { Some(EventHandler::new(choose)) },
                }
                div { class: "run-actions",
                    if timed {
                        button { class: "button ghost", disabled: at == 0, onclick: move |_| go(at.saturating_sub(1)), {tx::PREV} }
                        button {
                            class: "button ghost",
                            "aria-pressed": if a.flagged { "true" } else { "false" },
                            onclick: move |_| toggle_flag(),
                            if a.flagged { "⚑ {tx::FLAGGED}" } else { "⚐ {tx::FLAG}" }
                        }
                    }
                    if last {
                        button {
                            class: "button primary",
                            disabled: !timed && a.chosen.is_none(),
                            // A whole timed paper never goes in on a stray Enter.
                            "aria-keyshortcuts": if timed { "" } else { "Enter" },
                            onclick: move |e| if timed { submit(e) } else { finish() },
                            if timed { {tx::SUBMIT} } else { {tx::FINISH} }
                        }
                    } else {
                        button {
                            class: "button primary",
                            disabled: !timed && a.chosen.is_none(),
                            "aria-keyshortcuts": "Enter",
                            onclick: move |_| go(at + 1),
                            {tx::NEXT}
                        }
                    }
                }
            }
            if timed {
                aside { class: "run-side card",
                    div { class: "navigator", role: "list",
                        for (i, done, flagged) in cells {
                            button {
                                key: "{i}",
                                role: "listitem",
                                class: "nav-cell",
                                "data-done": if done { "yes" } else { "no" },
                                "data-flag": if flagged { "yes" } else { "no" },
                                "aria-current": if i == at { "true" } else { "false" },
                                onclick: move |_| go(i),
                                "{i + 1}"
                            }
                        }
                    }
                    p { class: "muted small", bdi { "{n - answered}" } " {tx::UNANSWERED}" }
                    button { class: "button primary", onclick: submit, {tx::SUBMIT} }
                }
            }
            if timed && confirming() {
                ConfirmSubmit {
                    blank: n - answered,
                    flagged,
                    left,
                    first_blank,
                    onclose: move |_| confirming.set(false),
                }
            }
        }
    }
}

/// "Hand it in?" for a timed paper with blanks or flags. The safe choice,
/// going back to the first blank, comes first and is what Escape does.
#[component]
fn ConfirmSubmit(
    blank: usize,
    flagged: usize,
    left: u64,
    first_blank: Option<usize>,
    onclose: EventHandler<()>,
) -> Element {
    let back = move |_| {
        onclose.call(());
        if let Some(i) = first_blank {
            go(i);
        }
    };
    rsx! {
        div { class: "bank-scrim", onclick: move |_| onclose.call(()),
            div {
                class: "bank-sheet card",
                role: "alertdialog",
                "aria-modal": "true",
                "aria-labelledby": "bank-confirm-title",
                onclick: move |e| e.stop_propagation(),
                h2 { id: "bank-confirm-title", {tx::CONFIRM_TITLE} }
                if blank > 0 {
                    p { {tx::CONFIRM_BLANK_A} " " bdi { "{blank}" } " " {tx::CONFIRM_BLANK_B} }
                }
                if flagged > 0 {
                    p { bdi { "{flagged}" } " " {tx::CONFIRM_FLAGGED} }
                }
                p { class: "muted small", "{tx::LEFT_ABOUT} " bdi { "{left}" } " {tx::MINUTES}" }
                div { class: "start-row",
                    button {
                        class: "button primary",
                        autofocus: true,
                        "aria-keyshortcuts": "Escape",
                        onclick: back,
                        if let Some(i) = first_blank {
                            {tx::CONFIRM_BACK_TO} " " bdi { "{i + 1}" }
                        } else {
                            {tx::CONFIRM_BACK}
                        }
                    }
                    button {
                        class: "button",
                        onclick: move |_| {
                            onclose.call(());
                            finish();
                        },
                        {tx::CONFIRM_SUBMIT}
                    }
                }
            }
        }
    }
}

/// The clock: a bar that drains over the paper's time. It is pure CSS, so
/// nothing ticks; its delay is fixed when it mounts, and the end of the
/// animation submits the paper. Choosing or moving also checks the time, for
/// systems with animations turned off.
#[component]
fn Timer(n: usize, started_ms: i64) -> Element {
    let elapsed_s = use_hook(|| (now_ms() - started_ms).max(0) as f64 / 1000.0);
    let limit_s = exam::time_limit_ms(n) as f64 / 1000.0;
    rsx! {
        div { class: "timer", aria_hidden: "true",
            div {
                class: "timer-fill",
                style: "animation-duration: {limit_s}s; animation-delay: -{elapsed_s}s;",
                onanimationend: move |_| {
                    if matches!(&*PHASE.read(), Phase::Running(p) if p.mode != Mode::Tutor) {
                        finish();
                    }
                },
            }
        }
    }
}
