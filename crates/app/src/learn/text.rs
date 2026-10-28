//! Lesson text: Egyptian Arabic with English terms. `batna_core::lesson`
//! splits it into paragraphs, bullets, bold and Latin runs; this draws them,
//! each Latin run isolated left-to-right so "Child-Pugh class C" never flips.

use batna_core::lesson::{Block, Span, blocks, spans};
use dioxus::prelude::*;

/// Paragraphs and bullet lists.
#[component]
pub fn Rich(text: String, #[props(default)] class: Option<&'static str>) -> Element {
    let blocks = blocks(&text);
    rsx! {
        div { class: class.unwrap_or("ls-rich"),
            for (i , b) in blocks.into_iter().enumerate() {
                match b {
                    Block::Para(s) => rsx! { p { key: "{i}", {runs(s)} } },
                    Block::Bullets(items) => rsx! {
                        ul { key: "{i}",
                            for (j , s) in items.into_iter().enumerate() {
                                li { key: "{j}", {runs(s)} }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// One line, no paragraphs (option labels, recap lines, titles).
#[component]
pub fn Line(text: String) -> Element {
    runs(spans(&text))
}

fn runs(spans: Vec<Span>) -> Element {
    rsx! {
        for (i , s) in spans.into_iter().enumerate() {
            {
                let inner = if s.latin {
                    rsx! { bdi { dir: "ltr", lang: "en", "{s.text}" } }
                } else {
                    rsx! { "{s.text}" }
                };
                if s.bold {
                    rsx! { strong { key: "{i}", {inner} } }
                } else {
                    rsx! { span { key: "{i}", {inner} } }
                }
            }
        }
    }
}

/// Mostly-English text (an option, a table cell) runs left-to-right.
pub fn is_latin(text: &str) -> bool {
    let latin = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let arabic = text.chars().filter(|c| ('\u{0600}'..='\u{06FF}').contains(c)).count();
    latin > arabic
}
