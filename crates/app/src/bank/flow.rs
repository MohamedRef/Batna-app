//! The question bank tapped through (`crate::testdom`): setup and its
//! filters, a tutor set, a timed paper with flags and the submit question,
//! leaving and coming back, a mock, the result and its review list.
//! Each test has its own store (`store::use_thread_store`), so saved
//! setups, open papers and finished papers never cross tests.

use std::cell::RefCell;

use batna_core::exam::{Filter, Mode, Only};
use dioxus::prelude::*;

use super::*;
use crate::Ui;
use crate::i18n::bank as tx;
use crate::prefs::DevicePrefs;
use crate::store::Review;
use crate::testdom::TestDom;

thread_local! {
    static REVIEW: RefCell<Option<Review>> = const { RefCell::new(None) };
    static UI: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

#[component]
fn Root() -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Bank),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    UI.with(|u| *u.borrow_mut() = Some(ui));
    let review = use_context_provider(crate::store::test_review);
    REVIEW.with(|r| *r.borrow_mut() = Some(review.clone()));
    use_context_provider(|| {
        StudyStoreHandle(
            review.backend.clone() as std::rc::Rc<std::cell::RefCell<dyn batna_core::review::StudyStore>>
        )
    });
    rsx! { Bank {} }
}

/// A fresh store for this test, then the bank on it.
fn mount() -> TestDom {
    crate::store::use_thread_store();
    TestDom::new(Root)
}

/// The bank again on the same store, like the next launch.
fn relaunch() -> TestDom {
    TestDom::new(Root)
}

fn logged() -> usize {
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow().reviews_logged())
}

fn paper(t: &mut TestDom) -> Paper {
    t.in_root(|| match &*PHASE.peek() {
        Phase::Running(p) => p.clone(),
        _ => panic!("no paper running"),
    })
}

/// The option button (0-based, as shown) holding the right answer.
fn right_option(t: &mut TestDom) -> usize {
    let p = paper(t);
    let q = &p.questions[p.at];
    q.order.iter().position(|&o| o == q.answer).unwrap()
}

fn tap_option(t: &mut TestDom, shown: usize) {
    t.click_nth("button.q-option", shown);
}

/// Runs `f` where contexts can be read (no store in context there).
fn outside_the_screen<T>(f: impl FnOnce() -> T) -> T {
    fn app() -> Element {
        rsx! {}
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, f)
}

fn saved_papers() -> usize {
    crate::store::shared().settings_with_prefix::<batna_core::exam::Outcome>(PAPER_PREFIX).len()
}

