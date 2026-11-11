//! The lesson player and the lessons list, driven the way he uses them: a
//! real lesson from the library opened at a step, its buttons tapped
//! (`test_dom`), and the HTML and the saved attempt checked after each tap.

use std::collections::HashSet;
use std::sync::Mutex;

use batna_core::lesson::{Lesson, StepBody};
use batna_core::response::{Attempt, Draft, sig};
use dioxus::prelude::*;

use super::*;
use crate::prefs::DevicePrefs;
use crate::test_dom::{El, Screen as Dom, norm, shows};

/// Lessons already taken by a test: tests run in parallel on one shared
/// store, so each one works on lessons nobody else touches.
static CLAIMED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// A lesson nobody else uses, with the first step that `pick` accepts.
fn claim(pick: impl Fn(&Lesson, usize, &StepBody) -> bool) -> (&'static Lesson, usize) {
    crate::store::use_memory_store();
    let mut claimed = CLAIMED.lock().unwrap();
    let taken = claimed.get_or_insert_with(super::browser::fixture_lessons);
    for (_, l) in Library::get().all_lessons() {
        if taken.contains(&l.id)
            || crate::store::shared().setting::<Mark>(&format!("{PREFIX}{}", l.id)).is_some()
        {
            continue;
        }
        if let Some(at) = l.steps.iter().enumerate().position(|(k, s)| pick(l, k, &s.body)) {
            taken.insert(l.id.clone());
            return (l, at);
        }
    }
    panic!("no free lesson has such a step");
}

/// A step with nothing to answer: Continue is on at once.
fn reading(b: &StepBody) -> bool {
    matches!(b.kind(), "explain" | "aside" | "figure" | "update")
}

fn kind(k: &'static str) -> impl Fn(&Lesson, usize, &StepBody) -> bool {
    move |_, _, b| b.kind() == k
}

fn saved(id: &str) -> Option<Attempt> {
    crate::store::shared().place(&player::pos_key(id))
}

#[derive(Props, Clone, PartialEq)]
struct Open {
    lesson: Option<String>,
    at: usize,
    #[props(default)]
    view: View,
    /// A response already on the step, and its first result.
    #[props(default)]
    draft: Option<(Draft, Option<bool>)>,
    /// Only the first result, no saved response (an older app's record).
    #[props(default)]
    answered: Option<bool>,
    /// Where the lesson was opened from.
    #[props(default = crate::Screen::Today)]
    from: crate::Screen,
}

/// The app's frame around Learn: the Ui and review contexts, the lesson's
/// saved attempt, and a line naming the screen in view.
fn root(o: Open) -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Learn),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    crate::store::use_provide_review();
    use_hook(|| {
        // Every course in view, whatever another test left in the store.
        crate::courses::focus::set(None);
        if let Some(id) = &o.lesson {
            let (_, lesson) = Library::get().lesson(id).expect("lesson");
            let mut a = Attempt { at: o.at, ..Attempt::new(lesson.steps.len(), crate::store::now_ms()) };
            if let Some((draft, answer)) = o.draft.clone() {
                a.put(o.at, sig(&lesson.steps[o.at].body), draft);
                a.answers[o.at] = answer;
            }
            if let Some(right) = o.answered {
                a.answers[o.at] = Some(right);
            }
            crate::store::shared().put_place(&player::pos_key(id), &a).unwrap();
        }
        *FROM.write() = o.from;
        *OPEN.write() = o.lesson.clone();
        *VIEW.write() = o.view;
    });
    rsx! {
        p { class: "t-screen", "{ui.screen:?}" }
        Learn {}
    }
}

fn open(lesson: &Lesson, at: usize) -> Dom {
    Dom::with_props(
        root,
        Open {
            lesson: Some(lesson.id.clone()),
            at,
            view: View::Steps,
            draft: None,
            answered: None,
            from: crate::Screen::Today,
        },
    )
}

fn screen_is(d: &Dom, s: &str) -> bool {
    d.find(|e| e.has_class("t-screen")).first().is_some_and(|e| e.text == s)
}

fn count(d: &Dom) -> String {
    d.find(|e| e.has_class("ls-count")).first().map(|e| e.text.clone()).unwrap_or_default()
}

fn next(e: &El) -> bool {
    e.has_class("ls-next")
}

fn is(class: &'static str) -> impl Fn(&El) -> bool {
    move |e| e.has_class(class)
}

/// The visible answer buttons of the current step, in screen order.
fn answers(d: &Dom) -> Vec<El> {
    d.find(|e| e.tag == "button" && e.has_class("answer"))
}

#[test]
fn continue_back_and_close_walk_the_lesson_and_save_the_place() {
    // A lesson whose first two steps are reading, not questions.
    let (l, _) = claim(|l, k, _| {
        k == 0 && l.steps.len() > 2 && reading(&l.steps[0].body) && reading(&l.steps[1].body)
    });
    let n = l.steps.len();
    let mut d = open(l, 0);
    assert_eq!(count(&d), format!("1/{n}"));
    let html = d.html();
    assert!(html.contains("data-view=\"steps\""), "{html}");
    assert!(
        d.find(|e| e.has_class("ls-context-title"))
            .first()
            .is_some_and(|e| e.text.contains(l.title.split_whitespace().next().unwrap()))
    );
    // Back is off on the first step; Continue moves on and is saved.
    assert!(d.find(|e| e.attr("data-nav") == Some("back")).first().unwrap().disabled());
    d.click(next);
    assert_eq!(count(&d), format!("2/{n}"));
    assert_eq!(saved(&l.id).unwrap().at, 1);
    assert_eq!(d.read(|| resume_step(&l.id)), Some(1));
    assert_eq!(d.find(|e| e.attr("data-state") == Some("now") && e.has_class("ls-seg")).len(), 1);
    assert!(d.html().contains("data-dir=\"fwd\""));
    // Back returns to the first step, which stayed mounted.
    d.click(|e| e.attr("data-nav") == Some("back"));
    assert_eq!(count(&d), "1/".to_string() + &n.to_string());
    assert!(d.html().contains("data-dir=\"back\""));
    assert_eq!(saved(&l.id).unwrap().at, 0);
    // × closes the player and goes back to where it was opened (Today).
    d.click(is("ls-close"));
    assert!(screen_is(&d, "Today"), "{}", d.html());
    assert!(!shows(&d, "ls-player"));
}

