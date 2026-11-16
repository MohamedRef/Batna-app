//! Full-text search (#1350): the words a lesson teaches in its steps and
//! a card asks or answers, not only titles. The index is
//! `batna_core::search::Index`, built once per library from every lesson
//! and item in book order. The Lessons screen lists what it finds under
//! the title matches; the quick search (⌘K) offers the first few.
//!
//! A lesson opens at the first step that holds the words, with them
//! marked by the CSS Custom Highlight API (no change to the page's DOM).
//! An item opens where it is played: a study session of that one card,
//! a tutor question in the bank, or its case.

use batna_core::content::{Body, Item};
use batna_core::search::{Found, Index, LessonHit, Snippet, snippet, words};
use dioxus::prelude::*;

use crate::Ui;
use crate::content::{Derived, Library};
use crate::i18n::search as tx;
use crate::look::{Css, css};

pub(crate) const CSS: Css = css!("src/search/search.css");

/// Fewer folded letters than this search titles only: one or two letters
/// would match most of the library.
pub const MIN_LETTERS: usize = 3;
/// Rows each group shows before «اعرض كمان».
const PAGE: usize = 20;

/// Every lesson and item of the library, folded once.
fn index() -> &'static Index {
    static INDEX: Derived<Index> = Derived::new();
    INDEX.get(|lib| {
        let items = lib.index.lectures.iter().filter_map(|e| lib.lectures.get(&e.id)).flat_map(|l| &l.items);
        Index::build(lib.all_lessons().map(|(_, l)| l), items)
    })
}

/// Builds the index off the UI thread at launch, so the first search
/// doesn't wait for it (about 0.1 s on a Mac, more on a phone). The web
/// build has one thread: it builds on the first search instead.
pub fn warm() {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(|| {
        index();
    });
}

/// What `query` finds in lesson steps and items; `None` when it is too short.
pub fn find(query: &str) -> Option<Found> {
    let w = words(query);
    let letters: usize = w.iter().map(|w| w.chars().filter(|c| c.is_alphanumeric()).count()).sum();
    (letters >= MIN_LETTERS).then(|| index().find(&w))
}

/// An item by id, with its lecture id.
fn item(id: &str) -> Option<(&'static str, &'static Item)> {
    let lib = Library::get();
    let lecture = batna_core::program::lecture_of(id);
    let l = lib.lectures.get(lecture)?;
    l.items.iter().find(|i| i.id == id).map(|i| (l.lecture.as_str(), i))
}

/// The text around the match in step `step` of a lesson.
pub fn step_snippet(lesson: &str, step: usize, query: &str) -> Option<Snippet> {
    let (_, l) = Library::get().lesson(lesson)?;
    let s = l.steps.get(step)?;
    snippet(&batna_core::search::step_text(s), &words(query), 40, 80)
}

/// The text around the match in an item.
pub fn item_snippet(id: &str, query: &str) -> Option<Snippet> {
    let (_, i) = item(id)?;
    snippet(&batna_core::search::item_text(i), &words(query), 40, 80)
}

/// Opens a lesson at a found step and marks the words there.
pub fn open_step(ui: Ui, lesson: &str, step: usize, query: &str) {
    crate::learn::open_at(ui, lesson, step);
    mark_words(step, query);
}

/// Marks `query`'s words in step `step` of the lesson about to show. The
/// player mounts on the next frames, so the script waits for the step.
fn mark_words(step: usize, query: &str) {
    let w = serde_json::to_string(&words(query)).unwrap_or_else(|_| "[]".into());
    document::eval(&format!("{MARK_JS}\n__batnaFind({w}, {});", step + 1));
}

const MARK_JS: &str = include_str!("mark.js");

/// Opens an item where it is played: cards, clozes, lists, essays, tables
/// and orders as a study session of that one item; an MCQ as a tutor
/// question; a case or a calc item on the Cases screen.
pub fn open_item(mut ui: Ui, id: &str) {
    let Some((lecture, it)) = item(id) else { return };
    crate::courses::focus::follow(lecture);
    match &it.body {
        Body::Mcq { .. } => {
            let lib = Library::get();
            let pool: Vec<batna_core::exam::Mcq> = batna_core::exam::mcqs(lib.lectures.get(lecture))
                .into_iter()
                .filter(|q| q.id == id)
                .collect();
            let questions = batna_core::exam::practice(&pool, 1, crate::store::now_ms() as u64);
            crate::bank::start(batna_core::exam::Mode::Tutor, questions);
            ui.screen.set(crate::Screen::Bank);
        }
        Body::Case { .. } | Body::Calc { .. } => crate::case::open_item(ui, id, it.body.kind() == "calc"),
        _ => {
            *crate::study::START.write() =
                Some(crate::study::Start { lectures: None, items: Some(vec![id.to_string()]), minutes: 10 });
            ui.screen.set(crate::Screen::Study);
        }
    }
}

