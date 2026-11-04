//! The Cases screen tapped the way he uses it (`test_dom`): the tabs, the
//! list, and a case played to its recap, with the review in the store.

use super::*;
use crate::prefs::DevicePrefs;
use crate::test_dom::{El, Screen as Dom, norm, shows};
use crate::{Screen, Ui};
use batna_core::review::StudyStoreHandle;

fn is(class: &'static str) -> impl Fn(&El) -> bool {
    move |e| e.has_class(class)
}

fn app() -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(Screen::Cases),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    crate::store::use_provide_review();
    rsx! {
        p { class: "t-screen", "{ui.screen:?}" }
        CasesScreen {}
    }
}

fn screen() -> Dom {
    crate::store::use_memory_store();
    Dom::new(app)
}

/// A case whose steps all have options, one with a step he grades himself.
fn case_where(pick: impl Fn(&[batna_core::content::CaseStep]) -> bool) -> Option<&'static Item> {
    items_in_library("case").into_iter().map(|(_, i)| i).find(|i| match &i.body {
        Body::Case { steps, .. } => pick(steps),
        _ => false,
    })
}

fn tab(label: &'static str) -> impl Fn(&El) -> bool {
    move |e| e.attr("role") == Some("tab") && e.text == label
}

fn first_words(s: &str) -> String {
    norm(&s.chars().take(60).collect::<String>())
}

#[test]
fn the_tabs_switch_lists_and_a_row_opens_its_case() {
    let mut d = screen();
    assert_eq!(d.find(|e| e.attr("role") == Some("tab")).len(), 3);
    assert_eq!(d.find(tab(tx::TAB_CASES)).remove(0).attr("aria-selected"), Some("true"));
    let cases = items_in_library("case").len();
    assert!(cases > 0);
    assert_eq!(d.find(is("case-row")).len(), cases);
    // Each row: its stem's first line and the number of steps.
    let (_, first) = items_in_library("case")[0];
    let Body::Case { steps, stem } = &first.body else { unreachable!() };
    assert!(d.html().contains(&tx::steps(steps.len())));
    // The number exercises.
    d.click(tab(tx::TAB_CALC_ITEMS));
    assert_eq!(d.find(tab(tx::TAB_CALC_ITEMS)).remove(0).attr("aria-selected"), Some("true"));
    let calcs = items_in_library("calc").len();
    if calcs == 0 {
        assert!(d.html().contains(tx::EMPTY_CALC));
    } else {
        assert_eq!(d.find(is("case-row")).len(), calcs);
    }
    d.click(tab(tx::TAB_CALCULATORS));
    assert!(d.find(is("case-row")).is_empty());
    // Back to cases; the first row opens its player.
    d.click(tab(tx::TAB_CASES));
    let stem_start = first_words(stem);
    d.click(|e| {
        e.has_class("case-row") && first_words(&e.text).starts_with(&stem_start[..stem_start.len().min(20)])
    });
    assert!(shows(&d, "case-stepper"), "{}", d.html());
    assert!(
        norm(&d.find(is("case-stem")).remove(0).text).starts_with(&stem_start[..stem_start.len().min(20)])
    );
    // ← back to the list keeps a half-done case.
    d.click_in(is("player-meta"), |e| e.tag == "button");
    assert!(shows(&d, "case-list"));
}