#[test]
fn finishing_a_lesson_marks_it_done_and_shows_the_recap() {
    // Every lesson ends on its cards (a Try step); skipping them finishes.
    let (l, at) = claim(|l, k, _| k + 1 == l.steps.len() && !l.practice.is_empty());
    let mut d = open(l, at);
    let n = l.steps.len();
    assert_eq!(count(&d), format!("{n}/{n}"));
    let finish = d.find(next).remove(0);
    assert!(finish.text.contains(tx::FINISH), "{}", finish.text);
    d.click(is("ls-skip"));
    let html = d.html();
    assert!(html.contains(tx::DONE_TITLE), "{html}");
    assert!(shows(&d, "ls-summary"));
    let m = d.read(|| mark(&l.id)).expect("the lesson is marked done");
    assert!(m.done_ms > 0 && m.right <= m.asked);
    // The position is cleared, so the list shows it done, not part-way.
    assert_eq!(d.read(|| resume_step(&l.id)), None);
    assert!(matches!(d.read(|| state(l, None)), State::Done(_)));
    for r in &l.recap {
        let first = r.split_whitespace().find(|w| w.chars().all(char::is_alphanumeric)).unwrap_or("");
        assert!(html.contains(first), "recap line {r} missing");
    }
    // Practice hands the lesson's items to a study session.
    let practise = d.find(|e| e.tag == "button" && e.text.contains(tx::PRACTICE)).remove(0);
    assert!(practise.text.contains(&l.practice.len().to_string()));
    d.click(|e| e.tag == "button" && e.text.contains(tx::PRACTICE));
    assert!(screen_is(&d, "Study"), "{}", d.html());
    let items = d.read(|| crate::study::START.peek().as_ref().and_then(|s| s.items.clone()));
    assert_eq!(items, Some(l.practice.clone()));
    assert_eq!(d.read(|| OPEN.peek().clone()), None, "the lesson closes behind the session");
}

#[test]
fn a_finish_the_store_refuses_stays_on_the_last_step_until_it_saves() {
    let (l, at) = claim(|l, k, _| k + 1 == l.steps.len() && !l.practice.is_empty());
    let mut d = open(l, at);
    let n = l.steps.len();
    crate::store::shared().set_read_only(true);
    d.click(is("ls-skip"));
    crate::store::shared().set_read_only(false);
    assert!(d.html().contains(tx::FINISH_UNSAVED), "{}", d.html());
    assert!(!shows(&d, "ls-summary"), "not done yet");
    assert_eq!(count(&d), format!("{n}/{n}"));
    assert!(d.read(|| mark(&l.id)).is_none(), "no mark without its cleared place");
    // Retry finishes it: the mark and the cleared place together.
    d.click(is("save-retry"));
    assert!(shows(&d, "ls-summary"), "{}", d.html());
    assert!(d.read(|| mark(&l.id)).is_some());
    assert_eq!(d.read(|| resume_step(&l.id)), None);
}

#[test]
fn a_question_gates_continue_until_answered_or_skipped() {
    let (l, at) = claim(kind("check"));
    let mut d = open(l, at);
    let html = d.html();
    // Continue is off, with Skip as the visible way on and a hint for
    // screen readers.
    assert!(d.find(next).remove(0).disabled(), "{html}");
    assert!(html.contains("aria-describedby=\"ls-gate-hint\""));
    assert!(html.contains(tx::NAV_REQUIRED));
    d.click(is("ls-skip"));
    let a = saved(&l.id).unwrap();
    assert!(a.skipped[at], "the skip is saved");
    assert_eq!(a.answers[at], None, "a skip is not an answer");
    if at + 1 < l.steps.len() {
        assert_eq!(count(&d), format!("{}/{}", at + 2, l.steps.len()));
        // Coming back to a skipped question: it can still be answered.
        d.click(|e| e.attr("data-nav") == Some("back"));
        assert_eq!(answers(&d).iter().filter(|b| !b.disabled()).count(), answers(&d).len());
    }
}

