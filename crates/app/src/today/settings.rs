//! Settings › Review: desired retention and new items a day. The export of
//! his content checks and the backup rows live in Settings › Your data.

use batna_core::srs::{MIN_REVIEWS_TO_OPTIMIZE, NEW_PER_DAY_CHOICES, Retention, ReviewSettings};
use dioxus::prelude::*;

use crate::i18n::study_settings as tx;
use crate::store::use_review;

#[component]
pub fn StudySection() -> Element {
    let review = use_review();
    let settings = review.backend.borrow().settings.clone();
    let reviews = review.backend.borrow().reviews_logged();
    let personal = settings.fsrs.as_ref().map(|p| p.reviews);

    let set = {
        let review = review.clone();
        move |s: ReviewSettings| review.backend.borrow_mut().set_settings(s)
    };
    let set_retention = set.clone();
    let set_new = set;

    rsx! {
        style { {super::CSS} }
        section { class: "card",
            h2 { {tx::RETENTION} }
            p { class: "muted small", {tx::RETENTION_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::RETENTION,
                for r in Retention::ALL {
                    button {
                        key: "{r.percent()}",
                        role: "radio",
                        "aria-checked": settings.retention == r,
                        onclick: {
                            let s = settings.clone();
                            let f = set_retention.clone();
                            move |_| f(ReviewSettings { retention: r, ..s.clone() })
                        },
                        "{r.percent()}%"
                        span { class: "seg-hint", {retention_hint(r)} }
                    }
                }
            }
            p { class: "muted small",
                match personal {
                    Some(n) => rsx! { "{tx::PERSONAL_1} {n} {tx::PERSONAL_2}" },
                    None => rsx! { "{tx::DEFAULT_PARAMS_1} {MIN_REVIEWS_TO_OPTIMIZE} {tx::DEFAULT_PARAMS_2} {reviews}." },
                }
            }
        }
        section { class: "card",
            h2 { {tx::NEW_PER_DAY} }
            p { class: "muted small", {tx::NEW_PER_DAY_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::NEW_PER_DAY,
                for n in NEW_PER_DAY_CHOICES {
                    button {
                        key: "{n}",
                        role: "radio",
                        "aria-checked": settings.new_per_day == n,
                        onclick: {
                            let s = settings.clone();
                            let f = set_new.clone();
                            move |_| f(ReviewSettings { new_per_day: n, ..s.clone() })
                        },
                        "{n}"
                    }
                }
            }
        }
    }
}

fn retention_hint(r: Retention) -> &'static str {
    match r {
        Retention::R85 => tx::R85,
        Retention::R90 => tx::R90,
        Retention::R95 => tx::R95,
    }
}
