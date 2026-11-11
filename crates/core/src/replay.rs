//! One answer applied without replaying the whole log (engineering audit
//! F15). Full [`Scheduler::replay_all`] stays the rule for a reload, an
//! import, a sync or a retraction; a local answer that sorts after every
//! answer already in the log changes only its own item, the same way a full
//! replay would, so it can be applied on its own.

use std::collections::BTreeMap;

use batna_protocol::Hlc;

use crate::review::LoggedReview;
use crate::srs::{CardState, Scheduler};

impl Scheduler {
    /// Applies `e` to `states` when it is newer than `newest` (the highest
    /// HLC in the log it joins) and returns true; else changes nothing and
    /// returns false, and the caller must replay the whole log.
    pub fn step(
        &self,
        states: &mut BTreeMap<String, CardState>,
        newest: Option<&Hlc>,
        e: &LoggedReview,
    ) -> bool {
        if newest.is_some_and(|n| e.hlc <= *n) {
            return false;
        }
        let id = e.event.item_id.as_str();
        let state = states.entry(id.to_owned()).or_insert_with(|| CardState::new(id));
        if self.admits(state, &e.event) {
            *state = self.apply(state, e.event.grade, e.event.at_ms);
        }
        true
    }
}

/// The highest HLC in a log.
pub fn newest(log: &[LoggedReview]) -> Option<&Hlc> {
    log.iter().map(|e| &e.hlc).max()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{Grade, Origin, ReviewEvent};

    /// A small deterministic generator, so failures reproduce.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self, n: u64) -> u64 {
            self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) % n
        }
    }

    fn event(rng: &mut Lcg, at_ms: i64, n: u64, items: u64) -> LoggedReview {
        let grade = [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy][rng.next(4) as usize];
        let origin = [Origin::Study, Origin::Review, Origin::Bank, Origin::Mock][rng.next(4) as usize];
        let item = format!("hb.L01.{:03}", rng.next(items));
        LoggedReview {
            id: format!("e{n}"),
            hlc: Hlc { wall_ms: at_ms, counter: (n % 3) as u32, device: "phone".into() },
            event: ReviewEvent::new(&item, grade, origin, at_ms, 3000),
        }
    }

    /// Step by step equals a full replay after every answer, over random
    /// grades, origins (evidence included) and gaps from minutes to weeks.
    #[test]
    fn stepping_matches_a_full_replay() {
        let s = Scheduler::default();
        for seed in 1..=20 {
            let mut rng = Lcg(seed);
            let (mut log, mut states) = (Vec::new(), BTreeMap::new());
            let mut at = 1_793_000_000_000_i64;
            for n in 0..300 {
                at += [60_000, 3_600_000, 86_400_000, 7 * 86_400_000][rng.next(4) as usize]
                    + rng.next(1000) as i64;
                let e = event(&mut rng, at, n, 12);
                assert!(s.step(&mut states, newest(&log), &e), "in order");
                log.push(e);
                assert_eq!(states, s.replay_all(&log), "seed {seed}, answer {n}");
            }
        }
    }

    /// An answer that sorts before one already logged (from another device,
    /// or a clock set back) is refused: only a full replay places it.
    #[test]
    fn an_out_of_order_answer_asks_for_a_full_replay() {
        let s = Scheduler::default();
        let mut rng = Lcg(7);
        let first = event(&mut rng, 2_000, 1, 3);
        let mut states = s.replay_all(std::slice::from_ref(&first));
        let before = states.clone();
        let earlier = event(&mut rng, 1_000, 2, 3);
        assert!(!s.step(&mut states, Some(&first.hlc), &earlier));
        assert_eq!(states, before, "nothing changed");
        let same = LoggedReview { id: "dup".into(), ..first.clone() };
        assert!(!s.step(&mut states, Some(&first.hlc), &same), "an equal stamp isn't newer");
    }

    /// Timings for the audit (run by hand in release:
    /// `cargo test -p batna-core --release replay_timings -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn replay_timings() {
        let s = Scheduler::default();
        for total in [10_000_u64, 50_000, 100_000] {
            let mut rng = Lcg(total);
            let mut at = 1_700_000_000_000_i64;
            let log: Vec<LoggedReview> = (0..total)
                .map(|n| {
                    at += 60_000 + rng.next(3_600_000) as i64;
                    event(&mut rng, at, n, 3_000)
                })
                .collect();
            let t = std::time::Instant::now();
            let mut states = s.replay_all(&log[..log.len() - 1]);
            let full = t.elapsed();
            let t = std::time::Instant::now();
            assert!(s.step(&mut states, newest(&log[..log.len() - 1]), &log[log.len() - 1]));
            let one = t.elapsed();
            println!("{total} answers: full replay {full:?}, one step {one:?}");
        }
    }
}
