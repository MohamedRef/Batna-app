//! Review events: what every study screen reports after an answer. Chunk 3
//! turns them into FSRS scheduling and stores them; until then screens can
//! write to [`MemorySink`].

use serde::{Deserialize, Serialize};

/// The four FSRS ratings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

/// Which screen produced the answer, for stats and for weighting later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Study,
    Review,
    Bank,
    Mock,
    Case,
    Oral,
}

/// One answer to one item. Append-only: never edited, never deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewEvent {
    pub item_id: String,
    pub grade: Grade,
    pub origin: Origin,
    /// Unix time in milliseconds.
    pub at_ms: i64,
    /// Time spent on the item, in milliseconds.
    pub took_ms: u32,
}

/// Where answers go. Chunk 3 implements it on the store.
pub trait ReviewSink {
    fn record(&mut self, event: ReviewEvent);
}

/// Keeps events in memory: for tests and for screens that land before the store.
#[derive(Debug, Default)]
pub struct MemorySink {
    pub events: Vec<ReviewEvent>,
}

impl ReviewSink for MemorySink {
    fn record(&mut self, event: ReviewEvent) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_sink_keeps_order() {
        let mut sink = MemorySink::default();
        for (i, grade) in [Grade::Again, Grade::Good].into_iter().enumerate() {
            sink.record(ReviewEvent {
                item_id: format!("hb.L01.00{i}"),
                grade,
                origin: Origin::Study,
                at_ms: i as i64,
                took_ms: 1000,
            });
        }
        assert_eq!(sink.events.len(), 2);
        assert_eq!(sink.events[1].grade, Grade::Good);
    }
}
