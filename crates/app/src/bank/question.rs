//! One MCQ on screen: stem, options, and (once revealed) the right answer,
//! why each option is right or wrong, the book page and any guideline update.

use batna_core::content::Body;
use batna_core::exam::Question;
use dioxus::prelude::*;

use super::{lecture_number, lookup, topic_title};
use crate::i18n::bank as tx;

#[component]
pub fn QuestionView(
    q: Question,
    chosen: Option<usize>,
    reveal: bool,
    /// Called with the original option index; `None` makes the options read-only.
    onchoose: Option<EventHandler<usize>>,
) -> Element {
    let Some((lecture, item)) = lookup(&q.id) else {
        return rsx! { p { class: "note", {tx::NOT_FOUND} } };
    };
    let Body::Mcq { stem, options, why, .. } = &item.body else {
        return rsx! { p { class: "note", {tx::NOT_FOUND} } };
    };
    let topic = item.topic.as_deref().map(|t| topic_title(&lecture.lecture, t));
    let rows = q.order.iter().enumerate().filter_map(|(shown, &orig)| {
        let text = options.get(orig)?.clone();
        let reason = why.get(orig).cloned().unwrap_or_default();
        let letter = char::from(b'A' + shown as u8);
        let state = match (reveal, orig == q.answer, chosen == Some(orig)) {
            (true, true, _) => "right",
            (true, false, true) => "wrong",
            (false, _, true) => "picked",
            _ => "",
        };
        Some((orig, letter, text, reason, state))
    });
    let locked = onchoose.is_none();
    rsx! {
        article { class: "q card",
            div { class: "q-meta",
                span { class: "chip", bdi { "L{lecture_number(&lecture.lecture)} · {lecture.title}" } }
                if let Some(t) = topic {
                    span { class: "chip", bdi { "{t}" } }
                }
            }
            p { class: "q-stem", lang: "en", dir: "ltr", "{stem}" }
            ol { class: "q-options", lang: "en", dir: "ltr",
                for (orig, letter, text, reason, state) in rows {
                    li { key: "{orig}",
                        button {
                            class: "q-option",
                            "data-state": state,
                            disabled: locked,
                            "aria-pressed": if chosen == Some(orig) { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(h) = onchoose {
                                    h.call(orig);
                                }
                            },
                            span { class: "q-letter", "{letter}" }
                            span { class: "q-text", "{text}" }
                            if reveal && state == "right" {
                                span { class: "q-mark", lang: "ar", dir: "rtl", "✓ {tx::RIGHT}" }
                            }
                            if reveal && state == "wrong" {
                                span { class: "q-mark", lang: "ar", dir: "rtl", "✗ {tx::YOUR_ANSWER}" }
                            }
                        }
                        if reveal && !reason.is_empty() {
                            p { class: "q-why", "{reason}" }
                        }
                    }
                }
            }
            if reveal {
                div { class: "q-foot",
                    span { class: "chip page", bdi { "{tx::BOOK_PAGE} {item.src.page}" } }
                }
                if let Some(u) = &item.update {
                    aside { class: "q-update",
                        strong { {tx::UPDATE} }
                        p { lang: "en", dir: "ltr", "{u.text}" }
                        p { class: "muted small", bdi { "{u.reference}" } " · " {tx::UPDATE_NOTE} }
                    }
                }
            }
        }
    }
}
