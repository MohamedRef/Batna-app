//! Answers the review log refused (a full or broken disk). They are never
//! dropped quietly: the paper says so and offers to save them again, and each
//! one is logged exactly once when the store takes writes again.

use batna_core::review::{ReviewEvent, StudyStoreHandle};
use dioxus::prelude::*;

use super::{Logged, PHASE, Phase};
use crate::i18n::bank as tx;

/// One refused answer: the tutor question it belongs to (so a later reason
/// or Undo finds it once saved), and the event itself.
#[derive(Clone, Debug, PartialEq)]
pub struct Unsaved {
    pub at: Option<usize>,
    pub event: ReviewEvent,
}

pub static UNSAVED: GlobalSignal<Vec<Unsaved>> = Signal::global(Vec::new);

/// Logs `event`, or keeps it for [`retry`] when the store refuses. Returns
/// the logged event's id.
pub fn log(at: Option<usize>, event: ReviewEvent) -> Option<String> {
    let store = try_consume_context::<StudyStoreHandle>()?;
    let saved = store.0.borrow_mut().record_logged(event.clone());
    match saved {
        Ok(id) => id,
        Err(_) => {
            UNSAVED.write().push(Unsaved { at, event });
            None
        }
    }
}

/// Tries every kept answer again; the ones that save leave the list.
pub fn retry() {
    let Some(store) = try_consume_context::<StudyStoreHandle>() else { return };
    let pending = std::mem::take(&mut *UNSAVED.write());
    let mut left = Vec::new();
    for u in pending {
        let saved = store.0.borrow_mut().record_logged(u.event.clone());
        match saved {
            Ok(Some(event_id)) => {
                if let (Some(at), Phase::Running(p)) = (u.at, &mut *PHASE.write())
                    && p.at == at
                {
                    p.logged = Some(Logged { at, event_id, at_ms: u.event.at_ms });
                }
            }
            Ok(None) => {}
            Err(_) => left.push(u),
        }
    }
    *UNSAVED.write() = left;
}

/// Says when answers are waiting to be saved, with one button to retry.
#[component]
pub fn UnsavedNote() -> Element {
    let n = UNSAVED.read().len();
    if n == 0 {
        return rsx! {};
    }
    rsx! {
        div { class: "verdict bad bank-unsaved", role: "alert",
            p { {tx::UNSAVED} }
            button { class: "btn btn-secondary bank-unsaved-retry", onclick: move |_| retry(), {tx::UNSAVED_RETRY} }
        }
    }
}