#[test]
fn a_check_step_picks_checks_explains_and_retries_without_rescoring() {
    let (l, at) = claim(kind("check"));
    let StepBody::Check { options, answer, why, .. } = &l.steps[at].body else { unreachable!() };
    let wrong = (answer + 1) % options.len();
    let mut d = open(l, at);
    assert_eq!(answers(&d).len(), options.len());
    assert!(d.html().contains("role=\"radiogroup\""));
    // Pick the wrong one: it is selected and saved, not checked.
    let pick = |d: &mut Dom, i: usize| {
        let t = norm(&options[i]);
        d.click(move |e| e.has_class("answer") && e.tag == "button" && norm(&e.text).ends_with(&t));
    };
    pick(&mut d, wrong);
    assert_eq!(answers(&d).iter().filter(|b| b.attr("data-state") == Some("selected")).count(), 1);
    assert_eq!(saved(&l.id).unwrap().drafts[at].draft, Draft::Check { pick: Some(wrong), checked: false });
    // The footer's primary is now "check it".
    let check = d.find(|e| e.attr("data-nav") == Some("check")).remove(0);
    assert!(check.text.contains(tx::CHECK_IT));
    d.click(|e| e.attr("data-nav") == Some("check"));
    let html = d.html();
    assert!(html.contains(tx::NOT_QUITE), "{html}");
    assert!(html.contains("role=\"list\""));
    assert!(html.contains(tx::MARK_MINE_WRONG) && html.contains(tx::MARK_RIGHT));
    assert!(norm(&d.find(is("ls-check")).remove(0).text).contains(&norm(&why[*answer])));
    assert!(answers(&d).iter().all(El::disabled), "checked options are locked");
    let a = saved(&l.id).unwrap();
    assert_eq!(a.answers[at], Some(false));
    assert_eq!(a.drafts[at].draft, Draft::Check { pick: Some(wrong), checked: true });
    // Continue is on now.
    assert!(!d.find(next).remove(0).disabled());
    // ↻ starts the step over; the right answer then counts as practice only.
    d.click(is("ls-retry"));
    assert!(!d.html().contains(tx::NOT_QUITE));
    assert_eq!(saved(&l.id).unwrap().drafts[at].draft, Draft::None);
    pick(&mut d, *answer);
    d.click(|e| e.attr("data-nav") == Some("check"));
    assert!(d.html().contains(tx::RIGHT));
    assert_eq!(saved(&l.id).unwrap().answers[at], Some(false), "only the first try counts");
}

#[test]
fn an_order_step_places_undoes_and_grades_the_order() {
    let (l, at) = claim(kind("order"));
    let StepBody::Order { steps, .. } = &l.steps[at].body else { unreachable!() };
    let n = steps.len();
    let mut d = open(l, at);
    assert!(d.html().contains(&tx::placed_of(0, n)));
    let tile = |s: &str| {
        let s = s.to_string();
        move |e: &El| e.has_class("ls-tile") && norm(&e.text) == norm(&s)
    };
    // Undo is off until something is placed; Undo and a tap on a slot take
    // steps back out.
    assert!(d.find(is("ls-undo")).remove(0).disabled());
    d.click(tile(&steps[1]));
    assert!(d.html().contains(&tx::placed_of(1, n)));
    d.click(is("ls-undo"));
    assert!(d.html().contains(&tx::placed_of(0, n)));
    d.click(tile(&steps[1]));
    d.click(is("ls-slot"));
    assert_eq!(saved(&l.id).unwrap().drafts[at].draft, Draft::Order { placed: vec![], checked: false });
    // Placed backwards: the first and last slots are wrong.
    for s in steps.iter().rev() {
        d.click(tile(s));
    }
    assert!(!shows(&d, "ls-pool"), "the pool empties when all are placed");
    d.click(|e| e.attr("data-nav") == Some("check"));
    let html = d.html();
    let wrong = (0..n).filter(|&i| i != n - 1 - i).count();
    assert!(html.contains(&tx::in_place(n - wrong, n)), "{html}");
    assert!(html.contains("ls-key"), "the right order is shown");
    assert_eq!(saved(&l.id).unwrap().answers[at], Some(false));
    // ↻ and the right order.
    d.click(is("ls-retry"));
    for s in steps {
        d.click(tile(s));
    }
    d.click(|e| e.attr("data-nav") == Some("check"));
    assert!(d.html().contains(tx::RIGHT));
    assert_eq!(d.find(|e| e.attr("data-state") == Some("right")).len(), n);
}

#[test]
fn a_match_step_counts_wrong_taps() {
    let (l, at) = claim(kind("match"));
    let StepBody::Match { pairs, .. } = &l.steps[at].body else { unreachable!() };
    let n = pairs.len();
    let mut d = open(l, at);
    assert!(d.html().contains(tx::MATCH_HINT));
    let left = |i: usize| {
        let t = pairs[i][0].clone();
        move |e: &El| e.has_class("ls-tile") && norm(&e.text) == norm(&t) && !e.disabled()
    };
    let right = |i: usize| {
        let t = pairs[i][1].clone();
        move |e: &El| e.has_class("ls-tile") && norm(&e.text) == norm(&t)
    };
    // The right column waits for a left pick.
    assert!(d.find(right(0)).iter().all(El::disabled));
    d.click(left(0));
    d.click(right(1 % n));
    if n > 1 {
        assert_eq!(d.find(|e| e.attr("data-state") == Some("wrong")).len(), 1, "the wrong tap flashes");
    }
    for i in 0..n {
        d.click(left(i));
        d.click(right(i));
    }
    let html = d.html();
    let misses = usize::from(n > 1);
    assert!(html.contains(&if misses == 0 { tx::RIGHT.to_string() } else { tx::misses(misses) }), "{html}");
    let a = saved(&l.id).unwrap();
    assert_eq!(a.answers[at], Some(misses == 0));
    assert_eq!(a.drafts[at].draft, Draft::Match { done: (0..n).collect(), misses: misses as u32 });
}

