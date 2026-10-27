//! Settings → review: desired retention, new items a day, and the export of
//! his content checks. Other chunks can add their own sections here.

use batna_core::srs::{MIN_REVIEWS_TO_OPTIMIZE, NEW_PER_DAY_CHOICES, Retention, ReviewSettings};
use dioxus::prelude::*;

use crate::i18n::settings as tx;
use crate::store::{self, use_review};
use crate::verify::ExportVerdicts;

const CSS: &str = include_str!("today.css");

#[component]
pub fn Settings() -> Element {
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
        style { {CSS} }
        style { {crate::verify::CSS} }
        section { class: "card",
            h1 { {tx::TITLE} }
        }
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
        ExportVerdicts {}
        crate::data_export::ExportCard {}
        crate::import::ImportCard {}
        section { class: "card",
            h2 { {tx::DATA} }
            p { class: "muted small",
                match &store::shared().path {
                    Some(p) => rsx! { "{tx::DATA_AT} " bdi { dir: "ltr", "{p.display()}" } },
                    None => rsx! { {tx::DATA_MEMORY} },
                }
            }
            p { class: "muted small", "{tx::DEVICE} " bdi { dir: "ltr", "{store::shared().device()}" } }
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
