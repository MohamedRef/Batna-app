//! `try` steps: the lecture's own cards and questions, inside the lesson.
//! They use the study player's renderers and grade row, so an answer here is
//! a real review: it goes to the store and FSRS schedules the item.

use std::collections::HashMap;
use std::sync::OnceLock;

use batna_core::content::{Item, Lecture};
use batna_core::review::{Grade, Origin, ReviewEvent};
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::learn as tx;
use crate::study::{ItemView, PLAYABLE, use_store};

/// Every item by id, with its lecture.
pub fn find(id: &str) -> Option<(&'static Lecture, &'static Item)> {
    static MAP: OnceLock<HashMap<&'static str, (&'static Lecture, &'static Item)>> = OnceLock::new();
    MAP.get_or_init(|| {
        Library::get()
            .lectures
            .values()
            .flat_map(|l| l.items.iter().map(move |i| (i.id.as_str(), (l, i))))
            .collect()
    })
    .get(id)
    .copied()
}

#[component]
pub fn TryItems(items: Vec<String>, on_answer: EventHandler<bool>) -> Element {
    let store = use_store();
    // Kinds the study player can't show yet are left out, and hidden (✗)
    // items. The list is taken once: a ✗ he marks here hides the card from
    // the next visit on, so the position always points at the same list.
    let playable: Vec<String> = use_hook(|| {
        let store = store.0.borrow();
        items
            .iter()
            .filter(|id| find(id).is_some_and(|(_, i)| PLAYABLE.contains(&i.body.kind())))
            .filter(|id| !store.is_hidden(id))
            .cloned()
            .collect()
    });
    let n = playable.len();
    let mut at = use_signal(|| 0usize);
    let mut again = use_signal(|| 0usize);
    let mut started = use_signal(std::time::Instant::now);
    let mut reported = use_signal(|| false);
    use_effect(move || {
        if n == 0 && !reported() {
            reported.set(true);
            on_answer.call(true);
        }
    });

    let body = match playable.get(at()).and_then(|id| find(id)) {
        Some((lecture, item)) => {
            let item = store.0.borrow().effective(item);
            let fresh = store.0.borrow().is_new(&item.id);
            let topic_title = item
                .topic
                .as_ref()
                .and_then(|t| lecture.topics.iter().find(|x| &x.id == t))
                .map(|t| t.title.clone());
            let id = item.id.clone();
            let store = store.clone();
            let on_grade = move |grade: Grade| {
                store.0.borrow_mut().record(ReviewEvent {
                    item_id: id.clone(),
                    grade,
                    origin: Origin::Study,
                    at_ms: crate::store::now_ms(),
                    took_ms: started().elapsed().as_millis().min(u32::MAX as u128) as u32,
                });
                if grade == Grade::Again {
                    *again.write() += 1;
                }
                started.set(std::time::Instant::now());
                let next = at() + 1;
                at.set(next);
                if next == n && !reported() {
                    reported.set(true);
                    on_answer.call(again() == 0);
                }
            };
            rsx! {
                div { key: "{at()}-{item.id}", class: "ls-try-item",
                    p { class: "muted small", "{tx::TRY_OF} {at() + 1} {tx::OF} {n}" }
                    ItemView {
                        item,
                        lecture_title: lecture.title.clone(),
                        topic_title,
                        fresh,
                        retry: false,
                        on_grade,
                    }
                }
            }
        }
        None => rsx! {
            p { class: "ls-verdict good", {tx::TRY_DONE} }
            p { class: "muted small", {tx::TRY_DONE_HINT} }
        },
    };
    rsx! {
        div { class: "ls-try",
            p { class: "ls-kicker", {tx::TRY} }
            p { class: "muted small", {tx::TRY_HINT} }
            {body}
        }
    }
}
