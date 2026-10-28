//! A book page's sheet. Every "ص N" chip in the app opens it (Mohamed,
//! 2026-10-28: no dead chips or links): the page's short quote when there
//! is one, the lecture the page is in, what Batna has made from that page,
//! and two ways on: a session on just that page, or the lecture's page.
//!
//! It opens over whatever is on screen (a lesson, a running session, a
//! paper) and changes nothing until he picks one of its buttons.

use batna_core::review::StudyStore;
use dioxus::prelude::*;

use crate::content::{Library, lecture_code};
use crate::i18n::book as tx;
use crate::i18n::count::{CASE, ITEM, QUESTION, count};
use crate::store::use_review;
use crate::{Screen, Ui};

const CSS: &str = include_str!("book.css");

/// The open page and the quote its chip carried.
static OPEN: GlobalSignal<Option<(u32, Option<String>)>> = Signal::global(|| None);

/// Opens the sheet for book page `page`.
pub fn open(page: u32, quote: Option<String>) {
    document::eval("window.__batnaHover && window.__batnaHover.hide();");
    *OPEN.write() = Some((page, quote.filter(|q| !q.trim().is_empty())));
}

pub fn close() {
    *OPEN.write() = None;
}

/// Whether the sheet is open (Back closes it first).
pub fn is_open() -> bool {
    OPEN.read().is_some()
}

/// What Batna has made from one book page.
#[derive(Debug, Default, PartialEq)]
pub struct PageContent {
    /// The lecture whose page range holds the page.
    pub lecture: Option<String>,
    /// Items a session can play, in book order.
    pub playable: Vec<String>,
    pub mcqs: usize,
    pub cases: usize,
    /// The first short quote an item on the page carries.
    pub quote: Option<String>,
}

/// Collects the page's items from the library, leaving out hidden ones.
pub fn page_content(lib: &Library, page: u32, hidden: impl Fn(&str) -> bool) -> PageContent {
    let lecture =
        lib.index.lectures.iter().find(|e| (e.pages[0]..=e.pages[1]).contains(&page)).map(|e| e.id.clone());
    let mut out = PageContent { lecture, ..Default::default() };
    for item in lib.lectures.values().flat_map(|l| l.items.iter()) {
        if item.src.page != page || hidden(&item.id) {
            continue;
        }
        let kind = item.body.kind();
        if crate::study::PLAYABLE.contains(&kind) {
            out.playable.push(item.id.clone());
        }
        match kind {
            "mcq" => out.mcqs += 1,
            "case" => out.cases += 1,
            _ => {}
        }
        if out.quote.is_none() {
            out.quote = item.src.quote.clone().filter(|q| !q.trim().is_empty());
        }
    }
    out
}

/// The sheet, drawn once by the shell; empty while no page is open.
#[component]
pub fn BookSheet() -> Element {
    let ui = use_context::<Ui>();
    let review = use_review();
    let Some((page, chip_quote)) = OPEN.read().clone() else {
        return rsx! {};
    };
    let lib = Library::get();
    let content = {
        let b = review.backend.borrow();
        page_content(lib, page, |id| b.is_hidden(id))
    };
    let quote = chip_quote.or(content.quote.clone());
    let entry = content.lecture.as_ref().and_then(|id| lib.index.lectures.iter().find(|e| &e.id == id));
    let playable = content.playable.clone();
    let n = playable.len();
    let lecture_id = content.lecture.clone();

    rsx! {
        style { {CSS} }
        div { class: "bk-backdrop", onclick: move |_| close() }
        div { class: "bk-sheet", role: "dialog", aria_modal: "true", aria_labelledby: "bk-title",
            div { class: "bk-head",
                span { class: "bk-page", aria_hidden: "true", bdi { "{page}" } }
                div {
                    h2 { id: "bk-title", "{tx::TITLE} " bdi { "{page}" } }
                    if let Some(e) = entry {
                        p { class: "muted small",
                            bdi { "{lecture_code(&e.id)}" }
                            " · "
                            bdi { dir: "ltr", lang: "en", "{e.title}" }
                        }
                    } else {
                        p { class: "muted small", {tx::NO_LECTURE} }
                    }
                }
                button {
                    class: "icon-button bk-close",
                    "data-back": "true",
                    aria_label: tx::CLOSE,
                    title: tx::CLOSE,
                    onclick: move |_| close(),
                    span { aria_hidden: "true", "✕" }
                }
            }
            div { class: "bk-body",
                if let Some(q) = &quote {
                    figure { class: "bk-quote",
                        blockquote { lang: "en", dir: "ltr", "“{q}”" }
                        figcaption { class: "muted small", {tx::QUOTE_NOTE} }
                    }
                }
                if n + content.mcqs + content.cases > 0 {
                    p { class: "bk-made",
                        {tx::MADE}
                        " "
                        span { class: "bk-counts",
                            {
                                let mut parts = Vec::new();
                                if n > 0 { parts.push(count(n, &ITEM)); }
                                if content.mcqs > 0 { parts.push(format!("{} MCQ", count(content.mcqs, &QUESTION))); }
                                if content.cases > 0 { parts.push(count(content.cases, &CASE)); }
                                parts.join(" · ")
                            }
                        }
                    }
                } else {
                    p { class: "muted", {tx::NOTHING_YET} }
                }
            }
            div { class: "bk-foot",
                if n > 0 {
                    button {
                        class: "btn btn-primary",
                        onclick: {
                            let lecture_id = lecture_id.clone();
                            move |_| {
                                *crate::study::START.write() = Some(crate::study::Start {
                                    lectures: lecture_id.clone().map(|l| vec![l]),
                                    items: Some(playable.clone()),
                                    minutes: crate::onboarding::session_minutes(),
                                });
                                close();
                                crate::lecture::close();
                                let mut screen = ui.screen;
                                screen.set(Screen::Study);
                            }
                        },
                        "{tx::STUDY} ({n})"
                    }
                }
                if let Some(id) = lecture_id {
                    button {
                        class: "btn btn-tonal",
                        onclick: move |_| {
                            close();
                            crate::lecture::open(ui, &id);
                        },
                        {tx::OPEN_LECTURE}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_one_is_in_l01_and_has_items() {
        let c = page_content(Library::get(), 1, |_| false);
        assert_eq!(c.lecture.as_deref(), Some("hb.L01"));
        assert!(!c.playable.is_empty());
        assert!(c.playable.iter().all(|id| id.starts_with("hb.L01.")));
    }

    #[test]
    fn hidden_items_are_left_out() {
        let all = page_content(Library::get(), 1, |_| false);
        let first = all.playable[0].clone();
        let some = page_content(Library::get(), 1, |id| id == first);
        assert_eq!(some.playable.len(), all.playable.len() - 1);
    }
}
