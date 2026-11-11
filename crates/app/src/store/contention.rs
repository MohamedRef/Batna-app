//! Two connections share the database file: the UI's and the sync engine's
//! (`cloud::start`). With WAL only one writes at a time, and SQLite's
//! default is to fail at once when the other holds the lock (engineering
//! audit F16). Each side now waits a bounded time:
//!
//! - the UI waits at most [`UI_BUSY`], its latency budget for a tap. Past
//!   that the write fails, and the screen's own saving state takes over (the
//!   answer stays with «حاول تاني», `bank::unsaved`, the lesson's Retry…),
//!   never a silent loss and never a frozen screen;
//! - the sync thread, off the UI thread, waits up to [`SYNC_BUSY`]: its
//!   transactions are short, so the UI's writes rarely wait on it at all.

use std::time::Duration;

use super::{Result, Store};

/// How long a UI write waits for the sync thread's lock.
pub const UI_BUSY: Duration = Duration::from_millis(750);
/// How long the sync thread waits for the UI's lock.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // the web syncs in-process
pub const SYNC_BUSY: Duration = Duration::from_secs(5);

impl Store {
    /// How long this connection waits for another's write lock.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn set_busy_timeout(&self, wait: Duration) -> Result<()> {
        self.conn.lock().unwrap().busy_timeout(wait)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use batna_core::review::{Grade, Origin, ReviewEvent};

    use super::*;

    /// The sync connection holds the write lock: a UI write fails within
    /// its budget instead of freezing (or failing at once), and goes
    /// through as soon as the lock is free.
    #[test]
    fn a_ui_write_waits_its_budget_then_fails_and_works_again() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("batna.db");
        let ui = Store::open(&path).unwrap();
        let sync = Store::open(&path).unwrap();
        sync.set_busy_timeout(SYNC_BUSY).unwrap();
        let answer = || ReviewEvent::new("hb.L01.003", Grade::Good, Origin::Study, 1, 3000);

        sync.conn.lock().unwrap().execute_batch("BEGIN IMMEDIATE").unwrap();
        let t = Instant::now();
        assert!(ui.append_review(answer()).is_err(), "the lock is held");
        let waited = t.elapsed();
        assert!(waited >= UI_BUSY.mul_f32(0.9), "it waited for the lock: {waited:?}");
        assert!(waited < UI_BUSY * 3, "within the budget: {waited:?}");
        // A write that reads first (last-writer-wins compares stamps) waits
        // too: its transaction takes the lock up front.
        let t = Instant::now();
        assert!(ui.put_setting("theme", &"night").is_err());
        assert!(t.elapsed() >= UI_BUSY.mul_f32(0.9), "it waited: {:?}", t.elapsed());

        sync.conn.lock().unwrap().execute_batch("COMMIT").unwrap();
        ui.append_review(answer()).unwrap();
        assert_eq!(sync.reviews().unwrap().len(), 1);
    }

    /// A lock held briefly (a sync page) doesn't fail a UI write at all.
    #[test]
    fn a_short_sync_write_only_delays_the_ui() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("batna.db");
        let ui = Store::open(&path).unwrap();
        let sync = std::sync::Arc::new(Store::open(&path).unwrap());
        sync.conn.lock().unwrap().execute_batch("BEGIN IMMEDIATE").unwrap();
        let held = sync.clone();
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            held.conn.lock().unwrap().execute_batch("COMMIT").unwrap();
        });
        ui.append_review(ReviewEvent::new("hb.L01.003", Grade::Good, Origin::Study, 1, 3000)).unwrap();
        release.join().unwrap();
    }
}
