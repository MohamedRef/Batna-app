//! Go to a book page: pick the course whose book is open on the desk (or,
//! for a course taught from slides, its lecture), type the printed page
//! number (or a range like 30–35) and get that page's sheet, the same one a
//! «ص N» chip opens: its lecture, what Batna made from it, a session on it,
//! his notes on it. From More, the tools rail, or Ctrl/⌘ + G.
//!
//! Adapted from Steps-app main: crates/app/src/gopage/mod.rs. Steps has one
//! book; Batna's books number their pages per course (Gastroenterology
//! starts again at 1) and the research course numbers slides per lecture,
//! so the box asks which book first: the course in view, else the last one
//! used here.

use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::gopage as tx;
use crate::look::{Css, css};
use crate::pagesheet::Scope;

const CSS: Css = css!("src/gopage/gopage.css");
/// Device-only: the book last opened here.
const LAST_BOOK: &str = "gopage.book";

static OPEN: GlobalSignal<bool> = Signal::global(|| std::env::var_os("BATNA_GO_PAGE").is_some());
static TEXT: GlobalSignal<String> = Signal::global(String::new);
/// The book: a course id (`hb`) or, for slides, a lecture id (`res.L02`).
static BOOK: GlobalSignal<Option<String>> = Signal::global(|| None);

pub fn open(_ui: crate::Ui) {
    crate::pagesheet::close();
    crate::abbrsheet::close();
    *TEXT.write() = String::new();
    *BOOK.write() = Some(start_book(Library::get()));
    *OPEN.write() = true;
}

pub fn close() {
    if *OPEN.peek() {
        *OPEN.write() = false;
    }
}

/// A course that has lectures in the index, in the program's order: its
/// id, short name and whether it cites slides.
fn courses(lib: &Library) -> Vec<(String, String, bool)> {
    lib.program
        .courses
        .iter()
        .filter(|c| lib.index.lectures.iter().any(|e| batna_core::program::course_of(&e.id) == c.id))
        .map(|c| (c.id.clone(), c.short.clone(), lib.program.slides(&c.id)))
        .collect()
}

/// The book a course opens on: the course itself, or its first lecture
/// when it is taught from slides.
fn book_of_course(lib: &Library, course: &str) -> String {
    if lib.program.slides(course) {
        lib.index
            .lectures
            .iter()
            .find(|e| batna_core::program::course_of(&e.id) == course)
            .map_or_else(|| course.to_string(), |e| e.id.clone())
    } else {
        course.to_string()
    }
}

/// Whether `book` still names a course (or slide lecture) with lectures.
fn known(lib: &Library, book: &str) -> bool {
    let scope = Scope::of_book(book);
    lib.index.lectures.iter().any(|e| crate::pagesheet::in_scope(lib, &scope, &e.id))
}

/// Where the box starts: the course in view, else the book last used here,
/// else the first course.
fn start_book(lib: &Library) -> String {
    let last: Option<String> = crate::store::shared().local_setting(LAST_BOOK);
    let focus = crate::courses::focus::current();
    let pick = match (focus, last) {
        // The last slide lecture of the course in view stays picked.
        (Some(f), Some(l)) if batna_core::program::course_of(&l) == f => Some(l),
        (Some(f), _) => Some(book_of_course(lib, &f)),
        (None, l) => l,
    };
    pick.filter(|b| known(lib, b))
        .or_else(|| courses(lib).first().map(|(id, ..)| book_of_course(lib, id)))
        .unwrap_or_else(|| "hb".into())
}

fn set_book(book: String) {
    let _ = crate::store::shared().put_local_setting(LAST_BOOK, &book);
    *BOOK.write() = Some(book);
}

/// What a typed page leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A printed page inside a lecture's range, with that lecture.
    Page { page: u32, lecture: String },
    /// Outside every lecture of the book: the closest page that has one.
    Nearest { asked: u32, page: u32, lecture: String },
    /// Not a page number.
    Invalid,
}

/// The first number of "30", "30-35" or "ص ٣٠–٣٥" (Arabic digits too);
/// a range opens its first page.
pub fn parse(text: &str) -> Option<u32> {
    let digits: String = text
        .chars()
        .map(|c| match c {
            '٠'..='٩' => char::from_digit(c as u32 - '٠' as u32, 10).unwrap_or(c),
            _ => c,
        })
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok().filter(|n| *n > 0)
}

/// A page number typed on its own, for the quick search: "30", "ص 30",
/// "p 30" or "page 30" (Arabic digits too).
pub fn page_query(text: &str) -> Option<u32> {
    let t = text.trim().to_lowercase();
    let rest = ["ص", "صفحة", "صفحه", "page", "p.", "p"]
        .into_iter()
        .find_map(|p| t.strip_prefix(p))
        .unwrap_or(&t)
        .trim();
    let all_digits = !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit() || ('٠'..='٩').contains(&c));
    if all_digits { parse(rest) } else { None }
}

