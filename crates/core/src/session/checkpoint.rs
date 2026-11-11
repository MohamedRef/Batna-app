//! Versioned, device-local Study checkpoints. A checkpoint is only safe to
//! reopen when the review log still matches the state that wrote it.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::review::ReviewEvent;

use super::Session;

/// A checkpoint is local to this device, never a synced setting or review event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    version: u32,
    review_marker: String,
    session: Session,
}

impl SessionCheckpoint {
    pub const VERSION: u32 = 1;

    pub fn new(session: &Session, review_marker: String) -> Self {
        Self { version: Self::VERSION, review_marker, session: session.clone() }
    }

    /// Version prefix lets the app validate old checkpoints once, using the
    /// original full-log fingerprint, before writing new O(1) v2 markers.
    pub fn uses_legacy_review_marker(&self) -> bool {
        self.review_marker.starts_with("v1:")
    }

    /// Reject old/corrupt snapshots, missing/tombstoned grades and unplayable
    /// items rather than risking a second FSRS event after a crash.
    ///
    /// The marker is captured *after* the latest committed review. If a review
    /// committed but its new checkpoint did not, the marker differs at boot.
    pub fn recover(
        self,
        current_marker: &str,
        live_reviews: &HashMap<String, ReviewEvent>,
        playable: impl Fn(&str) -> bool,
    ) -> Option<Session> {
        (self.version == Self::VERSION
            && self.review_marker == current_marker
            && self.session.valid_for_restore(live_reviews, playable))
        .then_some(self.session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{Grade, Origin};
    use crate::session::Planned;

    fn plan(id: &str) -> Planned {
        Planned {
            item_id: id.into(),
            lecture: "hb.L01".into(),
            topic: "hb.L01/definition".into(),
            est_secs: 20,
            fresh: true,
        }
    }

    fn event(id: &str, grade: Grade) -> ReviewEvent {
        ReviewEvent::new(id, grade, Origin::Study, 42, 20)
    }

    #[test]
    fn checkpoint_round_trips_retry_order_summary_and_real_undo_event() {
        let mut session = Session::new(vec![plan("a"), plan("b")]);
        session.grade_logged(Grade::Again, Some("review-a".into()), |_| false);
        let logged = HashMap::from([("review-a".into(), event("a", Grade::Again))]);
        let raw = serde_json::to_string(&SessionCheckpoint::new(&session, "1:0".into())).unwrap();
        let copy: SessionCheckpoint = serde_json::from_str(&raw).unwrap();
        let mut restored = copy.recover("1:0", &logged, |_| true).unwrap();
        assert_eq!(restored.progress(), (1, 3));
        assert_eq!(restored.current().unwrap().item_id, "b");
        assert_eq!(restored.summary(), session.summary());
        assert_eq!(restored.undo().unwrap().event.as_deref(), Some("review-a"));
        assert_eq!(restored.progress(), (0, 2));
    }

    #[test]
    fn failed_or_stale_checkpoints_never_advance_or_duplicate_reviews() {
        let mut session = Session::new(vec![plan("a"), plan("b")]);
        session.grade_logged(Grade::Good, Some("review-a".into()), |_| false);
        let live = HashMap::from([("review-a".into(), event("a", Grade::Good))]);
        let checkpoint = SessionCheckpoint::new(&session, "1:0".into());
        assert!(checkpoint.clone().recover("2:0", &live, |_| true).is_none());
        assert!(checkpoint.clone().recover("1:0", &HashMap::new(), |_| true).is_none());
        assert!(checkpoint.clone().recover("1:0", &live, |id| id != "b").is_none());
        let mut wrong = checkpoint.clone();
        wrong.version += 1;
        assert!(wrong.recover("1:0", &live, |_| true).is_none());
        let mut altered = live.clone();
        altered.insert("review-a".into(), event("a", Grade::Easy));
        assert!(checkpoint.recover("1:0", &altered, |_| true).is_none());
    }

    #[test]
    fn unanswered_session_needs_no_replayed_grade() {
        let session = Session::new(vec![plan("a")]);
        assert!(
            SessionCheckpoint::new(&session, "empty".into())
                .recover("empty", &HashMap::new(), |_| true)
                .is_some()
        );
    }
}