#[test]
fn a_tutor_set_checks_each_answer_once_and_logs_it() {
    let mut t = mount();
    assert!(t.text().contains(tx::TITLE));
    assert_eq!(t.attr("button[role=radio][aria-checked=true]", "role").as_deref(), Some("radio"));
    assert!(t.texts("button[role=radio][aria-checked=true]")[0].contains(tx::MODE_TUTOR_NAME));
    t.click_text("button.chip.toggle", "10");
    assert_eq!(t.texts("button.chip.toggle[aria-pressed=true]"), ["10"]);
    t.click("button.bank-go");
    assert_eq!(t.texts("span.run-count"), ["1 / 10"]);
    assert_eq!(t.attr("div.bank-run", "data-mode").as_deref(), Some("tutor"));
    // Check waits for a pick; a pick is only a draft.
    assert!(t.text().contains(tx::PICK_FIRST));
    assert_eq!(t.count("button.run-main-action[disabled]"), 1);
    let right = right_option(&mut t);
    let wrong = (right + 1) % paper(&mut t).questions[0].order.len();
    tap_option(&mut t, wrong);
    assert_eq!(t.count("button.q-option[data-state=selected]"), 1);
    tap_option(&mut t, right);
    assert_eq!(t.count("button.q-option[aria-pressed=true]"), 1);
    assert_eq!(logged(), 0);
    t.click_text("button.run-main-action", tx::CHECK);
    assert_eq!(logged(), 1, "checked once, sent to review");
    assert_eq!(t.texts("p.bank-sr[role=status]"), [tx::checked(true, char::from(b'A' + right as u8))]);
    assert_eq!(t.count("button.q-option[data-state=correct]"), 1);
    assert!(t.text().contains(tx::RIGHT_PICKED));
    assert_eq!(t.count("button.q-option[disabled]"), paper(&mut t).questions[0].order.len(), "locked");
    assert!(t.texts("div.run-sub span.muted")[0].contains("1 / 1"));
    // The explanation pane: why for each option and the check bar.
    assert_eq!(t.count("ol.q-why-list li"), paper(&mut t).questions[0].order.len());
    // Next; then a wrong answer says which letter was right.
    t.click_text("button.run-main-action", tx::NEXT);
    assert_eq!(t.texts("span.run-count"), ["2 / 10"]);
    let right = right_option(&mut t);
    let wrong = (right + 1) % paper(&mut t).questions[1].order.len();
    tap_option(&mut t, wrong);
    t.click_text("button.run-main-action", tx::CHECK);
    assert_eq!(t.texts("p.bank-sr[role=status]"), [tx::checked(false, char::from(b'A' + right as u8))]);
    assert_eq!(t.count("button.q-option[data-state=wrong]"), 1);
    assert_eq!(t.count("button.q-option[data-state=missed]"), 1);
    assert_eq!(logged(), 2);
    // Leaving a tutor set just ends it: nothing is kept to resume.
    t.click("button.run-leave");
    assert!(t.text().contains(tx::START_TUTOR));
    assert_eq!(t.count(".paused"), 0);
    assert_eq!(saved_papers(), 0, "tutor sets are not papers");
}

fn read_only(on: bool) {
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow().set_read_only(on));
}

#[test]
fn a_tutor_answer_the_log_refuses_is_kept_and_saved_once_on_retry() {
    let mut t = mount();
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    let right = right_option(&mut t);
    tap_option(&mut t, right);
    read_only(true);
    t.click_text("button.run-main-action", tx::CHECK);
    assert_eq!(logged(), 0);
    assert_eq!(t.texts("div.bank-unsaved p"), [tx::UNSAVED]);
    // Still refused: nothing moves, the answer stays kept.
    t.click("button.bank-unsaved-retry");
    assert_eq!((logged(), t.count("div.bank-unsaved")), (0, 1));
    read_only(false);
    t.click("button.bank-unsaved-retry");
    assert_eq!(logged(), 1, "saved exactly once");
    assert_eq!(t.count("div.bank-unsaved"), 0);
    assert!(paper(&mut t).logged.is_some(), "a reason or Undo now finds it");
    t.in_root(super::unsaved::retry);
    assert_eq!(logged(), 1, "a second retry logs nothing more");
}

#[test]
fn a_timed_paper_the_log_refuses_says_so_instead_of_sent_to_review() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    let right = right_option(&mut t);
    let wrong = (right + 1) % paper(&mut t).questions[0].order.len();
    tap_option(&mut t, wrong);
    read_only(true);
    t.click(".run-side button.btn-secondary");
    t.click_text("[role=alertdialog] button", tx::CONFIRM_SUBMIT);
    assert!(t.text().contains(tx::RESULT));
    assert_eq!(logged(), 0);
    assert!(!t.text().contains(tx::SENT_TO_REVIEW));
    assert_eq!(t.count("div.bank-unsaved"), 1);
    read_only(false);
    t.click("button.bank-unsaved-retry");
    assert_eq!(logged(), 1);
    assert!(t.text().contains(tx::SENT_TO_REVIEW));
}

