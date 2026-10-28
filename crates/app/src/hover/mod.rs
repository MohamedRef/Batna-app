//! Hover peeks with the S Pen or the mouse (PARITY.md §6.3, §6.4): a glossary
//! word's expansion, a book page's short quote, an MCQ option's `why`. The
//! pointer work is in `assets/hover.js`; this side hands it the glossary and
//! gives feature screens [`BookPage`].

use std::sync::OnceLock;

use batna_core::glossary::Glossary;
use dioxus::prelude::*;

use crate::i18n::hover as tx;

const HOVER_JS: &str = include_str!("../../assets/hover.js");
const GLOSSARY_YAML: &str = include_str!("../../../../content/glossary.yaml");

/// The glossary bundled with the app. A file that doesn't parse gives an
/// empty glossary (CI's `batna-tools validate` fails on it first).
pub fn glossary() -> &'static Glossary {
    static G: OnceLock<Glossary> = OnceLock::new();
    G.get_or_init(|| {
        serde_yaml_ng::from_str(GLOSSARY_YAML)
            .unwrap_or(Glossary { schema: String::new(), entries: Vec::new() })
    })
}

pub fn term_count() -> usize {
    glossary().entries.len()
}

/// Installs the hover script once and gives it the glossary.
pub fn use_hover() {
    use_hook(|| {
        let table = glossary().peek_table();
        document::eval(&format!(
            "{HOVER_JS}\nwindow.__batnaHover.setGlossary({table}, {note:?});",
            note = tx::UNVERIFIED
        ));
    });
}

/// The "ص 14" chip. Hovering it shows the item's short `src.quote` (when
/// there is one), and a tap or click opens the page's sheet
/// ([`crate::book`]): the quote, the lecture, and a session on that page.
#[component]
pub fn BookPage(page: u32, quote: Option<String>) -> Element {
    let peek = quote.clone().unwrap_or_default();
    rsx! {
        button {
            class: "chip page tap-chip",
            "data-peek": peek,
            aria_label: "{crate::i18n::book::CHIP_HINT} {page}",
            aria_haspopup: "dialog",
            onclick: move |e| {
                // Inside a card or row that has its own action, the chip only opens the page.
                e.stop_propagation();
                crate::pagesheet::open(page, quote.clone());
            },
            bdi { {crate::i18n::cite::page(page)} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_glossary_parses() {
        assert!(term_count() > 50);
        assert!(glossary().find("SAAG").is_some());
    }
}
