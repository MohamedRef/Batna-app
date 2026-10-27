//! The seam between the study player and the review store (chunk 3).
//!
//! `StudyStore` mirrors the trait chunk 3 is adding to `batna_core::review`;
//! once that lands, this file re-exports it instead. Until a store is provided
//! in `App`, the player keeps answers in memory for the run.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use batna_core::content::Item;
use batna_core::review::{MemorySink, ReviewEvent, ReviewSink};
use dioxus::prelude::*;

pub trait StudyStore {
    fn record(&mut self, event: ReviewEvent);
    /// Due item ids, most overdue first.
    fn due(&self, now_ms: i64) -> Vec<String>;
    fn is_new(&self, item_id: &str) -> bool;
    /// Marked wrong (✗) and kept out of study until fixed.
    fn is_hidden(&self, item_id: &str) -> bool;
    /// The item with the learner's own edit (✎) applied.
    fn effective(&self, item: &Item) -> Item {
        item.clone()
    }
    fn next_due_ms(&self, now_ms: i64) -> Option<i64>;
    /// New items that may still start today.
    fn new_left_today(&self, _now_ms: i64) -> usize {
        usize::MAX
    }
    /// Days until the next review for Again, Hard, Good and Easy.
    fn intervals(&self, _item_id: &str, _now_ms: i64) -> Option<[f32; 4]> {
        None
    }
}

#[derive(Clone)]
pub struct StudyStoreHandle(pub Rc<RefCell<dyn StudyStore>>);

/// Answers kept in memory for this run, used until the real store exists.
#[derive(Default)]
struct MemoryStore {
    sink: MemorySink,
    seen: HashSet<String>,
}

impl StudyStore for MemoryStore {
    fn record(&mut self, event: ReviewEvent) {
        self.seen.insert(event.item_id.clone());
        self.sink.record(event);
    }
    fn due(&self, _now_ms: i64) -> Vec<String> {
        Vec::new()
    }
    fn is_new(&self, item_id: &str) -> bool {
        !self.seen.contains(item_id)
    }
    fn is_hidden(&self, _item_id: &str) -> bool {
        false
    }
    fn next_due_ms(&self, _now_ms: i64) -> Option<i64> {
        None
    }
}

/// The store from `App`'s context, or the in-memory one.
pub fn use_store() -> StudyStoreHandle {
    use_hook(|| {
        try_consume_context::<StudyStoreHandle>().unwrap_or_else(|| {
            thread_local! {
                static FALLBACK: StudyStoreHandle =
                    StudyStoreHandle(Rc::new(RefCell::new(MemoryStore::default())));
            }
            FALLBACK.with(Clone::clone)
        })
    })
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}