#[test]
fn a_timed_paper_flags_navigates_asks_before_submitting_and_scores() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    assert!(t.text().contains(tx::PRESET_TIMED));
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    assert_eq!(t.attr("div.bank-run", "data-mode").as_deref(), Some("timed"));
    assert_eq!(t.count("button.nav-cell"), 10);
    assert_eq!(t.count("div.timer-fill"), 1);
    // Answer the first right, flag the second, jump to the last.
    let right = right_option(&mut t);
    tap_option(&mut t, right);
    assert_eq!(t.attr("button.nav-cell[aria-current=true]", "data-done").as_deref(), Some("yes"));
    t.click_text("button.btn-primary", tx::NEXT);
    t.click_text("button", tx::FLAG);
    assert_eq!(t.attr("button.nav-cell[data-flag=yes]", "aria-label").unwrap(), {
        format!("{} 2، {}، {}، {}", tx::QUESTION, tx::UNANSWERED, tx::FLAGGED, tx::NOW_HERE)
    });
    t.click_text("button", tx::PREV);
    assert_eq!(t.texts("span.run-count"), ["1 / 10"]);
    t.click_text("button.nav-cell", "10");
    assert_eq!(t.texts("span.run-count"), ["10 / 10"]);
    // Submit with blanks and a flag asks; "back" goes to the first blank.
    t.click_text("div.run-foot button", tx::SUBMIT);
    assert_eq!(t.count("[role=alertdialog]"), 1);
    assert!(t.text().contains(tx::CONFIRM_FLAGGED));
    t.click_text("[role=alertdialog] button", tx::CONFIRM_BACK_TO);
    assert_eq!(t.count("[role=alertdialog]"), 0);
    assert_eq!(t.texts("span.run-count"), ["2 / 10"]);
    // The backdrop closes it too; then hand it in.
    t.click(".run-side button.btn-secondary");
    t.click("div.sheet-backdrop");
    assert_eq!(t.count("[role=alertdialog]"), 0);
    t.click(".run-side button.btn-secondary");
    t.click_text("[role=alertdialog] button", tx::CONFIRM_SUBMIT);
    // The result: 1 of 10, the one answer logged, the paper saved.
    assert!(t.text().contains(tx::RESULT));
    assert_eq!(t.texts("bdi.pct")[0], "10%");
    assert!(t.text().contains(tx::SENT_TO_REVIEW));
    assert_eq!(logged(), 1, "only answered questions go to review");
    assert_eq!(saved_papers(), 1);
    assert_eq!(t.count("button.review-item"), 9, "wrong ones first");
    t.click_text("button.chip.toggle", tx::SHOW_ALL);
    assert_eq!(t.count("button.review-item"), 10);
    assert_eq!(t.count("button.review-item[data-right=yes]"), 1);
    t.click_text("button.chip.toggle", tx::SHOW_FLAGGED);
    assert_eq!(t.count("button.review-item"), 1);
    // A review question opens with its answer, and Close shuts it.
    t.click("button.review-item");
    assert_eq!(t.count("div.review-detail"), 1);
    assert_eq!(t.count("button.q-option[data-state=missed]"), 1, "the flagged blank shows the right one");
    t.click_text("button.bank-close", tx::CLOSE);
    assert_eq!(t.count("div.review-detail"), 0);
    // Retry the wrong ones: a tutor set of 9.
    t.click_text("button", tx::RETRY_WRONG);
    assert_eq!(t.texts("span.run-count"), ["1 / 9"]);
    assert_eq!(t.attr("div.bank-run", "data-mode").as_deref(), Some("tutor"));
    // Back on setup the finished paper is in the history.
    t.click("button.run-leave");
    assert!(t.text().contains(tx::HISTORY));
    assert_eq!(t.texts("ol.history summary bdi.score"), ["1/10"]);
}

#[test]
fn a_full_right_paper_offers_a_new_one_and_praises_strong_topics() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    for i in 0..10 {
        let right = right_option(&mut t);
        tap_option(&mut t, right);
        if i < 9 {
            t.click_text("button.btn-primary", tx::NEXT);
        }
    }
    // Nothing blank or flagged: Submit goes straight in.
    t.click_text("div.run-foot button", tx::SUBMIT);
    assert_eq!(t.texts("bdi.pct")[0], "100%");
    assert!(!t.text().contains(tx::SENT_TO_REVIEW));
    assert_eq!(logged(), 10);
    assert_eq!(t.count("button.review-item"), 10, "all shown when all right");
    t.click_text("button", tx::NEW_PAPER);
    assert!(t.text().contains(tx::START_TIMED));
}

