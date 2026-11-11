//! Mohamed's notes: a note chip on every item, MCQ explanation and lesson
//! step opens a short free-text note; the page sheet collects a page's
//! notes under «ملاحظاتك على ص N»; and one searchable list, sorted by course
//! and printed page and filtered by course, is a page on the lecture stack
//! (More, the rail).
//!
//! Notes are a synced table of their own (`store/notes.rs`), so a content
//! update never touches them. Their text goes nowhere but his devices and
//! his exports.
//!
//! Adapted from Steps-app main: crates/app/src/notes/mod.rs. Batna's books
//! number pages per course and the research course cites slides per
//! lecture, so every page here carries its book (`pagesheet::Scope::book`).

use batna_core::content::Item;
use batna_core::note::{self as core_note, Anchor, Listed, Note, Place, Problem};
use dioxus::prelude::*;

use crate::Ui;
use crate::content::Library;
use crate::i18n::notes as tx;
use crate::look::{Css, css};
use crate::store::{NoteError, use_review};

const CSS: Css = css!("src/notes/notes.css");

/// The list's page id on the lecture page stack.
pub const PAGE: &str = "notes";

/// The note being written: its anchor and the page it cites.
#[derive(Clone, Debug, PartialEq)]
pub struct Editing {
    pub anchor: Anchor,
    pub page: Option<u32>,
}

static EDITING: GlobalSignal<Option<Editing>> = Signal::global(|| None);
/// The course the list is filtered to (`None`: every course).
static COURSE: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Opens the note sheet on `anchor`.
pub fn edit(anchor: Anchor, page: Option<u32>) {
    *EDITING.write() = Some(Editing { anchor, page });
}

pub fn close() {
    *EDITING.write() = None;
}

/// Opens the list over the current screen.
pub fn open_list(ui: Ui) {
    crate::pagesheet::close();
    crate::gopage::close();
    crate::lecture::open(ui, PAGE);
}

/// Whether the list is the page on show (the sidebar marks its tool).
pub fn list_shown(ui: Ui) -> bool {
    crate::lecture::shown(*ui.screen.read()).as_deref() == Some(PAGE)
}

/// A content item by id, his own cards included.
fn item(lib: &'static Library, id: &str) -> Option<&'static Item> {
    if batna_core::own::is_own(id) {
        return crate::own::find(id).map(|(_, i)| i);
    }
    lib.lectures.get(batna_core::program::lecture_of(id))?.items.iter().find(|i| i.id == id)
}

/// A lesson step from its id (`cvs.L01.lesson1#3`): the lesson and the step.
fn step<'a>(
    lib: &'a Library,
    id: &str,
) -> Option<(&'a batna_core::lesson::Lesson, &'a batna_core::lesson::Step)> {
    let (lesson, k) = id.split_once('#')?;
    let (_, lesson) = lib.lesson(lesson)?;
    let step = lesson.steps.get(k.parse::<usize>().ok()?)?;
    Some((lesson, step))
}

/// An anchor's printed page in the library as it is now.
pub fn page_of(lib: &'static Library, anchor: &Anchor) -> Option<u32> {
    match anchor {
        Anchor::Item(id) => item(lib, id).map(|i| i.src.page).filter(|p| *p > 0),
        Anchor::Lesson(id) => step(lib, id).and_then(|(_, s)| s.page),
        Anchor::Page { page, .. } => Some(*page),
    }
}

/// The book an anchor's page numbers count in (`pagesheet::Scope::book`);
/// his own cards cite no book.
fn book_of(lib: &Library, anchor: &Anchor) -> Option<String> {
    match anchor {
        Anchor::Item(id) if batna_core::own::is_own(id) => None,
        Anchor::Item(id) | Anchor::Lesson(id) => {
            Some(crate::pagesheet::scope(lib, id.split('#').next().unwrap_or(id)).book().to_string())
        }
        Anchor::Page { book, .. } => Some(book.clone()),
    }
}

