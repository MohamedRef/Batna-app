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

/// Set once the item's answer is out (its `GradeRow` is shown), so the frame
/// can keep the book quote, which often is the answer, hidden until then.
#[derive(Clone, Copy)]
pub struct Revealed(pub Signal<bool>);

/// The item came back through Undo: he has seen its answer already, so a
/// recall card opens with it shown.
#[derive(Clone, Copy)]
pub struct Reopened(pub bool);

/// The study player logs confidence and reasons for this item (other
/// screens that show an `ItemView` don't): the frame offers the confidence
/// chips, and the rating row the reason chips.
#[derive(Clone, Copy)]
pub struct Tagging(pub bool);

/// Item kinds this player can show; the session composer picks only these.
/// A chunk that adds a renderer adds its kind here and an arm below.
pub const PLAYABLE: &[&str] = &["card", "cloze", "enumerate", "essay", "table", "order"];

#[component]
pub fn ItemView(
    item: Item,
    lecture_title: String,
    topic_title: Option<String>,
    fresh: bool,
    retry: bool,
    #[props(default)] reopened: bool,
    #[props(default)] tagging: bool,
    on_grade: EventHandler<Grade>,
) -> Element {
    let revealed = use_context_provider(|| Revealed(Signal::new(false))).0;
    use_context_provider(|| Reopened(reopened));
    use_context_provider(|| Tagging(tagging));
    // Every item starts with no taps (the item is keyed, so this runs per item).
    use_hook(|| {
        if tagging {
            *super::TAGS.write() = super::Tags::default();
        }
    });
    let confidence = super::TAGS.read().confidence;
    let badge = if retry {
        tx::BADGE_RETRY
    } else if fresh {
        tx::BADGE_NEW
    } else {
        tx::BADGE_REVIEW
    };
    rsx! {
        crate::ui::Card { element: crate::ui::CardElement::Article, class: "st-item", tabindex: "-1", "data-kind": item.body.kind(),
            // Screen readers only: a heading to jump to for each item; the
            // chips below show the same on screen.
            h1 { class: "st-sr-only", "{kind_label(&item.body)} · {lecture_title}" }
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
            // Before the answer is out: how sure is he? One optional tap.
            if tagging && !revealed() && !reopened {
                crate::metacog::ConfidencePicker {
                    value: confidence,
                    on_pick: move |c| super::TAGS.write().confidence = c,
                }
            }
            footer { class: "st-item-foot",
                crate::learn::ExplainLink { item_id: item.id.clone() }
                // His note stays out of sight until the answer is (it may hold it).
                crate::notes::NoteChip {
                    anchor: batna_core::note::Anchor::Item(item.id.clone()),
                    page: (item.src.page > 0).then_some(item.src.page),
                    show_text: revealed(),
                }
                // His own cards may have no page.
                if item.src.page > 0 {
                    crate::hover::BookPage {
                        page: item.src.page,
                        quote: if revealed() { item.src.quote.clone() } else { None },
                        held: !revealed() && item.src.quote.is_some(),
                        of: item.id.clone(),
                        hook: match &item.body {
                            batna_core::content::Body::Enumerate { hook: Some(h), .. } => {
                                Some(crate::pagesheet::Hook { text: h.clone(), parts: Vec::new() })
                            }
                            _ => None,
                        },
                    }
                }
                if let Some(q) = item.src.quote.as_ref().filter(|_| revealed()) {
                    details { class: "st-quote",
                        summary { {tx::BOOK_QUOTE} }
                        crate::pagesheet::Quote { text: q.clone() }
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
        Body::Table { .. } => rsx! { crate::table::TableItem { item, on_grade } },
        Body::Order { .. } => rsx! { crate::order::OrderItem { item, on_grade } },
        // Not in PLAYABLE yet: other chunks bring these screens.
        _ => rsx! {
            p { class: "muted", {tx::NOT_HERE_YET} }
            button { class: "btn btn-secondary btn-lg", onclick: move |_| on_grade.call(Grade::Good), {tx::SKIP} }
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

    fn first(kind: &str) -> &'static Item {
        crate::content::Library::get()
            .lectures
            .values()
            .flat_map(|l| &l.items)
            .find(|i| i.body.kind() == kind)
            .unwrap_or_else(|| panic!("no {kind} item"))
    }

    #[test]
    fn every_kind_has_a_label_and_a_one_line_name() {
        let kinds = ["card", "cloze", "enumerate", "essay", "mcq", "case", "table", "order", "calc"];
        let mut labels = std::collections::HashSet::new();
        for kind in kinds {
            let item = first(kind);
            assert!(labels.insert(kind_label(&item.body)), "{kind} has its own label");
            let line = short_text(item);
            assert!(!line.is_empty() && !line.contains('\n') && !line.contains("{{"), "{kind}: {line}");
            assert!(line.chars().count() <= 89, "{kind}: {line}");
            let head = match &item.body {
                Body::Card { front, .. } => front,
                Body::Mcq { stem, .. } | Body::Case { stem, .. } | Body::Calc { stem, .. } => stem,
                Body::Enumerate { prompt, .. } | Body::Essay { prompt, .. } | Body::Order { prompt, .. } => {
                    prompt
                }
                Body::Table { title, .. } => title,
                Body::Cloze { .. } => continue,
            };
            let words: String = head.split_whitespace().collect::<Vec<_>>().join(" ");
            let shown = line.trim_end_matches('…');
            assert!(words.starts_with(shown), "{kind}: {line:?} starts {words:?}");
        }
        let long: Item = serde_yaml_ng::from_str(&format!(
            "{{ id: x.2, kind: card, front: '{}', back: b, src: {{ page: 1 }}, status: unverified }}",
            "word ".repeat(40)
        ))
        .unwrap();
        let line = short_text(&long);
        assert_eq!(line.chars().count(), 89);
        assert!(line.ends_with('…'));
    }

    #[test]
    fn an_item_this_player_cannot_show_offers_skip() {
        use std::cell::Cell;
        thread_local! {
            static GOT: Cell<Option<Grade>> = const { Cell::new(None) };
        }
        fn app() -> Element {
            use_context_provider(|| crate::Ui {
                screen: Signal::new(crate::Screen::Study),
                prefs: Signal::new(crate::prefs::DevicePrefs::default()),
                focus: Signal::new(false),
            });
            use_context_provider(crate::store::test_review);
            rsx! {
                ItemView {
                    item: first("mcq").clone(),
                    lecture_title: "Liver".to_string(),
                    topic_title: Some("Child-Pugh".to_string()),
                    fresh: false,
                    retry: false,
                    on_grade: move |g| GOT.set(Some(g)),
                }
            }
        }
        crate::store::use_memory_store();
        let mut t = crate::testdom::TestDom::new(app);
        assert!(t.text().contains(tx::NOT_HERE_YET));
        assert_eq!(t.attr("span.st-badge", "data-badge").as_deref(), Some("review"));
        assert_eq!(t.texts("p.st-where"), ["Liver · Child-Pugh"]);
        assert!(t.texts("h1.st-sr-only")[0].starts_with(tx::KIND_MCQ));
        t.click_text("button", tx::SKIP);
        assert_eq!(GOT.get(), Some(Grade::Good));
    }
}
