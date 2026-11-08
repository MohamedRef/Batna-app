//! The study screen tapped through end to end (`crate::testdom`): setup,
//! each renderer, grading into the review store, Undo, a save that fails,
//! "study this" while a session runs, and the summary.

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::Ui;
use crate::prefs::DevicePrefs;
use crate::store::Review;
use crate::testdom::TestDom;
use batna_core::content::Body;
use batna_core::review::{ReviewEvent, StudyStore, StudyStoreHandle};

/// The first playable item of `kind` in the library.
pub(crate) fn first(kind: &str) -> &'static Item {
    ready_lectures()
        .into_iter()
        .flat_map(|l| &l.items)
        .find(|i| i.body.kind() == kind)
        .unwrap_or_else(|| panic!("no {kind} item"))
}

/// A store that refuses to save or to take back, for the failure paths.
#[derive(Default)]
struct Broken {
    save_fails: bool,
    retract_fails: bool,
    saved: Vec<ReviewEvent>,
}

impl StudyStore for Broken {
    fn record(&mut self, event: ReviewEvent) {
        self.saved.push(event);
    }
    fn record_logged(&mut self, event: ReviewEvent) -> Result<Option<String>, String> {
        if self.save_fails {
            return Err("disk full".into());
        }
        self.saved.push(event);
        Ok(Some(format!("ev-{}", self.saved.len())))
    }
    fn retract(&mut self, _event_id: &str) -> bool {
        !self.retract_fails
    }
    fn due(&self, _now_ms: i64) -> Vec<String> {
        Vec::new()
    }
    fn is_new(&self, _item_id: &str) -> bool {
        true
    }
    fn is_hidden(&self, _item_id: &str) -> bool {
        false
    }
    fn next_due_ms(&self, _now_ms: i64) -> Option<i64> {
        None
    }
}

thread_local! {
    static REVIEW: RefCell<Option<Review>> = const { RefCell::new(None) };
    static BROKEN: RefCell<Option<Rc<RefCell<Broken>>>> = const { RefCell::new(None) };
    static START_WITH: RefCell<Option<Start>> = const { RefCell::new(None) };
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
    use_context_provider(|| match BROKEN.with(|b| b.borrow().clone()) {
        Some(b) => StudyStoreHandle(b as Rc<RefCell<dyn StudyStore>>),
        None => StudyStoreHandle(review.backend.clone() as Rc<RefCell<dyn StudyStore>>),
    });
    use_hook(|| {
        if let Some(start) = START_WITH.with(|s| s.borrow_mut().take()) {
            *START.write() = Some(start);
        }
    });
    rsx! { Study {} }
}

fn mount(start: Option<Start>, broken: Option<Broken>) -> TestDom {
    crate::store::use_memory_store();
    START_WITH.with(|s| *s.borrow_mut() = start);
    BROKEN.with(|b| *b.borrow_mut() = broken.map(|b| Rc::new(RefCell::new(b))));
    TestDom::new(Root)
}

fn items(ids: &[&str]) -> Option<Start> {
    Some(Start { lectures: None, items: Some(ids.iter().map(|s| s.to_string()).collect()), minutes: 10 })
}

fn logged() -> usize {
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow().reviews_logged())
}

fn grade(t: &mut TestDom, g: &str) {
    t.click(&format!("button.rating[data-grade={g}]"));
}