/// Where a note sits now: its book and page in the library as it is,
/// else the page it cited when saved.
pub fn place(lib: &'static Library, note: &Note) -> Option<Place> {
    let page = page_of(lib, &note.anchor).or(note.page)?;
    Some((book_of(lib, &note.anchor)?, page))
}

/// A course's place in the program, for sorting; no course sorts last.
fn rank(lib: &Library, course: Option<&str>) -> usize {
    course.and_then(|c| lib.program.courses.iter().position(|x| x.id == c)).unwrap_or(usize::MAX)
}

/// "ص 28", or "سلايد 7" in a lecture taught from slides.
pub fn cite(book: &str, page: u32) -> String {
    if book.contains('.') { crate::i18n::cite::slide(page) } else { crate::i18n::cite::page(page) }
}

/// "كبد · ص 28": where a note sits, with its course.
fn where_label(lib: &Library, (book, page): &Place) -> String {
    let at = cite(book, *page);
    if book.contains('.') {
        format!("{} · {at}", lib.lecture_tag(book))
    } else {
        format!("{} · {at}", lib.course_short(book))
    }
}

/// What a note is written on, in one line.
pub fn label(lib: &'static Library, anchor: &Anchor) -> String {
    match anchor {
        Anchor::Item(id) => item(lib, id).map(crate::study::short_text).unwrap_or_else(|| id.clone()),
        Anchor::Lesson(id) => match step(lib, id) {
            Some((lesson, _)) => format!("{} · {}", tx::ON_LESSON, lesson.title),
            None => id.clone(),
        },
        Anchor::Page { book, page } => tx::on(&where_label(lib, &(book.clone(), *page))),
    }
}

/// The note chip: "＋ ملاحظة", or "ملاحظتك" when there is one. With
/// `show_text`, the note itself sits under it (held back on a card's front,
/// where it could give the answer away).
#[component]
pub fn NoteChip(anchor: Anchor, page: Option<u32>, #[props(default = true)] show_text: bool) -> Element {
    let review = use_review();
    let note = review.backend.borrow().note(&anchor).cloned();
    let has = note.is_some();
    rsx! {
        style { {CSS.text()} }
        button {
            class: "chip tap-chip nt-chip",
            "data-has": if has { "true" } else { "false" },
            aria_haspopup: "dialog",
            aria_label: if has { tx::EDIT_HINT } else { tx::ADD_HINT },
            title: if has { tx::EDIT_HINT } else { tx::ADD_HINT },
            onclick: {
                let anchor = anchor.clone();
                move |e: Event<MouseData>| {
                    e.stop_propagation();
                    edit(anchor.clone(), page);
                }
            },
            if has { {tx::CHIP_HAS} } else { {tx::CHIP_ADD} }
        }
        if let Some(n) = note.filter(|_| show_text) {
            p { class: "nt-inline", dir: "auto", "{n.text}" }
        }
    }
}

/// A page's notes on its sheet, and a way to write the page's own note.
/// `book` is the page's book (`pagesheet::Scope::book`).
#[component]
pub fn PageNotes(book: String, page: u32) -> Element {
    let review = use_review();
    let lib = Library::get();
    let notes: Vec<Note> = {
        let b = review.backend.borrow();
        core_note::on_page(b.notes(), &book, page, |n| place(lib, n)).into_iter().cloned().collect()
    };
    let own = Anchor::page(book.clone(), page);
    let has_page_note = notes.iter().any(|n| n.anchor == own);
    rsx! {
        style { {CSS.text()} }
        section { class: "nt-page", aria_labelledby: "nt-page-title",
            h3 { id: "nt-page-title", class: "nt-page-title", {tx::on_page_title(&cite(&book, page))} }
            if notes.is_empty() {
                p { class: "muted small", {tx::PAGE_NONE} }
            }
            ul { class: "nt-list",
                for n in notes {
                    NoteRow { key: "{n.anchor.row_id()}", note: n, at: None }
                }
            }
            if !has_page_note {
                button {
                    class: "btn btn-tonal nt-add",
                    onclick: move |_| edit(own.clone(), Some(page)),
                    {tx::ADD_PAGE}
                }
            }
        }
    }
}

