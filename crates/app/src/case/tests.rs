//! Existing behavior regressions for the feature facade.

use super::*;

#[test]
fn grades_from_steps_right() {
    assert_eq!(grade_for(3, 3), Grade::Good);
    assert_eq!(grade_for(2, 3), Grade::Hard);
    assert_eq!(grade_for(1, 2), Grade::Hard);
    assert_eq!(grade_for(1, 3), Grade::Again);
    assert_eq!(grade_for(0, 0), Grade::Again);
}

#[test]
fn options_are_marked_after_a_pick() {
    assert_eq!(option_state(0, None, None, Some(1)), "");
    assert_eq!(option_state(0, None, Some(0), Some(1)), "selected");
    assert_eq!(option_state(1, Some(1), None, Some(1)), "correct");
    assert_eq!(option_state(1, Some(0), None, Some(1)), "missed");
    assert_eq!(option_state(0, Some(0), None, Some(1)), "wrong");
    assert_eq!(option_state(2, Some(0), None, Some(1)), "dim");
}

fn opts(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|x| (*x).to_owned()).collect()
}

// Issue #86: the key was "A" in 61% of steps because options were
// never shuffled.
#[test]
fn options_are_shuffled_per_attempt() {
    let o = opts(&["Propranolol", "Carvedilol", "Band ligation", "TIPS", "Octreotide"]);
    let a = option_order(&o, 1, 0);
    assert_eq!(a, option_order(&o, 1, 0), "steady while the step is open");
    let mut sorted = a.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, [0, 1, 2, 3, 4]);
    // Across attempts the written first option lands in every position.
    let firsts: std::collections::BTreeSet<usize> =
        (0..200u64).map(|seed| option_order(&o, seed, 0).iter().position(|&i| i == 0).unwrap()).collect();
    assert_eq!(firsts.len(), 5);
    // "All of the above" steps keep their written order.
    let fixed = opts(&["a", "b", "c", "All of the above"]);
    assert_eq!(option_order(&fixed, 7, 0), [0, 1, 2, 3]);
}

#[test]
fn a_step_is_selected_then_checked_once() {
    let mut r = CaseRun::new("hb.L03.050");
    // Nothing selected: Check does nothing.
    assert!(!r.check(Some(1)));
    r.select(0);
    r.select(1);
    assert_eq!((r.draft, r.results.len()), (Some(1), 0));
    assert!(r.check(Some(1)));
    assert_eq!((r.picked, r.results.clone()), (Some(1), vec![(true, Some(1))]));
    // Locked: a second Check or a new selection changes nothing.
    assert!(!r.check(Some(1)));
    r.select(2);
    assert_eq!((r.picked, r.draft, r.results.len()), (Some(1), None, 1));
    r.next();
    assert_eq!((r.step, r.picked, r.draft), (1, None, None));
    // Next without an answer stays put.
    r.next();
    assert_eq!(r.step, 1);
    r.grade_self(false);
    r.grade_self(true);
    assert_eq!(r.results, vec![(true, Some(1)), (false, None)]);
}

#[test]
fn record_keeps_the_case_as_one_review() {
    record(None, "hb.L03.050", 2, 3, now_ms() - 1500);
    let sink = EVENTS.lock().unwrap();
    let e = sink.events.iter().find(|e| e.item_id == "hb.L03.050").unwrap();
    assert_eq!(e.grade, Grade::Hard);
    assert_eq!(e.origin, Origin::Case);
    assert!(e.took_ms >= 1500);
}
