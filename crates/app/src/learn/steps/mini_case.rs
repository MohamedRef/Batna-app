//! Lesson mini case interaction and response restoration.

use super::*;

/// A mini case, part by part: pick, check with the footer, read why, go on.
/// The last part's check finishes the case.
#[component]
pub(super) fn MiniCase(
    stem: String,
    steps: Vec<CaseStep>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let n = steps.len();
    // Each question's options in a stable shuffled order, like a check step.
    let orders = use_hook(|| {
        steps
            .iter()
            .enumerate()
            .map(|(si, s)| shuffled(s.options.len(), &format!("{seed}.{si}")))
            .collect::<Vec<_>>()
    });
    let (saved, put) = use_draft();
    let (picks0, at0, pick0) = match saved {
        Draft::Case { picks, at, pick } if picks.len() == n && at < n => (picks, at, pick),
        _ => (vec![None; n], 0, None),
    };
    let mut at = use_signal(|| at0);
    // The checked pick per part (a part without options: Some(0) once shown).
    let mut picks = use_signal(|| picks0);
    // The current part's pick, before Check.
    let mut pick = use_signal(|| pick0);
    let save = use_callback(move |()| {
        put.call(Draft::Case { picks: picks.peek().clone(), at: *at.peek(), pick: *pick.peek() })
    });
    let answers: Vec<Option<usize>> = steps.iter().map(|s| s.answer).collect();
    let has_options: Vec<bool> = steps.iter().map(|s| !s.options.is_empty()).collect();
    let part_answers = answers.clone();
    // A part is settled: the whole case is done once the last one is.
    let settle = use_callback(move |(si, p): (usize, usize)| {
        picks.write()[si] = Some(p);
        pick.set(None);
        save.call(());
        if si + 1 == n {
            let v = picks.peek().clone();
            let right = v.iter().zip(&answers).zip(&has_options).all(|((p, a), o)| !*o || *p == *a);
            on_answer.call(right);
        }
    });
    use_check_bar(
        move || pick().is_some(),
        move |_| {
            let si = *at.peek();
            let Some(p) = *pick.peek() else { return };
            if picks.peek()[si].is_none() {
                settle.call((si, p));
                cue(part_answers[si] == Some(p));
            }
        },
    );
    rsx! {
        div { class: "ls-case",
            p { class: "ls-kicker", {tx::CASE} }
            div { class: "ls-case-stem", dir: "ltr", lang: "en", Rich { text: stem } }
            for (si , s) in steps.iter().take(at() + 1).enumerate() {
                {
                    let done = picks.read()[si];
                    let last = si == at();
                    rsx! {
                        div { key: "{si}", class: "ls-case-step ls-pop",
                            if let Some(r) = &s.reveal { p { class: "ls-case-reveal", dir: "ltr", lang: "en", Line { text: r.clone() } } }
                            p { class: "ls-case-q", dir: "ltr", lang: "en", strong { Line { text: s.question.clone() } } }
                            if s.options.is_empty() {
                                if done.is_none() {
                                    button { class: "btn btn-secondary", "aria-keyshortcuts": "Space", onclick: move |_| settle.call((si, 0)), {tx::CASE_SHOW} }
                                }
                            } else {
                                div { class: "ls-options", dir: "ltr", lang: "en",
                                    for (pos , &oi) in orders[si].iter().enumerate() {
                                        {
                                            let (state, mark) = match done {
                                                None => (if last && pick() == Some(oi) { Some("selected") } else { None }, None),
                                                Some(p) => answer_state(true, Some(p), oi, s.answer.unwrap_or(usize::MAX)),
                                            };
                                            rsx! {
                                                button {
                                                    key: "{oi}",
                                                    class: "answer",
                                                    "aria-keyshortcuts": if last && done.is_none() { LETTERS.get(pos).copied().unwrap_or("") } else { "" },
                                                    "data-state": state,
                                                    disabled: done.is_some(),
                                                    onclick: move |_| {
                                                        if picks.peek()[si].is_none() {
                                                            pick.set(Some(oi));
                                                            save.call(());
                                                        }
                                                    },
                                                    span { class: "answer-key", "{LETTERS.get(pos).copied().unwrap_or(\"\")}" }
                                                    span { class: "answer-text", Line { text: s.options[oi].clone() } }
                                                    if let Some(m) = mark {
                                                        span { class: "answer-mark", dir: "rtl", lang: "ar", {m} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if done.is_some() {
                                div { class: "ls-why-box ls-pop", dir: "ltr", lang: "en", Rich { text: s.why.clone() } }
                                if last && si + 1 < n {
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| {
                                            at.set(si + 1);
                                            save.call(());
                                        },
                                        {tx::CASE_NEXT}
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
