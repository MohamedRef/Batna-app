//! «اتصلّحوا»: when a content update changes an item he marked ✗, the item
//! goes back into review and asks ✓ / ✎ / ✗ again (its mark no longer fits
//! the new text, `verify::Judgement::current`). Today says so once, opens
//! those items in a session, and «تمام» puts the note away (#1352).
//!
//! What he has seen is the synced setting `verify.fixed_seen`: item id → the
//! fingerprint of the fixed text. A later fix of the same item is new again.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::i18n::verify as tx;
use crate::store::Review;

pub const SEEN_KEY: &str = "verify.fixed_seen";

fn seen() -> BTreeMap<String, String> {
    crate::store::shared().setting(SEEN_KEY).unwrap_or_default()
}

/// Fixed ✗ items Today hasn't shown as fixed yet, in id order.
pub fn unseen(review: &Review) -> Vec<String> {
    let seen = seen();
    review
        .backend
        .borrow()
        .fixed_wrong()
        .into_iter()
        .filter(|(id, print)| seen.get(id) != Some(print))
        .map(|(id, _)| id)
        .collect()
}

/// Puts the note away for these items as they are written now.
fn mark_seen(review: &Review, ids: &[String]) {
    let mut seen = seen();
    for (id, print) in review.backend.borrow().fixed_wrong() {
        if ids.contains(&id) {
            seen.insert(id, print);
        }
    }
    if let Err(e) = crate::store::shared().put_setting(SEEN_KEY, &seen) {
        crate::diag::report!("fixed marks seen: {e}");
    }
}

/// Today's note; nothing when no fixed ✗ item is new.
#[component]
pub fn FixedNote() -> Element {
    let review = crate::store::use_review();
    // Re-read after «تمام»: the setting isn't a signal.
    let mut shown = use_signal(|| 0u32);
    let _ = shown();
    let ids = unseen(&review);
    if ids.is_empty() {
        return rsx! {};
    }
    let playable: Vec<String> = ids.iter().filter(|id| crate::study::is_playable(id)).cloned().collect();
    let ui = use_context::<crate::Ui>();
    rsx! {
        div { class: "note td-fixed", role: "status",
            p { {tx::fixed(ids.len())} }
            div { class: "verify-actions",
                if !playable.is_empty() {
                    button {
                        class: "btn btn-primary td-fixed-open",
                        onclick: {
                            let review = review.clone();
                            let ids = ids.clone();
                            move |_| {
                                mark_seen(&review, &ids);
                                let minutes = crate::onboarding::session_minutes();
                                *crate::study::START.write() =
                                    Some(crate::study::Start { lectures: None, items: Some(playable.clone()), minutes });
                                let mut screen = ui.screen;
                                screen.set(crate::Screen::Study);
                            }
                        },
                        {tx::FIXED_OPEN}
                    }
                }
                button {
                    class: "btn btn-secondary td-fixed-ok",
                    onclick: move |_| {
                        mark_seen(&review, &ids);
                        shown += 1;
                    },
                    {tx::FIXED_OK}
                }
            }
        }
    }
}