#[test]
fn a_case_is_played_step_by_step_and_recorded_once() {
    let item = case_where(|s| s.len() >= 2 && s.iter().all(|s| s.options.len() >= 2 && s.answer.is_some()))
        .expect("a case with options on every step");
    let Body::Case { steps, .. } = &item.body else { unreachable!() };
    let n = steps.len();
    let mut d = screen();
    d.read(|| *OPEN.write() = Some(item.id.clone()));
    d.settle();
    // Nothing picked: the hint and a disabled Check.
    assert!(d.html().contains(tx::PICK_FIRST));
    assert!(d.find(is("case-check")).remove(0).disabled());
    let option = |d: &mut Dom, k: usize, i: usize| {
        let t = norm(&steps[k].options[i]);
        d.click(move |e| e.has_class("case-option") && norm(&e.text).ends_with(&t));
    };
    #[allow(clippy::needless_range_loop)] // k indexes the step and its options
    for k in 0..n {
        let answer = steps[k].answer.unwrap();
        // The first step wrong, the rest right.
        let pick = if k == 0 { (answer + 1) % steps[k].options.len() } else { answer };
        option(&mut d, k, pick);
        assert_eq!(d.find(|e| e.has_class("case-option") && e.attr("aria-pressed") == Some("true")).len(), 1);
        d.click(is("case-check"));
        let html = d.html();
        assert!(html.contains(if k == 0 { tx::WRONG } else { tx::RIGHT }), "{html}");
        assert!(shows(&d, "case-why"));
        assert!(d.find(is("case-option")).iter().all(El::disabled), "a checked step is locked");
        let label = if k + 1 == n { tx::FINISH } else { tx::NEXT };
        d.click(|e| e.tag == "button" && e.text == label);
        if k + 1 < n {
            // The stepper marks the step done, ✗ for the miss.
            let done = d.find(|e| {
                e.has_class("case-stepper-step")
                    && e.attr("data-state") == Some(if k == 0 { "miss" } else { "ok" })
            });
            assert!(!done.is_empty());
        }
    }
    // The recap: score, message, each step's reasoning.
    let summary = d.find(is("case-summary")).remove(0);
    assert!(summary.text.contains(&format!("{} / {n}", n - 1)));
    let msg = match grade_for((n - 1) as u32, n as u32) {
        Grade::Good | Grade::Easy => tx::ALL_RIGHT,
        Grade::Hard => tx::SOME_RIGHT,
        Grade::Again => tx::FEW_RIGHT,
    };
    assert!(summary.text.contains(msg));
    assert!(summary.text.contains(tx::CORRECT_WAS), "the missed step shows the right answer");
    assert_eq!(d.find_in(is("case-steps-done"), |e| e.tag == "li").len(), n);
    // One review in the store, from the case.
    let reviewed = d.read(|| {
        let s = dioxus::core::consume_context_from_scope::<StudyStoreHandle>(ScopeId::APP).unwrap();
        !s.0.borrow().is_new(&item.id)
    });
    assert!(reviewed);
    // Again starts a fresh run; Back returns to the list and forgets it.
    d.click(|e| e.tag == "button" && e.text == tx::AGAIN);
    assert!(d.html().contains(tx::PICK_FIRST));
    assert_eq!(d.read(|| RUN.peek().as_ref().map(|r| r.results.len())), Some(0));
    d.click_in(is("player-meta"), |e| e.tag == "button");
    assert!(shows(&d, "case-list"));
}

#[test]
fn a_step_without_options_is_shown_and_graded_by_him() {
    let Some(item) = case_where(|s| s.iter().any(|s| s.options.is_empty())) else { return };
    let Body::Case { steps, .. } = &item.body else { unreachable!() };
    let k = steps.iter().position(|s| s.options.is_empty()).unwrap();
    let mut d = screen();
    d.read(|| {
        *OPEN.write() = Some(item.id.clone());
        // Earlier steps answered right, as he would have.
        let mut r = CaseRun::new(&item.id);
        for s in &steps[..k] {
            if s.options.is_empty() {
                r.grade_self(true);
            } else {
                r.select(s.answer.unwrap_or(0));
                r.check(s.answer);
            }
            r.next();
        }
        *RUN.write() = Some(r);
    });
    d.settle();
    d.click(|e| e.tag == "button" && e.text == tx::SHOW_ANSWER);
    assert!(shows(&d, "case-why"));
    d.click(|e| e.tag == "button" && e.text == tx::MISSED);
    assert_eq!(d.read(|| RUN.peek().as_ref().unwrap().results[k]), (false, None));
    let label = if k + 1 == steps.len() { tx::FINISH } else { tx::NEXT };
    d.click(|e| e.tag == "button" && e.text == label);
    if k + 1 == steps.len() {
        assert!(d.html().contains(tx::MISSED), "the recap says he missed it");
    } else {
        assert!(d.html().contains(tx::MISSED));
        assert_eq!(d.read(|| RUN.peek().as_ref().unwrap().step), k + 1);
    }
}

