//! Mohamed's own notes: a short free-text note on an item (card, cloze,
//! MCQ, case), a lesson step or a book page, kept beside the book like a
//! margin line.
//!
//! Adapted from Steps-app main: crates/core/src/note.rs. Batna's books
//! number their pages per course (Gastroenterology starts again at 1) and
//! the research course cites slides per lecture, so a page anchor names
//! its book: a course id (`hb`) or, for slides, a lecture id (`res.L02`),
//! the same key the page sheet's scope uses.
//!
//! Notes live in the synced `notes` table, one row per thing noted, keyed
//! by [`Anchor::row_id`], last writer wins. They never live in content: a
//! content-pack update that rewrites an item leaves its note alone, because
//! the note points at the item's id, not its text. Note text never leaves
//! the device except through Batna Cloud sync and his own exports.

use serde::{Deserialize, Serialize};

use crate::program::course_of;

/// Longest note, in characters. A note is a margin line, not a lesson.
pub const MAX_TEXT: usize = 2000;

/// What a note is written on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "on", content = "id", rename_all = "snake_case")]
pub enum Anchor {
    /// A content item or one of his own cards: `hb.L04.012`, `own.…`.
    Item(String),
    /// A lesson step: `hb.L04.lesson1#3`.
    Lesson(String),
    /// A printed page of a book: `book` is the course (`hb`) or, for a
    /// course that cites slides, the lecture (`res.L02`).
    Page { book: String, page: u32 },
}

impl Anchor {
    /// A page of `book` (a course id, or a lecture id for slides).
    pub fn page(book: impl Into<String>, page: u32) -> Self {
        Anchor::Page { book: book.into(), page }
    }

    /// The row's key in the `notes` table: one note per anchor.
    pub fn row_id(&self) -> String {
        match self {
            Anchor::Item(id) => format!("item:{id}"),
            Anchor::Lesson(id) => format!("lesson:{id}"),
            Anchor::Page { book, page } => format!("page:{book}:{page}"),
        }
    }

    /// The course the anchor belongs to; his own cards have none.
    pub fn course(&self) -> Option<&str> {
        match self {
            Anchor::Item(id) if crate::own::is_own(id) => None,
            Anchor::Item(id) | Anchor::Lesson(id) => Some(course_of(id)),
            Anchor::Page { book, .. } => Some(course_of(book)),
        }
    }
}

/// One note, as stored (JSON in `notes.value`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub anchor: Anchor,
    pub text: String,
    /// The printed page the anchor cited when the note was saved. The list
    /// prefers the item's current page and falls back to this one, so a
    /// note on an item a later pack removed still sorts where it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    pub created_ms: i64,
    pub edited_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    TooLong,
}

/// What saving `text` on an anchor does: an empty note is a delete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Save {
    Put(Note),
    Delete,
}

/// The note to store for `text` typed on `anchor`, keeping when it was
/// first written. Text is trimmed; empty text deletes the note.
pub fn save(
    anchor: Anchor,
    text: &str,
    page: Option<u32>,
    existing: Option<&Note>,
    now_ms: i64,
) -> Result<Save, Problem> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(Save::Delete);
    }
    if text.chars().count() > MAX_TEXT {
        return Err(Problem::TooLong);
    }
    let page = match &anchor {
        Anchor::Page { page, .. } => Some(*page),
        _ => page.filter(|p| *p > 0).or(existing.and_then(|n| n.page)),
    };
    Ok(Save::Put(Note {
        anchor,
        text: text.to_string(),
        page,
        created_ms: existing.map_or(now_ms, |n| n.created_ms),
        edited_ms: now_ms,
    }))
}

/// Where a note sits in the books: the book (course id, or lecture id for
/// slides) and the printed page.
pub type Place = (String, u32);

/// A note in the list, with where it sits now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed<'a> {
    pub note: &'a Note,
    /// The page it sorts under; `None` sorts last in its course.
    pub place: Option<Place>,
    pub course: Option<String>,
}

