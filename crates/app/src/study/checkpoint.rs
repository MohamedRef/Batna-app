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
    let marker = db.study_review_marker().ok()?;
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
