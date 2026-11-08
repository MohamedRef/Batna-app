//! Informational SQLite Study checkpoint cost baseline.
//! Run with: cargo test -p batna-app store::marker_perf::profile_study_checkpoint_marker -- --ignored --nocapture
//! Timings on a hosted Linux runner are not Android frame/persistence latencies.

use super::*;
use std::hint::black_box;
use std::time::Instant;

fn microseconds(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    let last = samples.len() - 1;
    (samples[samples.len() / 2], samples[(last * 95).div_ceil(100)], samples[last])
}

#[test]
#[ignore = "informational performance baseline, no hosted-runner timing gate"]
fn profile_study_checkpoint_marker() {
    let db = Store::open_in_memory().expect("temporary SQLite store");
    let mut inserted = 0;
    for count in [100_usize, 1_000, 3_000] {
        for n in inserted..count {
            db.append_review(ReviewEvent {
                item_id: format!("hb.L01.profile.{}", n % 64),
                grade: Grade::Good,
                origin: Origin::Study,
                at_ms: 1_800_000_000_000 + n as i64,
                took_ms: 1_200,
            })
            .expect("seed committed review events");
        }
        inserted = count;

        // Measure the read/hash and the full checkpoint write independently.
        // Save is deliberately unchanged by this diagnostic PR.
        let mut read_us = Vec::new();
        let mut save_us = Vec::new();
        for _ in 0..7 {
            let started = Instant::now();
            let marker = black_box(db.study_review_marker().expect("review marker"));
            read_us.push(started.elapsed().as_micros());

            let started = Instant::now();
            db.put_local_setting("benchmark.study.marker", &marker).expect("local checkpoint write");
            save_us.push(started.elapsed().as_micros());
        }
        let (r50, r95, rmax) = microseconds(&mut read_us);
        let (s50, s95, smax) = microseconds(&mut save_us);
        println!(
            "STUDY_CHECKPOINT_MARKER_BASELINE events={count} marker_p50_us={r50} marker_p95_us={r95} marker_max_us={rmax} local_write_p50_us={s50} local_write_p95_us={s95} local_write_max_us={smax}"
        );
    }
}