#[test]
fn a_sort_step_drops_cards_in_bins_and_takes_them_back() {
    let (l, at) = claim(kind("sort"));
    let StepBody::Sort { bins, cards, .. } = &l.steps[at].body else { unreachable!() };
    let n = cards.len();
    let mut d = open(l, at);
    assert!(d.html().contains(tx::SORT_HINT));
    let card = |c: usize| {
        let c = c.to_string();
        move |e: &El| e.has_class("ls-tile") && e.attr("data-card") == Some(c.as_str())
    };
    let in_bin = |b: usize| {
        let b = b.to_string();
        move |e: &El| e.has_class("ls-bin") && e.attr("data-bin") == Some(b.as_str())
    };
    // No card held: the bins can't take anything.
    assert!(d.find(|e| e.has_class("ls-bin-head")).iter().all(El::disabled));
    d.click(card(0));
    assert!(d.html().contains(tx::SORT_PICKED));
    assert_eq!(d.find(|e| e.attr("data-armed") == Some("yes")).len(), bins.len());
    // Tapping the held card again lets go of it.
    d.click(card(0));
    assert!(d.html().contains(tx::SORT_HINT));
    d.click(card(0));
    d.click(in_bin(cards[0].bin));
    assert!(d.html().contains(&tx::sort_left(n - 1, n)));
    // A placed card goes back to the pool on a tap.
    d.click(is("ls-chiplet"));
    assert!(d.html().contains(&tx::sort_left(n, n)));
    // Every card in a wrong bin first (when there are two bins).
    for (c, sc) in cards.iter().enumerate() {
        d.click(card(c));
        let to = if c == 0 { (sc.bin + 1) % bins.len() } else { sc.bin };
        d.click_in(in_bin(to), is("ls-bin-head"));
    }
    d.click(|e| e.attr("data-nav") == Some("check"));
    let html = d.html();
    let wrong = usize::from(bins.len() > 1);
    assert!(
        html.contains(&if wrong == 0 { tx::RIGHT.to_string() } else { tx::in_place(n - wrong, n) }),
        "{html}"
    );
    if wrong > 0 {
        assert!(shows(&d, "ls-should"), "the wrong card names its bin");
    }
    assert_eq!(saved(&l.id).unwrap().answers[at], Some(wrong == 0));
    assert!(d.find(is("ls-chiplet")).is_empty(), "checked cards are read-only");
}

#[test]
fn a_try_step_grades_the_lectures_cards_as_real_reviews() {
    let all_cards = |b: &StepBody| match b {
        StepBody::Try { items } => {
            items.len() >= 2
                && items.iter().all(|i| practice::find(i).is_some_and(|(_, it)| it.body.kind() == "card"))
        }
        _ => false,
    };
    let (l, at) = claim(move |_, _, b| all_cards(b));
    let StepBody::Try { items } = &l.steps[at].body else { unreachable!() };
    let n = items.len();
    let mut d = open(l, at);
    assert!(d.html().contains(&format!("{} 1 {} {n}", tx::TRY_OF, tx::OF)), "{}", d.html());
    // The first card: reveal, then "again"; the rest "good".
    for k in 0..n {
        d.click(|e| e.tag == "button" && e.text.contains(crate::i18n::study::REVEAL));
        let grade = if k == 0 { "again" } else { "good" };
        d.click(move |e| e.has_class("rating") && e.attr("data-grade") == Some(grade));
        let a = saved(&l.id).unwrap();
        assert_eq!(a.tries[at].graded, items[..=k].to_vec(), "each grade is logged once");
        assert_eq!(a.tries[at].again, 1);
    }
    assert!(d.html().contains(tx::TRY_DONE));
    // One "again" makes the step's result a miss; the reviews are in the store.
    assert_eq!(saved(&l.id).unwrap().answers[at], Some(false));
    let reviewed = d.read(|| {
        let store =
            dioxus::core::consume_context_from_scope::<batna_core::review::StudyStoreHandle>(ScopeId::APP)
                .unwrap();
        let s = store.0.borrow();
        items.iter().filter(|i| !s.is_new(i)).count()
    });
    assert_eq!(reviewed, n);
}

#[test]
fn a_try_grade_the_log_refuses_keeps_the_card_until_it_saves() {
    let (l, at) = claim(|_, _, b| match b {
        StepBody::Try { items } => {
            items.len() >= 2
                && items.iter().all(|i| practice::find(i).is_some_and(|(_, it)| it.body.kind() == "card"))
        }
        _ => false,
    });
    let StepBody::Try { items } = &l.steps[at].body else { unreachable!() };
    let mut d = open(l, at);
    d.click(|e| e.tag == "button" && e.text.contains(crate::i18n::study::REVEAL));
    crate::store::shared().set_read_only(true);
    d.click(|e| e.has_class("rating") && e.attr("data-grade") == Some("good"));
    crate::store::shared().set_read_only(false);
    assert!(d.html().contains(tx::TRY_SAVE_FAILED), "{}", d.html());
    assert!(d.html().contains(&format!("{} 1 {} {}", tx::TRY_OF, tx::OF, items.len())), "same card");
    assert!(saved(&l.id).is_none_or(|a| a.tries[at].graded.is_empty()), "nothing counted as graded");
    // Rated again once the disk takes writes: logged once, on to card 2.
    d.click(|e| e.has_class("rating") && e.attr("data-grade") == Some("good"));
    assert!(!d.html().contains(tx::TRY_SAVE_FAILED));
    assert_eq!(saved(&l.id).unwrap().tries[at].graded, items[..1].to_vec());
    assert!(d.html().contains(&format!("{} 2 {} {}", tx::TRY_OF, tx::OF, items.len())));
}

#[test]
fn a_reveal_opens_its_answer_and_unlocks_continue() {
    let (l, at) = claim(kind("reveal"));
    let StepBody::Reveal { answer, .. } = &l.steps[at].body else { unreachable!() };
    let mut d = open(l, at);
    assert!(d.html().contains(tx::THINK_HINT));
    assert!(d.find(next).remove(0).disabled());
    d.click(|e| e.tag == "button" && e.text.contains(tx::REVEAL));
    assert!(
        norm(&d.find(is("ls-answer")).remove(0).text)
            .contains(&norm(answer).chars().take(20).collect::<String>())
    );
    assert!(!d.find(next).remove(0).disabled());
    let a = saved(&l.id).unwrap();
    assert_eq!(a.drafts[at].draft, Draft::Reveal { open: true });
    assert_eq!(a.answers[at], Some(true));
}

