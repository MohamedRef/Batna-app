//! His own cards tapped through (`crate::testdom`): the add-a-card sheet
//! with its checks and preview, saving, editing, deleting (and a delete or
//! save that fails), and text shared in from another app.

use std::cell::RefCell;

use super::*;
use crate::Ui;
use crate::prefs::DevicePrefs;
use crate::store::Review;
use crate::testdom::TestDom;

thread_local! {
    static REVIEW: RefCell<Option<Review>> = const { RefCell::new(None) };
    static STUDIED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
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
    rsx! {
        OwnCardsCard { on_study: move |_| STUDIED.set(STUDIED.get() + 1) }
        ShareInbox {}
    }
}

fn mount() -> TestDom {
    crate::store::use_thread_store();
    TestDom::new(Root)
}

fn saved() -> Vec<OwnCard> {
    crate::own::cards()
}

fn break_table(table: &str) {
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow().break_table(table));
}

#[test]
fn a_new_card_is_checked_previewed_and_saved() {
    let mut t = mount();
    assert!(t.text().contains(tx::NONE_YET));
    assert_eq!(t.count("[role=dialog]"), 0);
    assert!(!t.in_root(crate::own::drafting));
    t.click_text("button", tx::NEW);
    assert!(t.in_root(crate::own::drafting));
    assert!(t.text().contains(tx::FORM_NEW));
    // Empty: the problem sits under the front, nothing is saved.
    t.click_text("div.own-actions button", tx::SAVE);
    assert_eq!(t.texts("span.field-error"), [tx::E_FRONT]);
    assert_eq!(
        t.attr("label.own-field[data-invalid=true] textarea", "aria-invalid").as_deref(),
        Some("true")
    );
    t.input_nth("[role=dialog] textarea", 0, "SAAG ≥ 1.1 means?");
    t.click_text("div.own-actions button", tx::SAVE);
    assert_eq!(t.texts("span.field-error"), [tx::E_BACK]);
    t.input_nth("[role=dialog] textarea", 1, "Portal hypertension");
    assert_eq!(t.texts("bdi.own-count")[0], format!("17/{}", core_own::MAX_FRONT));
    // Lecture, page and source show in the preview.
    let lecture = Library::get().index.lectures[0].id.clone();
    t.change_nth("[role=dialog] select", 0, &lecture);
    t.input("input[type=number]", "47");
    t.input("input[dir=auto]", "Ward round");
    let preview = t.texts("div.own-preview-card")[0].clone();
    assert!(preview.contains("SAAG ≥ 1.1 means?") && preview.contains("Portal hypertension"));
    assert!(preview.contains("Ward round") && preview.contains(&crate::i18n::cite::page(47)));
    t.input("input[type=number]", "x");
    t.change_nth("[role=dialog] select", 0, "");
    t.input("input[type=number]", "47");
    t.click_text("div.own-actions button", tx::SAVE);
    assert_eq!(t.count("[role=dialog]"), 0, "saved and closed");
    let cards = saved();
    assert_eq!(cards.len(), 1);
    assert_eq!(
        (cards[0].front.as_str(), cards[0].page, cards[0].source.as_deref()),
        ("SAAG ≥ 1.1 means?", Some(47), Some("Ward round"))
    );
    assert!(t.text().contains(&tx::cards(1)));
    assert!(t.text().contains("SAAG ≥ 1.1 means?"));
    t.click_text("button", tx::STUDY_THEM);
    assert_eq!(STUDIED.get(), 1);
}