/// "ص 179", or "سلايد 25" in a course taught from slides.
fn cite(id: &str, page: u32) -> String {
    if Library::get().program.slides(id) {
        crate::i18n::cite::slide(page)
    } else {
        crate::i18n::cite::page(page)
    }
}

/// The course of a lesson or item id, and whether it is the one in view.
fn course_rank(id: &str, focused: Option<&str>) -> u8 {
    u8::from(focused.is_some_and(|f| batna_core::program::course_of(id) != f))
}

/// Hits per course, the course in view first, then book order.
fn per_course<'a>(ids: impl Iterator<Item = &'a str>, focused: Option<&str>) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for id in ids {
        let c = batna_core::program::course_of(id);
        match out.iter_mut().find(|(k, _)| k == c) {
            Some((_, n)) => *n += 1,
            None => out.push((c.to_string(), 1)),
        }
    }
    out.sort_by_key(|(c, _)| course_rank(c, focused));
    out
}

/// Lessons and items in the order shown: the course in view first.
fn ordered(found: &Found, focused: Option<&str>) -> (Vec<LessonHit>, Vec<String>) {
    let mut lessons = found.lessons.clone();
    lessons.sort_by_key(|h| course_rank(&h.lesson, focused));
    let mut items = found.items.clone();
    items.sort_by_key(|id| course_rank(id, focused));
    (lessons, items)
}

/// Any Arabic letter makes it an Arabic line; "auto" would follow the
/// first Latin word and scramble a mostly Arabic one.
fn dir_of(text: &str) -> &'static str {
    if text.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)) { "rtl" } else { "ltr" }
}

/// The quoted text with the match marked.
#[component]
fn Snip(s: Snippet) -> Element {
    let dir = dir_of(&format!("{}{}{}", s.before, s.hit, s.after));
    rsx! {
        bdi { class: "sr-snip", dir,
            "{s.before}"
            mark { class: "sr-hit", "{s.hit}" }
            "{s.after}"
        }
    }
}

/// The Lessons screen's full-text results under its title matches.
/// `skip` holds lessons the title search already lists.
#[component]
pub fn FullText(q: String, skip: Vec<String>) -> Element {
    let lib = Library::get();
    let mut course = use_signal(|| None::<String>);
    let mut more_lessons = use_signal(|| 1usize);
    let mut more_items = use_signal(|| 1usize);
    // A new query starts from the first page and every course.
    let mut last = use_signal(String::new);
    if *last.peek() != q {
        last.set(q.clone());
        course.set(None);
        more_lessons.set(1);
        more_items.set(1);
    }
    let Some(found) = find(&q) else { return rsx! {} };
    let mut found = found;
    found.lessons.retain(|h| !skip.contains(&h.lesson));
    if found.is_empty() {
        return rsx! {};
    }
    let focused = crate::courses::focus::current();
    let (lessons, items) = ordered(&found, focused.as_deref());
    let courses = per_course(
        lessons.iter().map(|h| h.lesson.as_str()).chain(items.iter().map(String::as_str)),
        focused.as_deref(),
    );
    let picked = course();
    let shows = |id: &str| picked.as_deref().is_none_or(|c| batna_core::program::course_of(id) == c);
    let lessons: Vec<LessonHit> = lessons.into_iter().filter(|h| shows(&h.lesson)).collect();
    let items: Vec<String> = items.into_iter().filter(|id| shows(id)).collect();
    let (n_lessons, n_items) = (lessons.len(), items.len());
    let lessons_shown: Vec<LessonHit> = lessons.into_iter().take(PAGE * more_lessons()).collect();
    let items_shown: Vec<String> = items.into_iter().take(PAGE * more_items()).collect();
    rsx! {
        style { {CSS.text()} }
        section { class: "sr-full", aria_label: tx::FULL_LABEL,
            if courses.len() > 1 {
                div { class: "sr-courses", role: "group", aria_label: tx::BY_COURSE,
                    button {
                        class: "chip sr-course",
                        aria_pressed: if picked.is_none() { "true" } else { "false" },
                        onclick: move |_| course.set(None),
                        {tx::ALL}
                    }
                    for (c , n) in courses {
                        button {
                            key: "{c}",
                            class: "chip sr-course",
                            aria_pressed: if picked.as_deref() == Some(c.as_str()) { "true" } else { "false" },
                            onclick: { let c = c.clone(); move |_| course.set(Some(c.clone())) },
                            "{lib.course_short(&c)} "
                            span { class: "num-ltr", "{n}" }
                        }
                    }
                }
            }
            if n_lessons > 0 {
                h2 { class: "ls-sec sr-head", {tx::in_lessons(n_lessons)} }
                ol { class: "tap-list sr-list",
                    for h in lessons_shown {
                        LessonHitRow { key: "{h.lesson}", hit: h, q: q.clone() }
                    }
                }
                if n_lessons > PAGE * more_lessons() {
                    button { class: "btn btn-ghost sr-more", onclick: move |_| more_lessons += 1, {tx::MORE} }
                }
            }
            if n_items > 0 {
                h2 { class: "ls-sec sr-head", {tx::in_items(n_items)} }
                ol { class: "tap-list sr-list",
                    for id in items_shown {
                        ItemHitRow { key: "{id}", id: id.clone(), q: q.clone() }
                    }
                }
                if n_items > PAGE * more_items() {
                    button { class: "btn btn-ghost sr-more", onclick: move |_| more_items += 1, {tx::MORE} }
                }
            }
        }
    }
}