#[test]
fn setup_picks_time_lectures_and_starts() {
    let mut t = mount(None, None);
    assert!(t.text().contains(tx::TITLE));
    // Ten minutes by default; a tap moves the radio.
    assert_eq!(t.attr("button[role=radio][aria-checked=true]", "role").as_deref(), Some("radio"));
    t.click_text("button[role=radio]", "20");
    assert_eq!(t.texts("button[role=radio][aria-checked=true]").len(), 1);
    assert!(t.texts("button[role=radio][aria-checked=true]")[0].contains("20"));
    // Everything starts picked; "clear" leaves nothing to study.
    let chips = t.count("button.st-chip[aria-pressed=true]");
    assert!(chips > 1, "every ready lecture is picked");
    t.click_text("button.btn-link", tx::CLEAR_ALL);
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), 0);
    assert!(t.text().contains(tx::NOTHING_NOW));
    assert_eq!(t.count("button.st-start"), 0);
    // One lecture: its topics show, and a topic chip toggles.
    t.click("button.st-chip");
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), 1);
    assert!(t.text().contains(tx::TOPICS));
    let before = t.count("button.st-chip[aria-pressed=true]");
    let topic = t.texts("button.st-chip").last().cloned().unwrap();
    t.click_text("button.st-chip", &topic);
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), before + 1);
    t.click_text("button.st-chip", &topic);
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), before);
    // Picking all again, then start: a running session with a progress bar.
    t.click_text("button.btn-link", tx::PICK_ALL);
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), chips);
    assert!(t.text().contains(tx::START));
    t.click("button.st-start");
    assert_eq!(t.count("div.st-progress"), 1);
    assert_eq!(t.count("article.st-item"), 1);
    // "End" goes to the summary of an unanswered session, then back to setup.
    t.click_text("button.btn-link", tx::END);
    assert!(t.text().contains(tx::SUMMARY_EMPTY));
    t.click_text("button", tx::ANOTHER);
    assert!(t.text().contains(tx::LECTURES));
}

#[test]
fn a_card_reveals_grades_saves_and_undoes() {
    let card = first("card");
    let cloze = first("cloze");
    let mut t = mount(items(&[&card.id, &cloze.id]), None);
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("card"));
    assert_eq!(t.texts("span.st-count"), ["1 / 2"]);
    assert_eq!(t.attr("span.st-badge", "data-badge").as_deref(), Some("new"));
    // The answer and the rating row only after Reveal.
    assert_eq!(t.count("button.rating"), 0);
    t.click_text("button", tx::REVEAL);
    assert_eq!(t.count("div.st-answer"), 1);
    assert_eq!(t.count("button.rating"), 4);
    assert_eq!(logged(), 0);
    grade(&mut t, "good");
    assert_eq!(logged(), 1, "the grade is in the review log");
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("cloze"));
    assert_eq!(t.texts("span.st-count"), ["2 / 2"]);
    // Undo takes the grade back out of the log and reopens the card, answer shown.
    t.click("button.st-undo");
    assert_eq!(logged(), 0);
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("card"));
    assert_eq!(t.texts("p.st-undo-note"), [tx::UNDONE]);
    assert_eq!(t.attr("p.st-undo-note", "data-failed").as_deref(), Some("no"));
    assert_eq!(t.count("div.st-answer"), 1, "a reopened card shows its answer");
    grade(&mut t, "again");
    assert_eq!(logged(), 1);
    // The cloze: reveal the blanks, then the last grade ends the session.
    t.click_text("button", tx::REVEAL_BLANKS);
    assert_eq!(t.attr("p.st-verdict", "data-match").as_deref(), Some("wrong"));
    grade(&mut t, "good");
    // "Again" brings the card back once at the end, as a retry.
    assert_eq!(t.attr("span.st-badge", "data-badge").as_deref(), Some("retry"));
    t.click_text("button", tx::REVEAL);
    grade(&mut t, "good");
    assert_eq!(logged(), 3, "good, again, good, and the retry's good, minus the undone one");
    assert!(t.text().contains(tx::SUMMARY_TITLE), "{}", t.text());
    let stats = t.texts("dl.st-stats dd");
    assert_eq!(stats, ["2", "1", "2"], "2 items, 1 right first time, 2 new");
    assert!(t.text().contains(tx::SUM_BACK_SOON));
    assert_eq!(t.count("ul.st-shaky li"), 1);
    // The summary's Undo reopens the last item.
    t.click_text("button", tx::UNDO_LAST);
    assert_eq!(logged(), 2);
    assert_eq!(t.count("article.st-item"), 1);
}

