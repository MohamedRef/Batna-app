//! The frame around every item: what kind it is, where it comes from in the
//! book, a guideline-update note when there is one, and the renderer.

use batna_core::content::{Body, Item, Status};
use batna_core::review::Grade;
use dioxus::prelude::*;

use super::card::CardItem;
use super::cloze::ClozeItem;
use super::enumerate::EnumerateItem;
use super::essay::EssayItem;
use crate::i18n::study as tx;

/// Item kinds this player can show; the session composer picks only these.
/// A chunk that adds a renderer adds its kind here and an arm below.
pub const PLAYABLE: &[&str] = &["card", "cloze", "enumerate", "essay"];

#[component]
pub fn ItemView(
    item: Item,
    lecture_title: String,
    topic_title: Option<String>,
    fresh: bool,
    retry: bool,
    on_grade: EventHandler<Grade>,
) -> Element {
    let badge = if retry {
        tx::BADGE_RETRY
    } else if fresh {
        tx::BADGE_NEW
    } else {
        tx::BADGE_REVIEW
    };
    rsx! {
        article { class: "card st-item", "data-kind": item.body.kind(),
            header { class: "st-item-head",
                span { class: "chip st-badge", "data-badge": if retry { "retry" } else if fresh { "new" } else { "review" }, {badge} }
                span { class: "chip", {kind_label(&item.body)} }
                if item.status == Status::Unverified {
                    span { class: "chip st-unverified", title: tx::UNVERIFIED_HINT, {tx::UNVERIFIED} }
                }
            }
            p { class: "st-where muted small",
                bdi {
                    match &topic_title {
                        Some(t) => format!("{lecture_title} · {t}"),
                        None => lecture_title.clone(),
                    }
                }
            }
            div { class: "st-body",
                {render(&item, on_grade)}
            }
            footer { class: "st-item-foot",
                span { class: "st-page", "{tx::BOOK_PAGE} " bdi { "{item.src.page}" } }
                if let Some(q) = &item.src.quote {
                    details { class: "st-quote",
                        summary { {tx::BOOK_QUOTE} }
                        q { lang: "en", dir: "ltr", "{q}" }
                    }
                }
            }
            if let Some(update) = &item.update {
                details { class: "st-update",
                    summary {
                        strong { {tx::UPDATE_TITLE} }
                        span { class: "muted small", " · {tx::UPDATE_AFTER}" }
                    }
                    p { lang: "en", dir: "ltr", "{update.text}" }
                    p { class: "muted small", lang: "en", dir: "ltr", "{update.reference}" }
                    p { class: "small", {tx::UPDATE_BOOK_WINS} }
                }
            }
        }
    }
}

fn render(item: &Item, on_grade: EventHandler<Grade>) -> Element {
    let item = item.clone();
    match &item.body {
        Body::Card { .. } => rsx! { CardItem { item, on_grade } },
        Body::Cloze { .. } => rsx! { ClozeItem { item, on_grade } },
        Body::Enumerate { .. } => rsx! { EnumerateItem { item, on_grade } },
        Body::Essay { .. } => rsx! { EssayItem { item, on_grade } },
        // Not in PLAYABLE yet: other chunks bring these screens.
        _ => rsx! {
            p { class: "muted", {tx::NOT_HERE_YET} }
            button { class: "st-secondary", onclick: move |_| on_grade.call(Grade::Good), {tx::SKIP} }
        },
    }
}

fn kind_label(body: &Body) -> &'static str {
    match body {
        Body::Card { .. } => tx::KIND_CARD,
        Body::Cloze { .. } => tx::KIND_CLOZE,
        Body::Enumerate { .. } => tx::KIND_ENUMERATE,
        Body::Essay { .. } => tx::KIND_ESSAY,
        Body::Mcq { .. } => tx::KIND_MCQ,
        Body::Case { .. } => tx::KIND_CASE,
        Body::Table { .. } => tx::KIND_TABLE,
        Body::Order { .. } => tx::KIND_ORDER,
        Body::Calc { .. } => tx::KIND_CALC,
    }
}

/// One line that names an item in a list (the summary's "coming back" list).
pub fn short_text(item: &Item) -> String {
    let s = match &item.body {
        Body::Card { front, .. } => front.clone(),
        Body::Cloze { text, .. } => {
            let mut out = String::new();
            let mut rest = text.as_str();
            while let (Some(a), Some(b)) = (rest.find("{{"), rest.find("}}")) {
                if b < a {
                    break;
                }
                out.push_str(&rest[..a]);
                out.push_str("___");
                rest = &rest[b + 2..];
            }
            out.push_str(rest);
            out
        }
        Body::Mcq { stem, .. } | Body::Case { stem, .. } | Body::Calc { stem, .. } => stem.clone(),
        Body::Enumerate { prompt, .. } | Body::Essay { prompt, .. } | Body::Order { prompt, .. } => {
            prompt.clone()
        }
        Body::Table { title, .. } => title.clone(),
    };
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() > 90 { format!("{}…", s.chars().take(88).collect::<String>()) } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_hides_cloze_answers() {
        let item: Item = serde_yaml_ng::from_str(
            "{ id: x.1, kind: cloze, text: 'Class A = {{c1::5-6}} points', src: { page: 1 }, status: unverified }",
        )
        .unwrap();
        assert_eq!(short_text(&item), "Class A = ___ points");
    }
}
