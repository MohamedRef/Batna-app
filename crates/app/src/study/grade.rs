//! The Again / Hard / Good / Easy row every item shows once its answer is
//! out. Shared by all renderers, so the verify prompt and the intervals look
//! the same for every kind.

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
    rsx! {
        crate::verify::VerifyBar { item_id: item_id.clone() }
        div { class: "st-grades", role: "group", aria_label: tx::GRADE_LABEL,
            if suggested.is_some() {
                p { class: "st-suggest muted small", {tx::SUGGESTED} }
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

/// "١٠ د", "٣ ي", "٢ ش": short enough to sit under the button.
fn interval_label(days: f32) -> String {
    if days < 1.0 / 24.0 {
        format!("{} {}", (days * 24.0 * 60.0).round().max(1.0), tx::UNIT_MIN)
    } else if days < 1.0 {
        format!("{} {}", (days * 24.0).round(), tx::UNIT_HOUR)
    } else if days < 60.0 {
        format!("{} {}", days.round(), tx::UNIT_DAY)
    } else {
        format!("{} {}", (days / 30.0).round(), tx::UNIT_MONTH)
    }
}