#[test]
fn leaving_a_timed_paper_keeps_it_across_a_relaunch() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    let right = right_option(&mut t);
    tap_option(&mut t, right);
    t.click("button.run-leave");
    assert!(t.text().contains(tx::PAUSED_TITLE));
    assert!(t.texts(".paused p")[0].contains("1/10"));
    t.click_text(".paused button", tx::RESUME);
    assert_eq!(t.texts("span.run-count"), ["1 / 10"]);
    assert_eq!(t.count("button.q-option[aria-pressed=true]"), 1, "his answer is still there");
    t.click("button.run-leave");
    drop(t);
    // Android killed the app: the left paper is back as paused.
    let mut t = relaunch();
    assert!(t.text().contains(tx::PAUSED_TITLE));
    // A new paper scores the left one first.
    let before = saved_papers();
    t.click("button.bank-go");
    assert_eq!(saved_papers(), before + 1);
    assert_eq!(t.texts("span.run-count"), ["1 / 10"]);
    t.click("button.run-leave");
    t.click_text(".paused button", tx::SUBMIT_NOW);
    assert!(t.text().contains(tx::RESULT));
    assert_eq!(saved_papers(), before + 2);
    t.click_text("button", tx::BACK);
    assert_eq!(t.count(".paused"), 0);
    drop(t);
    let t = relaunch();
    assert_eq!(t.count(".paused"), 0, "a submitted paper never comes back");
}

#[test]
fn time_running_out_hands_the_paper_in() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    // The clock's animation ends: in it goes, no question asked.
    t.animation_end("div.timer-fill");
    assert!(t.text().contains(tx::RESULT));
    assert_eq!(t.count("[role=alertdialog]"), 0);
    // A paper whose time passed while away is handed in on the next tap.
    t.click_text("button", tx::BACK);
    t.click("button.bank-go");
    t.in_root(|| {
        if let Phase::Running(p) = &mut *PHASE.write() {
            p.started_ms = 0;
        }
    });
    tap_option(&mut t, 0);
    assert!(t.text().contains(tx::RESULT));
    // And one left past its time is submitted when he resumes it.
    t.click_text("button", tx::BACK);
    t.click("button.bank-go");
    t.click("button.run-leave");
    t.in_root(|| {
        if let Some(p) = &mut *PAUSED.write() {
            p.started_ms = 0;
        }
    });
    assert!(t.text().contains(tx::PAUSED_OVER));
    t.click_text(".paused button", tx::RESUME);
    assert!(t.text().contains(tx::RESULT));
    // Moving to another question after the limit hands it in as well.
    t.click_text("button", tx::BACK);
    t.click("button.bank-go");
    t.in_root(|| {
        if let Phase::Running(p) = &mut *PHASE.write() {
            p.started_ms = 0;
        }
    });
    t.click_text("button.nav-cell", "3");
    assert!(t.text().contains(tx::RESULT));
}

#[test]
fn a_mock_uses_the_whole_pool_and_its_own_count() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_MOCK_NAME);
    assert!(t.text().contains(tx::MOCK_SCOPE_NOTE));
    assert_eq!(t.count("button.bank-customise"), 0, "no filters for a mock");
    t.click_text("button.chip.toggle", "20");
    t.click("button.bank-go");
    assert_eq!(t.texts("span.run-count"), ["1 / 20"]);
    assert!(t.texts("div.run-sub span.chip")[0].contains(tx::MODE_MOCK_NAME));
    t.click(".run-side button.btn-secondary");
    t.click_text("[role=alertdialog] button", tx::CONFIRM_SUBMIT);
    assert!(t.text().contains(tx::MOCK_TITLE));
    t.click_text("button", tx::BACK);
    assert_eq!(t.texts("ol.history span.chip"), [tx::MODE_MOCK]);
}

