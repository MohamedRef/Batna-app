//! The planner tab as drawn (Dioxus SSR), and the setup card's checks.

use super::*;
use batna_core::content::Lecture;
use batna_core::review::{Grade, Origin, ReviewEvent};
use batna_core::stats::Clock;

const DAY_MS: i64 = 86_400_000;
/// Midday on a fixed day, so every run sees the same plan.
const NOW_MS: i64 = 20_400 * DAY_MS + 12 * 3_600_000;

fn clock() -> Clock {
    Clock { now_ms: NOW_MS, zone: batna_core::zone::Zone::fixed(0), rollover_hour: 4 }
}

fn today() -> Day {
    clock().today()
}

fn lectures(n: usize) -> Vec<Lecture> {
    let lib = crate::content::Library::get();
    lib.index.lectures.iter().filter_map(|e| lib.lectures.get(&e.id).cloned()).take(n).collect()
}

/// Three lectures, with a few answers over the last days.
fn data(n: usize) -> StudyData {
    let lectures = lectures(n);
    let mut events = Vec::new();
    for (i, item) in lectures[0].items.iter().take(4).enumerate() {
        events.push(ReviewEvent {
            confidence: None,
            reason: None,
            item_id: item.id.clone(),
            grade: Grade::Good,
            origin: Origin::Review,
            at_ms: NOW_MS - (i as i64 + 1) * DAY_MS,
            took_ms: 5000,
        });
    }
    StudyData {
        lectures,
        events,
        marks: Default::default(),
        forecast: Some(vec![3, 5, 0, 8]),
        clock: clock(),
        demo: false,
    }
}

#[derive(Props, Clone, PartialEq)]
struct Case {
    settings: PlanSettings,
    n: usize,
}

fn root(case: Case) -> Element {
    use_hook(|| *SETTINGS.write() = case.settings.clone());
    let data = use_hook(|| Rc::new(data(case.n)));
    rsx! {
        crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false,
            Planner { data }
        }
    }
}

