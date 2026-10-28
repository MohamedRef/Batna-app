//! A recall card: read the question, answer in your head, reveal, grade.

use batna_core::content::{Body, Item};
use batna_core::review::Grade;
use dioxus::prelude::*;

use super::grade::GradeRow;
use crate::i18n::study as tx;

#[component]
pub fn CardItem(item: Item, on_grade: EventHandler<Grade>) -> Element {
    let Body::Card { front, back } = &item.body else { return rsx! {} };
    let mut shown = use_signal(|| false);
    // Tablet and Mac: the question on one side, the answer on the other.
    rsx! {
        crate::desk::Panes {
            primary: rsx! {
                p { class: "st-prompt", lang: "en", dir: "ltr", "{front}" }
                if !shown() {
                    p { class: "muted small", {tx::THINK_FIRST} }
                    button {
                        class: "btn btn-primary btn-lg",
                        autofocus: true,
                        "aria-keyshortcuts": "Space",
                        onclick: move |_| shown.set(true),
                        {tx::REVEAL}
                    }
                }
            },
            secondary: shown().then(|| rsx! {
                div { class: "st-answer", lang: "en", dir: "ltr", "{back}" }
                GradeRow { item_id: item.id.clone(), suggested: None, on_grade }
            }),
        }
    }
}
