//! ✓ / ✎ / ✗ tapped through (`crate::testdom`): each verdict is stored on
//! the item, Undo takes it back, the editor checks what he changed, and a
//! verdict that can't be saved offers "try again".

use std::cell::RefCell;

use batna_core::content::Item;
use batna_core::review::verify::Check;

use super::*;
use crate::Ui;
use crate::prefs::DevicePrefs;
use crate::store::Review;
use crate::testdom::TestDom;

thread_local! {
    static REVIEW: RefCell<Option<Review>> = const { RefCell::new(None) };
    static ITEM: RefCell<String> = const { RefCell::new(String::new()) };
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
    let id = ITEM.with(|i| i.borrow().clone());
    rsx! {
        VerifyBar { item_id: id }
        ExportVerdicts {}
    }
}

/// An unchecked book item of `kind`.
fn unchecked(kind: &str) -> &'static Item {
    crate::content::Library::get()
        .lectures
        .values()
        .flat_map(|l| &l.items)
        .find(|i| i.body.kind() == kind && i.status == batna_core::content::Status::Unverified)
        .unwrap_or_else(|| panic!("no unverified {kind}"))
}

fn mount(item: &Item) -> TestDom {
    crate::store::use_thread_store();
    ITEM.with(|i| *i.borrow_mut() = item.id.clone());
    TestDom::new(Root)
}

fn with_review<T>(f: impl FnOnce(&Review) -> T) -> T {
    REVIEW.with(|r| f(r.borrow().as_ref().unwrap()))
}

fn verdict(id: &str) -> Option<Verdict> {
    with_review(|b| b.backend.borrow().judgement(id).map(|j| j.verdict))
}

#[test]
fn right_is_stored_and_undo_asks_again() {
    let item = unchecked("card");
    let mut t = mount(item);
    assert_eq!(with_review(|b| b.backend.borrow().check(&item.id)), Check::Ask);
    assert!(t.text().contains(tx::QUESTION));
    // Nothing judged yet: the export has nothing to write.
    assert_eq!(t.count("button.btn-primary[disabled]"), 1);
    t.click_text("button.verify-btn", tx::RIGHT);
    assert_eq!(verdict(&item.id), Some(Verdict::Approved));
    assert_eq!(t.attr("p.verify-saved", "data-verdict").as_deref(), Some("approved"));
    assert!(t.text().contains("✓ 1 · ✎ 0 · ✗ 0"));
    assert_eq!(t.count("button.btn-primary[disabled]"), 0);
    t.click("button.verify-undo");
    assert_eq!(verdict(&item.id), None);
    assert!(t.text().contains(tx::QUESTION));
}

#[test]
fn wrong_keeps_his_note_and_cancel_goes_back() {
    let item = unchecked("card");
    let mut t = mount(item);
    t.click_text("button.verify-btn", tx::WRONG);
    assert!(t.text().contains(tx::WRONG_NOTE));
    t.click_text("button.verify-btn", tx::CANCEL);
    assert!(t.text().contains(tx::QUESTION));
    t.click_text("button.verify-btn", tx::WRONG);
    t.input("textarea", "  the page says 8 points  ");
    t.click_text("button.verify-btn", tx::WRONG_CONFIRM);
    assert_eq!(verdict(&item.id), Some(Verdict::Wrong));
    assert_eq!(
        with_review(|b| b.backend.borrow().judgement(&item.id).unwrap().note.clone()).as_deref(),
        Some("the page says 8 points")
    );
    assert_eq!(t.attr("p.verify-saved", "data-verdict").as_deref(), Some("wrong"));
    assert!(t.text().contains(tx::SAVED_WRONG));
}

#[test]
fn the_editor_needs_a_change_and_stores_the_edit() {
    let item = unchecked("card");
    let mut t = mount(item);
    t.click_text("button.verify-btn", tx::EDIT);
    // One box per field, labelled, plus the note.
    assert_eq!(t.count("div.verify-editor textarea"), verify::editable_fields(&item.body).len() + 1);
    assert!(t.text().contains(tx::F_FRONT) && t.text().contains(tx::F_BACK));
    t.click_text("button.verify-btn", tx::SAVE);
    assert_eq!(t.texts("p.verify-error"), [tx::NOTHING_CHANGED]);
    assert_eq!(verdict(&item.id), None);
    // Typing clears the message; a changed back saves as ✎ with the edit.
    t.input_nth("div.verify-editor textarea", 1, "A better answer");
    assert_eq!(t.count("p.verify-error"), 0);
    t.input_nth("div.verify-editor textarea", 2, "fixed the back");
    t.click_text("button.verify-btn", tx::SAVE);
    assert_eq!(verdict(&item.id), Some(Verdict::Edited));
    let edited = with_review(|b| b.backend.borrow().judgement(&item.id).unwrap().edit.clone());
    assert!(!edited.is_empty());
    assert!(t.text().contains(tx::SAVED_EDITED));
    // And the effective item now shows his words.
    let shown = with_review(|b| batna_core::review::StudyStore::effective(&*b.backend.borrow(), item));
    assert!(matches!(shown.body, batna_core::content::Body::Card { back, .. } if back == "A better answer"));
}

