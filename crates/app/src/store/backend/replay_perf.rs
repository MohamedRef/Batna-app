//! Informational Study review-replay baseline using the actual FSRS backend.
//! Run with: cargo test -p batna-app store::backend::replay_perf::profile_review_replay -- --ignored --nocapture
//! Synthetic histories are not a substitute for native Android frame profiling.

use super::*;
use batna_core::review::{Grade, Origin};
use std::hint::black_box;
use std::time::Instant;

fn percentiles(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    let last = samples.len() - 1;
    (samples[samples.len() / 2], samples[(last * 95).div_ceil(100)], samples[last])
}

#[test]
#[ignore = "informational hosted-runner timing, never a pass/fail latency threshold"]
fn profile_review_replay() {
    for count in [100_usize, 1_000, 3_000] {
        let db: &'static Store = Box::leak(Box::new(Store::open_in_memory().unwrap()));
        for n in 0..count {
            db.append_review(ReviewEvent {
                item_id: format!("hb.L01.replay.{}", n % 64),
                grade: match n % 4 {
                    0 => Grade::Again,
                    1 => Grade::Hard,
                    2 => Grade::Good,
                    _ => Grade::Easy,
                },
                origin: Origin::Study,
                at_ms: 1_793_610_000_000 + n as i64 * 60_000,
                took_ms: 1_200,
            })
            .expect("seed committed review event");
        }

        let mut backend = ReviewBackend::new(db, Catalog::new());
        assert_eq!(backend.reviews_logged(), count);
        let mut replay_us = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            backend.replay();
            replay_us.push(start.elapsed().as_micros());
            black_box(backend.states.len());
        }
        let (p50, p95, max) = percentiles(&mut replay_us);

        // The real grade path commits the event, pushes the in-memory log,
        // replays FSRS and signals the UI. It intentionally does *not* include
        // the separate checkpoint marker or WebView rendering time.
        let start = Instant::now();
        let event_id = StudyStore::record_logged(
            &mut backend,
            ReviewEvent {
                item_id: "hb.L01.replay.0".into(),
                grade: Grade::Good,
                origin: Origin::Study,
                at_ms: 1_793_610_000_000 + count as i64 * 60_000,
                took_ms: 1_200,
            },
        )
        .expect("persist grade")
        .expect("committed review id");
        let grade_us = start.elapsed().as_micros();
        assert_eq!(backend.reviews_logged(), count + 1);
        assert!(db.reviews().unwrap().iter().any(|entry| entry.id == event_id));

        println!(
            "STUDY_REPLAY_BASELINE events={count} replay_p50_us={p50} replay_p95_us={p95} replay_max_us={max} grade_store_replay_us={grade_us}"
        );
    }
}
