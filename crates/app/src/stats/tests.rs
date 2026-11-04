//! The progress screen as drawn (Dioxus SSR): its three tabs, the period
//! head, the lecture cards and the mistakes log.

use std::rc::Rc;

use batna_core::content::Body;
use batna_core::review::{Grade, Origin};

use super::*;

const DAY_MS: i64 = 86_400_000;
const NOW_MS: i64 = 20_400 * DAY_MS + 12 * 3_600_000;

fn lectures() -> Vec<Lecture> {
    let lib = Library::get();
    lib.index.lectures.iter().filter_map(|e| lib.lectures.get(&e.id).cloned()).collect()
}

fn clock() -> Clock {
    Clock { now_ms: NOW_MS, zone: batna_core::zone::Zone::fixed(0), rollover_hour: ROLLOVER_HOUR }
}

/// The demo's few weeks of answers on the first three lectures.
fn studied() -> StudyData {
    let lectures = lectures();
    let events = demo::events(&lectures, NOW_MS);
    StudyData { lectures, events, marks: demo::marks(), forecast: None, clock: clock(), demo: true }
}

fn fresh() -> StudyData {
    StudyData {
        lectures: lectures(),
        events: Vec::new(),
        marks: Default::default(),
        forecast: Some(vec![2, 4]),
        clock: clock(),
        demo: false,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Progress,
    Mistakes,
    Month,
}

#[derive(Props, Clone, PartialEq)]
struct Case {
    view: View,
    studied: bool,
}

fn root(case: Case) -> Element {
    use_context_provider(crate::settings::SettingsNav::new);
    let data = use_hook(|| Rc::new(if case.studied { studied() } else { fresh() }));
    use_hook(|| {
        *period::SPAN.write() =
            if case.view == View::Month { period::Span::Month } else { period::Span::Week }
    });
    rsx! {
        crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false,
            match case.view {
                View::Mistakes => rsx! { mistakes::Mistakes { data } },
                _ => rsx! { Progress { data } },
            }
        }
    }
}

