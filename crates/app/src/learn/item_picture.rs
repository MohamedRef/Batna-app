//! A book figure on a card, question or case (#1154): the same picture as
//! the lesson figure (zoom, full-screen viewer), shown above the question.
//! Its marks stay off until the answer is out, so he finds the sign first.

use batna_core::content::Item;
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::learn as tx;

/// The item's picture, or nothing when it has none. `answered` brings the
/// marks (and their list) in.
#[component]
pub fn ItemPicture(item: Item, answered: bool) -> Element {
    let Some(image) = Library::get().item_picture(&item) else { return rsx! {} };
    let has_marks = !item.marks.is_empty();
    let marks = if answered { item.marks.clone() } else { Vec::new() };
    let caption = if has_marks && !answered { tx::ITEM_PICTURE_HINT } else { "" };
    rsx! {
        style { {crate::learn::CSS.text()} }
        div { class: "st-picture", "data-answered": if answered { "yes" } else { "no" },
            // Keyed: the marks come in shown once the answer is out.
            super::steps::Figure {
                key: "{answered}",
                title: None,
                svg: String::new(),
                image: Some(image),
                caption: caption.to_string(),
                alt: tx::ITEM_PICTURE.to_string(),
                marks,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use batna_core::content::{Body, Item, Source, Status};
    use batna_core::lesson::{FigureMark, MarkKind, Picture, StepBody};
    use dioxus::prelude::*;

    use super::ItemPicture;
    use crate::content::Library;

    /// A card on the first book picture of the built-in lessons.
    fn picture_card() -> Item {
        let (lecture, file) = Library::get()
            .lessons
            .values()
            .flat_map(|f| f.lessons.iter().flat_map(|l| &l.steps).map(move |s| (f, s)))
            .find_map(|(f, s)| match &s.body {
                StepBody::Figure { image: Some(p), .. } if p.book => {
                    Some((f.lecture.clone(), p.file.clone()))
                }
                _ => None,
            })
            .expect("a book picture in the lessons");
        Item {
            id: format!("{lecture}.999"),
            body: Body::Card { front: "Name the sign.".into(), back: "The sign.".into() },
            topic: None,
            src: Source { page: 1, quote: None },
            status: Status::Unverified,
            update: None,
            tags: Vec::new(),
            image: Some(Picture { file, book: true, ..Picture::default() }),
            marks: vec![FigureMark {
                at: [0.5, 0.5],
                label: "Here".into(),
                kind: MarkKind::Point,
                size: None,
            }],
        }
    }

    fn html(item: Item, answered: bool) -> String {
        fn app(p: (Item, bool)) -> Element {
            rsx! { ItemPicture { item: p.0, answered: p.1 } }
        }
        let mut dom = VirtualDom::new_with_props(app, (item, answered));
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn the_picture_shows_and_its_marks_wait_for_the_answer() {
        let item = picture_card();
        let before = html(item.clone(), false);
        assert!(before.contains("st-picture") && before.contains("src=\"data:image/"), "{before}");
        assert!(!before.contains("class=\"ls-marks-layer\"") && !before.contains(">Here<"), "marks wait");
        assert!(before.contains(crate::i18n::learn::ITEM_PICTURE_HINT));
        let after = html(item.clone(), true);
        assert!(after.contains("class=\"ls-marks-layer\"") && after.contains("Here"), "{after}");
        // No picture, or one the lessons don't have: nothing at all.
        let none = html(Item { image: None, ..item.clone() }, false);
        assert!(!none.contains("st-picture"));
        let gone = Item {
            image: Some(Picture { file: "hb/none.webp".into(), book: true, ..Picture::default() }),
            ..item
        };
        assert!(!html(gone, false).contains("st-picture"));
    }
}
