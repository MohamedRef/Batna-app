//! Today as drawn (Dioxus SSR): the screen on a fresh store, the hero with
//! and without an exam plan, and each focus of the plan's day.

use batna_core::plan::{DayPlan, PlanSettings};
use batna_core::review::{Grade, Origin, ReviewEvent};
use batna_core::stats::Clock;

use super::*;

fn shell(children: Element) -> Element {
    crate::store::use_provide_review();
    use_context_provider(crate::settings::SettingsNav::new);
    rsx! { crate::browser_fixture::Shell { screen: Screen::Today, focus: false, {children} } }
}

fn render(app: fn() -> Element) -> (String, VirtualDom) {
    crate::store::use_memory_store();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    (dioxus::ssr::render(&dom), dom)
}

#[test]
fn today_offers_a_session_the_counts_and_the_whole_library() {
    let (page, dom) = render(|| {
        use_hook(|| *crate::plan::SETTINGS.write() = PlanSettings::default());
        shell(rsx! { Today {} })
    });
    let lib = Library::get();
    for text in [tx::EYEBROW, tx::TIME_Q, tx::DUE, tx::NEW, tx::CONTINUE_HEAD, tx::LIBRARY, tx::MORE_HEAD] {
        assert!(page.contains(text), "missing {text}: {page}");
    }
    assert!(page.contains(tx::QUICK_BANK) && page.contains(tx::CHECKED_ROW), "{page}");
    assert!(page.contains(tx::UNVERIFIED_NOTE), "{page}");
    // One radio per session length, the chosen one checked.
    let chosen = dom.in_runtime(crate::onboarding::session_minutes);
    assert_eq!(page.matches("class=\"td-chip\"").count(), crate::onboarding::MINUTE_CHOICES.len(), "{page}");
    assert!(page.contains(&tx::minutes_short(chosen)));
    // Every lecture in the index has its row.
    assert_eq!(page.matches("class=\"tap tap-row lecture-row\"").count(), lib.index.lectures.len(), "{page}");
    assert!(page.contains(tx::READY) || page.contains(tx::WRITING));
    // A fresh store has new items waiting: the hero starts a session.
    assert!(page.contains(tx::READY_Q) || page.contains(tx::DONE_Q), "{page}");
    assert!(page.contains(tx::HERO_KICKER_NEW) || page.contains(tx::HERO_KICKER), "{page}");
}

fn target(focus: Focus) -> DailyTarget {
    const DAY_MS: i64 = 86_400_000;
    let lib = Library::get();
    let lectures: Vec<_> =
        lib.index.lectures.iter().filter_map(|e| lib.lectures.get(&e.id).cloned()).take(3).collect();
    let now_ms = 20_400 * DAY_MS + 12 * 3_600_000;
    let clock = Clock { now_ms, zone: batna_core::zone::Zone::fixed(0), rollover_hour: 4 };
    let events = vec![ReviewEvent {
        item_id: lectures[0].items[0].id.clone(),
        grade: Grade::Good,
        origin: Origin::Review,
        at_ms: now_ms - 3_600_000,
        took_ms: 4000,
    }];
    let data = StudyData {
        lectures,
        events,
        marks: Default::default(),
        forecast: Some(vec![4; 60]),
        clock,
        demo: false,
    };
    let settings = PlanSettings { exam: Some(clock.today().plus(60)), ..PlanSettings::default() };
    let mut t = crate::plan::daily_target(&data, &settings).expect("a plan");
    let day: &mut DayPlan = &mut t.plan.days[0];
    day.focus = focus;
    t
}

#[derive(Props, Clone, PartialEq)]
struct HeroCase {
    t: Option<DailyTarget>,
    due: usize,
    new_today: usize,
    unseen: usize,
    nothing: bool,
}

fn hero(c: HeroCase) -> Element {
    shell(rsx! {
        Hero { target: c.t.clone(), due: c.due, new_today: c.new_today, unseen: c.unseen, due_minutes: 7, due_tomorrow: 9, nothing: c.nothing, minutes: 15 }
    })
}

