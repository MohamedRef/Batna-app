//! «ابدأ من الأول» (#1341) end to end over real stores: the reset marker
//! syncs, survives a restore and an offline device, undoes exactly, and
//! leaves everything that is not progress alone.

use batna_core::reset::Scope;
use batna_core::review::{Grade, Origin, ReviewEvent, StudyStore as _};
use batna_protocol::sync::SeqOp;

use super::*;

fn store() -> &'static Store {
    Box::leak(Box::new(Store::open_in_memory().unwrap()))
}

fn backend(s: &'static Store) -> ReviewBackend {
    ReviewBackend::new(s, catalog(Library::get()))
}

fn answer(s: &Store, item: &str, at_ms: i64) {
    s.append_review(ReviewEvent::new(item, Grade::Good, Origin::Study, at_ms, 3000)).unwrap();
}

/// What the hub would pass on from `from`, numbered from `seq`.
fn relay(from: &Store, to: &Store, seq: u64) -> u64 {
    let ops = from.outbox(500).unwrap();
    from.remove_from_outbox(ops.iter().map(|o| o.id.as_str())).unwrap();
    let n = ops.len() as u64;
    let ops: Vec<SeqOp> =
        ops.into_iter().enumerate().map(|(i, op)| SeqOp { seq: seq + i as u64, op }).collect();
    to.apply_remote(&ops, seq + n).unwrap();
    to.observe_stored().unwrap();
    seq + n
}

/// Two items of the hepatology course and one of another course.
fn items() -> (String, String, String) {
    let lib = Library::get();
    let mut hb = lib.lectures.values().filter(|l| l.lecture.starts_with("hb.")).flat_map(|l| &l.items);
    let a = hb.next().unwrap().id.clone();
    let b = hb.next().unwrap().id.clone();
    let other = lib.lectures.values().find(|l| !l.lecture.starts_with("hb.")).unwrap().items[0].id.clone();
    (a, b, other)
}

fn hb() -> Scope {
    Scope::Course("hb".into())
}

fn now() -> i64 {
    super::super::now_ms()
}

#[test]
fn an_offline_tablets_older_answers_stay_out_and_its_newer_ones_count() {
    let (a, b, other) = items();
    let (phone, tablet) = (store(), store());
    answer(phone, &a, now());
    answer(phone, &other, now());
    let seq = relay(phone, tablet, 1);
    // The tablet studies offline while the phone resets hepatology.
    answer(tablet, &b, now());
    // Order is by HLC: the reset comes at least a millisecond later.
    std::thread::sleep(std::time::Duration::from_millis(3));
    phone.put_reset(&hb(), |_| {}).unwrap();
    let seq = relay(phone, tablet, seq);
    relay(tablet, phone, seq);
    for s in [phone, tablet] {
        let be = backend(s);
        assert!(be.is_new(&a) && be.is_new(&b), "hepatology is new on both devices");
        assert!(!be.is_new(&other), "the other course is untouched");
        assert_eq!(be.reviews_logged(), 3, "nothing was deleted");
    }
    // An answer the tablet makes now, after the reset, counts.
    answer(tablet, &b, now());
    let be = backend(tablet);
    assert!(!be.is_new(&b) && be.is_new(&a));
}

#[test]
fn a_backup_from_before_the_reset_brings_no_progress_back_until_undo() {
    let (a, _, _) = items();
    let phone = store();
    answer(phone, &a, now());
    let before = phone.export("test", now()).unwrap().document;
    let schedule = backend(phone).states().clone();
    phone.put_reset(&hb(), |_| {}).unwrap();
    phone.apply_import(&before).unwrap();
    let be = backend(phone);
    assert!(be.is_new(&a), "the restored answers stay behind the marker");
    phone.remove_reset(&hb()).unwrap();
    assert_eq!(backend(phone).states(), &schedule);
}