#[test]
fn an_mcq_answer_is_picked_from_its_options() {
    let item = unchecked("mcq");
    let mut t = mount(item);
    t.click_text("button.verify-btn", tx::EDIT);
    assert_eq!(t.count("div.verify-editor select"), 1);
    let batna_core::content::Body::Mcq { options, answer, .. } = &item.body else { unreachable!() };
    assert_eq!(t.count("div.verify-editor option"), options.len());
    assert!(t.texts("div.verify-editor option")[0].starts_with("A. "));
    let other = (answer + 1) % options.len();
    t.change_nth("div.verify-editor select", 0, &other.to_string());
    t.click_text("button.verify-btn", tx::SAVE);
    assert_eq!(verdict(&item.id), Some(Verdict::Edited));
    let shown = with_review(|b| batna_core::review::StudyStore::effective(&*b.backend.borrow(), item));
    assert!(matches!(shown.body, batna_core::content::Body::Mcq { answer, .. } if answer == other));
    // Cancel from the editor goes back to the question (after an undo).
    t.click("button.verify-undo");
    t.click_text("button.verify-btn", tx::EDIT);
    t.click_text("button.verify-btn", tx::CANCEL);
    assert!(t.text().contains(tx::QUESTION));
}

#[test]
fn a_verdict_that_isnt_saved_offers_try_again() {
    let item = unchecked("card");
    let mut t = mount(item);
    with_review(|b| b.backend.borrow().break_table("item_status"));
    t.click_text("button.verify-btn", tx::RIGHT);
    assert_eq!(verdict(&item.id), None);
    assert!(t.text().contains(tx::NOT_SAVED));
    assert!(t.text().contains(tx::QUESTION), "still asking");
    // Try again repeats the same tap (and fails the same way here).
    t.click("button.save-retry");
    assert!(t.text().contains(tx::NOT_SAVED));
}

#[test]
fn an_item_that_needs_no_check_shows_nothing() {
    crate::store::use_thread_store();
    ITEM.with(|i| *i.borrow_mut() = "hb.L99.404".into());
    let t = TestDom::new(Root);
    assert_eq!(t.count("section.verify"), 0);
}

#[test]
fn small_layout_helpers() {
    assert_eq!(rows_for("one"), 2);
    assert_eq!(rows_for(&"line\n".repeat(30)), 10);
    assert_eq!(letter(0), 'A');
    assert_eq!(letter(40), 'Z');
    assert_eq!(field_label("nope"), "");
}

#[test]
fn his_checks_export_as_yaml_he_can_read() {
    let item = unchecked("card");
    let dir = tempfile::tempdir().unwrap();
    crate::store::export_to_for_test(dir.path().to_path_buf());
    let mut t = mount(item);
    t.click_text("button.verify-btn", tx::RIGHT);
    t.click_text("button.btn-primary", tx::EXPORT_BUTTON);
    assert!(t.text().contains(tx::EXPORT_SAVED));
    let file = std::fs::read_dir(dir.path()).unwrap().next().unwrap().unwrap().path();
    assert!(file.file_name().unwrap().to_str().unwrap().starts_with("batna-verdicts-"));
    let yaml = std::fs::read_to_string(&file).unwrap();
    assert!(yaml.contains(&item.id) && yaml.contains("approved"), "{yaml}");
    // The same text is on screen to copy.
    assert!(t.html().contains(&item.id));
    // A folder that went away: the reason and a retry.
    crate::store::export_to_for_test(dir.path().join("gone"));
    t.click_text("button.btn-primary", tx::EXPORT_BUTTON);
    assert!(t.text().contains(tx::EXPORT_FAILED));
    std::fs::create_dir(dir.path().join("gone")).unwrap();
    t.click("button.save-retry");
    assert!(t.text().contains(tx::EXPORT_SAVED));
}