fn render_hero(c: HeroCase) -> String {
    crate::store::use_memory_store();
    let mut dom = VirtualDom::new_with_props(hero, c);
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

#[test]
fn the_hero_says_what_today_holds() {
    let base = HeroCase { t: None, due: 3, new_today: 5, unseen: 50, nothing: false };
    let page = render_hero(base.clone());
    assert!(page.contains(&tx::headline(3, 5)) && page.contains(&tx::hero_body(3, 5)), "{page}");
    assert!(page.contains(&tx::start_session(15)) && page.contains(tx::HERO_KICKER), "{page}");

    // All done for today, more new items waiting behind the daily cap.
    let page = render_hero(HeroCase { due: 0, new_today: 0, ..base.clone() });
    assert!(page.contains(tx::ALL_DONE) && page.contains(tx::NEW_CAP_REACHED), "{page}");
    assert!(page.contains(&format!("{} 9", tx::TOMORROW)), "{page}");
    assert!(page.contains(tx::OPEN_NEXT_LESSON) && page.contains(tx::HERO_KICKER_NEW), "{page}");
    // Nothing unseen left: just tomorrow.
    let page = render_hero(HeroCase { due: 0, new_today: 0, unseen: 0, ..base.clone() });
    assert!(!page.contains(tx::NEW_CAP_REACHED) && page.contains(tx::TOMORROW), "{page}");
    // Nothing written for him yet.
    let page = render_hero(HeroCase { due: 0, new_today: 0, unseen: 0, nothing: true, ..base });
    assert!(page.contains(tx::NOTHING_WRITTEN) && !page.contains(tx::ALL_DONE), "{page}");
}

#[test]
fn with_an_exam_plan_the_hero_follows_the_plans_day() {
    let t = target(Focus::Learn);
    let goal = t.new_goal();
    assert!(goal > 0, "the plan gives new items on a learning day");
    let page = render_hero(HeroCase {
        t: Some(t.clone()),
        due: 4,
        new_today: goal as usize,
        unseen: 100,
        nothing: false,
    });
    assert!(page.contains(tx::HERO_KICKER_PLAN) && page.contains(&tx::exam_in(t.days_left)), "{page}");
    assert!(page.contains(tx::PLAN_TODAY) && page.contains(&tx::plan_line(goal, 4)), "{page}");
    assert!(page.contains("role=\"progressbar\"") && page.contains(tx::ABOUT), "{page}");

    // Only new items, no reviews due.
    let page =
        render_hero(HeroCase { t: Some(t.clone()), due: 0, new_today: 1, unseen: 100, nothing: false });
    assert!(page.contains(&format!("{goal} {}", tx::PLAN_NEW)), "{page}");

    // Today's new items done, reviews still due.
    let mut done = t.clone();
    done.done.new = goal;
    let page =
        render_hero(HeroCase { t: Some(done.clone()), due: 2, new_today: 0, unseen: 100, nothing: false });
    assert!(page.contains(tx::NEW_DONE), "{page}");
    // Some done, some left.
    let mut some = t.clone();
    some.done.new = 1;
    let page = render_hero(HeroCase { t: Some(some), due: 2, new_today: 1, unseen: 100, nothing: false });
    assert!(page.contains(tx::NEW_LEFT_1) && page.contains(tx::NEW_LEFT_2), "{page}");
    // Everything done with a plan: a "get ahead" session.
    let page = render_hero(HeroCase { t: Some(done), due: 0, new_today: 0, unseen: 30, nothing: false });
    assert!(page.contains(tx::AHEAD) && page.contains(tx::ALL_DONE), "{page}");
}

#[test]
fn mock_weak_and_rest_days_each_say_what_to_do() {
    let page = render_hero(HeroCase {
        t: Some(target(Focus::Mock)),
        due: 0,
        new_today: 0,
        unseen: 9,
        nothing: false,
    });
    assert!(page.contains(tx::MOCK) && page.contains(tx::OPEN_BANK), "{page}");
    let page = render_hero(HeroCase {
        t: Some(target(Focus::WeakTopics)),
        due: 1,
        new_today: 0,
        unseen: 9,
        nothing: false,
    });
    assert!(page.contains(tx::WEAK) && page.contains(tx::WEAK_START), "{page}");
    let page = render_hero(HeroCase {
        t: Some(target(Focus::Rest)),
        due: 2,
        new_today: 0,
        unseen: 9,
        nothing: false,
    });
    assert!(page.contains(tx::REST) && page.contains(tx::REST_DUE), "{page}");
    let mut revisit = target(Focus::Revisit);
    revisit.plan.days[0].revisit =
        vec![revisit.plan.days[0].new_lectures.first().cloned().unwrap_or_else(|| "hb.L01".into())];
    let page = render_hero(HeroCase { t: Some(revisit), due: 2, new_today: 1, unseen: 9, nothing: false });
    assert!(page.contains(tx::REVISIT), "{page}");
}

#[test]
fn a_tight_plan_adds_one_calm_note() {
    for (fix, line) in [
        (TightFix::MoreDays(2), Some(tx::tight_days(2))),
        (TightFix::HigherCap(220), Some(tx::tight_cap(220))),
        (TightFix::HighYield, Some(tx::TIGHT_HIGH_YIELD.to_string())),
        (TightFix::FinalWindow, None),
    ] {
        let mut t = target(Focus::Learn);
        t.plan.status = PlanStatus::Tight { left_over: 40, fix };
        let page = render_hero(HeroCase { t: Some(t), due: 1, new_today: 1, unseen: 9, nothing: false });
        match line {
            Some(l) => assert!(page.contains(&l) && page.contains("class=\"note small\""), "{fix:?}: {page}"),
            None => {
                assert!(!page.contains("class=\"note small\""), "the final window says it already: {page}")
            }
        }
    }
}

#[test]
fn a_lecture_in_progress_shows_how_far_it_is() {
    let lib = Library::get();
    let id = lib.index.lectures[0].id.clone();
    #[derive(Props, Clone, PartialEq)]
    struct P {
        l: LectureProgress,
    }
    fn app(p: P) -> Element {
        shell(rsx! { ul { Progress { lecture: p.l.clone() } } })
    }
    crate::store::use_memory_store();
    let l = LectureProgress { id: id.clone(), title: lib.title(&id).to_string(), seen: 5, total: 20, due: 3 };
    let mut dom = VirtualDom::new_with_props(app, P { l });
    dom.rebuild_in_place();
    let page = dioxus::ssr::render(&dom);
    assert!(page.contains("inline-size: 25%") && page.contains("aria-valuenow=\"5\""), "{page}");
    assert!(page.contains(&format!("3 {}", tx::DUE_CHIP)), "{page}");
    assert!(page.contains(&format!("5 {} 20 {}", tx::OF, tx::STARTED)), "{page}");
}

#[test]
fn due_cases_point_to_the_cases_screen() {
    let lib = Library::get();
    let ids: Vec<String> = lib
        .lectures
        .values()
        .flat_map(|l| &l.items)
        .filter(|i| i.body.kind() == "case")
        .take(2)
        .map(|i| i.id.clone())
        .collect();
    #[derive(Props, Clone, PartialEq)]
    struct P {
        ids: Vec<String>,
    }
    fn app(p: P) -> Element {
        shell(rsx! { DueCases { ids: p.ids.clone() } })
    }
    crate::store::use_memory_store();
    let n = ids.len();
    let mut dom = VirtualDom::new_with_props(app, P { ids });
    dom.rebuild_in_place();
    let page = dioxus::ssr::render(&dom);
    assert!(page.contains(&tx::due_cases(n)), "{page}");
}

#[test]
fn review_settings_offer_retention_and_new_items_unless_the_plan_sets_them() {
    use crate::i18n::study_settings as st;
    let (page, _) = render(|| {
        use_hook(|| *crate::plan::SETTINGS.write() = PlanSettings::default());
        shell(rsx! { StudySection {} })
    });
    for text in [st::RETENTION, st::R85, st::R90, st::R95, st::NEW_PER_DAY] {
        assert!(page.contains(text), "missing {text}: {page}");
    }
    assert!(page.contains("90%") && page.contains("95%"), "{page}");
    // One retention and one new-per-day choice are checked.
    assert_eq!(page.matches("aria-checked=true").count(), 2, "{page}");
    assert!(!page.contains(st::NEW_FROM_PLAN_1), "{page}");

    let (page, _) = render(|| {
        use_hook(|| {
            let today = batna_core::plan::Day::from_unix_ms(crate::store::now_ms(), 0, 4);
            *crate::plan::SETTINGS.write() =
                PlanSettings { exam: Some(today.plus(90)), ..PlanSettings::default() };
        });
        shell(rsx! { StudySection {} })
    });
    assert!(page.contains(st::NEW_FROM_PLAN_1) && page.contains(st::NEW_FROM_PLAN_2), "{page}");
    assert_eq!(page.matches("aria-checked=true").count(), 1, "only retention is a choice: {page}");
}

thread_local! {
    static SEEN: std::cell::Cell<Option<Ui>> = const { std::cell::Cell::new(None) };
    static NAV: std::cell::Cell<Option<crate::settings::SettingsNav>> = const { std::cell::Cell::new(None) };
}

/// Remembers the shell's navigation so a test can see where a tap went.
#[component]
fn Spy() -> Element {
    let ui = use_context::<Ui>();
    let nav = use_context::<crate::settings::SettingsNav>();
    SEEN.set(Some(ui));
    NAV.set(Some(nav));
    rsx! {}
}

fn screen(t: &crate::tap_dom::TestDom) -> Screen {
    t.with(|| *SEEN.get().expect("the spy ran").screen.peek())
}

/// Back on Today, sheets closed, before the next tap.
fn home(t: &crate::tap_dom::TestDom) {
    t.with(|| {
        let mut ui = SEEN.get().unwrap();
        ui.screen.set(Screen::Today);
        NAV.get().unwrap().close();
    });
}

#[test]
fn every_row_on_today_opens_what_it_names() {
    use crate::tap_dom::TestDom;
    crate::store::use_memory_store();
    let mut t = TestDom::of(|| {
        use_hook(|| *crate::plan::SETTINGS.write() = PlanSettings::default());
        shell(rsx! {
            Spy {}
            Today {}
        })
    });
    // A session length is a choice, not a move.
    let other = crate::onboarding::MINUTE_CHOICES[0];
    t.click(&tx::minutes_short(other));
    assert_eq!(t.with(crate::onboarding::session_minutes), other);
    assert_eq!(screen(&t), Screen::Today);

    for (button, to) in [
        (tx::start_session(other), Screen::Study),
        (format!("{}.", tx::NEW), Screen::Study),
        (tx::ALL_LESSONS.to_string(), Screen::Learn),
        (tx::QUICK_BANK.to_string(), Screen::Bank),
    ] {
        home(&t);
        t.click(&button);
        assert_eq!(screen(&t), to, "{button}");
    }
    for button in [tx::CHECKED_ROW, tx::UNVERIFIED_NOTE] {
        home(&t);
        t.click(button);
        assert!(t.with(|| *NAV.get().unwrap().open.peek()), "{button} opens Settings › Data");
    }
    home(&t);
    t.click("class=\"tap tap-row lecture-row\"");
    assert!(t.with(crate::lecture::is_open), "a lecture row opens its lecture");
    t.with(crate::lecture::close_all);
}

#[test]
fn the_plan_hero_buttons_open_the_plan_and_the_bank() {
    use crate::tap_dom::TestDom;
    crate::store::use_memory_store();
    #[derive(Props, Clone, PartialEq)]
    struct P {
        t: DailyTarget,
    }
    fn app(p: P) -> Element {
        shell(rsx! {
            Spy {}
            Hero { target: Some(p.t.clone()), due: 2, new_today: 0, unseen: 30, due_minutes: 7, due_tomorrow: 9, nothing: false, minutes: 15 }
        })
    }
    let mock = target(Focus::Mock);
    let days = mock.days_left;
    let mut t = TestDom::new(VirtualDom::new_with_props(app, P { t: mock }));
    t.click(tx::OPEN_BANK);
    assert_eq!(screen(&t), Screen::Bank);
    home(&t);
    t.click(&tx::exam_in(days));
    assert_eq!(screen(&t), Screen::Stats, "the pill opens the plan");
    home(&t);
    t.click(&format!("aria-label=\"{}\"", tx::PLAN_HINT));
    assert_eq!(screen(&t), Screen::Stats, "so does the ring");
    // The card itself starts today's session.
    home(&t);
    t.click("class=\"btn btn-lg btn-on-hero td-go\"");
    assert_eq!(screen(&t), Screen::Study);
}

#[test]
fn with_nothing_ready_the_hero_opens_the_next_lesson() {
    use crate::tap_dom::TestDom;
    crate::store::use_memory_store();
    let mut t = TestDom::of(|| {
        shell(rsx! {
            Spy {}
            Hero { target: None, due: 0, new_today: 0, unseen: 0, due_minutes: 0, due_tomorrow: 3, nothing: false, minutes: 15 }
        })
    });
    t.click(tx::OPEN_NEXT_LESSON);
    assert_ne!(screen(&t), Screen::Today, "a lesson or the Lessons screen");
    assert_ne!(screen(&t), Screen::Study);
}

#[test]
fn due_cases_open_the_cases_and_a_progress_row_its_lecture() {
    use crate::tap_dom::TestDom;
    crate::store::use_memory_store();
    let mut t = TestDom::of(|| {
        let lib = Library::get();
        let ids: Vec<String> = lib
            .lectures
            .values()
            .flat_map(|l| &l.items)
            .filter(|i| i.body.kind() == "case")
            .take(1)
            .map(|i| i.id.clone())
            .collect();
        let id = lib.index.lectures[0].id.clone();
        let l =
            LectureProgress { id: id.clone(), title: lib.title(&id).to_string(), seen: 5, total: 20, due: 3 };
        shell(rsx! {
            Spy {}
            DueCases { ids }
            ul { Progress { lecture: l } }
        })
    });
    t.click(&tx::due_cases(1));
    assert_eq!(screen(&t), Screen::Cases);
    home(&t);
    t.click("class=\"tap tap-row progress-row\"");
    assert!(t.with(crate::lecture::is_open));
    t.with(crate::lecture::close_all);
}

#[test]
fn wide_desktop_today_keeps_study_primary_and_untraps_scrolling() {
    let css = include_str!("today.css");
    for required in [
        "@media (min-width: 1200px) and (min-height: 740px)",
        ".today-cols.tab-cols[data-wide=\"aside\"]",
        "grid-template-columns: minmax(0, 3fr) minmax(18rem, 2fr)",
        ".today-cols > .tab-main.tab-sticky",
        "max-block-size: none",
        ".today-cols > .tab-aside",
        "overflow-y: auto",
    ] {
        assert!(css.contains(required), "desktop task-first layout needs {required}");
    }
}

#[test]
fn resume_button_returns_to_existing_study_without_queuing_a_new_start() {
    use crate::tap_dom::TestDom;
    crate::store::use_memory_store();
    let mut t = TestDom::of(|| {
        shell(rsx! {
            Spy {}
            ResumeStudy { index: 2, total: 7 }
        })
    });
    assert!(t.html().contains(tx::RESUME_TITLE), "resume is clearly labeled");
    assert!(t.html().contains("2 / 7"), "progress is visible");
    assert_eq!(screen(&t), Screen::Today);
    t.click(tx::RESUME_SESSION);
    assert_eq!(screen(&t), Screen::Study);
    assert!(t.with(|| crate::study::START.peek().is_none()), "resume must not create a new session request");
}
