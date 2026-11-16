//! The reset card driven by taps (`test_dom`): pick, confirm, backup, undo,
//! and the refusals.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use batna_core::review::{Grade, Origin, ReviewEvent, StudyStore, StudyStoreHandle};

use super::*;
use crate::test_dom::{El, Screen};

fn scratch() -> PathBuf {
    let dir = tempfile::tempdir().unwrap().keep();
    crate::store::export_to_for_test(dir.clone());
    dir
}

#[derive(Props, Clone, PartialEq)]
struct Show {
    course: Option<String>,
}

#[allow(non_snake_case)]
fn Host(p: Show) -> Element {
    let review = use_context_provider(|| crate::store::test_review_on(crate::store::shared()));
    use_context_provider(|| StudyStoreHandle(review.backend.clone() as Rc<RefCell<dyn StudyStore>>));
    match p.course {
        Some(course) => rsx! { CourseReset { course } },
        None => rsx! { ResetCard {} },
    }
}

/// A fresh store with one hepatology answer, and the card mounted.
fn mount(course: Option<&str>) -> (Screen, String) {
    let store = crate::store::use_thread_store();
    let item =
        Library::get().lectures.values().find(|l| l.lecture.starts_with("hb.")).unwrap().items[0].id.clone();
    store
        .append_review(ReviewEvent::new(&item, Grade::Good, Origin::Study, crate::store::now_ms(), 3000))
        .unwrap();
    let s = Screen::with_props(Host, Show { course: course.map(str::to_string) });
    (s, item)
}

fn is_new(s: &Screen, item: &str) -> bool {
    s.read(|| use_context::<Review>().backend.borrow().is_new(item))
}

fn button(text: String) -> impl Fn(&El) -> bool {
    move |e: &El| e.tag == "button" && e.text.contains(&text)
}

fn hb() -> String {
    name(&Scope::Course("hb".into()))
}

#[test]
fn one_course_takes_a_second_tap_saves_a_backup_and_undoes() {
    let dir = scratch();
    let (mut s, item) = mount(None);
    assert!(!is_new(&s, &item));
    s.click(|e| e.attr("role") == Some("radio") && e.text.contains(&hb()));
    let text = s.html();
    assert!(text.contains(tx::RESTARTS) && text.contains(&tx::restarts_line(1, 0)), "{text}");
    assert!(text.contains(tx::STAYS_LINE));
    s.click(button(tx::GO.into()));
    assert!(crate::store::shared().resets().is_empty(), "the first tap only arms it");
    s.click(button(tx::confirm(&hb())));
    assert!(crate::store::shared().resets().courses.contains_key("hb"));
    assert!(is_new(&s, &item));
    assert!(s.html().contains(&tx::done(&hb())));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "a backup was written first");
    // The reset stands in the list with its own undo, and the snackbar's undo works.
    assert!(s.html().contains(&tx::undo_scope(&hb())));
    s.click(|e| e.has_class("reset-undo") && e.text.contains(tx::UNDO));
    assert!(crate::store::shared().resets().is_empty());
    assert!(!is_new(&s, &item));
    assert!(s.html().contains(tx::UNDONE));
}

#[test]
fn the_whole_curriculum_waits_for_the_typed_word() {
    scratch();
    let (mut s, item) = mount(None);
    s.click(|e| e.attr("role") == Some("radio") && e.text.contains(tx::ALL));
    assert!(s.find(button(tx::GO.into())).iter().all(El::disabled));
    s.input(|e| e.tag == "input" && e.attr("type") == Some("text"), "كده");
    assert!(s.find(button(tx::GO.into())).iter().all(El::disabled));
    s.input(|e| e.tag == "input" && e.attr("type") == Some("text"), tx::TYPE_WORD);
    s.click(button(tx::GO.into()));
    assert!(crate::store::shared().resets().all.is_some());
    assert!(is_new(&s, &item));
}

#[test]
fn a_failed_backup_resets_nothing() {
    let file = tempfile::NamedTempFile::new().unwrap().into_temp_path().keep().unwrap();
    crate::store::export_to_for_test(file.join("sub"));
    let (mut s, item) = mount(None);
    s.click(|e| e.attr("role") == Some("radio") && e.text.contains(&hb()));
    s.click(button(tx::GO.into()));
    s.click(button(tx::confirm(&hb())));
    assert!(crate::store::shared().resets().is_empty());
    assert!(!is_new(&s, &item));
    assert!(s.html().contains(tx::BACKUP_FAILED));
}

#[test]
fn an_open_paper_refuses_with_a_clear_line() {
    scratch();
    let (mut s, _) = mount(Some("hb"));
    s.read(|| crate::bank::leave_paper_for_test(true));
    s.click(button(tx::COURSE_BUTTON.into()));
    assert!(s.html().contains(tx::BLOCKED_PAPER));
    assert!(s.find(button(tx::GO.into())).is_empty(), "no way to run it under the paper");
    assert!(s.read(|| run(&use_context::<Review>(), &Scope::Course("hb".into()), false)).is_err());
    assert!(crate::store::shared().resets().is_empty());
    s.read(|| crate::bank::leave_paper_for_test(false));
}

#[test]
fn a_finished_lesson_reads_as_not_done_until_undo() {
    scratch();
    let (s, _) = mount(Some("hb"));
    let lesson =
        Library::get().lessons.values().find(|f| f.lecture.starts_with("hb.")).unwrap().lessons[0].id.clone();
    let mark = crate::learn::Mark { done_ms: crate::store::now_ms() - 1, right: 1, asked: 1 };
    crate::store::shared().put_setting(&format!("lesson.{lesson}"), &mark).unwrap();
    assert_eq!(s.read(|| crate::learn::mark(&lesson)), Some(mark));
    let review = s.read(use_context::<Review>);
    s.read(|| apply(&review, &Scope::Course("hb".into()), false)).unwrap();
    assert_eq!(s.read(|| crate::learn::mark(&lesson)), None);
    s.read(|| undo(&review, &Scope::Course("hb".into()))).unwrap();
    assert_eq!(s.read(|| crate::learn::mark(&lesson)), Some(mark));
}

/// #1353: another device's clock ran fast, so the reset carries a time 3
/// minutes ahead of this one. A lesson finished right after it is done.
#[test]
fn a_lesson_finished_just_after_a_reset_ahead_of_this_clock_reads_as_done() {
    scratch();
    let (s, _) = mount(Some("hb"));
    let lesson =
        Library::get().lessons.values().find(|f| f.lecture.starts_with("hb.")).unwrap().lessons[0].id.clone();
    let ahead = batna_protocol::hlc::Hlc {
        wall_ms: crate::store::now_ms() + 180_000,
        counter: 0,
        device: "fast".into(),
    };
    let store = crate::store::shared();
    store.put_setting(&Scope::Course("hb".into()).key(), &Marker::new(ahead)).unwrap();
    s.read(reload);
    let plain = crate::learn::Mark { done_ms: crate::store::now_ms(), right: 1, asked: 1 };
    store.put_setting(&format!("lesson.{lesson}"), &plain).unwrap();
    assert_eq!(s.read(|| crate::learn::mark(&lesson)), None, "the plain clock reads as before the reset");
    let mark = crate::learn::Mark { done_ms: s.read(|| stamp(&lesson)), right: 1, asked: 1 };
    store.put_setting(&format!("lesson.{lesson}"), &mark).unwrap();
    assert_eq!(s.read(|| crate::learn::mark(&lesson)), Some(mark));
    s.read(|| undo(&use_context::<Review>(), &Scope::Course("hb".into()))).unwrap();
}