#[test]
fn today_and_more_open_the_right_tab() {
    fn opener() -> Element {
        let ui = use_context_provider(|| Ui {
            screen: Signal::new(Screen::Today),
            prefs: Signal::new(DevicePrefs::default()),
            focus: Signal::new(false),
        });
        crate::store::use_provide_review();
        let on = *ui.screen.read();
        rsx! {
            p { class: "t-screen", "{on:?}" }
            button { class: "t-due", onclick: move |_| open_due(ui, true), "due" }
            button { class: "t-calc", onclick: move |_| open_calculators(ui), "calc" }
            if on == Screen::Cases { CasesScreen {} }
        }
    }
    crate::store::use_memory_store();
    let mut d = Dom::new(opener);
    d.click(is("t-due"));
    assert_eq!(d.find(is("t-screen")).remove(0).text, "Cases");
    assert_eq!(d.find(tab(tx::TAB_CALC_ITEMS)).remove(0).attr("aria-selected"), Some("true"));
    let mut d = Dom::new(opener);
    d.click(is("t-calc"));
    assert_eq!(d.find(tab(tx::TAB_CALCULATORS)).remove(0).attr("aria-selected"), Some("true"));
}

#[test]
fn list_rows_cut_long_stems() {
    let (_, item) = items_in_library("case")
        .into_iter()
        .find(|(_, i)| match &i.body {
            Body::Case { stem, .. } => stem.chars().count() > 120,
            _ => false,
        })
        .unwrap_or(items_in_library("case")[0]);
    let line = first_line(item);
    assert!(line.chars().count() <= 121);
    if let Body::Case { stem, .. } = &item.body {
        assert_eq!(line.ends_with('…'), stem.chars().count() > 120);
    }
    assert_eq!(find_item(&item.id).map(|(_, i)| i.id.clone()), Some(item.id.clone()));
    assert!(find_item("nope.L01.001").is_none());
}

#[test]
fn the_stepper_and_the_recap_lead_back() {
    let item = case_where(|s| !s.is_empty() && s.iter().all(|s| s.options.len() >= 2 && s.answer.is_some()))
        .expect("a case with options on every step");
    let Body::Case { steps, .. } = &item.body else { unreachable!() };
    let mut d = screen();
    d.read(|| *OPEN.write() = Some(item.id.clone()));
    d.settle();
    let meta = d.find(is("player-meta")).remove(0);
    assert_eq!(meta.text.contains(tx::UNVERIFIED), item.status == Status::Unverified);
    // The current step's stepper button says "now" and stays on the case.
    let now =
        d.find(|e| e.has_class("case-stepper-step") && e.attr("aria-current") == Some("step")).remove(0);
    assert!(now.text.contains(tx::NOW));
    d.click(|e| e.has_class("case-stepper-step"));
    assert!(shows(&d, "case-stepper"));
    // Every step right, to the recap.
    for (k, s) in steps.iter().enumerate() {
        d.read(|| {
            if let Some(r) = RUN.write().as_mut() {
                r.select(s.answer.unwrap());
            }
        });
        d.settle();
        d.click(is("case-check"));
        let label = if k + 1 == steps.len() { tx::FINISH } else { tx::NEXT };
        d.click(|e| e.tag == "button" && e.text == label);
    }
    assert!(d.html().contains(tx::ALL_RIGHT));
    // The recap's Back forgets the finished case.
    d.click_in(is("case-summary"), |e| e.tag == "button" && e.text == tx::BACK);
    assert!(shows(&d, "case-list"));
    assert_eq!(d.read(|| RUN.peek().clone()), None);
}

#[test]
fn an_unknown_case_shows_nothing() {
    let mut d = screen();
    d.read(|| *OPEN.write() = Some("nope.L01.001".into()));
    d.settle();
    assert!(!shows(&d, "case-stepper"));
    assert!(!shows(&d, "case-list"));
}