/// A lesson that teaches the words: its title, where it is, and the step's
/// text around them. Opens at that step.
#[component]
fn LessonHitRow(hit: LessonHit, q: String) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let Some((_, l)) = lib.lesson(&hit.lesson) else { return rsx! {} };
    let page = l.steps.get(hit.step).and_then(|s| s.page);
    let snip = step_snippet(&hit.lesson, hit.step, &q);
    let crumb = lib.crumb(&hit.lesson);
    let (id, step) = (hit.lesson.clone(), hit.step);
    let id_page = id.clone();
    rsx! {
        li {
            button {
                class: "tap tap-row sr-row",
                title: tx::OPEN_STEP,
                onclick: move |_| open_step(ui, &id, step, &q),
                span { class: "tap-row-main sr-main",
                    strong { class: "sr-title", crate::learn::text::Line { text: l.title.clone() } }
                    span { class: "muted small sr-where",
                        bdi { "{crumb}" }
                        " · "
                        {tx::step_n(step + 1, l.steps.len())}
                        if let Some(p) = page {
                            " · "
                            {cite(&id_page, p)}
                        }
                        if hit.steps > 1 {
                            " · "
                            {tx::also_in(hit.steps - 1)}
                        }
                    }
                    if let Some(s) = snip {
                        Snip { s }
                    }
                }
                span { class: "chev", aria_hidden: "true" }
            }
        }
    }
}

/// An item that holds the words: its kind, where it is, and its text
/// around them. Opens the item; «افتح الدرس اللي بيشرحها» opens its lesson.
#[component]
fn ItemHitRow(id: String, q: String) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let Some((lecture, it)) = item(&id) else { return rsx! {} };
    let snip = item_snippet(&id, &q);
    let crumb = lib.crumb(lecture);
    let lesson = crate::learn::lesson_for_item(&id).map(|l| l.id.clone());
    let open = id.clone();
    let kind = crate::study::kind_label(&it.body);
    let short = crate::study::short_text(it);
    let place = cite(&id, it.src.page);
    rsx! {
        li { class: "sr-item",
            button {
                class: "tap tap-row sr-row",
                title: tx::OPEN_ITEM,
                onclick: move |_| open_item(ui, &open),
                span { class: "tap-row-main sr-main",
                    span { class: "muted small sr-where",
                        span { class: "chip sr-kind", {kind} }
                        " "
                        bdi { "{crumb}" }
                        " · "
                        {place}
                    }
                    // The question first, then where the word is when that
                    // is elsewhere (an option, the answer, the feedback).
                    strong { class: "sr-title", dir: dir_of(&short), crate::learn::text::Line { text: short.clone() } }
                    if let Some(s) = snip.filter(|s| !short.contains(&format!("{}{}", s.hit, s.after.trim_end_matches('…')))) {
                        Snip { s }
                    }
                }
                span { class: "chev", aria_hidden: "true" }
            }
            if let Some(lesson) = lesson {
                button {
                    class: "btn-link sr-lesson",
                    onclick: move |_| {
                        let at = Library::get()
                            .lesson(&lesson)
                            .and_then(|(_, l)| batna_core::search::find_step(l, &words(&q)));
                        match at {
                            Some(k) => open_step(ui, &lesson, k, &q),
                            None => crate::learn::open(ui, &lesson),
                        }
                    },
                    {tx::OPEN_LESSON}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
