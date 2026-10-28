//! "Mention / Enumerate": recall as many points as you can (typed, or said
//! to yourself), then tick the ones you got. Typed recall pre-ticks matches.

use batna_core::content::{Body, Item};
use batna_core::review::{Grade, enumerate_grade};
use batna_core::session::recalled;
use dioxus::prelude::*;

use super::grade::GradeRow;
use crate::i18n::study as tx;

#[component]
pub fn EnumerateItem(item: Item, on_grade: EventHandler<Grade>) -> Element {
    let Body::Enumerate { prompt, points, pass } = &item.body else { return rsx! {} };
    let (total, pass) = (points.len(), *pass);
    let mut typed = use_signal(String::new);
    let mut ticks = use_signal(|| None::<Vec<bool>>);
    let pts = points.clone();
    let open = move |_| ticks.set(Some(recalled(&pts, &typed.read())));

    let got = ticks.read().as_ref().map(|t| t.iter().filter(|x| **x).count());
    let wrote = typed.read().trim().to_string();
    // Tablet and Mac: the prompt and the recall on one side, the points on the
    // other. Once open, the points take the wide column; grading sits under both.
    rsx! {
        crate::desk::Panes {
            wide_side: got.is_some(),
            primary: rsx! {
                p { class: "st-prompt", lang: "en", dir: "ltr", "{prompt}" }
                p { class: "muted small", "{tx::ENUM_GOAL_A} {total} {tx::ENUM_GOAL_B} {pass} {tx::ENUM_GOAL_C}" }
                if got.is_none() {
                    textarea {
                        class: "st-recall",
                        lang: "en",
                        dir: "ltr",
                        rows: "5",
                        placeholder: tx::ENUM_PLACEHOLDER,
                        value: "{typed}",
                        oninput: move |e| typed.set(e.value()),
                    }
                    div { class: "st-actions",
                        button { class: "st-primary", "aria-keyshortcuts": "Enter", onclick: open, {tx::SHOW_POINTS} }
                    }
                } else if !wrote.is_empty() {
                    section { class: "st-mine-col",
                        h3 { {tx::WHAT_I_WROTE} }
                        p { lang: "en", dir: "ltr", class: "st-pre", "{wrote}" }
                    }
                }
            },
            secondary: got.map(|n| rsx! {
                p { class: "muted small",
                    {tx::TICK_WHAT_YOU_GOT}
                    if !wrote.is_empty() {
                        " {tx::PRE_TICKED}"
                    }
                }
                ul { class: "st-points", lang: "en", dir: "ltr",
                    for (i , point) in points.iter().enumerate() {
                        li { key: "{i}",
                            label { class: "st-point",
                                input {
                                    r#type: "checkbox",
                                    checked: ticks.read().as_ref().is_some_and(|t| t[i]),
                                    onchange: move |e| {
                                        if let Some(t) = ticks.write().as_mut() {
                                            t[i] = e.checked();
                                        }
                                    },
                                }
                                span { "{point}" }
                            }
                        }
                    }
                }
                p {
                    class: "st-verdict",
                    "data-match": if n >= pass { "right" } else { "wrong" },
                    "{n} {tx::OF} {total} · "
                    if n >= pass { {tx::ENUM_PASS} } else { "{tx::ENUM_SHORT_A} {pass - n} {tx::ENUM_SHORT_B}" }
                }
            }),
            footer: got.map(|n| rsx! {
                GradeRow { item_id: item.id.clone(), suggested: Some(enumerate_grade(n, total, pass)), on_grade }
            }),
        }
    }
}
