//! The review revision must be transactionally bound to every possible
//! mutation, even imports or sync code that writes SQL outside Store::append_review.

use super::*;
use batna_core::review::{Grade, Origin};

fn sample() -> ReviewEvent {
    ReviewEvent {
        confidence: None,
        reason: None,
        item_id: "hb.L01.001".into(),
        grade: Grade::Good,
        origin: Origin::Study,
        at_ms: 100,
        took_ms: 20,
    }
}

#[test]
fn revision_is_constant_time_to_read_and_changes_on_mutations() {
    let store = Store::open_in_memory().unwrap();
    let initial = store.study_review_marker().unwrap();
    assert!(initial.starts_with("v2:"));
    assert_eq!(initial.len(), 35, "v2 prefix plus 128-bit random token");
    store.put_local_setting("theme", &"dark").unwrap();
    assert_eq!(store.study_review_marker().unwrap(), initial);

    let logged = store.append_review(sample()).unwrap();
    let inserted = store.study_review_marker().unwrap();
    assert_ne!(inserted, initial);
    assert_eq!(store.study_review_marker().unwrap(), inserted);

    {
        let mut conn = store.conn.lock().unwrap();
        let tx = conn.transaction().unwrap();
        tx.execute("UPDATE review_events SET took_ms = 40 WHERE id = ?1", [&logged.id]).unwrap();
        tx.rollback().unwrap();
    }
    assert_eq!(store.study_review_marker().unwrap(), inserted, "rolled back review must not change marker");

    store
        .conn
        .lock()
        .unwrap()
        .execute("UPDATE review_events SET took_ms = 41 WHERE id = ?1", [&logged.id])
        .unwrap();
    let updated = store.study_review_marker().unwrap();
    assert_ne!(updated, inserted, "remote merge or import updates must invalidate");

    assert!(store.retract_review(&logged.id).unwrap());
    let undone = store.study_review_marker().unwrap();
    assert_ne!(updated, undone);
    assert!(!store.retract_review(&logged.id).unwrap());
    assert_eq!(store.study_review_marker().unwrap(), undone, "duplicate undo does not mutate");
    assert!(store.reviews().unwrap().is_empty());

    let reopened = Store::from_snapshot(&store.snapshot().unwrap().1).unwrap();
    assert_eq!(reopened.study_review_marker().unwrap(), undone);

    reopened.conn.lock().unwrap().execute("DELETE FROM review_events WHERE id = ?1", [&logged.id]).unwrap();
    assert_ne!(reopened.study_review_marker().unwrap(), undone);
}

#[test]
fn old_and_new_markers_both_detect_new_committed_reviews() {
    let store = Store::open_in_memory().unwrap();
    let old = store.study_review_marker_legacy().unwrap();
    let new = store.study_review_marker().unwrap();
    store.append_review(sample()).unwrap();
    assert_ne!(store.study_review_marker_legacy().unwrap(), old);
    assert_ne!(store.study_review_marker().unwrap(), new);
}

#[test]
fn two_branches_with_same_review_count_cannot_share_marker() {
    let store = Store::open_in_memory().unwrap();
    let bytes = store.snapshot().unwrap().1;
    let fork_a = Store::from_snapshot(&bytes).unwrap();
    let fork_b = Store::from_snapshot(&bytes).unwrap();
    fork_a.append_review(sample()).unwrap();
    let mut other = sample();
    other.item_id = "hb.L01.002".into();
    fork_b.append_review(other).unwrap();
    assert_ne!(fork_a.study_review_marker().unwrap(), fork_b.study_review_marker().unwrap());
}

#[test]
fn borrowed_device_wipe_keeps_a_valid_future_revision() {
    let store = Store::open_in_memory().unwrap();
    store.append_review(sample()).unwrap();
    let before = store.study_review_marker().unwrap();
    store.forget_study().unwrap();
    let wiped = store.study_review_marker().unwrap();
    assert_ne!(wiped, before, "forgetting data invalidates the older checkpoint");
    assert!(store.reviews().unwrap().is_empty());

    store.append_review(sample()).unwrap();
    assert_ne!(store.study_review_marker().unwrap(), wiped, "new answers after wipe still invalidate");
}