/// Every note that matches `query` (all its words, any case, in the note
/// or its anchor id) and `course`, sorted by course (`rank`, the program's
/// order), then printed page, then anchor. `place` gives a note's book and
/// page as the library has them now (falling back to the saved page).
pub fn list<'a>(
    notes: &'a [Note],
    query: &str,
    course: Option<&str>,
    place: impl Fn(&Note) -> Option<Place>,
    rank: impl Fn(Option<&str>) -> usize,
) -> Vec<Listed<'a>> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut out: Vec<Listed<'a>> = notes
        .iter()
        .filter(|n| {
            if words.is_empty() {
                return true;
            }
            let hay = format!("{} {}", n.text, n.anchor.row_id()).to_lowercase();
            words.iter().all(|w| hay.contains(w.as_str()))
        })
        .map(|n| Listed { note: n, place: place(n), course: n.anchor.course().map(str::to_string) })
        .filter(|l| course.is_none_or(|c| l.course.as_deref() == Some(c)))
        .collect();
    out.sort_by(|a, b| {
        let key = |l: &Listed<'a>| {
            (rank(l.course.as_deref()), l.place.is_none(), l.place.clone(), l.note.anchor.clone())
        };
        key(a).cmp(&key(b))
    });
    out
}

/// The notes that belong on the sheet of page `page` of `book`: the page's
/// own note first, then notes on items and lesson steps citing it.
pub fn on_page<'a>(
    notes: &'a [Note],
    book: &str,
    page: u32,
    place: impl Fn(&Note) -> Option<Place>,
) -> Vec<&'a Note> {
    let mut out: Vec<&Note> =
        notes.iter().filter(|n| place(n).is_some_and(|(b, p)| b == book && p == page)).collect();
    out.sort_by(|a, b| {
        let own = |n: &Note| !matches!(n.anchor, Anchor::Page { .. });
        (own(a), &a.anchor).cmp(&(own(b), &b.anchor))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(anchor: Anchor, text: &str, page: Option<u32>) -> Note {
        Note { anchor, text: text.into(), page, created_ms: 1, edited_ms: 1 }
    }

    fn item(id: &str) -> Anchor {
        Anchor::Item(id.into())
    }

    #[test]
    fn anchors_have_stable_row_ids_and_json() {
        assert_eq!(item("hb.L04.012").row_id(), "item:hb.L04.012");
        assert_eq!(Anchor::Lesson("hb.L04.l1#3".into()).row_id(), "lesson:hb.L04.l1#3");
        assert_eq!(Anchor::page("hb", 28).row_id(), "page:hb:28");
        assert_eq!(Anchor::page("res.L02", 7).row_id(), "page:res.L02:7");
        let n = note(Anchor::page("git", 1), "x", Some(1));
        let json = serde_json::to_value(&n).unwrap();
        assert_eq!(json["anchor"], serde_json::json!({"on": "page", "id": {"book": "git", "page": 1}}));
        assert_eq!(serde_json::from_value::<Note>(json).unwrap(), n);
        // A note saved without a page reads back without one.
        let bare: Note = serde_json::from_str(
            r#"{"anchor":{"on":"item","id":"hb.L01.001"},"text":"t","created_ms":1,"edited_ms":2}"#,
        )
        .unwrap();
        assert_eq!(bare.page, None);
    }

    #[test]
    fn courses_come_from_ids_and_books_but_not_own_cards() {
        assert_eq!(item("neph.L03.004").course(), Some("neph"));
        assert_eq!(Anchor::Lesson("git.L02.l1#2".into()).course(), Some("git"));
        assert_eq!(item("own.0011").course(), None);
        assert_eq!(Anchor::page("hb", 3).course(), Some("hb"));
        assert_eq!(Anchor::page("res.L05", 3).course(), Some("res"));
    }

    #[test]
    fn saving_trims_keeps_the_first_date_and_deletes_when_empty() {
        let first = match save(item("a.L01.1"), "  why ascites?  ", Some(30), None, 10).unwrap() {
            Save::Put(n) => n,
            Save::Delete => panic!("a note with text is kept"),
        };
        assert_eq!(
            (first.text.as_str(), first.page, first.created_ms, first.edited_ms),
            ("why ascites?", Some(30), 10, 10)
        );
        let again = match save(item("a.L01.1"), "SAAG", None, Some(&first), 20).unwrap() {
            Save::Put(n) => n,
            Save::Delete => panic!(),
        };
        assert_eq!((again.page, again.created_ms, again.edited_ms), (Some(30), 10, 20));
        assert_eq!(save(item("a.L01.1"), "   \n ", None, Some(&first), 30), Ok(Save::Delete));
        assert_eq!(save(item("a.L01.1"), &"x".repeat(MAX_TEXT + 1), None, None, 1), Err(Problem::TooLong));
        let page = save(Anchor::page("hb", 12), "p", Some(99), None, 1).unwrap();
        assert!(matches!(page, Save::Put(n) if n.page == Some(12)));
    }

    #[test]
    fn the_list_sorts_by_course_then_page_filters_and_searches() {
        let notes = vec![
            note(item("hb.L04.002"), "Kayser-Fleischer ring", Some(31)),
            note(Anchor::page("hb", 29), "draw the portal flow", None),
            note(item("git.L03.001"), "Rome IV", Some(29)),
            note(item("own.aa"), "my mnemonic", None),
            note(item("hb.L01.009"), "old page", Some(99)),
        ];
        // hb.L01.009 moved to page 5 in a later pack: the current page wins.
        let place = |n: &Note| match &n.anchor {
            Anchor::Item(id) if id == "hb.L01.009" => Some(("hb".to_string(), 5)),
            Anchor::Item(id) if id.starts_with("own.") => None,
            Anchor::Item(id) => n.page.map(|p| (course_of(id).to_string(), p)),
            Anchor::Page { book, page } => Some((book.clone(), *page)),
            Anchor::Lesson(_) => None,
        };
        // The program's order: hb, then git; no course sorts last.
        let rank = |c: Option<&str>| match c {
            Some("hb") => 0,
            Some("git") => 1,
            _ => 9,
        };
        let all = list(&notes, "", None, place, rank);
        let order: Vec<String> = all.iter().map(|l| l.note.anchor.row_id()).collect();
        assert_eq!(
            order,
            ["item:hb.L01.009", "page:hb:29", "item:hb.L04.002", "item:git.L03.001", "item:own.aa"]
        );
        assert_eq!(all[1].course.as_deref(), Some("hb"));
        assert_eq!(list(&notes, "", Some("hb"), place, rank).len(), 3);
        assert_eq!(list(&notes, "kayser", None, place, rank).len(), 1);
        assert_eq!(list(&notes, "git.L03", None, place, rank).len(), 1);
        assert!(list(&notes, "zzz", None, place, rank).is_empty());
    }

    #[test]
    fn a_page_sheet_shows_its_own_note_first_and_stays_in_its_book() {
        let notes = vec![
            note(item("hb.L01.003"), "item on 28", Some(28)),
            note(Anchor::page("hb", 28), "page note", None),
            note(Anchor::Lesson("hb.L01.l1#2".into()), "step", Some(28)),
            note(item("hb.L01.004"), "elsewhere", Some(30)),
            // Gastroenterology's own page 28 is another page.
            note(item("git.L01.004"), "other book", Some(28)),
        ];
        let place = |n: &Note| match &n.anchor {
            Anchor::Page { book, page } => Some((book.clone(), *page)),
            Anchor::Item(id) | Anchor::Lesson(id) => n.page.map(|p| (course_of(id).to_string(), p)),
        };
        let on = on_page(&notes, "hb", 28, place);
        let texts: Vec<&str> = on.iter().map(|n| n.text.as_str()).collect();
        assert_eq!(texts, ["page note", "item on 28", "step"]);
    }
}
