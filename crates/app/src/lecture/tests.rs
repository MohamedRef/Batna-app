//! The lecture page and its stack, rendered with the app's review store and
//! driven by taps (`test_dom`).

use super::*;
use crate::learn::{Mark, tests::claim_lecture};
use crate::prefs::DevicePrefs;
use crate::test_dom::{El, Screen as Dom, norm, shows};

#[derive(Props, Clone, PartialEq)]
struct Page {
    id: String,
}

/// The app's frame: Ui on Learn, the review store, the page on the stack.
fn root(p: Page) -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(Screen::Learn),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    crate::store::use_provide_review();
    use_hook(|| {
        crate::courses::focus::set(None);
        open(ui, &p.id)
    });
    let on = shown(Screen::Learn);
    rsx! {
        p { class: "t-screen", "{ui.screen:?}" }
        p { class: "t-shown", "{on:?}" }
        if let Some(id) = on {
            LecturePage { id }
        }
    }
}

fn page(id: &str) -> Dom {
    crate::store::use_memory_store();
    Dom::with_props(root, Page { id: id.to_string() })
}

fn is(class: &'static str) -> impl Fn(&El) -> bool {
    move |e| e.has_class(class)
}

fn screen(d: &Dom) -> String {
    d.find(is("t-screen")).remove(0).text
}

#[test]
fn a_lecture_page_shows_its_pages_lessons_and_ways_to_practise() {
    let (lec, lessons) = claim_lecture();
    let lib = Library::get();
    let entry = lib.index.lectures.iter().find(|e| e.id == lec).unwrap();
    // One lesson done with a score, one done with nothing asked.
    let put = |l: &batna_core::lesson::Lesson, m: Mark| {
        crate::store::shared().put_setting(&format!("lesson.{}", l.id), &m).unwrap();
    };
    put(&lessons[0], Mark { done_ms: 1, right: 1, asked: 2 });
    put(&lessons[1], Mark { done_ms: 1, right: 0, asked: 0 });
    let mut d = page(lec);
    let html = d.html();
    assert!(norm(&d.find(|e| e.tag == "h1").remove(0).text).contains(&norm(&entry.title)));
    let [first, last] = entry.pages;
    assert!(html.contains(&format!("{first}–{last}")), "{html}");
    // Every lesson as a card, the done ones ticked.
    let cards = d.find(is("ls-card"));
    assert_eq!(cards.len(), lessons.len());
    assert_eq!(cards.iter().filter(|c| c.attr("data-done") == Some("yes")).count(), 2);
    assert!(html.contains("✓ 1/2"));
    assert!(d.find(is("ls-done")).iter().any(|c| c.text == "✓"));
    assert!(html.contains(&format!("2 {} {}", tx::OF, lessons.len())));
    if lib.lectures.contains_key(lec) {
        assert!(shows(&d, "progress"));
        // Study this lecture: a session on it, the page closed behind it.
        d.click(|e| e.tag == "button" && e.text.contains(tx::STUDY));
        assert_eq!(screen(&d), "Study");
        let start = d.read(|| crate::study::START.peek().clone()).expect("a session");
        assert_eq!(start.lectures, Some(vec![lec.to_string()]));
        assert!(!d.read(is_open));
    }
}

#[test]
fn a_lesson_card_opens_the_lesson() {
    let (lec, lessons) = claim_lecture();
    let mut d = page(lec);
    d.click(|e| e.has_class("ls-card-btn") && norm(&e.text).contains(&norm(&lessons[1].title)));
    assert_eq!(d.read(|| crate::learn::OPEN.peek().clone()), Some(lessons[1].id.clone()));
}

#[test]
fn practice_ways_lead_to_the_bank_and_cases() {
    let lib = Library::get();
    let with = |kind: &str| {
        lib.index
            .lectures
            .iter()
            .find(|e| lib.lectures.get(&e.id).is_some_and(|l| l.items.iter().any(|i| i.body.kind() == kind)))
            .map(|e| e.id.clone())
    };
    if let Some(lec) = with("mcq") {
        let mut d = page(&lec);
        d.click(|e| e.has_class("lec-way") && e.text.contains(tx::MCQS));
        assert_eq!(screen(&d), "Bank");
    }
    if let Some(lec) = with("case") {
        let mut d = page(&lec);
        d.click(|e| e.has_class("lec-way") && e.text.contains(tx::CASES));
        assert_eq!(screen(&d), "Cases");
    }
    // A topic row starts a session on that topic (or its questions).
    let lec = with("card").expect("a lecture with cards");
    let mut d = page(&lec);
    assert!(d.html().contains(tx::TOPICS));
    d.click(|e| e.has_class("tap-row") && !e.has_class("lec-outline-row"));
    assert!(["Study", "Bank"].contains(&screen(&d).as_str()));
}

#[test]
fn back_pops_the_stack_and_the_crumb_opens_the_course_map() {
    let lec = &Library::get().index.lectures[0].id;
    let mut d = page(lec);
    // The course crumb pushes the map on top.
    d.click_in(is("lec-crumbs"), |e| e.tag == "button");
    assert_eq!(d.find(is("t-shown")).remove(0).text, format!("{:?}", Some(crate::courses::MAP)));
    assert!(d.html().contains(crate::i18n::courses::TITLE));
    // Back from the map returns to the lecture, then closes it.
    d.click(is("lec-back"));
    assert_eq!(d.find(is("t-shown")).remove(0).text, format!("{:?}", Some(lec)));
    d.click(is("lec-back"));
    assert_eq!(d.find(is("t-shown")).remove(0).text, "None");
}