#[test]
fn a_failed_save_keeps_the_item_and_lets_him_rate_again() {
    let card = first("card");
    let mut t = mount(items(&[&card.id]), Some(Broken { save_fails: true, ..Broken::default() }));
    t.click_text("button", tx::REVEAL);
    grade(&mut t, "good");
    assert_eq!(t.texts("p.st-undo-note"), [tx::SAVE_FAILED]);
    assert_eq!(t.attr("p.st-undo-note", "data-failed").as_deref(), Some("yes"));
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("card"));
    // The rating row is live again after the failure.
    assert_eq!(t.count("button.rating[disabled]"), 0);
    assert_eq!(t.count("div.st-answer"), 1, "his answer stays on screen");
}

#[test]
fn an_undo_the_store_refuses_says_so_and_moves_nothing() {
    let card = first("card");
    let cloze = first("cloze");
    let mut t =
        mount(items(&[&card.id, &cloze.id]), Some(Broken { retract_fails: true, ..Broken::default() }));
    t.click_text("button", tx::REVEAL);
    grade(&mut t, "good");
    t.click("button.st-undo");
    assert_eq!(t.texts("p.st-undo-note"), [tx::UNDO_FAILED]);
    assert_eq!(t.attr("p.st-undo-note", "data-failed").as_deref(), Some("yes"));
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("cloze"), "still on the next item");
}

#[test]
fn study_this_during_a_session_asks_first() {
    let card = first("card");
    let cloze = first("cloze");
    let essay = first("essay");
    let mut t = mount(items(&[&card.id]), None);
    let ask = |t: &mut TestDom, ids: &[&str]| {
        let req = items(ids).unwrap();
        t.in_root(move || *START.write() = Some(req));
    };
    // Keep: the request goes away, the session is untouched.
    ask(&mut t, &[&cloze.id, &essay.id]);
    assert_eq!(t.count("[role=alertdialog]"), 1);
    assert!(t.text().contains(&format!("{} (2)", tx::PENDING_ADD)));
    t.click_text("button", tx::PENDING_KEEP);
    assert_eq!(t.count("[role=alertdialog]"), 0);
    assert_eq!(t.texts("span.st-count"), ["1 / 1"]);
    // Add: the new items go after the current session.
    ask(&mut t, &[&cloze.id, &essay.id]);
    t.click_text("button", tx::PENDING_ADD);
    assert_eq!(t.texts("span.st-count"), ["1 / 3"]);
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("card"));
    // Replace: a new session on just those items.
    ask(&mut t, &[&essay.id]);
    t.click_text("button", tx::PENDING_REPLACE);
    assert_eq!(t.texts("span.st-count"), ["1 / 1"]);
    assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some("essay"));
}

#[test]
fn a_session_of_nothing_playable_goes_straight_to_the_summary() {
    let t = mount(items(&["nope.L99.001"]), None);
    assert!(t.text().contains(tx::SUMMARY_EMPTY));
    assert!(t.text().contains(tx::NEXT_TOMORROW));
    assert_eq!(t.count("dl.st-stats"), 0);
}

#[test]
fn a_lecture_start_plans_by_time() {
    let lecture = ready_lectures()[0].lecture.clone();
    let start = Start { lectures: Some(vec![lecture.clone()]), items: None, minutes: 5 };
    let t = mount(Some(start), None);
    assert_eq!(t.count("article.st-item"), 1);
    let len: usize = t.texts("span.st-count")[0].split(" / ").nth(1).unwrap().parse().unwrap();
    assert!(len >= 1);
    // Every item comes from the asked lecture.
    let text = t.text();
    let title = lecture_title(ready_lectures()[0]);
    assert!(text.contains(&title), "{text}");
}