/// Where a printed page of `book` sits in the index: its lecture, or the
/// nearest page that belongs to one.
pub fn target(lib: &Library, book: &str, text: &str) -> Target {
    let Some(page) = parse(text) else { return Target::Invalid };
    let scope = Scope::of_book(book);
    let ranges = lib
        .index
        .lectures
        .iter()
        .filter(|e| crate::pagesheet::in_scope(lib, &scope, &e.id))
        .map(|e| (e.pages, e.id.as_str()));
    if let Some((_, id)) = ranges.clone().find(|(p, _)| (p[0]..=p[1]).contains(&page)) {
        return Target::Page { page, lecture: id.to_string() };
    }
    let nearest = ranges.flat_map(|(p, id)| [(p[0], id), (p[1], id)]).min_by_key(|(p, _)| p.abs_diff(page));
    match nearest {
        Some((p, id)) => Target::Nearest { asked: page, page: p, lecture: id.to_string() },
        None => Target::Invalid,
    }
}

/// Every course book that has printed page `page` in a lecture: the
/// course and that lecture, in the program's order (the quick search).
pub fn books_with(lib: &Library, page: u32) -> Vec<(String, String)> {
    courses(lib)
        .into_iter()
        .filter(|(_, _, slides)| !slides)
        .filter_map(|(id, ..)| match target(lib, &id, &page.to_string()) {
            Target::Page { lecture, .. } => Some((id, lecture)),
            _ => None,
        })
        .collect()
}

pub fn go(page: u32, lecture: &str) {
    close();
    crate::pagesheet::open(page, lecture, None, false, None);
}

/// The page box, drawn once by the shell; empty while closed.
#[component]
pub fn GoPage() -> Element {
    if !OPEN() {
        return rsx! {};
    }
    let lib = Library::get();
    let text = TEXT();
    let book = BOOK().unwrap_or_else(|| start_book(lib));
    let course = batna_core::program::course_of(&book).to_string();
    let slides = lib.program.slides(&course);
    let found = if text.trim().is_empty() { None } else { Some(target(lib, &book, &text)) };
    let can_go = matches!(found, Some(Target::Page { .. }));
    let title = if slides { tx::SLIDE_TITLE } else { tx::TITLE };
    let hint = if slides { tx::SLIDE_HINT } else { tx::HINT };
    let lectures: Vec<(String, String)> = if slides {
        lib.index
            .lectures
            .iter()
            .filter(|e| batna_core::program::course_of(&e.id) == course)
            .map(|e| (e.id.clone(), format!("{} · {}", lib.lecture_tag(&e.id), e.title)))
            .collect()
    } else {
        Vec::new()
    };
    let submit = {
        let book = book.clone();
        move |e: FormEvent| {
            e.prevent_default();
            match target(Library::get(), &book, &TEXT.peek()) {
                Target::Page { page, lecture } => go(page, &lecture),
                Target::Nearest { .. } | Target::Invalid => {}
            }
        }
    };
    rsx! {
        style { {CSS.text()} }
        crate::sheet::Sheet {
            label_id: "gp-title",
            on_close: move |_| close(),
            head: rsx! {
                h2 { id: "gp-title", class: "gp-title", {title} }
            },
            form { class: "gp-form", onsubmit: submit,
                div { class: "gp-books", role: "group", aria_label: tx::BOOKS,
                    for (id , short , _) in courses(lib) {
                        button {
                            key: "{id}",
                            r#type: "button",
                            class: "chip tap-chip",
                            "data-tone": if id == course { "accent" } else { "" },
                            aria_pressed: if id == course { "true" } else { "false" },
                            onclick: {
                                let id = id.clone();
                                move |_| set_book(book_of_course(Library::get(), &id))
                            },
                            "{short}"
                        }
                    }
                }
                if slides {
                    label { class: "field",
                        span { {tx::LECTURE} }
                        select {
                            class: "gp-lecture",
                            dir: "auto",
                            value: "{book}",
                            onchange: move |e| set_book(e.value()),
                            for (id , name) in lectures {
                                option { key: "{id}", value: "{id}", selected: id == book, "{name}" }
                            }
                        }
                    }
                }
                label { class: "muted small", r#for: "gp-input", {hint} }
                div { class: "gp-row",
                    input {
                        id: "gp-input",
                        class: "gp-input",
                        r#type: "text",
                        inputmode: "numeric",
                        dir: "ltr",
                        autocomplete: "off",
                        placeholder: tx::PLACEHOLDER,
                        value: "{text}",
                        onmounted: move |e: MountedEvent| async move {
                            let _ = e.set_focus(true).await;
                        },
                        oninput: move |e| *TEXT.write() = e.value(),
                    }
                    button { class: "btn btn-primary", r#type: "submit", disabled: !can_go,
                        {tx::GO}
                    }
                }
                match found {
                    Some(Target::Page { lecture, .. }) => rsx! {
                        p { class: "small gp-where", bdi { "{lib.crumb(&lecture)}" } }
                    },
                    Some(Target::Nearest { asked, page, lecture }) => rsx! {
                        div { class: "note gp-note", role: "status",
                            p { class: "small", {tx::outside(asked, slides)} }
                            button {
                                class: "btn btn-tonal",
                                r#type: "button",
                                onclick: move |_| go(page, &lecture),
                                {tx::nearest(page)}
                            }
                        }
                    },
                    Some(Target::Invalid) => rsx! {
                        p { class: "muted small", role: "status", {tx::INVALID} }
                    },
                    None => rsx! {},
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
