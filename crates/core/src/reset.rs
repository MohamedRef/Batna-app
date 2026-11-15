//! «ابدأ من الأول»: restarting progress for one course or the whole
//! curriculum (Batna #1341) without deleting anything.
//!
//! A reset is one synced last-writer-wins setting, `reset.<course id>` or
//! `reset.all`, holding a [`Marker`]: the reset's HLC. Readers leave out
//! progress in that scope made at or before the marker:
//!
//! - answers (`review_events`) by HLC, so an offline device that syncs its
//!   older answers later still finds them left out, and its answers made
//!   after the reset count;
//! - finished lessons (`lesson.<id>`) and places inside a lesson
//!   (`lessonpos.<id>`) by their own wall-clock time against the marker's;
//! - finished papers, only on a whole-curriculum reset that chose to
//!   include them ([`Marker::papers`]).
//!
//! Nothing is deleted or tombstoned, so undo is removing the marker (a
//! tombstone on that one setting) and everything reads as before. A second
//! reset of the same scope replaces the marker with a later one.
//!
//! His verdicts on content, notes, highlights, own cards (their text),
//! course statuses, the exam date and every setting are not progress and no
//! reader here touches them. His own cards' schedules restart only with
//! `reset.all`: their ids (`own.…`) belong to no course.

use std::collections::BTreeMap;

use batna_protocol::Hlc;
use serde::{Deserialize, Serialize};

use crate::program::course_of;
use crate::review::LoggedReview;

/// Synced settings keys start with this.
pub const PREFIX: &str = "reset.";
/// The scope id of the whole curriculum (`reset.all`).
pub const ALL: &str = "all";

/// What a reset covers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// Every course, and his own cards.
    All,
    /// One course, by id (`hb`).
    Course(String),
}

impl Scope {
    /// The synced setting's key: `reset.all`, `reset.hb`.
    pub fn key(&self) -> String {
        format!("{PREFIX}{}", self.id())
    }

    pub fn id(&self) -> &str {
        match self {
            Scope::All => ALL,
            Scope::Course(c) => c,
        }
    }

    /// The scope a settings key names, if it is a reset key.
    pub fn from_key(key: &str) -> Option<Scope> {
        match key.strip_prefix(PREFIX)? {
            "" => None,
            ALL => Some(Scope::All),
            c => Some(Scope::Course(c.to_owned())),
        }
    }

    /// Whether an item, lesson or lecture id is in this scope.
    pub fn covers(&self, id: &str) -> bool {
        match self {
            Scope::All => true,
            Scope::Course(c) => course_of(id) == c,
        }
    }
}

/// One reset, as stored in its setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marker {
    /// The reset's own stamp: progress stamped at or before it is left out.
    pub hlc: Hlc,
    /// Unique per reset, so a replaced marker is told apart from its undo.
    pub id: String,
    /// Finished papers and mocks restart too (whole curriculum only).
    #[serde(default)]
    pub papers: bool,
    /// The catch-up plan that was running when the reset was made (opaque
    /// to core), so undo can put it back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catchup: Option<serde_json::Value>,
}

impl Marker {
    pub fn new(hlc: Hlc) -> Marker {
        let id = format!("{}-{}-{}", hlc.device, hlc.wall_ms, hlc.counter);
        Marker { hlc, id, papers: false, catchup: None }
    }

    /// The reset's wall-clock time.
    pub fn at_ms(&self) -> i64 {
        self.hlc.wall_ms
    }
}

/// Every reset in force.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resets {
    pub all: Option<Marker>,
    pub courses: BTreeMap<String, Marker>,
}