#[test]
fn enumerate_typed_recall_pre_ticks_and_suggests() {
    let item = first("enumerate");
    let Body::Enumerate { points, pass, .. } = &item.body else { unreachable!() };
    let mut t = mount(items(&[&item.id]), None);
    assert_eq!(t.count("textarea.st-recall"), 1);
    t.input("textarea.st-recall", &points[0]);
    t.click_text("button", tx::SHOW_POINTS);
    assert_eq!(t.count("ul.st-points li"), points.len());
    assert!(t.text().contains(tx::WHAT_I_WROTE));
    assert!(t.text().contains(tx::PRE_TICKED));
    let ticked = t.count("ul.st-points input[checked]");
    assert!(ticked >= 1, "the typed point is pre-ticked");
    // Tick every point: the verdict turns to a pass and Easy/Good is suggested.
    for i in 0..points.len() {
        t.change_nth("ul.st-points input", i, "true");
    }
    assert_eq!(t.attr("p.st-verdict", "data-match").as_deref(), Some("right"));
    assert!(t.texts("p.st-verdict")[0].contains(&format!("{} {} {}", points.len(), tx::OF, points.len())));
    assert!(*pass <= points.len());
    assert_eq!(t.count("button.rating[data-suggested=yes]"), 1);
    // Untick all: short of the pass mark, and it says how many are missing.
    for i in 0..points.len() {
        t.change_nth("ul.st-points input", i, "false");
    }
    assert_eq!(t.attr("p.st-verdict", "data-match").as_deref(), Some("wrong"));
    assert!(t.texts("p.st-verdict")[0].contains(tx::ENUM_SHORT_A));
    assert_eq!(t.attr("button.rating[data-suggested=yes]", "data-grade").as_deref(), Some("again"));
}

#[test]
fn essay_compares_headings_with_the_outline() {
    let item = first("essay");
    let Body::Essay { outline, .. } = &item.body else { unreachable!() };
    let sections = batna_core::session::outline_sections(outline);
    let mut t = mount(items(&[&item.id]), None);
    assert!(t.text().contains(tx::ESSAY_HOW));
    t.input("textarea.st-recall", &sections[0].0);
    t.click_text("button", tx::COMPARE);
    assert!(t.text().contains(tx::MODEL_OUTLINE));
    assert_eq!(t.count("ol.st-outline > li"), sections.len());
    assert!(t.texts("p.st-pre")[0].contains(&sections[0].0));
    let got = |t: &TestDom| t.texts("p.st-verdict")[0].clone();
    assert!(got(&t).starts_with(&format!("{} 1 ", tx::HEADINGS_GOT)), "{}", got(&t));
    for i in 0..sections.len() {
        t.change_nth("ol.st-outline input", i, "true");
    }
    assert!(got(&t).contains(&format!("{n} {} {n}", tx::OF, n = sections.len())));
    assert_eq!(t.attr("button.rating[data-suggested=yes]", "data-grade").as_deref(), Some("easy"));
    grade(&mut t, "easy");
    assert_eq!(logged(), 1);
}

#[test]
fn cloze_checks_typed_blanks_with_enter() {
    let item = first("cloze");
    let Body::Cloze { text, .. } = &item.body else { unreachable!() };
    let answers: Vec<String> = text
        .split("{{c")
        .skip(1)
        .filter_map(|s| s.split("::").nth(1).map(|a| a.split("}}").next().unwrap().trim().to_string()))
        .collect();
    let mut t = mount(items(&[&item.id]), None);
    assert_eq!(t.count("input.st-blank"), answers.len());
    for (i, a) in answers.iter().enumerate() {
        t.input_nth("input.st-blank", i, a);
    }
    t.key("input.st-blank", dioxus::html::Key::Enter);
    assert_eq!(t.attr("p.st-verdict", "data-match").as_deref(), Some("right"));
    assert_eq!(t.texts("p.st-verdict"), [tx::CLOZE_RIGHT]);
    assert_eq!(t.count("span.st-filled[data-match=right]"), answers.len());
    assert_eq!(t.count("s.st-typed"), 0, "nothing crossed out when right");
    assert_eq!(t.attr("button.rating[data-suggested=yes]", "data-grade").as_deref(), Some("good"));
}

