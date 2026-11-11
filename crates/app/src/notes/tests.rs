//! His notes tapped through (`crate::testdom`): writing, editing and
//! deleting a note from its chip, the page sheet's list, and the list of
//! every note. Adapted from Steps-app main: crates/app/src/notes/mod.rs
//! (tests).

use std::cell::RefCell;

use super::*;
use crate::Ui;
use crate::prefs::DevicePrefs;
use crate::store::Review;
use crate::testdom::TestDom;

thread_local! {
    static REVIEW: RefCell<Option<Review>> = const { RefCell::new(None) };
}

/// The first hepatology item that cites a page.
fn hb_item() -> &'static Item {
    let lib = Library::get();
    lib.lectures["hb.L01"].items.iter().find(|i| i.src.page > 0).expect("hb.L01 cites pages")
}

#[test]
fn anchors_find_their_book_page_and_label_in_the_library() {
    let lib = Library::get();
    let it = hb_item();
    let a = Anchor::Item(it.id.clone());
    assert_eq!(page_of(lib, &a), Some(it.src.page));
    assert_eq!(label(lib, &a), crate::study::short_text(it));
    let n = Note { anchor: a, text: "t".into(), page: None, created_ms: 1, edited_ms: 1 };
    assert_eq!(place(lib, &n), Some(("hb".to_string(), it.src.page)));
    // A page names its course; a slide its lecture.
    assert_eq!(label(lib, &Anchor::page("hb", 28)), tx::on(&format!("{} · ص 28", lib.course_short("hb"))));
    assert!(label(lib, &Anchor::page("res.L02", 7)).contains("سلايد 7"));
    // Gone from the library: the saved page stands in, the id is the label.
    let gone = Anchor::Item("hb.L01.999".into());
    assert_eq!(page_of(lib, &gone), None);
    assert_eq!(label(lib, &gone), "hb.L01.999");
    let n = Note { anchor: gone, text: "t".into(), page: Some(4), created_ms: 1, edited_ms: 1 };
    assert_eq!(place(lib, &n), Some(("hb".to_string(), 4)));
    // His own cards cite no book.
    let own = Note {
        anchor: Anchor::Item("own.x".into()),
        text: "t".into(),
        page: Some(3),
        created_ms: 1,
        edited_ms: 1,
    };
    assert_eq!(place(lib, &own), None);
}

#[test]
fn lesson_steps_resolve_by_index_and_slides_stay_in_their_lecture() {
    let lib = Library::get();
    let (lecture, lesson) =
        lib.all_lessons().find(|(_, l)| l.steps.iter().any(|s| s.page.is_some())).unwrap();
    let (k, st) = lesson.steps.iter().enumerate().find(|(_, s)| s.page.is_some()).unwrap();
    let a = Anchor::Lesson(format!("{}#{k}", lesson.id));
    assert_eq!(page_of(lib, &a), st.page);
    assert!(label(lib, &a).contains(&lesson.title));
    assert_eq!(a.course(), Some(batna_core::program::course_of(lecture)));
    assert_eq!(page_of(lib, &Anchor::Lesson(format!("{}#999", lesson.id))), None);
    assert_eq!(page_of(lib, &Anchor::Lesson("nope".into())), None);
    // A research step's page is a slide of its own lecture.
    let n = Note {
        anchor: Anchor::Lesson("res.L02.lesson1#0".into()),
        text: "t".into(),
        page: Some(3),
        created_ms: 1,
        edited_ms: 1,
    };
    assert_eq!(place(lib, &n).map(|p| p.0), Some("res.L02".to_string()));
}

#[component]
fn Root() -> Element {
    use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Study),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    let review = use_context_provider(crate::store::test_review);
    REVIEW.with(|r| *r.borrow_mut() = Some(review.clone()));
    use_hook(close);
    let it = hb_item();
    rsx! {
        NoteChip { anchor: Anchor::Item(it.id.clone()), page: Some(it.src.page) }
        PageNotes { book: "hb", page: it.src.page }
        PageNotes { book: "git", page: it.src.page }
        NoteSheet {}
    }
}

fn mount() -> TestDom {
    crate::store::use_thread_store();
    TestDom::new(Root)
}

fn notes() -> Vec<Note> {
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow().notes().to_vec())
}