#[test]
fn a_mnemonic_opens_letter_by_letter_or_all_at_once() {
    let (l, at) = claim(|_, _, b| matches!(b, StepBody::Mnemonic { parts, .. } if parts.len() >= 2));
    let StepBody::Mnemonic { parts, word, .. } = &l.steps[at].body else { unreachable!() };
    let n = parts.len();
    let mut d = open(l, at);
    assert!(d.html().contains(&word.replace('&', "&amp;")) || norm(&d.html()).contains(&norm(word)));
    assert_eq!(d.find(|e| e.tag == "li" && e.attr("data-open") == Some("no")).len(), n);
    d.click(|e| e.has_class("ls-mn-key"));
    assert_eq!(d.find(|e| e.tag == "li" && e.attr("data-open") == Some("yes")).len(), 1);
    let mut shown = vec![false; n];
    shown[0] = true;
    assert_eq!(saved(&l.id).unwrap().drafts[at].draft, Draft::Mnemonic { shown });
    d.click(|e| e.tag == "button" && e.text.contains(tx::SHOW_ALL));
    assert_eq!(d.find(|e| e.tag == "li" && e.attr("data-open") == Some("yes")).len(), n);
    assert!(d.find(|e| e.text.contains(tx::SHOW_ALL) && e.tag == "button").is_empty());
    for p in parts {
        assert!(norm(&d.html()).contains(&norm(&p.text)), "{} shows", p.text);
    }
}

#[test]
fn a_table_hides_cells_until_tapped() {
    let (l, at) = claim(|_, _, b| matches!(b, StepBody::Table { hide, .. } if hide.len() >= 2));
    let StepBody::Table { hide, rows, .. } = &l.steps[at].body else { unreachable!() };
    let mut d = open(l, at);
    assert!(d.html().contains(tx::TABLE_HINT));
    assert_eq!(d.find(is("ls-cell-hidden")).len(), hide.len());
    d.click(is("ls-cell-hidden"));
    assert_eq!(d.find(is("ls-cell-hidden")).len(), hide.len() - 1);
    let Draft::Table { open } = saved(&l.id).unwrap().drafts[at].draft.clone() else { panic!("table draft") };
    assert_eq!(open.len(), 1);
    let [r, c] = open[0];
    let cell = d.find(|e| e.attr("data-opened") == Some("yes")).remove(0);
    assert_eq!(norm(&cell.text), norm(&rows[r][c]), "the tapped cell shows its value");
    d.click(|e| e.tag == "button" && e.text.contains(tx::SHOW_ALL));
    assert!(d.find(is("ls-cell-hidden")).is_empty());
    assert!(!d.html().contains(tx::TABLE_HINT));
    assert_eq!(d.find(|e| e.attr("data-opened") == Some("yes")).len(), hide.len());
}

#[test]
fn a_figure_opens_full_screen_and_closes() {
    let (l, at) = claim(|_, _, b| matches!(b, StepBody::Figure { .. }));
    let StepBody::Figure { alt, .. } = &l.steps[at].body else { unreachable!() };
    let mut d = open(l, at);
    assert!(!shows(&d, "ls-viewer"));
    d.click(is("ls-svg"));
    let viewer = d.find(is("ls-viewer")).remove(0);
    assert_eq!(viewer.attr("role"), Some("dialog"));
    assert_eq!(viewer.attr("aria-label"), Some(alt.as_str()));
    assert_eq!(d.find(|e| e.attr("data-zoom").is_some()).len(), 3);
    d.click_in(is("ls-viewer"), is("ls-close"));
    assert!(!shows(&d, "ls-viewer"));
    assert!(shows(&d, "ls-player"), "closing the viewer keeps the lesson open");
}

#[test]
fn a_mini_case_goes_part_by_part_and_each_part_can_be_redone() {
    let (l, at) = claim(
        |_, _, b| matches!(b, StepBody::Case { steps, .. } if steps.len() >= 2 && steps.iter().all(|s| s.options.len() >= 2 && s.answer.is_some())),
    );
    let StepBody::Case { steps, .. } = &l.steps[at].body else { unreachable!() };
    let n = steps.len();
    let mut d = open(l, at);
    let pick = |d: &mut Dom, si: usize, oi: usize| {
        let t = norm(&steps[si].options[oi]);
        d.click(move |e| {
            e.has_class("answer") && e.tag == "button" && !e.disabled() && norm(&e.text).ends_with(&t)
        });
    };
    // Part 1, wrong.
    let a0 = steps[0].answer.unwrap();
    pick(&mut d, 0, (a0 + 1) % steps[0].options.len());
    assert_eq!(d.find(|e| e.attr("data-state") == Some("selected")).len(), 1);
    d.click(|e| e.attr("data-nav") == Some("check"));
    assert!(shows(&d, "ls-why-box"), "why opens after the check");
    assert!(d.html().contains(tx::MARK_MINE_WRONG));
    // ↻ on part 1, then the right answer: part 1 now shows right.
    d.click(is("ls-retry"));
    pick(&mut d, 0, a0);
    d.click(|e| e.attr("data-nav") == Some("check"));
    assert!(d.html().contains(tx::MARK_MINE_RIGHT));
    // Every later part, right.
    #[allow(clippy::needless_range_loop)] // si indexes the part and its options
    for si in 1..n {
        d.click(|e| e.tag == "button" && e.text.contains(tx::CASE_NEXT));
        pick(&mut d, si, steps[si].answer.unwrap());
        d.click(|e| e.attr("data-nav") == Some("check"));
    }
    let a = saved(&l.id).unwrap();
    // The first try of part 1 was wrong: the case counts as missed.
    assert_eq!(a.answers[at], Some(false));
    let Draft::Case { picks, first, .. } = a.drafts[at].draft.clone() else { panic!("case draft") };
    assert_eq!(picks, steps.iter().map(|s| s.answer).collect::<Vec<_>>());
    assert_ne!(first[0], steps[0].answer, "the redo never moves the first pick");
    assert!(!d.find(next).remove(0).disabled());
}