#[test]
fn filters_narrow_the_scope_and_an_empty_scope_offers_the_way_back() {
    let mut t = mount();
    let pool = t.in_root(|| exam::mcqs(&study_lectures()).len());
    assert!(t.text().contains(tx::SCOPE_ALL));
    t.click("button.bank-customise");
    assert_eq!(t.attr("button.bank-customise", "aria-expanded").as_deref(), Some("true"));
    assert!(t.text().contains(tx::PICK_LECTURE_FOR_TOPICS));
    // One lecture: its topics show and the scope names it.
    let lecture = t.attr(".bank-advanced button.chip.toggle", "title").unwrap();
    t.click(".bank-advanced button.chip.toggle");
    assert!(t.text().contains(tx::SCOPE_CHOSEN));
    assert!(t.texts("bdi.bank-scope")[0].len() > 1);
    assert!(lecture.len() > 1);
    assert!(!t.text().contains(tx::PICK_LECTURE_FOR_TOPICS));
    let topics_before = t.count(".bank-advanced button.chip.toggle[aria-pressed=true]");
    assert_eq!(topics_before, 1, "the lecture chip");
    // A topic of it (topic titles are the book's English).
    let chips = t.texts(".bank-advanced button.chip.toggle");
    let topic = chips.iter().find(|c| c.is_ascii()).expect("a topic chip").clone();
    t.click_text(".bank-advanced button.chip.toggle", &topic);
    assert_eq!(t.count(".bank-advanced button.chip.toggle[aria-pressed=true]"), 2);
    assert_eq!(t.texts("bdi.bank-scope")[0], topic, "one topic names the scope");
    // A kind tag narrows it further and is said under the title.
    if let Some(tag) =
        chips.iter().find(|c| [tx::TAG_HIGH_YIELD, tx::TAG_BOOK, tx::TAG_NUMBERS].contains(&c.as_str()))
    {
        t.click_text(".bank-advanced button.chip.toggle", tag);
        assert!(t.texts("button.bank-scope-tap span.muted")[0].contains(tag.as_str()));
        t.click_text(".bank-advanced button.chip.toggle", tag);
    }
    t.click_text(".bank-advanced button.chip.toggle", &topic);
    // Unpicking the lecture drops it.
    t.click(".bank-advanced button.chip.toggle[aria-pressed=true]");
    assert!(t.text().contains(tx::PICK_LECTURE_FOR_TOPICS));
    t.click(".bank-advanced button.chip.toggle");
    t.click_text("button", tx::RESET_SCOPE);
    assert!(t.text().contains(tx::SCOPE_ALL));
    // "Only wrong or unseen" with everything right last time: nothing left,
    // and the button undoes just that.
    t.in_root(|| {
        let ids: Vec<String> = exam::mcqs(&study_lectures()).iter().map(|q| q.id.to_string()).collect();
        let mut h = HISTORY.write();
        for id in ids {
            h.last.insert(id, true);
        }
    });
    t.click_text("button.chip.toggle", tx::ONLY_WRONG);
    assert!(t.text().contains(tx::NONE_MATCH));
    assert!(t.text().contains(tx::EMPTY_ALL_RIGHT));
    t.click_text("div.bank-empty button", tx::WIDEN_ALL_RIGHT);
    assert!(!t.text().contains(tx::NONE_MATCH));
    assert!(
        t.texts("p.bank-scope-note")[0]
            .contains(&crate::i18n::count::count(pool, &crate::i18n::count::QUESTION))
    );
    // Close the filters again.
    t.click("button.bank-customise");
    assert_eq!(t.count(".bank-advanced"), 0);
}

