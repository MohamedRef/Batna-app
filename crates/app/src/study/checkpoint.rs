//! Cold-process recovery for Study. Keep the snapshot on this device only;
//! it must not become a synced setting or replay review events on restoration.

use std::collections::HashMap;

use batna_core::session::{Session, SessionCheckpoint};
use dioxus::prelude::*;
use web_time::Instant;

use super::{PHASE, Phase, SESSION, STARTED, StudyStoreHandle};

const KEY: &str = "study.session.v1";

/// Called once by the app shell, after providing the real review backend, so
/// Today can display Continue on the very first screen after restarting.
pub(crate) fn use_restore() {
    let backend = super::use_store();
    use_hook(move || {
        if *PHASE.peek() != Phase::Setup || SESSION.peek().current().is_some() {
            return;
        }
        if let Some(saved) = load(&backend) {
            *SESSION.write() = saved;
            *PHASE.write() = Phase::Running;
            *STARTED.write() = Instant::now();
        }
    });
}

fn load(backend: &StudyStoreHandle) -> Option<Session> {
    let db = crate::store::shared();
    let checkpoint = db.local_setting::<SessionCheckpoint>(KEY)?;
    // Old checkpoints stay recoverable after the revision-trigger migration:
    // validate their *original* full-log marker once, then save v2 on the
    // next transition. Never weaken validation to a count-only comparison.
    let marker = if checkpoint.uses_legacy_review_marker() {
        db.study_review_marker_legacy().ok()?
    } else {
        db.study_review_marker().ok()?
    };
    let live = db.reviews().ok()?.into_iter().map(|entry| (entry.id, entry.event)).collect::<HashMap<_, _>>();
    let recovered =
        checkpoint.recover(&marker, &live, |id| super::is_playable(id) && !backend.0.borrow().is_hidden(id));
    if recovered.is_none() {
        clear();
    }
    recovered
}

/// Save the *already advanced* queue. We never checkpoint between a review
/// commit and its matching Session::grade_logged change: if a crash lands in
/// that gap, the previous marker will fail closed on the next launch.
pub(super) fn save_running(session: &Session) {
    let db = crate::store::shared();
    let result = db
        .study_review_marker()
        .and_then(|marker| db.put_local_setting(KEY, &SessionCheckpoint::new(session, marker)));
    if let Err(error) = result {
        crate::diag::report!("saving Study checkpoint: {error}");
    }
}

pub(super) fn clear() {
    if let Err(error) = crate::store::shared().put_local_setting(KEY, &Option::<SessionCheckpoint>::None) {
        crate::diag::report!("clearing Study checkpoint: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use batna_core::review::{Grade, Origin, ReviewEvent};
    use batna_core::session::Planned;

    fn memory_backend() -> StudyStoreHandle {
        StudyStoreHandle(Rc::new(RefCell::new(super::super::backend::MemoryStore::default())))
    }

    fn planned(id: &str) -> Planned {
        Planned {
            item_id: id.to_string(),
            lecture: "hb.L01".into(),
            topic: "hb.L01/definition".into(),
            est_secs: 20,
            fresh: true,
        }
    }

    #[test]
    fn sqlite_restart_loads_exact_queue_and_keeps_undo_event_id() {
        let db = crate::store::use_thread_store();
        let card = super::super::flow::first("card");
        let cloze = super::super::flow::first("cloze");
        let mut session = Session::new(vec![planned(&card.id), planned(&cloze.id)]);
        save_running(&session);
        let backend = memory_backend();
        assert_eq!(load(&backend).unwrap().progress(), (0, 2));

        // Grading saves the review first, then its corresponding checkpoint.
        let logged = db
            .append_review(ReviewEvent {
                confidence: None,
                reason: None,
                item_id: card.id.clone(),
                grade: Grade::Again,
                origin: Origin::Study,
                at_ms: 100,
                took_ms: 20,
            })
            .unwrap();
        session.grade_logged(Grade::Again, Some(logged.id.clone()), |_| false);
        save_running(&session);

        // Simulate a fresh loader reading SQLite rather than any global SESSION.
        let mut restored = load(&backend).expect("committed review and snapshot agree");
        assert_eq!(restored.progress(), (1, 3), "Again retry survives");
        assert_eq!(restored.current().unwrap().item_id, cloze.id);
        assert_eq!(restored.undo().unwrap().event.as_deref(), Some(logged.id.as_str()));

        clear();
        assert!(load(&backend).is_none(), "ended session should not reopen");
    }

    #[test]
    fn legacy_v1_checkpoint_recovers_once_and_new_save_uses_v2() {
        let db = crate::store::use_thread_store();
        let card = super::super::flow::first("card");
        let session = Session::new(vec![planned(&card.id)]);
        let original_marker = db.study_review_marker_legacy().unwrap();
        db.put_local_setting(KEY, &SessionCheckpoint::new(&session, original_marker)).unwrap();

        let backend = memory_backend();
        assert!(load(&backend).is_some(), "v1 checkpoint should validate against the original log");
        save_running(&session);
        let modern = db.local_setting::<SessionCheckpoint>(KEY).unwrap();
        assert!(!modern.uses_legacy_review_marker(), "next checkpoint writes v2 revision");
        assert!(load(&backend).is_some(), "v2 checkpoint also recovers");
        clear();
    }

    #[test]
    fn crash_after_review_commit_before_snapshot_fails_closed() {
        let db = crate::store::use_thread_store();
        let card = super::super::flow::first("card");
        let session = Session::new(vec![planned(&card.id)]);
        save_running(&session);
        db.append_review(ReviewEvent {
            confidence: None,
            reason: None,
            item_id: card.id.clone(),
            grade: Grade::Good,
            origin: Origin::Study,
            at_ms: 100,
            took_ms: 20,
        })
        .unwrap();

        // The old snapshot must not reopen the just-graded item or emit a
        // second review event. Existing committed reviews stay untouched.
        assert!(load(&memory_backend()).is_none());
        assert_eq!(db.reviews().unwrap().len(), 1);
        assert!(db.local_setting::<SessionCheckpoint>(KEY).is_none());
    }
}