#[test]
fn a_question_answered_in_an_older_app_shows_its_key_and_can_be_retried() {
    for k in ["check", "order", "match", "sort", "case"] {
        let (l, at) = claim(kind(k));
        let lesson = l.id.clone();
        let mut d = Dom::with_props(
            root,
            Open {
                lesson: Some(lesson),
                at,
                view: View::Steps,
                draft: None,
                answered: Some(true),
                from: crate::Screen::Learn,
            },
        );
        assert!(shows(&d, "ls-before"), "{k}: {}", d.html());
        assert!(d.html().contains(tx::KEY));
        assert!(!d.find(next).remove(0).disabled(), "{k}: already answered");
        d.click(is("ls-retry"));
        assert!(!shows(&d, "ls-before"), "{k}: the task comes back");
        assert_eq!(saved(&l.id).unwrap().answers[at], Some(true), "{k}: the old result stays");
    }
}

#[test]
fn the_pages_view_turns_with_arrows_and_the_view_switch_is_kept() {
    let (l, _) = claim(|l, k, _| {
        k == 0 && l.steps.len() > 2 && reading(&l.steps[0].body) && reading(&l.steps[1].body)
    });
    let n = l.steps.len();
    let mut d = Dom::with_props(
        root,
        Open {
            lesson: Some(l.id.clone()),
            at: 0,
            view: View::Pages,
            draft: None,
            answered: None,
            from: crate::Screen::Learn,
        },
    );
    let html = d.html();
    assert!(html.contains("data-view=\"pages\""));
    assert!(html.contains(&format!("{} 1 {} {n}", tx::PAGE_OF, tx::OF_N)), "{html}");
    d.click(|e| e.has_class("ls-arrow-next"));
    assert_eq!(count(&d), format!("2/{n}"));
    // The switch in the bar goes back to Steps and saves it.
    d.click(is("ls-view"));
    assert!(d.html().contains("data-view=\"steps\""));
    assert_eq!(crate::store::shared().setting::<View>(VIEW_KEY), Some(View::Steps));
    // × with the lesson opened from Learn stays on Learn, showing the list.
    d.click(is("ls-close"));
    assert!(screen_is(&d, "Learn"));
    assert!(d.html().contains(tx::HEAD));
}

#[test]
fn reading_steps_show_their_text_and_labels() {
    for (k, label) in [("aside", None), ("update", Some(tx::BOOK_WINS)), ("explain", None)] {
        let (l, at) = claim(kind(k));
        let d = open(l, at);
        let st = d.find(is("ls-stage")).remove(0);
        assert_eq!(st.attr("data-kind"), Some(k));
        assert_eq!(st.attr("data-n"), Some(&*(at + 1).to_string()));
        if let Some(label) = label {
            assert!(st.text.contains(label));
        }
        match &l.steps[at].body {
            StepBody::Aside { text, .. } | StepBody::Update { text, .. } | StepBody::Explain { text, .. } => {
                let first: String = norm(text).chars().take(12).collect();
                assert!(norm(&st.text).contains(&first), "{k}: {first}");
            }
            _ => unreachable!(),
        }
        assert!(!d.find(next).remove(0).disabled(), "{k}: nothing to answer");
    }
}

/// A whole lecture's lessons nobody else uses (at least two of them).
pub(crate) fn claim_lecture() -> (&'static str, &'static [Lesson]) {
    crate::store::use_memory_store();
    let mut claimed = CLAIMED.lock().unwrap();
    let taken = claimed.get_or_insert_with(super::browser::fixture_lessons);
    let lib = Library::get();
    for e in &lib.index.lectures {
        let Some(f) = lib.lessons.get(&e.id) else { continue };
        if f.lessons.len() >= 2 && f.lessons.iter().all(|l| !taken.contains(&l.id)) {
            taken.extend(f.lessons.iter().map(|l| l.id.clone()));
            return (e.id.as_str(), &f.lessons);
        }
    }
    panic!("no free lecture");
}

fn list() -> Dom {
    Dom::with_props(
        root,
        Open {
            lesson: None,
            at: 0,
            view: View::Steps,
            draft: None,
            answered: None,
            from: crate::Screen::Learn,
        },
    )
}

/// Leaves `l` part-way at step `at` on this device.
fn leave_at(l: &Lesson, at: usize, started_ms: i64) {
    let a = Attempt { at, ..Attempt::new(l.steps.len(), started_ms) };
    crate::store::shared().put_place(&player::pos_key(&l.id), &a).unwrap();
}