#[test]
fn undo_brings_back_the_exact_schedule_and_answer_undo_still_works() {
    let (a, b, other) = items();
    let s = store();
    for (k, id) in [&a, &b, &a, &other].into_iter().enumerate() {
        answer(s, id, now() - 86_400_000 * (4 - k as i64));
    }
    let schedule = backend(s).states().clone();
    let weak = backend(s).weak();
    s.put_reset(&Scope::All, |_| {}).unwrap();
    let mut be = backend(s);
    assert!(be.states().is_empty(), "the whole curriculum restarts");
    assert_eq!(be.reviews_logged(), 4, "the activity history keeps every answer");
    assert_eq!(be.answers_per_day(5, now()).iter().sum::<u32>(), 4);
    // Studying after the reset, then taking that answer back.
    let id =
        be.record_logged(ReviewEvent::new(&a, Grade::Again, Origin::Study, now(), 3000)).unwrap().unwrap();
    assert!(!be.is_new(&a));
    assert!(be.retract(&id));
    assert!(be.is_new(&a), "Undo of the last answer works after a reset");
    s.remove_reset(&Scope::All).unwrap();
    let be = backend(s);
    assert_eq!(be.states(), &schedule, "everything comes back exactly as it was");
    assert_eq!(be.weak(), weak);
}

#[test]
fn verdicts_notes_and_own_cards_are_untouched_byte_for_byte() {
    let (a, _, _) = items();
    let s = store();
    answer(s, &a, now());
    let j = Judgement {
        verdict: Verdict::Approved,
        edit: verify::Edit::new(),
        note: None,
        fingerprint: "f".into(),
        at_ms: 1,
    };
    s.set_judgement(&a, Some(&j)).unwrap();
    let note = batna_core::note::Note {
        anchor: batna_core::note::Anchor::Item(a.clone()),
        text: "mine".into(),
        page: None,
        created_ms: 1,
        edited_ms: 1,
    };
    s.put_note(&note).unwrap();
    s.put_own_card(&batna_core::own::OwnCard {
        id: "own.0123456789abcdef".into(),
        front: "f".into(),
        back: "b".into(),
        lecture: Some("hb.L01".into()),
        page: None,
        source: None,
        created_ms: 1,
    })
    .unwrap();
    s.put_setting("hl.hb.L01.lesson1.x", &"yellow").unwrap();
    let dump = |s: &Store| {
        let conn = s.conn.lock().unwrap();
        let mut out = Vec::new();
        for (t, k) in [("item_status", "item_id"), ("notes", "id"), ("own_cards", "id")] {
            let mut stmt = conn.prepare(&format!("SELECT * FROM {t} ORDER BY {k}")).unwrap();
            let n = stmt.column_count();
            let rows = stmt
                .query_map([], |r| {
                    (0..n).map(|i| r.get::<_, rusqlite::types::Value>(i).map(|v| format!("{v:?}"))).collect()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<Vec<String>>>>()
                .unwrap();
            out.push(rows);
        }
        out
    };
    let before = dump(s);
    let hl: Option<String> = s.setting("hl.hb.L01.lesson1.x");
    s.put_reset(&Scope::All, |_| {}).unwrap();
    let be = backend(s);
    assert!(be.is_new(&a));
    assert_eq!(dump(s), before);
    assert_eq!(s.setting::<String>("hl.hb.L01.lesson1.x"), hl);
    assert_eq!(be.judgement(&a), Some(&j));
    assert_eq!(be.notes().len(), 1);
}

#[test]
fn an_export_carries_the_marker_to_a_new_device() {
    let (a, _, _) = items();
    let phone = store();
    answer(phone, &a, now());
    let marker = phone.put_reset(&hb(), |_| {}).unwrap();
    let doc = phone.export("test", now()).unwrap().document;
    let fresh = store();
    fresh.apply_import(&doc).unwrap();
    assert_eq!(fresh.resets().courses.get("hb"), Some(&marker));
    assert!(backend(fresh).is_new(&a));
    assert_eq!(backend(fresh).reviews_logged(), 1);
}
