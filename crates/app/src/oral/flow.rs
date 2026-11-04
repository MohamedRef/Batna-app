//! Oral mode tapped through (`crate::testdom`) on paused tokio time, so the
//! think timer, "And what else?" and auto-continue run through in a test.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use batna_core::review::{Origin, StudyStore};

use super::*;
use crate::Ui;
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
        screen: Signal::new(crate::Screen::Oral),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    UI.with(|u| *u.borrow_mut() = Some(ui));
    let review = use_context_provider(crate::store::test_review);
    REVIEW.with(|r| *r.borrow_mut() = Some(review.clone()));
    use_context_provider(|| StudyStoreHandle(review.backend.clone() as Rc<RefCell<dyn StudyStore>>));
    rsx! {
        Oral {}
        // Study's START is read by the study screen; show it here.
        p { class: "start-asked", if crate::study::START.read().is_some() { "yes" } else { "no" } }
    }
}

/// Paused tokio time for the timers, entered for the whole test.
struct Clock(tokio::runtime::Runtime);

impl Clock {
    fn new() -> Self {
        let rt =
            tokio::runtime::Builder::new_current_thread().enable_time().start_paused(true).build().unwrap();
        Clock(rt)
    }

    /// Lets `secs` of think time pass, a tenth of a second at a time.
    fn pass(&self, t: &mut TestDom, secs: f32) {
        for _ in 0..(secs * 10.0).round() as u32 {
            self.0.block_on(tokio::time::sleep(Duration::from_millis(100)));
            t.settle();
        }
    }
}

fn mount(clock: &Clock, settings: Option<Settings>) -> TestDom {
    let _g = clock.0.enter();
    let store = crate::store::use_thread_store();
    if let Some(s) = settings {
        store.put_local_setting(SETTINGS_KEY, &s).unwrap();
    }
    TestDom::new(Root)
}

fn quiet() -> Option<Settings> {
    Some(Settings { voice: false, ..Settings::default() })
}

fn oral_reviews() -> Vec<batna_core::review::ReviewEvent> {
    REVIEW
        .with(|r| r.borrow().as_ref().unwrap().backend.borrow().log())
        .into_iter()
        .filter(|e| e.origin == Origin::Oral)
        .collect()
}

fn session<T>(t: &mut TestDom, f: impl FnOnce(&Session) -> T) -> T {
    t.in_root(|| f(SESSION.peek().as_ref().expect("a session")))
}

#[test]
fn setup_keeps_his_choices_for_next_time() {
    let clock = Clock::new();
    let _g = clock.0.enter();
    let mut t = mount(&clock, None);
    assert!(t.text().contains(tx::TITLE));
    assert!(t.text().contains(tx::AVAILABLE));
    assert_eq!(t.texts("button[role=radio][aria-checked=true]"), ["10", tx::RATES[1]]);
    t.click_text("button[role=radio]", "20");
    t.input("input[type=range]", "25");
    assert!(t.text().contains(&format!("{}: {}", tx::THINK, count(25u32, &SECOND))));
    t.input("input[type=range]", "nope");
    t.change_nth("input[type=checkbox]", 1, "true");
    // Voice off hides the speed.
    t.change_nth("input[type=checkbox]", 0, "false");
    assert_eq!(t.texts("button[role=radio][aria-checked=true]"), ["20"]);
    // One lecture narrows what can be asked.
    let all = t.texts("p.muted.small")[0].clone();
    let lecture = t.attr("option[value^=hb]", "value");
    let first_lecture = t.in_root(|| pool(&consume_context::<StudyStoreHandle>(), None)[0].lecture.clone());
    t.change_nth("select", 0, &first_lecture);
    assert_ne!(t.texts("p.muted.small")[0], all, "{lecture:?}");
    t.change_nth("select", 0, "");
    assert_eq!(t.texts("p.muted.small")[0], all);
    drop(t);
    let t = TestDom::new(Root);
    assert_eq!(t.texts("button[role=radio][aria-checked=true]"), ["20"]);
    assert!(t.text().contains(&count(25u32, &SECOND)));
    assert_eq!(t.count("input[type=checkbox][checked]"), 1, "auto-continue on, voice off");
}

#[test]
fn saved_settings_out_of_range_come_back_sensible() {
    let clock = Clock::new();
    let _g = clock.0.enter();
    let odd = Settings {
        lecture: Some("hb.L99".into()),
        size: 7,
        think: 99,
        voice: true,
        rate: 40,
        auto_continue: false,
    };
    let t = mount(&clock, Some(odd));
    let s = t.dom.in_scope(ScopeId::APP, || SETTINGS.peek().clone());
    assert_eq!(
        s,
        Settings {
            lecture: None,
            size: 10,
            think: THINK_MAX,
            voice: true,
            rate: tx::RATES.len() - 1,
            auto_continue: false
        }
    );
}