#[test]
fn a_note_is_written_shown_on_its_page_edited_and_deleted() {
    let mut t = mount();
    assert_eq!(t.texts("button.nt-chip"), [tx::CHIP_ADD]);
    assert_eq!(t.count("section.nt-page"), 2);
    assert_eq!(t.count("[role=dialog]"), 0);

    t.click("button.nt-chip");
    assert!(t.text().contains(tx::TITLE_NEW));
    t.input("textarea", "  SAAG ≥ 1.1 = portal hypertension  ");
    t.click_text("div.nt-actions button", tx::SAVE);
    assert_eq!(t.count("[role=dialog]"), 0, "saving closes the sheet");
    let saved = notes();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].text, "SAAG ≥ 1.1 = portal hypertension");
    assert_eq!(saved[0].page, Some(hb_item().src.page));
    assert_eq!(t.texts("button.nt-chip"), [tx::CHIP_HAS]);
    assert_eq!(t.count("p.nt-inline"), 1, "the note shows under its item");
    // On hepatology's page, not on Gastroenterology's page with the same number.
    assert_eq!(t.count("li.nt-row"), 1);

    // Edit: the old text is there; too long says so and keeps the text.
    t.click("button.nt-chip");
    assert!(t.text().contains(tx::TITLE_EDIT));
    t.input("textarea", &"x".repeat(batna_core::note::MAX_TEXT + 1));
    t.click_text("div.nt-actions button", tx::SAVE);
    assert!(t.text().contains(tx::TOO_LONG));
    assert_eq!(notes()[0].text, "SAAG ≥ 1.1 = portal hypertension");
    t.click_text("div.nt-actions button", tx::DELETE);
    assert!(notes().is_empty());
    assert_eq!(t.texts("button.nt-chip"), [tx::CHIP_ADD]);
}

#[test]
fn a_page_gets_its_own_note_from_its_sheet() {
    let mut t = mount();
    t.click_text("button.nt-add", tx::ADD_PAGE);
    t.input("textarea", "draw the portal flow");
    t.click_text("div.nt-actions button", tx::SAVE);
    let saved = notes();
    assert_eq!(saved.len(), 1);
    // The first PageNotes is hepatology's: the note names that book.
    assert_eq!(saved[0].anchor, Anchor::page("hb", hb_item().src.page));
    // Once it has one, the page offers no second note of its own.
    assert_eq!(t.count("button.nt-add"), 1, "only Gastroenterology's page still offers one");
}

#[component]
fn ListRoot() -> Element {
    use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Learn),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    let review = use_context_provider(crate::store::test_review);
    REVIEW.with(|r| *r.borrow_mut() = Some(review.clone()));
    use_hook(|| {
        *COURSE.write() = None;
        let mut b = review.backend.borrow_mut();
        let it = hb_item();
        b.save_note(Anchor::Item(it.id.clone()), "Wilson: ATP7B", Some(it.src.page)).unwrap();
        b.save_note(Anchor::page("git", 1), "Rome IV", None).unwrap();
        b.save_note(Anchor::page("res.L02", 3), "incidence vs prevalence", None).unwrap();
    });
    rsx! {
        NotesList {}
        NoteSheet {}
    }
}

#[test]
fn the_list_sorts_by_course_filters_and_searches() {
    crate::store::use_thread_store();
    let mut t = TestDom::new(ListRoot);
    assert!(t.text().contains(&tx::count(3)));
    // The program's order: research, then hepatology, then gastroenterology.
    let texts = t.texts("span.nt-text");
    assert_eq!(texts, ["incidence vs prevalence", "Wilson: ATP7B", "Rome IV"]);
    // One chip per course with a note, and "all".
    assert_eq!(t.count("div.ntl-courses button"), 4);
    t.click_text("div.ntl-courses button", Library::get().course_short("git"));
    assert_eq!(t.texts("span.nt-text"), ["Rome IV"]);
    t.click_text("div.ntl-courses button", tx::ALL);
    t.input("input[type=search]", "atp7b");
    assert_eq!(t.texts("span.nt-text"), ["Wilson: ATP7B"]);
    t.input("input[type=search]", "zzz");
    assert!(t.text().contains(tx::NO_MATCH));
    t.input("input[type=search]", "");
    // A tap opens the note to edit.
    t.click_text("button.nt-open", "Rome IV");
    assert!(t.text().contains(tx::TITLE_EDIT));
}
