//! A book page's sheet. Every "ص N" chip in the app opens it (Mohamed,
//! 2026-10-28: no dead chips or links): the short quote the chip itself
//! carries, when it has one, the lecture the page is in, what Batna has made from that page,
//! and two ways on: a session on just that page, or the lecture's page.
//!
//! It opens over whatever is on screen (a lesson, a running session, a
//! paper) and changes nothing until he picks one of its buttons.

use batna_core::review::StudyStore;
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::book as tx;
use crate::i18n::count::{CASE, ITEM, QUESTION, count};
use crate::look::{Css, css};
use crate::store::use_review;
use crate::{Screen, Ui};

const CSS: Css = css!("src/pagesheet/pagesheet.css");

/// The open page, where its number counts ([`Scope`]), the quote its chip
/// carried, and whether a quote is held back until he answers (it would give
/// the answer away).
type Open = (u32, Scope, Option<String>, bool);

/// Where a cited page number counts. Each course's book numbers its own
/// pages (Gastroenterology starts again at 1), and a course that cites
/// slides numbers them per lecture.
#[derive(Clone, Debug, PartialEq)]
pub enum Scope {
    /// A page of this course's book (`hb`).
    Book(String),
    /// A slide of this lecture (`res.L02`).
    Slides(String),
}
static OPEN: GlobalSignal<Option<Open>> = Signal::global(|| None);

/// Opens the sheet for page `page` cited by `of` (an item, lesson step or
/// lecture id; empty for the hepatology book). A course that cites slides
/// numbers them per lecture, so the sheet stays in `of`'s lecture.
pub fn open(page: u32, of: &str, quote: Option<String>, held: bool) {
    document::eval("window.__batnaHover && window.__batnaHover.hide();");
    *OPEN.write() = Some((page, scope(Library::get(), of), quote.filter(|q| !q.trim().is_empty()), held));
}

/// Where `of`'s page numbers count; empty `of` is the hepatology book.
pub fn scope(lib: &Library, of: &str) -> Scope {
    use batna_core::program::{course_of, lecture_of};
    if of.is_empty() {
        Scope::Book("hb".into())
    } else if lib.program.slides(of) {
        Scope::Slides(lecture_of(of).to_string())
    } else {
        Scope::Book(course_of(of).to_string())
    }
}

pub fn close() {
    *OPEN.write() = None;
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
}

/// Collects the page's items from the library, leaving out hidden ones.
/// `scope` keeps it to one course's book or one lecture's slides.
pub fn page_content(lib: &Library, page: u32, scope: &Scope, hidden: impl Fn(&str) -> bool) -> PageContent {
    let in_scope = |id: &str| match scope {
        Scope::Slides(l) => id == l,
        Scope::Book(c) => batna_core::program::course_of(id) == c && !lib.program.slides(id),
    };
    let lecture = lib
        .index
        .lectures
        .iter()
        .find(|e| in_scope(&e.id) && (e.pages[0]..=e.pages[1]).contains(&page))
        .map(|e| e.id.clone());
    let mut out = PageContent { lecture, ..Default::default() };
    for item in lib.lectures.values().filter(|l| in_scope(&l.lecture)).flat_map(|l| l.items.iter()) {
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
    }
    out
}

/// The sheet, drawn once by the shell; empty while no page is open.
#[component]
pub fn BookSheet() -> Element {
    let ui = use_context::<Ui>();
    let review = use_review();
    let Some((page, scope, chip_quote, held)) = OPEN.read().clone() else {
        return rsx! {};
    };
    let lib = Library::get();
    let content = {
        let b = review.backend.borrow();
        page_content(lib, page, &scope, |id| b.is_hidden(id))
    };
    let title = if matches!(scope, Scope::Slides(_)) { tx::SLIDE_TITLE } else { tx::TITLE };
    // Only the chip's own quote: another item's line from the same page
    // reads as the source of this one and points at the wrong fact
    // (Mohamed, 2026-10-29).
    let quote = chip_quote;
    let entry = content.lecture.as_ref().and_then(|id| lib.index.lectures.iter().find(|e| &e.id == id));
    let playable = content.playable.clone();
    let n = playable.len();
    let lecture_id = content.lecture.clone();

    rsx! {
        style { {CSS.text()} }
        crate::sheet::Sheet {
            label_id: "bk-title",
            on_close: move |_| close(),
            head: rsx! {
                div { class: "bk-head",
                    span { class: "bk-page", aria_hidden: "true", bdi { "{page}" } }
                    div {
                        h2 { id: "bk-title", "{title} " bdi { "{page}" } }
                        if let Some(e) = entry {
                            p { class: "muted small",
                                bdi { {lib.crumb(&e.id)} }
                            }
                        } else {
                            p { class: "muted small", {tx::NO_LECTURE} }
                        }
                    }
                }
            },
            div { class: "bk-body",
                if let Some(q) = &quote {
                    figure { class: "bk-quote",
                        blockquote { lang: "en", dir: "ltr", "“{q}”" }
                        figcaption { class: "muted small", {tx::QUOTE_NOTE} }
                    }
                } else if held {
                    p { class: "bk-quote muted", {tx::QUOTE_HELD} }
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
                                crate::lecture::close_all();
                                crate::learn::forget();
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

    fn hb() -> Scope {
        Scope::Book("hb".into())
    }

    #[test]
    fn a_book_page_stays_in_its_own_course() {
        // Another course's book starts again at page 1.
        let lib = Library::get();
        assert_eq!(scope(lib, "git.L01.004"), Scope::Book("git".into()));
        let other = page_content(lib, 1, &Scope::Book("git".into()), |_| false);
        assert!(other.playable.iter().all(|id| id.starts_with("git.")), "{other:?}");
    }

    #[test]
    fn page_one_is_in_l01_and_has_items() {
        let c = page_content(Library::get(), 1, &hb(), |_| false);
        assert_eq!(c.lecture.as_deref(), Some("hb.L01"));
        assert!(!c.playable.is_empty());
        assert!(c.playable.iter().all(|id| id.starts_with("hb.L01.")));
    }

    #[test]
    fn hidden_items_are_left_out() {
        let all = page_content(Library::get(), 1, &hb(), |_| false);
        let first = all.playable[0].clone();
        let some = page_content(Library::get(), 1, &hb(), |id| id == first);
        assert_eq!(some.playable.len(), all.playable.len() - 1);
    }

    #[test]
    fn a_slide_stays_in_its_own_lecture() {
        let lib = Library::get();
        assert_eq!(scope(lib, "res.L02.019"), Scope::Slides("res.L02".into()));
        assert_eq!(scope(lib, "hb.L02.019"), hb());
        assert_eq!(scope(lib, ""), hb());
        let c = page_content(lib, 1, &scope(lib, "res.L02.019"), |_| false);
        assert_eq!(c.lecture.as_deref(), Some("res.L02"));
        assert!(c.playable.iter().all(|id| id.starts_with("res.L02.")));
        // A book page never lands in a slide lecture.
        assert_eq!(page_content(lib, 30, &hb(), |_| false).lecture.as_deref(), Some("hb.L02"));
    }
}