#[test]
fn the_stack_keeps_one_screen_and_never_repeats_the_top() {
    fn app() -> Element {
        let ui = use_context_provider(|| Ui {
            screen: Signal::new(Screen::Learn),
            prefs: Signal::new(DevicePrefs::default()),
            focus: Signal::new(false),
        });
        rsx! { p { "{ui.screen:?}" } }
    }
    let d = Dom::new(app);
    d.read(|| {
        let mut ui = dioxus::core::consume_context_from_scope::<Ui>(ScopeId::APP).unwrap();
        open(ui, "hb.L01");
        open(ui, "hb.L01");
        open(ui, "hb.L02");
        assert_eq!(shown(Screen::Learn).as_deref(), Some("hb.L02"));
        assert_eq!(OPEN.peek().as_ref().unwrap().0.len(), 2, "the same page twice is one entry");
        assert_eq!(shown(Screen::Today), None, "the page belongs to the screen it opened on");
        close();
        assert_eq!(shown(Screen::Learn).as_deref(), Some("hb.L01"));
        // Opened from another screen: a fresh stack there.
        ui.screen.set(Screen::Today);
        open(ui, "hb.L03");
        assert_eq!(OPEN.peek().as_ref().unwrap().0, vec!["hb.L03".to_string()]);
        open_references(ui);
        assert_eq!(shown(Screen::Today).as_deref(), Some(REFERENCES));
        close();
        assert!(!is_open());
        open(ui, "hb.L04");
        close_all();
        assert!(!is_open());
    });
}

#[test]
fn a_lecture_still_being_written_says_so() {
    let lib = Library::get();
    let Some(e) = lib.index.lectures.iter().find(|e| !lib.lectures.contains_key(&e.id)) else { return };
    let d = page(&e.id);
    assert!(d.html().contains(tx::WRITING));
    assert!(!shows(&d, "lec-actions"));
}

#[test]
fn the_references_page_searches_lectures_and_opens_one() {
    let mut d = page(REFERENCES);
    let html = d.html();
    assert!(html.contains(crate::i18n::refs::TITLE));
    let lib = Library::get();
    assert_eq!(d.find(is("refs-row")).len(), lib.index.lectures.len());
    let e = &lib.index.lectures[1];
    d.input(|e| e.tag == "input", &e.title);
    let rows = d.find(is("refs-row"));
    assert!(rows.iter().any(|r| norm(&r.text).contains(&norm(&e.title))));
    assert!(rows.len() < lib.index.lectures.len(), "the search narrows the list");
    d.input(|e| e.tag == "input", "zzqx nothing");
    assert!(d.html().contains(crate::i18n::refs::NO_MATCH));
    d.input(|e| e.tag == "input", "");
    d.click(|r| r.has_class("refs-row") && norm(&r.text).contains(&norm(&e.title)));
    assert_eq!(d.find(is("t-shown")).remove(0).text, format!("{:?}", Some(&e.id)));
    // Back returns to References.
    d.click(is("lec-back"));
    assert_eq!(d.find(is("t-shown")).remove(0).text, format!("{:?}", Some(REFERENCES)));
}

#[test]
fn a_page_left_on_another_screen_closes_and_odd_ids_show_their_pages() {
    fn app() -> Element {
        let ui = use_context_provider(|| Ui {
            screen: Signal::new(Screen::Learn),
            prefs: Signal::new(DevicePrefs::default()),
            focus: Signal::new(false),
        });
        crate::store::use_provide_review();
        use_lecture_page(ui);
        let open = is_open();
        rsx! {
            p { class: "t-open", "{open}" }
            button { class: "t-today", onclick: move |_| { let mut ui = ui; ui.screen.set(Screen::Today) }, "today" }
        }
    }
    crate::store::use_memory_store();
    let mut d = Dom::new(app);
    d.read(|| {
        let ui = dioxus::core::consume_context_from_scope::<Ui>(ScopeId::APP).unwrap();
        open(ui, "hb.L01");
    });
    d.settle();
    assert_eq!(d.find(is("t-open")).remove(0).text, "true");
    // Moving to another screen drops the page stack of the old one.
    d.click(is("t-today"));
    assert_eq!(d.find(is("t-open")).remove(0).text, "false");
    // The abbreviations dictionary sits on the same stack; an unknown id is blank.
    let d = page(crate::abbrdict::PAGE);
    assert!(!shows(&d, "lec-hero"));
    assert!(d.html().len() > 200, "the dictionary renders");
    let d = page("nope.L99");
    assert!(!shows(&d, "lec-page"));
}

#[test]
fn the_book_order_outline_opens_each_pages_sheet() {
    // #831: the lecture page by page; a row opens that page's sheet.
    let lec = "hem.L09";
    let first = Library::get().index.lectures.iter().find(|e| e.id == lec).unwrap().pages[0];
    let mut d = page(lec);
    assert!(d.html().contains(tx::BOOK_ORDER));
    let rows = d.find(is("lec-outline-row"));
    assert!(rows.len() > 5);
    d.click(|e| e.has_class("lec-outline-row"));
    assert_eq!(d.read(crate::pagesheet::open_page), Some(first));
}