#[test]
fn the_list_searches_lessons_and_clears_the_search() {
    let (_, lessons) = claim_lecture();
    let l = &lessons[0];
    let mut d = list();
    let html = d.html();
    assert!(html.contains(tx::HEAD));
    // "n of all done", over the lessons in view (others finish lessons in
    // parallel, so only the total is fixed).
    let total =
        d.read(|| Library::get().all_lessons().filter(|(lec, _)| crate::courses::focus::shows(lec)).count());
    let sub = d.find(is("page-sub")).remove(0).text;
    assert!(sub.ends_with(&tx::done_of(0, total).replacen("0 ", "", 1)), "{sub}");
    // A lesson's own title finds it, grouped under its lecture.
    d.input(|e| e.tag == "input" && e.attr("type") == Some("search"), &l.title);
    let hits = d.find(|e| e.has_class("ls-item"));
    assert!(hits.iter().any(|h| norm(&h.text).contains(&norm(&l.title))), "{}", d.html());
    assert!(shows(&d, "ls-group"));
    assert!(!shows(&d, "ls-others"), "the other lectures hide while searching");
    // × clears it.
    d.click(is("ls-search-clear"));
    assert!(!shows(&d, "ls-group"));
    assert!(shows(&d, "ls-others"));
    // Nothing found: says so, and with one course in view offers all.
    d.input(|e| e.tag == "input", "zzqx-no-lesson");
    assert!(d.html().contains(&tx::no_match("zzqx-no-lesson")));
    d.read(|| crate::courses::focus::set(Some("hb".into())));
    d.settle();
    d.click(is("ls-search-all"));
    assert_eq!(d.read(crate::courses::focus::current), None);
}

#[test]
fn rows_show_each_lessons_state_and_open_it() {
    let (lec, lessons) = claim_lecture();
    let (a, b) = (&lessons[0], &lessons[1]);
    // The first lesson finished with a score, the second part-way.
    crate::store::shared()
        .put_setting(&format!("{PREFIX}{}", a.id), &Mark { done_ms: 1, right: 2, asked: 3 })
        .unwrap();
    leave_at(b, 2, crate::store::now_ms());
    let mut d = list();
    d.read(|| *SHOWN.write() = Some(lec.to_string()));
    d.settle();
    let rows = d.find(|e| e.has_class("ls-item"));
    let row = |id: &Lesson| {
        rows.iter().find(|r| norm(&r.text).contains(&norm(&id.title))).cloned().expect("a row per lesson")
    };
    assert_eq!(row(a).attr("data-state"), Some("done"));
    assert!(row(a).text.contains("2/3") && row(a).text.contains(tx::STATE_DONE));
    assert_eq!(row(b).attr("data-state"), Some("current"));
    assert!(row(b).text.contains(&tx::step_of(3, b.steps.len())));
    // The lecture card leads back into the lesson left part-way.
    let go = d.find(is("ls-lec-go")).remove(0);
    assert!(go.text.contains(tx::CONTINUE));
    assert!(d.html().contains(&tx::resume_at(3, b.steps.len())));
    d.click(is("ls-lec-go"));
    assert_eq!(d.read(|| OPEN.peek().clone()), Some(b.id.clone()));
    assert_eq!(count(&d), format!("3/{}", b.steps.len()), "it reopens where he left");
}

#[test]
fn a_lesson_left_in_another_lecture_gets_a_card_back_to_it() {
    let (_, lessons) = claim_lecture();
    let (other, _) = claim_lecture();
    let l = &lessons[0];
    leave_at(l, 1, crate::store::now_ms() + 60_000);
    let mut d = list();
    // Another lecture is picked: the card leads back.
    d.read(|| *SHOWN.write() = Some(other.to_string()));
    d.settle();
    let card = d.find(is("ls-resume")).remove(0);
    assert!(card.text.contains(tx::LEFT_OFF_KICKER));
    assert!(card.text.contains(&tx::step_of(2, l.steps.len())));
    d.click(is("ls-resume"));
    assert_eq!(d.read(|| OPEN.peek().clone()), Some(l.id.clone()));
}

#[test]
fn other_lectures_pick_a_lecture_or_open_its_page() {
    let (lec, _) = claim_lecture();
    let mut d = list();
    let title = Library::get().title(lec).to_string();
    d.click(|e| e.has_class("ls-other") && !e.has_class("ls-soon") && norm(&e.text).contains(&norm(&title)));
    assert_eq!(d.read(|| SHOWN.peek().clone()), Some(lec.to_string()));
    assert!(d.find(is("ls-lec")).remove(0).text.contains(&title));
    // The lecture page link on the card.
    d.click(is("ls-lec-page"));
    assert_eq!(d.read(|| crate::lecture::shown(crate::Screen::Learn)), Some(lec.to_string()));
    // A lecture with no lessons yet opens its page.
    let mut d = list();
    if d.find(is("ls-soon")).is_empty() {
        return;
    }
    d.click(is("ls-soon"));
    assert!(d.read(|| crate::lecture::shown(crate::Screen::Learn)).is_some());
}

#[derive(Props, Clone, PartialEq)]
struct Card {
    prefer: Vec<String>,
    item: Option<String>,
}

fn card_root(c: Card) -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(crate::Screen::Today),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    rsx! {
        p { class: "t-screen", "{ui.screen:?}" }
        TodayLesson { prefer: c.prefer.clone() }
        if let Some(id) = c.item.clone() {
            ExplainLink { item_id: id }
        }
    }
}

#[test]
fn today_offers_the_lesson_to_go_on_with() {
    let (lec, lessons) = claim_lecture();
    let mut d = Dom::with_props(card_root, Card { prefer: vec![lec.to_string()], item: None });
    let card = d.find(is("ls-now")).remove(0);
    assert!(card.text.contains(tx::NOW_KICKER) || card.text.contains(tx::NEXT_KICKER), "{}", card.text);
    d.click(is("ls-now"));
    assert!(screen_is(&d, "Learn"));
    assert!(d.read(|| OPEN.peek().clone()).is_some());
    // A lesson part-way comes first, with its step.
    leave_at(&lessons[0], 1, crate::store::now_ms());
    let d = Dom::with_props(card_root, Card { prefer: vec![lec.to_string()], item: None });
    let card = d.find(is("ls-now")).remove(0);
    assert!(card.text.contains(tx::NOW_KICKER));
}