#[test]
fn a_wrong_cloze_shows_what_he_typed_crossed_out() {
    let item = first("cloze");
    let mut t = mount(items(&[&item.id]), None);
    t.input_nth("input.st-blank", 0, "zzz");
    t.click_text("button", tx::CHECK);
    assert_eq!(t.texts("s.st-typed"), ["zzz"]);
    assert_eq!(t.texts("p.st-verdict"), [tx::CLOZE_WRONG]);
}

#[test]
fn every_playable_kind_opens_in_the_player() {
    for kind in PLAYABLE {
        let item = first(kind);
        let t = mount(items(&[&item.id]), None);
        assert_eq!(t.attr("article.st-item", "data-kind").as_deref(), Some(*kind));
        // The heading for screen readers names the kind and the lecture.
        assert_eq!(t.count("h1.st-sr-only"), 1);
    }
}

#[test]
fn his_own_cards_are_a_deck_he_can_study() {
    let card = batna_core::own::OwnCard {
        id: "own.flow1".into(),
        front: "Wilson disease gene?".into(),
        back: "ATP7B".into(),
        lecture: None,
        page: None,
        source: None,
        created_ms: 1,
    };
    let mut t = mount(None, None);
    REVIEW.with(|r| r.borrow().as_ref().unwrap().backend.borrow_mut().save_own(&card).unwrap());
    t.settle();
    // The deck shows as a lecture chip under "your cards", and in the card list.
    assert!(t.texts("button.st-chip").iter().any(|c| c.contains(crate::i18n::own::TITLE)));
    assert!(t.text().contains("Wilson disease gene?"));
    // "Study them" starts a session on just his cards.
    t.click_text("button", crate::i18n::own::STUDY_THEM);
    assert_eq!(t.texts("span.st-count"), ["1 / 1"]);
    assert!(t.texts("p.st-where").iter().any(|w| w.contains(crate::i18n::own::TITLE)));
    // No book page for his own card.
    assert!(!t.html().contains("data-page"));
    assert!(is_playable("own.flow1"));
}

#[test]
fn a_course_in_view_narrows_setup_and_counts_due_elsewhere() {
    let lectures = ready_lectures();
    let course = |id: &str| batna_core::program::course_of(id).to_string();
    let here = course(&lectures[0].lecture);
    let away = lectures.iter().find(|l| course(&l.lecture) != here).expect("two courses with study items");
    let away_item = away.items.iter().find(|i| PLAYABLE.contains(&i.body.kind())).unwrap();
    let mut t = mount(None, None);
    let all_chips = t.count("button.st-chip[aria-pressed=true]");
    // An answer a month ago makes the other course's item due now.
    REVIEW.with(|r| {
        r.borrow().as_ref().unwrap().backend.borrow_mut().record(ReviewEvent {
            item_id: away_item.id.clone(),
            grade: batna_core::review::Grade::Again,
            origin: batna_core::review::Origin::Study,
            at_ms: now_ms() - 30 * 86_400_000,
            took_ms: 5_000,
        })
    });
    t.in_root(|| crate::courses::focus::show_for_test(Some(here.clone())));
    let chips = t.count("button.st-chip[aria-pressed=true]");
    assert!(chips < all_chips, "only the course in view is picked ({chips} of {all_chips})");
    assert!(t.text().contains(&tx::due_elsewhere(1)), "{}", t.text());
    t.in_root(|| crate::courses::focus::show_for_test(None));
    assert_eq!(t.count("p.st-elsewhere"), 0);
    assert_eq!(t.count("button.st-chip[aria-pressed=true]"), all_chips);
}