#[test]
fn a_round_asks_reveals_ticks_and_rates_into_review() {
    let clock = Clock::new();
    let _g = clock.0.enter();
    let mut t = mount(&clock, quiet());
    t.click("button.oral-start");
    assert_eq!(t.attr("div.oral-run", "data-phase").as_deref(), Some("ask"));
    clock.pass(&mut t, 0.2);
    // The think timer counts down from 15, and pauses.
    assert_eq!(t.texts("div.oral-ring span"), ["15"]);
    t.click_text("button", tx::PAUSE_TIMER);
    assert_eq!(t.attr("section.oral-timer", "data-paused").as_deref(), Some("yes"));
    clock.pass(&mut t, 3.0);
    assert_eq!(t.texts("div.oral-ring span"), ["15"], "time stands still while paused");
    t.click_text("button", tx::RESUME_TIMER);
    clock.pass(&mut t, 3.0);
    assert_eq!(t.texts("div.oral-ring span"), ["12"]);
    // Restart puts the full time back.
    t.click_text("button", tx::RESTART_TIMER);
    clock.pass(&mut t, 0.2);
    assert_eq!(t.texts("div.oral-ring span"), ["15"]);
    // His own notes, then Show: the answer as a checklist.
    t.input("details.oral-notes textarea", "my point");
    t.click_text("button.oral-big", tx::SHOW);
    clock.pass(&mut t, 0.2);
    assert_eq!(t.attr("div.oral-run", "data-phase").as_deref(), Some("answer"));
    assert!(t.text().contains(tx::YOUR_NOTES) && t.text().contains("my point"));
    let points = t.count("button.oral-point");
    assert!(points >= 1);
    t.click("button.oral-point");
    assert_eq!(t.count("button.oral-point[aria-pressed=true]"), 1);
    assert!(t.texts("p.oral-said")[0].contains(&format!("1 / {points}")));
    t.click("button.oral-point");
    assert_eq!(t.count("button.oral-point[aria-pressed=true]"), 0);
    let first = session(&mut t, |s| s.current.clone().unwrap().prompt.item_id);
    t.click_text("button.oral-big", tx::GOOD);
    let logged = oral_reviews();
    assert_eq!(logged.len(), 1);
    assert_eq!((logged[0].item_id.as_str(), logged[0].grade), (first.as_str(), Grade::Good));
    assert_eq!(session(&mut t, |s| s.asked), 2, "the next question is asked");
    assert_eq!(t.attr("div.oral-run", "data-phase").as_deref(), Some("ask"));
    // Show right after a rating is ignored (a double tap)...
    t.click_text("button.oral-big", tx::SHOW);
    clock.pass(&mut t, 0.2);
    assert_eq!(t.attr("div.oral-run", "data-phase").as_deref(), Some("ask"));
    // ...and works once things settle.
    std::thread::sleep(SETTLE + Duration::from_millis(50));
    t.click_text("button.oral-big", tx::SHOW);
    clock.pass(&mut t, 0.2);
    t.click_text("button.oral-big", tx::AGAIN);
    assert_eq!(oral_reviews()[1].grade, Grade::Again);
    // End: the summary counts one good, one again, and lists the miss.
    t.click("button.oral-end");
    assert!(t.text().contains(tx::DONE));
    assert_eq!(t.texts("dl.oral-tally dd"), ["1", "1"]);
    assert_eq!(t.count("ul.oral-missed li"), 1);
    // The miss opens in a study session (or its lecture's page).
    t.click("ul.oral-missed button");
    let ui = UI.with(|u| u.borrow().unwrap());
    let screen = t.in_root(|| *ui.screen.peek());
    let started = t.texts("p.start-asked") == ["yes"];
    assert!((screen == crate::Screen::Study && started) || t.in_root(crate::lecture::is_open), "{screen:?}");
    // Another round, then back to the setup.
    t.click_text("button.oral-big", tx::ANOTHER);
    assert_eq!(t.count("div.oral-run"), 1);
    t.click("button.oral-end");
    t.click_text("button.oral-small", tx::BACK);
    assert!(t.text().contains(tx::AVAILABLE));
}

#[test]
fn the_timer_runs_to_the_answer_and_auto_continue_moves_on() {
    let clock = Clock::new();
    let _g = clock.0.enter();
    let mut t = mount(
        &clock,
        Some(Settings { voice: false, auto_continue: true, think: 10, size: 10, ..Settings::default() }),
    );
    t.click("button.oral-start");
    // Think time, then "what else" for a list, then the answer.
    let mut saw_what_else = false;
    for _ in 0..250 {
        clock.pass(&mut t, 0.1);
        saw_what_else |= t.text().contains(tx::WHAT_ELSE);
        if t.attr("div.oral-run", "data-phase").as_deref() == Some("answer") {
            break;
        }
    }
    assert_eq!(t.attr("div.oral-run", "data-phase").as_deref(), Some("answer"));
    let what_else = session(&mut t, |s| s.current.as_ref().unwrap().prompt.what_else);
    assert_eq!(saw_what_else, what_else);
    // Nobody rates it: after a while it counts as heard and the next one comes.
    clock.pass(&mut t, 9.0);
    assert_eq!(session(&mut t, |s| (s.tally.heard, s.asked)), (1, 2));
    assert!(oral_reviews().is_empty(), "heard is not a rating");
    t.click("button.oral-end");
    assert!(t.text().contains(tx::DONE_HEARD));
    assert!(t.text().contains(tx::NONE_MISSED));
}

#[test]
fn with_the_voice_on_text_still_shows() {
    let clock = Clock::new();
    let _g = clock.0.enter();
    let mut t = mount(&clock, Some(Settings::default()));
    t.click("button.oral-start");
    clock.pass(&mut t, 1.0);
    assert!(t.count(".oral-question p") >= 1);
    t.click_text("button.oral-small", tx::REPEAT);
    clock.pass(&mut t, 0.5);
    assert_eq!(t.count("div.oral-run"), 1);
}

#[test]
fn a_follow_up_and_a_repeat_say_so() {
    let entry = |follow_up, again| {
        let item = crate::study::flow::first("card");
        Entry { prompt: session::Prompt::from_item("hb.L01", item).unwrap(), follow_up, again }
    };
    assert!(spoken_question(&entry(true, false)).starts_with("Following on from that. "));
    assert!(spoken_question(&entry(false, true)).starts_with("Once more. "));
    let plain = entry(false, false);
    assert_eq!(spoken_question(&plain), plain.prompt.spoken_question());
}