fn render(view: View, studied: bool) -> String {
    let mut dom = VirtualDom::new_with_props(root, Case { view, studied });
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

#[test]
fn before_any_study_the_progress_tab_says_so_kindly() {
    let page = render(View::Progress, false);
    for text in [tx::SUMMARY, tx::SEEN, tx::SEEN_TWICE, tx::APPROVED, tx::NOTHING_YET, tx::WEAK_EMPTY] {
        assert!(page.contains(text), "missing {text}: {page}");
    }
    assert!(page.contains(tx::RECALL_NONE) && page.contains(tx::ACTIVITY_EMPTY), "{page}");
    // Every lecture of the library has its card, under its course.
    assert_eq!(page.matches("class=\"stat-lecture\"").count(), lectures().len(), "{page}");
    assert!(!page.contains(tx::WEAK_START), "no session from nothing");
}

#[test]
fn after_some_weeks_the_numbers_the_weak_topics_and_the_bars_fill_in() {
    let data = studied();
    let rows = data.stats();
    let seen: u32 = rows.iter().map(|l| l.counts.seen).sum();
    let total: u32 = rows.iter().map(|l| l.counts.total).sum();
    let page = render(View::Progress, true);
    assert!(page.contains(&format!("{seen} {} {total} {}", tx::OF, tx::SEEN)), "the seen count: {page}");
    assert!(!page.contains(tx::NOTHING_YET));
    assert!(page.contains(tx::WEAK_TITLE) && page.contains(tx::WEAK_START), "{page}");
    assert!(page.contains(tx::SESSION_COUNT), "{page}");
    // A week of bars, named by weekday, today last.
    assert_eq!(page.matches("class=\"st-bar\"").count(), 7, "{page}");
    assert_eq!(page.matches("data-today=true").count(), 1, "{page}");
    assert!(crate::i18n::plan::WEEKDAYS_SHORT.iter().any(|d| page.contains(d)));
    assert!(page.contains(tx::RECALL_SUB), "{page}");
    // The first lecture's topics table, with its retention.
    assert!(page.contains(tx::TOPICS) && page.contains(tx::OPEN_LECTURE), "{page}");
    assert!(page.contains('%'), "{page}");
    assert!(page.contains("data-empty=\"no\""), "studied lectures have bars: {page}");
}

#[test]
fn the_month_view_labels_its_ends() {
    let page = render(View::Month, true);
    assert_eq!(page.matches("class=\"st-bar\"").count(), 30, "{page}");
    assert!(page.contains(tx::MONTH_START) && page.contains(tx::TODAY), "{page}");
    assert!(page.contains("data-span=\"month\""));
}

#[test]
fn bars_show_a_little_for_any_day_with_answers() {
    assert_eq!(period::bar_height(0, 0, 10), 2);
    assert_eq!(period::bar_height(0, 3, 10), 6);
    assert_eq!(period::bar_height(5, 3, 10), 50);
    assert_eq!(period::bar_height(10, 3, 10), 100);
    assert_eq!(period::bar_height(4, 1, 0), 400, "no peak: the minutes themselves");
}

#[test]
fn the_mistakes_log_groups_open_mistakes_and_hides_cloze_answers() {
    let page = render(View::Mistakes, false);
    assert!(page.contains(tx::MISTAKES_TITLE) && page.contains(tx::MISTAKES_EMPTY), "{page}");
    assert!(!page.contains(tx::MISTAKES_START));

    let data = studied();
    let groups = stats::mistakes(&data.lectures, &data.events, &data.marks, data.clock.now_ms);
    assert!(!groups.is_empty(), "the demo has misses");
    let page = render(View::Mistakes, true);
    assert!(page.contains(tx::MISTAKES_START) && page.contains(tx::MISSED), "{page}");
    assert_eq!(page.matches("class=\"card mistake-group\"").count(), groups.len(), "{page}");
    let first = &groups[0].items[0];
    let item = data.lectures.iter().flat_map(|l| &l.items).find(|i| i.id == first.item_id).unwrap();
    let shown = mistakes::preview(item);
    assert!(!shown.contains("{{"), "{shown}");
}

#[test]
fn a_preview_is_the_question_cut_to_two_lines() {
    let lib = Library::get();
    let all: Vec<_> = lib.lectures.values().flat_map(|l| &l.items).collect();
    for kind in ["card", "cloze", "mcq", "enumerate", "table", "case", "calc", "essay", "order"] {
        let Some(item) = all.iter().find(|i| i.body.kind() == kind) else { continue };
        let p = mistakes::preview(item);
        assert!(!p.is_empty(), "{kind}");
        assert!(p.chars().count() <= 141, "{kind}: {p}");
        if let Body::Cloze { .. } = item.body {
            assert!(p.contains("[…]"), "{p}");
        }
    }
    let long =
        all.iter().find(|i| matches!(&i.body, Body::Card { front, .. } if front.chars().count() > 140));
    if let Some(item) = long {
        assert!(mistakes::preview(item).ends_with('…'));
    }
}

#[test]
fn the_screen_opens_on_the_tab_asked_for() {
    crate::store::use_memory_store();
    #[derive(Props, Clone, PartialEq)]
    struct Want {
        tab: Option<Tab>,
    }
    fn app(w: Want) -> Element {
        crate::store::use_provide_review();
        use_context_provider(crate::settings::SettingsNav::new);
        use_hook(|| *WANT.write() = w.tab);
        // No exam day, whatever another test saved in the shared store.
        use_hook(|| *crate::plan::SETTINGS.write() = Default::default());
        rsx! { crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false, Stats {} } }
    }
    for (tab, selected, shows) in [
        (None, tx::TAB_PROGRESS, tx::SUMMARY),
        (Some(Tab::Mistakes), tx::TAB_MISTAKES, tx::MISTAKES_TITLE),
        (Some(Tab::Plan), tx::TAB_PLAN, crate::i18n::plan::SETUP_TITLE),
    ] {
        let mut dom = VirtualDom::new_with_props(app, Want { tab });
        dom.rebuild_in_place();
        let page = dioxus::ssr::render(&dom);
        assert!(page.contains(&format!("aria-selected=\"true\">{selected}")), "{tab:?}: {page}");
        assert!(page.contains(shows), "{tab:?}: {page}");
        assert!(page.contains(tx::HEAD));
    }
}