#[test]
fn content_gone_under_a_session_still_lets_him_end_it() {
    let mut t = mount(None, None);
    t.in_root(|| {
        let gone = batna_core::session::Planned {
            item_id: "hb.L99.404".into(),
            lecture: "hb.L99".into(),
            topic: "hb.L99".into(),
            est_secs: 10,
            fresh: true,
        };
        *SESSION.write() = Session::new(vec![gone]);
        *PHASE.write() = Phase::Running;
    });
    assert_eq!(t.count("article.st-item"), 0);
    t.click_text("button", tx::END);
    assert!(t.text().contains(tx::SUMMARY_TITLE) || t.text().contains(tx::SUMMARY_EMPTY));
}

#[test]
fn the_memory_store_logs_and_takes_back() {
    let mut m = backend::MemoryStore::default();
    let ev = |id: &str| ReviewEvent {
        item_id: id.into(),
        grade: batna_core::review::Grade::Good,
        origin: batna_core::review::Origin::Study,
        at_ms: 1,
        took_ms: 1,
    };
    assert!(m.is_new("a"));
    let first = m.record_logged(ev("a")).unwrap().unwrap();
    m.record(ev("b"));
    assert!(!m.is_new("a") && !m.is_new("b"));
    assert!(m.retract(&first));
    assert!(m.is_new("a"), "taken back");
    assert!(m.retract("never-logged"), "nothing to take back is not a failure");
    assert!(m.due(0).is_empty() && !m.is_hidden("a") && m.next_due_ms(0).is_none());
    assert!(backend::now_ms() > 1_700_000_000_000);
}

#[test]
fn the_player_falls_back_to_memory_without_a_store() {
    fn app() -> Element {
        let store = backend::use_store();
        let n = store.0.borrow().is_new("x");
        rsx! { p { "{n}" } }
    }
    let t = TestDom::new(app);
    assert_eq!(t.texts("p"), ["true"]);
}

/// Diagnostic measurement of the real Study reveal -> Good -> next-card
/// event and VirtualDom mutation path. Run only on the CI runner with
/// \`--ignored --nocapture\`; it is NOT a native Android paint benchmark.
#[test]
#[ignore = "diagnostic: invoke the focused test from CI and print timings, not a timing gate"]
fn profile_study_card_transitions() {
    let ids: Vec<String> = ready_lectures()
        .into_iter()
        .flat_map(|lecture| lecture.items.iter())
        .filter(|item| matches!(&item.body, Body::Card { .. }))
        .take(16)
        .map(|item| item.id.clone())
        .collect();
    assert!(ids.len() >= 8, "need several distinct cards to measure the real transition path");

    let started = std::time::Instant::now();
    let mut t = mount(Some(Start { lectures: None, items: Some(ids.clone()), minutes: 15 }), None);
    let mount_us = started.elapsed().as_micros();
    assert_eq!(t.texts("span.st-count"), [format!("1 / {}", ids.len())]);
    let mut transition_us = Vec::with_capacity(ids.len());

    for (n, _) in ids.iter().enumerate() {
        t.click_text("button", tx::REVEAL);
        assert_eq!(t.count("button.rating"), 4);
        let started = std::time::Instant::now();
        grade(&mut t, "good");
        transition_us.push(started.elapsed().as_micros());
        if n + 1 < ids.len() {
            assert_eq!(t.texts("span.st-count"), [format!("{} / {}", n + 2, ids.len())]);
        }
    }
    assert_eq!(logged(), ids.len(), "each grade must be persisted by the test review store");
    assert!(t.text().contains(tx::SUMMARY_TITLE), "all items should lead to a summary");
    transition_us.sort_unstable();
    let p50 = transition_us[(transition_us.len() - 1) / 2];
    let p95 = transition_us[(transition_us.len() * 95).div_ceil(100) - 1];
    let max = transition_us[transition_us.len() - 1];
    println!(
        "STUDY_TRANSITION_BASELINE cards={} mount_us={} grade_p50_us={} grade_p95_us={} grade_max_us={} scope=TestDom_event_and_mutation_not_Android_paint",
        ids.len(), mount_us, p50, p95, max
    );
}
