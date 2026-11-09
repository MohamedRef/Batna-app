//! Adapted from Steps-app main: crates/app/src/weak/tests.rs.
use std::cell::RefCell;

use batna_core::review::Grade::{Again, Good};
use batna_core::review::{Grade, Origin, ReviewEvent, StudyStore as _};

use super::*;
use crate::Ui;
use crate::prefs::DevicePrefs;

thread_local! {
    static SEEN: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

/// Two hepatology cards on one page and a question on that page, from the
/// library as it is.
fn page_items() -> (u32, String, String, Option<String>) {
    let lib = Library::get();
    let items = &lib.lectures["hb.L01"].items;
    let playable = |i: &&Item| crate::study::PLAYABLE.contains(&i.body.kind()) && i.src.page > 0;
    let first = items.iter().find(playable).unwrap();
    let page = first.src.page;
    let other = items.iter().filter(playable).find(|i| i.src.page != page).unwrap();
    let mcq = items.iter().find(|i| i.src.page == page && i.body.kind() == "mcq").map(|i| i.id.clone());
    (page, first.id.clone(), other.id.clone(), mcq)
}

#[derive(Props, Clone, PartialEq)]
struct With {
    answers: Vec<(String, Grade)>,
    /// What to draw: "probe", "step", "sheet" or "settings".
    show: &'static str,
}

#[allow(non_snake_case)]
fn Host(p: With) -> Element {
    use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Learn),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    let review = use_context_provider(|| {
        let review = crate::store::test_review();
        for (k, (id, grade)) in p.answers.iter().enumerate() {
            review.backend.borrow_mut().record(ReviewEvent {
                item_id: id.clone(),
                grade: *grade,
                origin: Origin::Study,
                at_ms: 1_793_610_000_000 + k as i64 * 60_000,
                took_ms: 5_000,
            });
        }
        review
    });
    use_context_provider(|| {
        batna_core::review::StudyStoreHandle(
            review.backend.clone() as std::rc::Rc<RefCell<dyn batna_core::review::StudyStore>>
        )
    });
    SEEN.set(review.backend.borrow().weak());
    let (page, ..) = page_items();
    use_hook(move || {
        *SHOW.write() = true;
        if p.show == "sheet" {
            crate::pagesheet::open(page, "hb.L01", None, false, None);
        }
    });
    match p.show {
        "step" => rsx! { StepMark { of: "hb.L01.lesson1#0", page } },
        "sheet" => rsx! { crate::pagesheet::BookSheet {} },
        "settings" => rsx! { WeakSection {} },
        _ => rsx! {},
    }
}

fn render(show: &'static str, answers: &[(&str, Grade)]) -> String {
    crate::store::use_thread_store();
    let answers = answers.iter().map(|(id, g)| (id.to_string(), *g)).collect();
    let mut dom = VirtualDom::new_with_props(Host, With { answers, show });
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

fn weak_after(answers: &[(&str, Grade)]) -> BTreeSet<String> {
    render("probe", answers);
    SEEN.take()
}

#[test]
fn a_miss_marks_and_the_next_right_answer_clears() {
    let (_, a, b, _) = page_items();
    assert!(weak_after(&[]).is_empty());
    assert_eq!(weak_after(&[(&a, Again)]), BTreeSet::from([a.clone()]));
    assert!(weak_after(&[(&a, Again), (&a, Good)]).is_empty());
    assert_eq!(
        weak_after(&[(&a, Good), (&b, Again), (&a, Again)]),
        BTreeSet::from([a.clone(), b.clone()]),
        "each item keeps its own last answer"
    );
}

#[test]
fn only_the_books_own_page_counts() {
    let lib = Library::get();
    let (page, a, b, _) = page_items();
    let weak = BTreeSet::from([a.clone(), b.clone()]);
    let ids = |scope: Scope, page| {
        on_page(lib, &weak, &scope, page).iter().map(|i| i.id.clone()).collect::<Vec<_>>()
    };
    assert_eq!(ids(Scope::Book("hb".into()), page), std::slice::from_ref(&a));
    // Gastroenterology's page with the same number is another page.
    assert!(ids(Scope::Book("git".into()), page).is_empty());
    assert!(ids(Scope::Slides("res.L02".into()), page).is_empty());
}

#[test]
fn a_lesson_step_is_marked_only_after_a_miss_on_its_page() {
    let (_, a, b, _) = page_items();
    assert!(!render("step", &[]).contains("weak-mark"));
    assert!(!render("step", &[(&b, Again)]).contains("weak-mark"), "another page");
    let html = render("step", &[(&a, Again)]);
    assert!(html.contains("weak-mark"), "{html}");
    assert!(html.contains(tx::MARK) && html.contains(tx::WHY));
    assert!(!render("step", &[(&a, Again), (&a, Good)]).contains("weak-mark"));
}

#[test]
fn the_page_sheet_lists_the_facts_to_go_over_again() {
    let (_, a, _, mcq) = page_items();
    let mut answers = vec![(a.as_str(), Again)];
    if let Some(q) = &mcq {
        answers.push((q.as_str(), Again));
    }
    let html = render("sheet", &answers);
    assert!(html.contains("bk-weak"), "{html}");
    assert_eq!(html.matches(r#"class="weak-fact""#).count(), 1, "the card, not the MCQ");
    if mcq.is_some() {
        assert!(html.contains(&tx::questions(1)));
    }
    assert!(html.contains(&tx::study(1)));
    assert!(!render("sheet", &[]).contains("bk-weak"));
}

#[test]
fn the_setting_shows_its_toggle() {
    let html = render("settings", &[]);
    assert!(html.contains(tx::SETTING_TITLE) && html.contains(r#"type="checkbox""#), "{html}");
}
