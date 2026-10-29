//! The Again / Hard / Good / Easy row every item shows once its answer is
//! out ("افتكرتها قد إيه؟"), with the real FSRS interval under each grade.
//! Shared by all renderers, so the content check above it and the intervals
//! look the same for every kind.

use batna_core::review::Grade;
use dioxus::prelude::*;

use super::backend::{now_ms, use_store};
use crate::i18n::study as tx;

const GRADES: [Grade; 4] = [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy];

#[component]
pub fn GradeRow(item_id: String, suggested: Option<Grade>, on_grade: EventHandler<Grade>) -> Element {
    let store = use_store();
    let intervals = store.0.borrow().intervals(&item_id, now_ms());
    let mut done = use_signal(|| false);
    // The answer is out: let the frame show the book quote.
    let revealed = try_use_context::<super::item::Revealed>();
    use_effect(move || {
        if let Some(super::item::Revealed(mut r)) = revealed {
            r.set(true);
        }
    });
    rsx! {
        // Two separate judgements: whether the content is right (its own
        // box, only while the item is unchecked), then how well he
        // remembered it (the rating row, always in the same place).
        crate::verify::VerifyBar { item_id: item_id.clone() }
        div { class: "st-grades", role: "group", aria_labelledby: "st-grade-label",
            p { class: "st-grade-head",
                strong { id: "st-grade-label", {tx::GRADE_LABEL} }
                span { class: "st-suggest muted small",
                    {if suggested.is_some() { tx::SUGGESTED } else { tx::GRADE_HINT }}
                }
            }
            div { class: "st-grade-row",
                for (i , grade) in GRADES.into_iter().enumerate() {
                    button {
                        key: "{i}",
                        class: "st-grade",
                        "aria-keyshortcuts": "{i + 1}",
                        "data-grade": grade_name(grade),
                        "data-suggested": if suggested == Some(grade) { "yes" } else { "no" },
                        disabled: done(),
                        onclick: move |_| {
                            if !done() {
                                done.set(true);
                                on_grade.call(grade);
                            }
                        },
                        span { class: "st-grade-name", {grade_label(grade)} }
                        if let Some(days) = intervals.map(|v| v[i]) {
                            span { class: "st-grade-when small", {interval_label(days)} }
                        }
                    }
                }
            }
        }
    }
}

fn grade_name(g: Grade) -> &'static str {
    match g {
        Grade::Again => "again",
        Grade::Hard => "hard",
        Grade::Good => "good",
        Grade::Easy => "easy",
    }
}

pub fn grade_label(g: Grade) -> &'static str {
    match g {
        Grade::Again => tx::AGAIN,
        Grade::Hard => tx::HARD,
        Grade::Good => tx::GOOD,
        Grade::Easy => tx::EASY,
    }
}

/// "10 دقايق", "5 ساعات", "يومين", "3 شهور": when this grade brings it back.
fn interval_label(days: f32) -> String {
    use crate::i18n::count::{DAY, HOUR, MINUTE, MONTH, count};
    let n = |x: f32| x.round().max(1.0) as u64;
    if days < 1.0 / 24.0 {
        count(n(days * 24.0 * 60.0), &MINUTE)
    } else if days < 1.0 {
        count(n(days * 24.0), &HOUR)
    } else if days < 60.0 {
        count(n(days), &DAY)
    } else {
        count(n(days / 30.0), &MONTH)
    }
}