/// One note in a list: where it sits (`at`, in the list of every note),
/// what it is on, the note, and a tap to edit it.
#[component]
fn NoteRow(note: Note, at: Option<Place>) -> Element {
    let lib = Library::get();
    let on = label(lib, &note.anchor);
    let anchor = note.anchor.clone();
    let page = at.as_ref().map(|(_, p)| *p).or(note.page);
    // A page's own note names its place already.
    let at = at.filter(|_| !matches!(note.anchor, Anchor::Page { .. }));
    rsx! {
        li { class: "nt-row",
            button {
                class: "nt-open tap",
                aria_haspopup: "dialog",
                onclick: move |_| edit(anchor.clone(), page),
                span { class: "nt-on muted small",
                    if let Some(p) = &at {
                        span { class: "nt-pg", {where_label(lib, p)} }
                        " · "
                    }
                    bdi { dir: "auto", "{on}" }
                }
                span { class: "nt-text", dir: "auto", "{note.text}" }
            }
        }
    }
}

/// The sheet where a note is written, edited or deleted. Mounted once.
#[component]
pub fn NoteSheet() -> Element {
    rsx! {
        if let Some(e) = EDITING() {
            Sheet { key: "{e.anchor.row_id()}", editing: e }
        }
    }
}

#[component]
fn Sheet(editing: Editing) -> Element {
    let review = use_review();
    let lib = Library::get();
    let existing = review.backend.borrow().note(&editing.anchor).cloned();
    let had = existing.is_some();
    let mut text = use_signal(|| existing.map(|n| n.text).unwrap_or_default());
    let mut problem = use_signal(|| None::<String>);
    let on = label(lib, &editing.anchor);
    // Where it sits, unless the anchor is the page itself (its label says so).
    let sheet_at = match &editing.anchor {
        Anchor::Page { .. } => None,
        a => editing.page.and_then(|p| Some((book_of(lib, a)?, p))),
    };
    let save = {
        let review = review.clone();
        let editing = editing.clone();
        move |clear: bool| {
            let typed = if clear { String::new() } else { text.read().clone() };
            let result = review.backend.borrow_mut().save_note(editing.anchor.clone(), &typed, editing.page);
            match result {
                Ok(()) => close(),
                Err(NoteError::Check(Problem::TooLong)) => problem.set(Some(tx::TOO_LONG.to_string())),
                Err(NoteError::Store(e)) => problem.set(Some(format!("{} {e}", tx::FAILED))),
            }
        }
    };
    let n = text.read().chars().count();
    rsx! {
        style { {CSS.text()} }
        div { class: "sheet-backdrop nt-backdrop", onclick: move |_| close() }
        div {
            class: "sheet nt-sheet",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "nt-title",
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    close();
                }
            },
            div { class: "sheet-head",
                h2 { id: "nt-title", if had { {tx::TITLE_EDIT} } else { {tx::TITLE_NEW} } }
                button {
                    class: "icon-button",
                    "data-back": "true",
                    aria_label: tx::CANCEL,
                    title: tx::CANCEL,
                    onclick: move |_| close(),
                    span { aria_hidden: "true", "✕" }
                }
            }
            p { class: "nt-on muted small",
                if let Some(p) = sheet_at {
                    span { class: "nt-pg", {where_label(lib, &p)} }
                    " · "
                }
                bdi { dir: "auto", "{on}" }
            }
            label { class: "field nt-field",
                span { class: "nt-label", {tx::FIELD}
                    bdi { class: "nt-count", dir: "ltr", "{n}/{core_note::MAX_TEXT}" }
                }
                textarea {
                    dir: "auto",
                    rows: 6,
                    autofocus: true,
                    placeholder: tx::HINT,
                    "aria-invalid": if problem().is_some() { "true" } else { "false" },
                    value: "{text}",
                    oninput: move |e| {
                        problem.set(None);
                        text.set(e.value());
                    },
                }
            }
            p { class: "muted small", {tx::PRIVATE} }
            if let Some(p) = problem() {
                p { class: "field-error", role: "alert", "{p}" }
            }
            div { class: "nt-actions",
                button {
                    class: "btn btn-primary",
                    onclick: {
                        let mut save = save.clone();
                        move |_| save(false)
                    },
                    {tx::SAVE}
                }
                if had {
                    button {
                        class: "btn btn-secondary",
                        onclick: {
                            let mut save = save.clone();
                            move |_| save(true)
                        },
                        {tx::DELETE}
                    }
                }
                button { class: "btn btn-secondary", onclick: move |_| close(), {tx::CANCEL} }
            }
        }
    }
}