fn render(settings: PlanSettings, n: usize) -> String {
    crate::store::use_memory_store();
    let mut dom = VirtualDom::new_with_props(root, Case { settings, n });
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

fn exam_in(days: i32) -> PlanSettings {
    PlanSettings { exam: Some(today().plus(days)), ..PlanSettings::default() }
}

#[test]
fn without_an_exam_day_the_planner_asks_for_one() {
    let page = render(PlanSettings::default(), 3);
    for text in [tx::SETUP_TITLE, tx::EXAM_DAY, tx::DAYS_LABEL, tx::CAP_LABEL, tx::SAVE] {
        assert!(page.contains(text), "missing {text}: {page}");
    }
    for name in tx::WEEKDAYS {
        assert!(page.contains(name), "{name}");
    }
    // Saturday to Thursday on, Friday off.
    assert_eq!(page.matches("aria-pressed=\"true\"").count(), 6, "{page}");
    assert_eq!(page.matches("aria-pressed=\"false\"").count(), 1);
    assert!(page.contains(&format!("min=\"{}\"", Day::iso(today().plus(1)))), "{page}");
    assert!(!page.contains(tx::CANCEL), "nothing to go back to: {page}");
    assert!(!page.contains(tx::EDIT));
}

#[test]
fn a_plan_shows_the_exam_today_the_load_the_pace_and_the_days_ahead() {
    let settings = exam_in(60);
    let page = render(settings.clone(), 3);
    let exam = settings.exam.unwrap();
    assert!(page.contains(&format!("{}: {}", tx::EXAM_ON, long_date(exam))), "{page}");
    for text in [
        tx::TODAY,
        tx::LOAD_TITLE,
        tx::LOAD_KEY_CAP,
        tx::PACE_TITLE,
        tx::PACE_KEY_PLAN,
        tx::PROGRESS,
        tx::FIRST_PASS,
        tx::SECOND_PASS,
        tx::WEEK,
        tx::ALL_DAYS,
        tx::EDIT,
        tx::FINAL_FROM,
    ] {
        assert!(page.contains(text), "missing {text}: {page}");
    }
    assert!(page.contains(&format!("{} ({})", tx::LOAD_KEY_CAP, settings.review_cap)), "{page}");
    assert!(page.matches("<svg").count() >= 2, "the two charts: {page}");
    // Three lectures fit easily in sixty days.
    assert!(page.contains(tx::ON_TRACK) || page.contains(tx::SNUG), "{page}");
    assert!(page.contains(tx::PACE_DONE_ON), "{page}");
    // A study day with new items names its lectures and opens the first.
    let lib = crate::content::Library::get();
    assert!(page.contains(lib.title(&lectures(1)[0].lecture)), "{page}");
    assert!(page.contains(tx::DAY_HINT));
    // The first-pass bar counts what was seen.
    assert!(page.contains(&format!("{}/", 4)) || page.contains("4 "), "{page}");
}

#[test]
fn a_day_off_today_points_to_the_next_study_day() {
    let mut settings = exam_in(30);
    let mut weekdays = [false; 7];
    weekdays[today().plus(1).weekday()] = true;
    settings.weekdays = weekdays;
    let page = render(settings, 3);
    assert!(page.contains(tx::REST_TODAY), "{page}");
    assert!(page.contains(tx::FOCUS_REST), "the week lists rest days: {page}");
}

#[test]
fn too_much_for_the_days_left_says_how_to_fix_it() {
    let page = render(exam_in(14), 40);
    assert!(page.contains(tx::TIGHT), "{page}");
    assert!(page.contains("class=\"note\""), "a tight plan is a note: {page}");
}

#[test]
fn inside_the_final_days_new_work_left_points_to_high_yield() {
    let page = render(exam_in(5), 3);
    assert!(page.contains(tx::TIGHT_FINAL), "{page}");
    assert!(page.contains(tx::FOCUS_MOCK) || page.contains(tx::FOCUS_WEAK), "{page}");
}

#[test]
fn the_setup_checks_the_day_the_weekdays_and_the_cap() {
    let old = PlanSettings::default();
    let all = [true; 7];
    let t = today();
    assert_eq!(edited(&old, t, "", all, 100), Err(tx::PICK_DAY));
    assert_eq!(edited(&old, t, "not a date", all, 100), Err(tx::PICK_DAY));
    assert_eq!(edited(&old, t, &t.iso(), all, 100), Err(tx::PICK_DAY), "today is too soon");
    assert_eq!(edited(&old, t, &t.plus(plan::MAX_DAYS_AHEAD + 1).iso(), all, 100), Err(tx::PICK_DAY_FAR));
    assert_eq!(edited(&old, t, &t.plus(30).iso(), [false; 7], 100), Err(tx::PICK_WEEKDAY));

    let ok = edited(&old, t, &t.plus(30).iso(), all, 5).unwrap();
    assert_eq!(ok.exam, Some(t.plus(30)));
    assert_eq!(ok.weekdays, all);
    assert_eq!(ok.review_cap, 30, "a cap below 30 is raised");
    assert_eq!(ok.final_days, old.final_days, "the rest is kept");
    assert_eq!(edited(&old, t, &t.plus(30).iso(), all, 50_000).unwrap().review_cap, plan::MAX_REVIEW_CAP);
}

#[test]
fn a_session_without_an_exam_follows_the_review_settings() {
    crate::store::use_memory_store();
    fn app() -> Element {
        crate::store::use_provide_review();
        let review = use_context::<crate::store::Review>();
        let backend = review.backend.borrow();
        let now = crate::store::now_ms();
        // No exam day: the settings' own daily cap.
        *SETTINGS.write() = PlanSettings::default();
        assert_eq!(new_left_today(&*backend, now), backend.new_left_today(now));
        assert!(target_for(&*backend, PlanSettings::default()).is_none());
        // With one: the plan's share for today, cached until something changes.
        let settings =
            PlanSettings { exam: Some(Day::from_unix_ms(now, 0, 4).plus(90)), ..PlanSettings::default() };
        *SETTINGS.write() = settings.clone();
        let target = target_now(&*backend).expect("a plan for today");
        assert_eq!(target.exam, settings.exam.unwrap());
        assert_eq!(new_left_today(&*backend, now), target.new_left() as usize);
        assert_eq!(target_for(&*backend, settings), Some(target), "the cached answer");
        rsx! { "ok" }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    assert_eq!(dioxus::ssr::render(&dom), "ok");
}

#[test]
fn days_list_their_lectures_and_reviews() {
    let lib = crate::content::Library::get();
    let ids: Vec<String> = lectures(2).iter().map(|l| l.lecture.clone()).collect();
    let d = DayPlan {
        day: today(),
        focus: Focus::Learn,
        new: 12,
        new_lectures: vec![ids[0].clone()],
        revisit: vec![ids[1].clone()],
        reviews: 3,
    };
    let html = dioxus::ssr::render_element(rsx! { DayDetail { d } });
    assert!(html.contains(&format!("12 {} {}", tx::NEW_ITEMS, tx::FROM)), "{html}");
    assert!(html.contains(lib.title(&ids[0])) && html.contains(lib.title(&ids[1])), "{html}");
    assert!(html.contains(tx::REVISIT) && html.contains(tx::APPROX), "{html}");
    for f in [Focus::Learn, Focus::Revisit, Focus::Mock, Focus::WeakTopics, Focus::Rest] {
        assert!(!focus_label(f).is_empty());
    }
    assert_ne!(focus_label(Focus::Mock), focus_label(Focus::WeakTopics));
}

thread_local! {
    static SEEN: std::cell::Cell<Option<crate::Ui>> = const { std::cell::Cell::new(None) };
}

#[component]
fn Spy() -> Element {
    SEEN.set(Some(use_context::<crate::Ui>()));
    rsx! {}
}

fn pressable(settings: PlanSettings, n: usize) -> crate::tap_dom::TestDom {
    fn app(case: Case) -> Element {
        use_hook(|| *SETTINGS.write() = case.settings.clone());
        let data = use_hook(|| Rc::new(data(case.n)));
        rsx! {
            crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false,
                Spy {}
                Planner { data }
            }
        }
    }
    crate::store::use_memory_store();
    crate::tap_dom::TestDom::new(VirtualDom::new_with_props(app, Case { settings, n }))
}

fn screen(t: &crate::tap_dom::TestDom) -> crate::Screen {
    t.with(|| *SEEN.get().unwrap().screen.peek())
}

#[test]
fn the_setup_card_saves_a_plan_or_says_what_is_missing() {
    let mut t = pressable(PlanSettings::default(), 3);
    t.click(tx::SAVE);
    assert!(t.html().contains(tx::PICK_DAY) && t.html().contains("role=\"alert\""), "{}", t.html());
    let day = today().plus(40).iso();
    t.input("type=\"date\"", &day);
    assert!(!t.html().contains("role=\"alert\""), "typing clears the error");
    // Every weekday off: at least one is needed.
    for name in tx::WEEKDAYS {
        if t.html().contains(&format!("aria-pressed=\"true\">{name}")) {
            t.click(name);
        }
    }
    assert_eq!(t.html().matches("aria-pressed=\"true\"").count(), 0, "{}", t.html());
    t.click(tx::SAVE);
    assert!(t.html().contains(tx::PICK_WEEKDAY), "{}", t.html());
    t.click(tx::WEEKDAYS[0]);
    t.input("type=\"number\"", "not a number");
    t.input("type=\"number\"", "150");
    t.click(tx::SAVE);
    let saved = t.with(|| SETTINGS.peek().clone());
    assert_eq!(saved.exam, Some(today().plus(40)));
    assert_eq!(saved.review_cap, 150);
    assert_eq!(saved.weekdays.iter().filter(|d| **d).count(), 1);
    assert!(t.html().contains(tx::EDIT), "the plan replaces the setup: {}", t.html());

    // Edit opens the setup with Cancel; Cancel goes back unchanged.
    t.click(tx::EDIT);
    assert!(t.html().contains(tx::CANCEL) && t.html().contains(&format!("value=\"{day}\"")), "{}", t.html());
    t.click(tx::CANCEL);
    assert!(t.html().contains(tx::EDIT));
    // What was saved comes back after a sync.
    t.with(|| {
        *SETTINGS.write() = PlanSettings::default();
        reload();
        assert_eq!(*SETTINGS.peek(), saved);
        // Leave the shared test store as other tests expect it.
        save(&PlanSettings::default());
    });
}

#[test]
fn the_plans_buttons_start_a_session_or_open_what_they_show() {
    let mut t = pressable(exam_in(60), 3);
    t.click(tx::REVIEWS);
    assert_eq!(screen(&t), crate::Screen::Study, "today's reviews");
    assert!(t.with(|| crate::study::START.peek().as_ref().is_some_and(|s| s.lectures.is_none())));
    t.with(|| SEEN.get().unwrap().screen.set(crate::Screen::Stats));
    if t.html().contains(&format!("<span class=\"big-label\">{}", tx::NEW_ITEMS)) {
        t.click(&format!("<span class=\"big-label\">{}", tx::NEW_ITEMS));
        assert!(t.with(|| crate::study::START.peek().as_ref().is_some_and(|s| s.lectures.is_some())));
    }
    t.click(tx::PROGRESS_HINT);
    assert_eq!(screen(&t), crate::Screen::Stats);
    t.click(tx::DAY_HINT);
    assert!(t.with(crate::lecture::is_open), "a day opens its first lecture");
    t.with(crate::lecture::close_all);
}
