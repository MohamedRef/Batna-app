//! Existing behavior regressions for the feature facade.

use super::*;

#[test]
fn explicit_items_keep_order_and_skip_unknown_and_duplicates() {
    let mut ids: Vec<String> = items()
        .values()
        .filter(|(_, i)| PLAYABLE.contains(&i.body.kind()))
        .map(|(_, i)| i.id.clone())
        .collect();
    ids.sort();
    let ids: Vec<String> = ids.into_iter().take(2).rev().collect();
    if ids.len() < 2 {
        return; // no content embedded
    }
    let store = StudyStoreHandle(std::rc::Rc::new(std::cell::RefCell::new(backend::MemoryStore::default())));
    let asked = [ids[0].clone(), "nope.L99.001".into(), ids[1].clone(), ids[0].clone()];
    let plan = plan_items(&store, &asked);
    let got: Vec<&str> = plan.iter().map(|p| p.item_id.as_str()).collect();
    assert_eq!(got, [ids[0].as_str(), ids[1].as_str()]);
    assert!(plan.iter().all(|p| p.fresh));
}

#[test]
fn when_labels() {
    // UTC, so study days start at 04:00 UTC. `now` is 05:00 on a day.
    let zone = batna_core::zone::Zone::fixed(0);
    let now = 20_000 * 86_400_000 + 5 * 3_600_000;
    let at = |h: i64| now + h * 3_600_000;
    let label = |due: i64| when_label(now, due, &zone);
    assert_eq!(label(now), tx::NOW);
    assert_eq!(label(now + 20 * 60_000), "بعد 20 دقيقة");
    assert_eq!(label(at(5)), "بعد 5 ساعات");
    assert_eq!(label(at(2)), "بعد ساعتين");
    assert_eq!(label(at(23)), tx::TOMORROW, "04:00 tomorrow");
    // A 2-day interval lands at 04:00 two study days on: 47 hours away.
    assert_eq!(label(at(47)), "بعد يومين");
    assert_eq!(label(at(71)), "بعد 3 أيام");
}