#[test]
fn the_forecast_reads_the_scheduler_or_falls_back_to_a_rough_one() {
    let f = fresh();
    assert_eq!(f.due_forecast(4), vec![2, 4, 0, 0], "padded with zeros");
    let demo = studied();
    assert_eq!(demo.due_forecast(5).len(), 5);
    assert!(demo.due_forecast(30).iter().sum::<u32>() > 0, "studied items come back");
    // A wrong answer and a right one later: the item still counts as seen.
    let mut d = fresh();
    let item = d.lectures[0].items[0].id.clone();
    for (back, grade) in [(3, Grade::Again), (1, Grade::Good)] {
        d.events.push(batna_core::review::ReviewEvent {
            item_id: item.clone(),
            grade,
            origin: Origin::Review,
            at_ms: NOW_MS - back * DAY_MS,
            took_ms: 1000,
        });
    }
    assert_eq!(d.stats()[0].counts.seen, 1);
}

thread_local! {
    static SEEN: std::cell::Cell<Option<(crate::Ui, crate::settings::SettingsNav)>> =
        const { std::cell::Cell::new(None) };
}

#[component]
fn Spy() -> Element {
    SEEN.set(Some((use_context::<crate::Ui>(), use_context::<crate::settings::SettingsNav>())));
    rsx! {}
}

fn pressable(studied_data: bool) -> crate::tap_dom::TestDom {
    #[derive(Props, Clone, PartialEq)]
    struct P {
        studied: bool,
    }
    fn app(p: P) -> Element {
        use_context_provider(crate::settings::SettingsNav::new);
        let data = use_hook(|| Rc::new(if p.studied { studied() } else { fresh() }));
        use_hook(|| *period::SPAN.write() = period::Span::Week);
        rsx! {
            crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false,
                Spy {}
                Progress { data }
            }
        }
    }
    crate::store::use_memory_store();
    crate::tap_dom::TestDom::new(VirtualDom::new_with_props(app, P { studied: studied_data }))
}

fn moved(t: &crate::tap_dom::TestDom) -> crate::Screen {
    t.with(|| {
        let (mut ui, nav) = SEEN.get().unwrap();
        let at = *ui.screen.peek();
        ui.screen.set(crate::Screen::Stats);
        nav.close();
        at
    })
}

#[test]
fn the_progress_numbers_open_study_and_the_checks() {
    let mut t = pressable(true);
    t.click(&format!("{}. {}", tx::SEEN, tx::STUDY_HINT));
    assert_eq!(moved(&t), crate::Screen::Study);
    t.click(&format!("{}. {}", tx::APPROVED, tx::APPROVED_HINT));
    assert!(t.with(|| *SEEN.get().unwrap().1.open.peek()), "Settings › Data");
    moved(&t);
    t.click(tx::WEAK_START);
    assert_eq!(moved(&t), crate::Screen::Study, "a session on the weak topics");
    assert!(t.with(|| crate::study::START.peek().as_ref().is_some_and(|s| s.items.is_some())));
    t.click(&format!("title=\"{}\"", tx::TOPIC_HINT));
    assert_ne!(moved(&t), crate::Screen::Stats, "a topic opens its practice");
    t.click(tx::OPEN_LECTURE);
    assert!(t.with(crate::lecture::is_open));
    t.with(crate::lecture::close_all);
}

#[test]
fn the_tabs_switch_on_tap() {
    crate::store::use_memory_store();
    fn app() -> Element {
        crate::store::use_provide_review();
        use_context_provider(crate::settings::SettingsNav::new);
        use_hook(|| *crate::plan::SETTINGS.write() = Default::default());
        rsx! {
            crate::browser_fixture::Shell { screen: crate::Screen::Stats, focus: false,
                Spy {}
                Stats {}
            }
        }
    }
    let mut t = crate::tap_dom::TestDom::of(app);
    t.click(&format!(">{}<", tx::TAB_MISTAKES));
    assert!(t.html().contains(tx::MISTAKES_TITLE), "{}", t.html());
    t.click(&format!(">{}<", tx::TAB_PLAN));
    assert!(t.html().contains(crate::i18n::plan::SETUP_TITLE));
    t.click(&format!(">{}<", tx::TAB_PROGRESS));
    assert!(t.html().contains(tx::SUMMARY));
    // Asked for a tab while already here.
    t.with(|| open(SEEN.get().unwrap().0, Tab::Mistakes));
    assert_eq!(moved(&t), crate::Screen::Stats);
}
