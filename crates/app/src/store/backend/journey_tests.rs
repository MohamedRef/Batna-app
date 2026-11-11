//! The whole life of an answer across the durable boundaries (engineering
//! audit F23): first launch on a file, grade, restart, restore from an
//! export on a new device, two devices syncing both ways. Each step must
//! land on the same review state. Native-only steps (process death, OS
//! kills, TalkBack) are in docs/NATIVE_QA.md.

use batna_core::content::Lecture;
use batna_core::review::{Grade, Origin};
use batna_protocol::sync::SeqOp;

use super::*;

const T0: i64 = 1_793_610_000_000;
const DAY: i64 = 86_400_000;

fn catalog() -> Catalog {
    let lec: Lecture =
        serde_yaml_ng::from_str(include_str!("../../../../../fixtures/content/sample.yaml")).unwrap();
    lec.items.iter().map(|i| (i.id.clone(), Entry { lecture: lec.lecture.clone(), item: i.clone() })).collect()
}

fn leak(store: Store) -> &'static Store {
    Box::leak(Box::new(store))
}

/// What the hub would pass on: `from`'s outbox, numbered from `seq`.
fn relay(from: &Store, seq: u64) -> Vec<SeqOp> {
    let ops = from.outbox(500).unwrap();
    from.remove_from_outbox(ops.iter().map(|o| o.id.as_str())).unwrap();
    ops.into_iter().enumerate().map(|(i, op)| SeqOp { seq: seq + i as u64, op }).collect()
}

#[test]
fn first_launch_grade_restart_restore_and_two_device_sync_agree() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("batna.db");
    let items: Vec<String> = catalog().keys().take(3).cloned().collect();
    assert_eq!(items.len(), 3, "the sample lecture has items");

    // First launch on a file; three answers over three days.
    let phone_store = leak(Store::open(&path).unwrap());
    let mut phone = ReviewBackend::new(phone_store, catalog());
    for (i, (id, grade)) in items.iter().zip([Grade::Good, Grade::Again, Grade::Easy]).enumerate() {
        let at = T0 + i as i64 * DAY;
        phone.record_logged(ReviewEvent::new(id.as_str(), grade, Origin::Study, at, 4000)).unwrap();
    }
    let graded = phone.states.clone();
    assert_eq!(graded.len(), 3);

    // Restart: the same file, opened again.
    let reopened = ReviewBackend::new(leak(Store::open(&path).unwrap()), catalog());
    assert_eq!(reopened.states, graded, "a restart reads back the same state");

    // Restore: an export imported on a new device.
    let doc = phone_store.export("test", T0).unwrap().document;
    let restored_store = leak(Store::open_in_memory().unwrap());
    restored_store.apply_import(&doc).unwrap();
    let restored = ReviewBackend::new(restored_store, catalog());
    assert_eq!(restored.states, graded, "a restore lands on the same state");

    // Two devices: the tablet gets the phone's answers through the hub…
    let tablet_store = leak(Store::open_in_memory().unwrap());
    tablet_store.apply_remote(&relay(phone_store, 1), 3).unwrap();
    let mut tablet = ReviewBackend::new(tablet_store, catalog());
    assert_eq!(tablet.states, graded);
    // …answers once itself, and the phone gets that back.
    let at = T0 + 5 * DAY;
    tablet.record_logged(ReviewEvent::new(items[1].as_str(), Grade::Good, Origin::Review, at, 3000)).unwrap();
    phone_store.apply_remote(&relay(tablet_store, 4), 4).unwrap();
    phone.reload();
    assert_eq!(phone.states, tablet.states, "both devices agree after syncing both ways");
    assert_ne!(phone.states, graded, "the tablet's answer moved the item");
}