#[test]
fn edit_delete_and_a_failed_delete() {
    let mut t = mount();
    t.click_text("button", tx::NEW);
    t.input_nth("[role=dialog] textarea", 0, "Q one");
    t.input_nth("[role=dialog] textarea", 1, "A one");
    t.click_text("div.own-actions button", tx::SAVE);
    // Edit opens the same card; Escape and ✕ close without saving.
    t.click_text("button", tx::EDIT);
    assert!(t.text().contains(tx::FORM_EDIT));
    t.input_nth("[role=dialog] textarea", 0, "Q changed");
    t.key("[role=dialog]", dioxus::html::Key::Escape);
    assert_eq!(t.count("[role=dialog]"), 0);
    assert_eq!(saved()[0].front, "Q one");
    t.click_text("button", tx::EDIT);
    t.click("[role=dialog] button.icon-button");
    assert_eq!(t.count("[role=dialog]"), 0);
    t.click_text("button", tx::EDIT);
    t.click("div.own-backdrop");
    assert_eq!(t.count("[role=dialog]"), 0);
    t.click_text("button", tx::EDIT);
    t.input_nth("[role=dialog] textarea", 0, "Q changed");
    t.click_text("div.own-actions button", tx::SAVE);
    assert_eq!(saved()[0].front, "Q changed");
    // Delete asks first; Cancel keeps it.
    t.click_text("button", tx::DELETE);
    assert!(t.text().contains(tx::DELETE_SURE));
    t.click_text("div.own-actions button", tx::CANCEL);
    assert!(!t.text().contains(tx::DELETE_SURE));
    // A delete that fails keeps the row open with the reason and a retry.
    break_table("sync_outbox");
    t.click_text("button", tx::DELETE);
    t.click_text("button", tx::DELETE_YES);
    assert!(t.text().contains(tx::DELETE_FAILED));
    assert!(t.text().contains(tx::DELETE_SURE));
    t.click("button.save-retry");
    assert!(t.text().contains(tx::DELETE_FAILED));
    assert_eq!(saved().len(), 1, "still there");
}

#[test]
fn a_delete_that_works_removes_the_card() {
    let mut t = mount();
    t.click_text("button", tx::NEW);
    t.input_nth("[role=dialog] textarea", 0, "Q");
    t.input_nth("[role=dialog] textarea", 1, "A");
    t.click_text("div.own-actions button", tx::SAVE);
    t.click_text("button", tx::DELETE);
    t.click_text("button", tx::DELETE_YES);
    assert!(saved().is_empty());
    assert!(t.text().contains(tx::NONE_YET));
}

#[test]
fn a_save_that_fails_keeps_every_word() {
    let mut t = mount();
    break_table("sync_outbox");
    t.click_text("button", tx::NEW);
    t.input_nth("[role=dialog] textarea", 0, "Q kept");
    t.input_nth("[role=dialog] textarea", 1, "A kept");
    t.click_text("div.own-actions button", tx::SAVE);
    assert_eq!(t.count("[role=dialog]"), 1, "the sheet stays open");
    assert!(t.texts("p.field-error")[0].starts_with(tx::FAILED));
    assert!(t.html().contains("Q kept") && t.html().contains("A kept"));
}

#[test]
fn shared_text_opens_a_draft_or_waits_for_the_one_open() {
    let mut t = mount();
    let shared = |text: &str| crate::own::share::Shared {
        text: text.into(),
        subject: Some("Lecture notes".into()),
        app: Some("com.android.chrome".into()),
    };
    let first = shared("Ascites with SAAG above 1.1 points to portal hypertension.");
    t.in_root(|| arrive(&first));
    assert!(t.text().contains(tx::FORM_SHARED));
    assert!(t.text().contains(tx::SHARED_HINT));
    assert!(t.html().contains("Lecture notes"), "the subject is its source");
    // A second share while this one is open waits, and says so.
    let second = shared(&"long text ".repeat(400));
    t.in_root(|| arrive(&second));
    assert!(t.text().contains(tx::SHARE_WAITING));
    t.click("[role=dialog] button.icon-button");
    // The waiting one opens next, cut to a card's length.
    assert_eq!(t.count("[role=dialog]"), 1);
    assert!(t.text().contains(tx::SHARED_TRIMMED));
    assert!(!t.text().contains(tx::SHARE_WAITING));
}

#[test]
fn a_share_needs_text_and_names_where_it_came_from() {
    use crate::own::share::{Shared, parse, take};
    let got = parse(r#"{"text":"SAAG","subject":"  ","app":"com.android.chrome"}"#).unwrap();
    assert_eq!(got.source().as_deref(), Some("com.android.chrome"), "a blank subject falls back to the app");
    assert!(parse(r#"{"text":"   "}"#).is_none(), "nothing to make a card of");
    assert!(parse("not json").is_none());
    let bare = Shared { text: "x".into(), subject: None, app: None };
    assert_eq!(bare.source(), None);
    assert_eq!(take(), None, "only Android has a share inbox");
}