#[test]
fn why_a_scope_is_empty_and_what_widens_it() {
    let pool_lectures = outside_the_screen(study_lectures);
    let pool = exam::mcqs(&pool_lectures);
    let mut last = exam::LastResults::new();
    for q in &pool {
        last.insert(q.id.to_string(), true);
    }
    use setup::{Empty, why_empty};
    let only = Filter { only: Only::WrongOrUnseen, ..Filter::default() };
    assert_eq!(why_empty(&only, &pool, &last), Empty::AllRight);
    let tag = Filter { tags: ["no-such-tag".to_string()].into(), ..Filter::default() };
    assert_eq!(why_empty(&tag, &pool, &last), Empty::Tags);
    let topic = Filter { topics: [("hb.L99".to_string(), "x".to_string())].into(), ..Filter::default() };
    assert_eq!(why_empty(&topic, &pool, &last), Empty::Topics);
}

#[test]
fn the_setup_comes_back_as_he_left_it_minus_what_is_gone() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "40");
    drop(t);
    let t = relaunch();
    assert!(t.texts("button[role=radio][aria-checked=true]")[0].contains(tx::MODE_TIMED_NAME));
    assert_eq!(t.texts("button.chip.toggle[aria-pressed=true]"), ["40"]);
    // A saved scope naming a lecture the content lost, and a count the screen
    // no longer offers, are dropped.
    let lectures = outside_the_screen(study_lectures);
    let pool = exam::mcqs(&lectures);
    let saved = setup::Saved {
        mode: Mode::Timed,
        filter: Filter { lectures: ["hb.L99".to_string()].into(), ..Filter::default() },
        count: 7,
        mock_count: 3,
        advanced: true,
    }
    .pruned(&pool);
    assert!(saved.filter.lectures.is_empty());
    assert_eq!((saved.count, saved.mock_count), (20, exam::MOCK_SIZE));
    assert_eq!(saved.mode, Mode::Timed);
}

#[test]
fn a_weak_topic_opens_the_bank_on_it() {
    let mut t = mount();
    t.click_text("button[role=radio]", tx::MODE_TIMED_NAME);
    t.click_text("button.chip.toggle", "10");
    t.click("button.bank-go");
    t.click(".run-side button.btn-secondary");
    t.click_text("[role=alertdialog] button", tx::CONFIRM_SUBMIT);
    // 0 right: every topic is weak.
    assert!(t.text().contains(tx::WEAK));
    assert!(t.count("button.chip.weak") >= 1);
    t.click("button.chip.weak");
    assert!(t.text().contains(tx::SCOPE_CHOSEN), "the setup, narrowed");
    assert!(t.texts("button[role=radio][aria-checked=true]")[0].contains(tx::MODE_TIMED_NAME));
    // A lecture row in the history opens its page.
    t.click("ol.history button.tap-row");
    assert!(t.in_root(crate::lecture::is_open));
}

#[test]
fn small_helpers_name_and_date_things() {
    assert_eq!(topic_title("hb.L99", "nope"), "nope");
    assert_eq!(lecture_title("hb.L99"), "hb.L99");
    let lec = outside_the_screen(study_lectures).remove(0);
    assert_eq!(lecture_title(&lec.lecture), Library::get().lectures[&lec.lecture].title);
    outside_the_screen(|| {
        assert!(lookup("hb.L99.001").is_none());
        assert!(lookup("nodot").is_none());
        let id = &lec.items[0].id;
        assert_eq!(&lookup(id).unwrap().1.id, id, "without a store the book's own item");
    });
}

#[test]
fn a_sync_reloads_the_papers() {
    let mut t = mount();
    let o = exam::score(&[], &[], Mode::Mock, 5, 0);
    crate::store::shared().put_setting(&format!("{PAPER_PREFIX}other.5"), &o).unwrap();
    assert_eq!(t.count("ol.history li"), 0);
    t.in_root(reload_history);
    assert_eq!(t.count("ol.history li"), 1);
    // A course switch keeps only that course's picks.
    t.in_root(|| {
        let first = study_lectures().remove(0).lecture;
        setup::focus(&first, None);
        course_changed();
    });
    assert!(t.text().contains(tx::SCOPE_CHOSEN));
}
