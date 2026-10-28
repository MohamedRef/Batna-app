//! After a paper: the score, per lecture and per topic, the weakest topics,
//! and a review of every question with its explanation.

use batna_core::content::Body;
use batna_core::exam::{self, Mode, Score};
use dioxus::prelude::*;

use super::question::QuestionView;
use super::{PHASE, Phase, back_to_setup, lookup, minutes, short_lecture_title, start, topic_title};
use crate::content::lecture_code;
use crate::i18n::bank as tx;
use crate::i18n::count::{MINUTE, count};

#[derive(Clone, Copy, PartialEq)]
enum Show {
    All,
    Wrong,
    Flagged,
}

#[component]
pub fn Result() -> Element {
    let Phase::Done { paper, outcome } = PHASE.read().clone() else {
        return rsx! {};
    };
    let mut show =
        use_signal(|| if outcome.total.right < outcome.total.total { Show::Wrong } else { Show::All });
    let timed = paper.mode.timed();

    let lectures: Vec<(String, Score)> = outcome.by_lecture.iter().map(|(l, s)| (l.clone(), *s)).collect();
    let topics: Vec<(String, String, Score)> = outcome
        .by_topic
        .iter()
        .map(|(k, s)| {
            let (l, t) = k.split_once('/').unwrap_or((k, ""));
            (l.to_string(), t.to_string(), *s)
        })
        .collect();
    let weak: Vec<(String, String, Score)> = exam::weak_topics(&outcome, 60, 2)
        .into_iter()
        .take(5)
        .map(|(k, s)| {
            let (l, t) = k.split_once('/').unwrap_or((k, ""));
            (l.to_string(), t.to_string(), s)
        })
        .collect();
    let several_lectures = lectures.len() > 1;

    let wrong: Vec<_> = paper
        .questions
        .iter()
        .zip(&paper.answers)
        .filter(|(q, a)| !a.correct(q))
        .map(|(q, _)| q.clone())
        .collect();
    let has_wrong = !wrong.is_empty();
    let filter = *show.read();
    let rows: Vec<_> = paper
        .questions
        .iter()
        .zip(&paper.answers)
        .enumerate()
        .filter(|(_, (q, a))| match filter {
            Show::All => true,
            Show::Wrong => !a.correct(q),
            Show::Flagged => a.flagged,
        })
        .map(|(i, (q, a))| (i, q.clone(), a.clone()))
        .collect();
    let any_flagged = paper.answers.iter().any(|a| a.flagged);
    let mut open = use_signal(|| None::<usize>);
    let detail = open().and_then(|i| Some((i, paper.questions.get(i)?.clone(), paper.answers.get(i)?.clone()))).map(
        |(i, q, a)| {
            rsx! {
                div { class: "review-detail",
                    div { class: "review-detail-head",
                        span { class: "run-count", "{tx::QUESTION} " bdi { "{i + 1}" } }
                        button { class: "btn-link", "aria-keyshortcuts": "Escape", onclick: move |_| open.set(None), {tx::CLOSE} }
                    }
                    QuestionView { key: "{i}", q, chosen: a.chosen, reveal: true }
                }
            }
        },
    );

    rsx! {
        section { class: "card result",
            h1 {
                if paper.mode == Mode::Mock { {tx::MOCK_TITLE} } else { {tx::RESULT} }
            }
            div { class: "big-score",
                bdi { class: "pct", "{outcome.total.percent()}%" }
                span { class: "muted", bdi { "{outcome.total.right}" } " {tx::OF} " bdi { "{outcome.total.total}" } }
            }
            div { class: "muted small",
                if timed {
                    "{tx::TOOK} " {count(minutes(outcome.took_ms), &MINUTE)}
                }
                if outcome.unanswered > 0 {
                    " · " bdi { "{outcome.unanswered}" } " {tx::UNANSWERED}"
                }
            }
            if has_wrong {
                p { class: "small", {tx::SENT_TO_REVIEW} }
            }
        }

        if several_lectures {
            section { class: "card",
                h2 { {tx::BY_LECTURE} }
                ScoreRows {
                    rows: lectures.into_iter().map(|(l, s)| (format!("{} · {}", lecture_code(&l), short_lecture_title(&l)), s)).collect::<Vec<_>>(),
                }
            }
        }
        if !topics.is_empty() {
            section { class: "card",
                h2 { {tx::BY_TOPIC} }
                ScoreRows {
                    rows: topics.into_iter().map(|(l, t, s)| (topic_title(&l, &t), s)).collect::<Vec<_>>(),
                }
                h2 { class: "label", {tx::WEAK} }
                if weak.is_empty() {
                    p { class: "muted small", {tx::ALL_STRONG} }
                } else {
                    div { class: "chips",
                        for (l, t, s) in weak {
                            span { key: "{l}/{t}", class: "chip weak",
                                bdi { {topic_title(&l, &t)} }
                                " · " bdi { "{s.right}/{s.total}" }
                            }
                        }
                    }
                }
            }
        }

        section { class: "card",
            h2 { {tx::REVIEW} }
            div { class: "chips",
                for (value, label) in [(Show::All, tx::SHOW_ALL), (Show::Wrong, tx::SHOW_WRONG), (Show::Flagged, tx::SHOW_FLAGGED)] {
                    if value != Show::Flagged || any_flagged {
                        button {
                            class: "chip toggle",
                            "aria-pressed": if filter == value { "true" } else { "false" },
                            onclick: move |_| show.set(value),
                            {label}
                        }
                    }
                }
            }
        }
        crate::desk::ListDetail {
            list: rsx! {
                div { class: "review-list",
                    for (i, q, a) in rows {
                        button {
                            key: "{i}",
                            class: "review-item",
                            "data-right": if a.correct(&q) { "yes" } else { "no" },
                            "aria-current": if open() == Some(i) { "true" } else { "false" },
                            onclick: move |_| open.set(Some(i)),
                            span { class: "mark", if a.correct(&q) { "✓" } else { "✗" } }
                            span { "{tx::QUESTION} " bdi { "{i + 1}" } }
                            bdi { class: "stem-peek", {stem_peek(&q.id)} }
                            if a.flagged {
                                span { class: "muted", " ⚑" }
                            }
                        }
                    }
                }
            },
            detail: detail,
        }

        div { class: "start-row",
            if has_wrong {
                button {
                    class: "btn btn-primary",
                    onclick: move |_| start(Mode::Tutor, wrong.clone()),
                    {tx::RETRY_WRONG}
                }
            }
            button { class: "btn btn-secondary", onclick: move |_| back_to_setup(), {tx::BACK} }
        }
    }
}

#[component]
fn ScoreRows(rows: Vec<(String, Score)>) -> Element {
    rsx! {
        ul { class: "score-rows",
            for (i, (label, s)) in rows.into_iter().enumerate() {
                li { key: "{i}",
                    bdi { class: "score-label", "{label}" }
                    div { class: "score-bar", aria_hidden: "true",
                        div { class: "score-fill", style: "inline-size: {s.percent()}%;" }
                    }
                    bdi { class: "score-num", "{s.right}/{s.total}" }
                }
            }
        }
    }
}

fn stem_peek(id: &str) -> String {
    match lookup(id).map(|(_, i)| i.body) {
        Some(Body::Mcq { stem, .. }) => stem.chars().take(90).collect(),
        _ => String::new(),
    }
}