#[test]
fn a_study_item_links_to_the_lesson_that_teaches_it() {
    let (_, lessons) = claim_lecture();
    let l = lessons.iter().find(|l| !l.practice.is_empty()).expect("a lesson with practice");
    let item = l.practice[0].clone();
    let taught = lesson_for_item(&item).expect("the item has a lesson");
    let mut d = Dom::with_props(card_root, Card { prefer: vec![], item: Some(item) });
    let link = d.find(is("ls-explain")).remove(0);
    assert!(link.text.contains(tx::EXPLAIN));
    assert!(norm(&link.text).contains(&norm(&taught.title)));
    d.click(is("ls-explain"));
    assert_eq!(d.read(|| OPEN.peek().clone()), Some(taught.id.clone()));
    // An item no lesson covers shows no link.
    assert!(lesson_for_item("nope.L99.001").is_none());
    let d = Dom::with_props(card_root, Card { prefer: vec![], item: Some("nope.L99.001".into()) });
    assert!(!shows(&d, "ls-explain"));
}

#[test]
fn the_view_setting_switches_between_steps_and_pages() {
    fn app() -> Element {
        rsx! { ViewSection {} }
    }
    crate::store::use_memory_store();
    let mut d = Dom::new(app);
    assert!(d.html().contains(tx::VIEW_STEPS_HINT) && d.html().contains(tx::VIEW_PAGES_HINT));
    d.click(|e| e.attr("role") == Some("radio") && e.text.contains(tx::VIEW_PAGES));
    assert_eq!(d.read(|| *VIEW.peek()), View::Pages);
    let checked = d.find(|e| e.attr("aria-checked") == Some("true")).remove(0);
    assert!(checked.text.contains(tx::VIEW_PAGES));
    d.click(|e| e.attr("role") == Some("radio") && e.text.contains(tx::VIEW_STEPS));
    assert_eq!(d.read(|| *VIEW.peek()), View::Steps);
}

#[test]
fn reading_choices_restyle_the_lesson_and_close_before_the_lesson() {
    // Adapted from Steps-app main: crates/app/src/render_tests/learn.rs.
    use crate::i18n::learn as tl;
    let (l, _) = claim(|l, k, _| k == 0 && reading(&l.steps[0].body));
    let n = l.steps.len();
    let mut d = open(l, 0);
    // The book's defaults, on the player the lesson CSS keys on.
    let html = d.html();
    for attr in ["data-read-size=\"m\"", "data-read-spacing=\"normal\"", "data-read-width=\"book\""] {
        assert!(html.contains(attr), "{attr}");
    }
    // Closed: × is the Escape target. Open: «Aa» is, so Escape and Android
    // Back close the popover before they close the lesson.
    assert!(!html.contains(tl::READ_SPACING));
    assert_eq!(html.matches("aria-keyshortcuts=\"Escape\"").count(), 1);
    d.click(is("ls-read-aa"));
    let html = d.html();
    assert!(html.contains(tl::READ_SPACING) && html.contains(tl::READ_SAMPLE[1]), "{html}");
    assert!(html.contains("aria-expanded=\"true\""), "the button says the popover is open");
    assert_eq!(html.matches("aria-keyshortcuts=\"Escape\"").count(), 1, "only the popover answers Escape");
    assert!(
        d.find(|e| e.has_class("ls-read-aa")).first().is_some_and(|e| e.attr("data-back") == Some("true"))
    );

    d.click(|e| e.attr("aria-label") == Some(tl::READ_SIZE_LARGER));
    d.click(|e| e.tag == "button" && e.text == tl::READ_SPACING_AIRY);
    d.click(|e| e.tag == "button" && e.text == tl::READ_WIDTH_NARROW);
    let player = d.find(is("ls-player")).first().cloned().expect("player");
    assert_eq!(player.attr("data-read-size"), Some("xl"));
    assert_eq!(player.attr("data-read-spacing"), Some("airy"));
    assert_eq!(player.attr("data-read-width"), Some("narrow"));
    // The preview follows the choice too.
    assert!(d.find(is("ls-read-preview")).first().is_some_and(|e| e.attr("data-read-size") == Some("xl")));

    // Done closes it; the lesson is where it was.
    d.click(is("ls-read-done"));
    assert!(!d.html().contains(tl::READ_SPACING));
    assert_eq!(count(&d), format!("1/{n}"));
}

#[test]
fn reading_css_scales_the_step_text_and_moves_the_reading_column() {
    let learn = include_str!("learn.css");
    for want in [
        "font-size: calc(1.05rem * var(--ls-scale, 1)); line-height: calc(1.95 * var(--ls-space, 1));",
        "max-inline-size: var(--ls-one, 40rem);",
        "max-inline-size: var(--ls-one-xl, 44rem);",
    ] {
        assert!(learn.contains(want), "learn.css lacks {want}");
    }
    let reading = include_str!("reading.css");
    for want in [
        "[data-read-size=\"xl\"] { --ls-scale: 1.3; }",
        ".ls-player[data-read-width=\"narrow\"]",
        ".ls-player[data-read-width=\"wide\"]",
        ".ls-read-pop {",
    ] {
        assert!(reading.contains(want), "reading.css lacks {want}");
    }
}
