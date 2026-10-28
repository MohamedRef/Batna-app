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
    // Neighbouring Latin spans (a bold word inside an English phrase) share
    // one isolate, so the phrase keeps its order.
    let mut groups: Vec<(bool, Vec<Span>)> = Vec::new();
    for s in spans {
        match groups.last_mut() {
            Some((latin, g)) if *latin && s.latin => g.push(s),
            _ => groups.push((s.latin, vec![s])),
        }
    }
    rsx! {
        for (i , (latin , g)) in groups.into_iter().enumerate() {
            if latin {
                bdi { key: "{i}", dir: "ltr", lang: "en", {styled(g)} }
            } else {
                span { key: "{i}", {styled(g)} }
            }
        }
    }
}

fn styled(spans: Vec<Span>) -> Element {
    rsx! {
        for (i , s) in spans.into_iter().enumerate() {
            if s.bold {
                strong { key: "{i}", "{s.text}" }
            } else {
                span { key: "{i}", "{s.text}" }
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