/// Every note, by printed page: search, a course filter, and a tap to edit.
#[component]
pub fn NotesList() -> Element {
    let review = use_review();
    let mut query = use_signal(String::new);
    let lib = Library::get();
    let all: Vec<Note> = review.backend.borrow().notes().to_vec();
    let course = COURSE();
    let listed: Vec<Listed> =
        core_note::list(&all, &query.read(), course.as_deref(), |n| place(lib, n), |c| rank(lib, c));
    // Course chips: only courses that have a note.
    let every = core_note::list(&all, "", None, |n| place(lib, n), |c| rank(lib, c));
    let courses: Vec<(String, String)> = lib
        .program
        .courses
        .iter()
        .filter(|c| every.iter().any(|l| l.course.as_deref() == Some(c.id.as_str())))
        .map(|c| (c.id.clone(), c.short.clone()))
        .collect();
    let rows: Vec<(Note, Option<Place>)> = listed.iter().map(|l| (l.note.clone(), l.place.clone())).collect();
    let total = all.len();

    rsx! {
        style { {crate::lecture::CSS.text()} }
        style { {CSS.text()} }
        div { class: "lec-page ntl",
            div { class: "ntl-head",
                button {
                    class: "icon-button lec-back",
                    "data-back": "true",
                    aria_label: crate::i18n::lecture::BACK,
                    title: crate::i18n::lecture::BACK,
                    onclick: move |_| crate::lecture::close(),
                    span { aria_hidden: "true", "→" }
                }
                header { class: "page-head",
                    h1 { {tx::LIST_TITLE} }
                    p { class: "page-sub", {tx::count(total)} }
                }
            }
            if total == 0 {
                p { class: "note", {tx::LIST_EMPTY} }
            } else {
                if courses.len() > 1 {
                    div { class: "ntl-courses", role: "group", aria_label: tx::COURSES,
                        button {
                            class: "chip tap-chip",
                            "data-tone": if course.is_none() { "accent" } else { "" },
                            aria_pressed: if course.is_none() { "true" } else { "false" },
                            onclick: move |_| *COURSE.write() = None,
                            {tx::ALL}
                        }
                        for (id , short) in courses {
                            button {
                                key: "{id}",
                                class: "chip tap-chip",
                                "data-tone": if course.as_deref() == Some(id.as_str()) { "accent" } else { "" },
                                aria_pressed: if course.as_deref() == Some(id.as_str()) { "true" } else { "false" },
                                onclick: {
                                    let id = id.clone();
                                    move |_| *COURSE.write() = Some(id.clone())
                                },
                                "{short}"
                            }
                        }
                    }
                }
                label { class: "search",
                    {crate::icons::search()}
                    input {
                        r#type: "search",
                        dir: "auto",
                        aria_label: tx::SEARCH,
                        placeholder: tx::SEARCH,
                        value: "{query}",
                        oninput: move |e| query.set(e.value()),
                    }
                }
                if rows.is_empty() {
                    p { class: "note", {tx::NO_MATCH} }
                }
                ul { class: "nt-list",
                    for (n , at) in rows {
                        NoteRow { key: "{n.anchor.row_id()}", note: n, at }
                    }
                }
                p { class: "muted small", {tx::PRIVATE} }
            }
        }
    }
}

#[cfg(test)]
mod tests;