impl Resets {
    /// From the synced settings under [`PREFIX`]; keys that are not reset
    /// keys are skipped.
    pub fn from_settings(rows: impl IntoIterator<Item = (String, Marker)>) -> Resets {
        let mut out = Resets::default();
        for (key, marker) in rows {
            match Scope::from_key(&key) {
                Some(Scope::All) => out.all = Some(marker),
                Some(Scope::Course(c)) => {
                    out.courses.insert(c, marker);
                }
                None => {}
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.all.is_none() && self.courses.is_empty()
    }

    pub fn get(&self, scope: &Scope) -> Option<&Marker> {
        match scope {
            Scope::All => self.all.as_ref(),
            Scope::Course(c) => self.courses.get(c),
        }
    }

    /// Every reset in force, the whole curriculum first.
    pub fn scopes(&self) -> Vec<(Scope, &Marker)> {
        self.all
            .iter()
            .map(|m| (Scope::All, m))
            .chain(self.courses.iter().map(|(c, m)| (Scope::Course(c.clone()), m)))
            .collect()
    }

    /// The latest marker that covers `id` (an item, lesson or lecture id).
    pub fn cutoff(&self, id: &str) -> Option<&Marker> {
        let course = self.courses.get(course_of(id));
        match (self.all.as_ref(), course) {
            (Some(a), Some(c)) => Some(if c.hlc > a.hlc { c } else { a }),
            (a, c) => a.or(c),
        }
    }

    /// Whether an answer still counts as progress.
    pub fn counts(&self, e: &LoggedReview) -> bool {
        self.cutoff(&e.event.item_id).is_none_or(|m| e.hlc > m.hlc)
    }

    /// Whether something in `id`'s scope done at `at_ms` (a finished lesson,
    /// a lesson's started attempt) still counts.
    pub fn counts_at(&self, id: &str, at_ms: i64) -> bool {
        self.cutoff(id).is_none_or(|m| at_ms > m.at_ms())
    }

    /// Whether a finished paper at `at_ms` still shows.
    pub fn paper_counts(&self, at_ms: i64) -> bool {
        self.all.as_ref().is_none_or(|m| !m.papers || at_ms > m.at_ms())
    }

    /// The answers that still count, in the log's order.
    pub fn progress(&self, log: &[LoggedReview]) -> Vec<LoggedReview> {
        if self.is_empty() {
            return log.to_vec();
        }
        log.iter().filter(|e| self.counts(e)).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{Grade, Origin, ReviewEvent};

    fn hlc(wall_ms: i64, device: &str) -> Hlc {
        Hlc { wall_ms, counter: 0, device: device.into() }
    }

    fn answer(item: &str, wall_ms: i64, device: &str) -> LoggedReview {
        LoggedReview {
            id: format!("{device}-{wall_ms}"),
            hlc: hlc(wall_ms, device),
            event: ReviewEvent::new(item, Grade::Good, Origin::Study, wall_ms, 1000),
        }
    }

    fn resets(rows: &[(&str, i64)]) -> Resets {
        Resets::from_settings(rows.iter().map(|(k, at)| (k.to_string(), Marker::new(hlc(*at, "phone")))))
    }

    #[test]
    fn keys_round_trip_and_other_keys_are_skipped() {
        assert_eq!(Scope::All.key(), "reset.all");
        assert_eq!(Scope::Course("hb".into()).key(), "reset.hb");
        assert_eq!(Scope::from_key("reset.all"), Some(Scope::All));
        assert_eq!(Scope::from_key("reset.hb"), Some(Scope::Course("hb".into())));
        assert_eq!(Scope::from_key("reset."), None);
        assert_eq!(Scope::from_key("lesson.hb.L01.lesson1"), None);
        let r = Resets::from_settings([("other".to_string(), Marker::new(hlc(1, "p")))]);
        assert!(r.is_empty());
    }

    #[test]
    fn a_course_reset_leaves_out_only_that_courses_older_answers() {
        let r = resets(&[("reset.hb", 100)]);
        assert!(!r.counts(&answer("hb.L01.001", 99, "tablet")), "older, from an offline tablet");
        assert!(!r.counts(&answer("hb.L01.001", 100, "aaa")), "same wall, smaller device sorts first");
        assert!(r.counts(&answer("hb.L01.001", 101, "tablet")), "made after the reset");
        assert!(r.counts(&answer("hem.L01.001", 50, "tablet")), "another course");
        assert!(r.counts(&answer("own.0123456789abcdef", 50, "tablet")), "own cards only on all");
    }

    #[test]
    fn the_whole_curriculum_covers_own_cards_and_the_later_marker_wins() {
        let r = resets(&[("reset.all", 100), ("reset.hb", 200)]);
        assert!(!r.counts(&answer("own.0123456789abcdef", 50, "p")));
        assert!(!r.counts(&answer("hem.L01.001", 100, "p")));
        assert!(r.counts(&answer("hem.L01.001", 150, "q")));
        assert!(!r.counts(&answer("hb.L01.001", 150, "q")), "hb was reset again later");
        let r = resets(&[("reset.all", 300), ("reset.hb", 200)]);
        assert_eq!(r.cutoff("hb.L01.001").unwrap().at_ms(), 300);
    }

    #[test]
    fn lessons_and_places_follow_wall_time() {
        let r = resets(&[("reset.hb", 1_000)]);
        assert!(!r.counts_at("hb.L01.lesson1", 1_000));
        assert!(r.counts_at("hb.L01.lesson1", 1_001));
        assert!(r.counts_at("hem.L01.lesson1", 5));
    }

    #[test]
    fn papers_restart_only_when_the_whole_reset_says_so() {
        let mut r = resets(&[("reset.all", 1_000), ("reset.hb", 2_000)]);
        assert!(r.paper_counts(10));
        r.all.as_mut().unwrap().papers = true;
        assert!(!r.paper_counts(10));
        assert!(r.paper_counts(1_001));
    }

    #[test]
    fn progress_keeps_the_logs_order_and_no_reset_keeps_everything() {
        let log = vec![
            answer("hb.L01.001", 10, "p"),
            answer("hem.L01.001", 20, "p"),
            answer("hb.L01.001", 30, "p"),
        ];
        assert_eq!(Resets::default().progress(&log), log);
        let kept = resets(&[("reset.hb", 25)]).progress(&log);
        assert_eq!(kept.iter().map(|e| e.hlc.wall_ms).collect::<Vec<_>>(), [20, 30]);
    }

    #[test]
    fn a_marker_survives_json_with_its_catchup_plan() {
        let mut m = Marker::new(hlc(5, "phone"));
        m.papers = true;
        m.catchup = Some(serde_json::json!({ "days": 3 }));
        let back: Marker = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(back, m);
        let old: Marker =
            serde_json::from_str(r#"{"hlc":{"wall_ms":5,"counter":0,"device":"p"},"id":"x"}"#).unwrap();
        assert!(!old.papers && old.catchup.is_none());
    }
}
