//! Marks on a figure's picture (#984), tapped the way he uses them.

use super::fixture::{MarkedFigure, MarkedFigureProps, marks};
use super::*;
use crate::test_dom::{El, Screen as Dom, shows};

fn picture() -> Picture {
    Picture {
        file: "hb/test.png".into(),
        book: true,
        data: "data:image/png;base64,AA==".into(),
        ..Picture::default()
    }
}

fn open(shown: bool) -> Dom {
    Dom::with_props(MarkedFigure, MarkedFigureProps { image: picture(), shown, zoomed: false })
}

fn toggle(e: &El) -> bool {
    e.has_class("ls-marks-toggle")
}

fn pressed(d: &Dom) -> Vec<String> {
    d.find(toggle).iter().map(|e| e.attr("aria-pressed").unwrap_or_default().to_string()).collect()
}

#[test]
fn marks_draw_numbered_with_their_labels_listed_in_the_same_numbers() {
    let d = open(true);
    let m = marks();
    assert_eq!(d.find(|e| e.has_class("ls-mk")).len(), m.len());
    let kinds: Vec<String> =
        d.find(|e| e.has_class("ls-mk")).iter().map(|e| e.attr("data-kind").unwrap().to_string()).collect();
    assert_eq!(kinds, ["point", "arrow", "box", "bracket", "bracket"]);
    // The layer is for the eye; the list is what a screen reader reads.
    let layer = d.find(|e| e.has_class("ls-marks-layer")).remove(0);
    assert_eq!(layer.attr("aria-hidden"), Some("true"));
    let list = d.find(|e| e.has_class("ls-marks-list")).remove(0);
    assert_eq!(list.attr("aria-label"), Some(crate::i18n::marks::LIST));
    let items = d.find_in(|e| e.has_class("ls-marks-list"), |e| e.tag == "li");
    for (i, (li, mark)) in items.iter().zip(&m).enumerate() {
        assert!(li.text.starts_with(&(i + 1).to_string()) && li.text.contains(&mark.label), "{}", li.text);
    }
    let html = d.html();
    assert!(html.contains(r#"<bdi dir="ltr" lang="en">Point near the top left</bdi>"#), "{html}");
    // The picture keeps its alt for the whole picture.
    assert!(html.contains(r#"alt="A test picture.""#));
    // Positions are percentages of the picture, left to right.
    assert!(html.contains("inset-inline-start: 30.00%; inset-block-start: 25.00%;"), "{html}");
    assert!(html.contains("inline-size: 30.00%; block-size: 20.00%;"));
}

#[test]
fn the_toggle_hides_and_shows_the_marks_and_their_list() {
    let mut d = open(true);
    assert_eq!(pressed(&d), ["true"]);
    let t = d.find(toggle).remove(0);
    assert_eq!(t.attr("aria-keyshortcuts"), Some("M"));
    assert!(t.text.contains(crate::i18n::marks::SHOW));
    d.click(toggle);
    assert_eq!(pressed(&d), ["false"]);
    assert!(!shows(&d, "ls-marks-layer") && !shows(&d, "ls-marks-list"));
    assert!(shows(&d, "ls-marked"), "the picture stays");
    d.click(toggle);
    assert!(shows(&d, "ls-marks-layer") && shows(&d, "ls-marks-list"));
}

#[test]
fn before_a_question_the_marks_start_hidden() {
    let mut d = open(false);
    assert_eq!(pressed(&d), ["false"]);
    assert!(!shows(&d, "ls-mk") && !shows(&d, "ls-marks-list"));
    d.click(toggle);
    assert_eq!(d.find(|e| e.has_class("ls-mk")).len(), marks().len());
}

#[test]
fn the_full_screen_view_has_the_marks_and_its_own_toggle() {
    let mut d = open(true);
    d.click(|e| e.has_class("ls-svg"));
    assert!(shows(&d, "ls-viewer"));
    let in_viewer =
        |d: &Dom, class: &'static str| d.find_in(|e| e.has_class("ls-viewer"), |e| e.has_class(class));
    assert_eq!(in_viewer(&d, "ls-mk").len(), marks().len());
    assert_eq!(in_viewer(&d, "ls-marks-list").len(), 1);
    let t = in_viewer(&d, "ls-marks-toggle").remove(0);
    assert_eq!(t.attr("aria-label"), Some(crate::i18n::marks::SHOW));
    assert_eq!(t.attr("aria-keyshortcuts"), Some("M"));
    d.click_in(|e| e.has_class("ls-viewer"), toggle);
    assert!(in_viewer(&d, "ls-mk").is_empty(), "hidden in the viewer");
    assert!(!shows(&d, "ls-mk"), "and in the lesson behind it: one switch");
    assert_eq!(pressed(&d), ["false", "false"]);
}

#[test]
fn a_figure_without_marks_or_a_drawn_one_has_no_toggle() {
    let d = Dom::new(|| {
        rsx! {
            Figure {
                title: None,
                svg: r#"<svg viewBox="0 0 1 1"></svg>"#.to_string(),
                image: None,
                caption: "c".to_string(),
                alt: "a".to_string(),
                marks: marks(),
            }
            Figure { title: None, svg: String::new(), image: Some(picture()), caption: "c".to_string(), alt: "a".to_string() }
        }
    });
    assert!(!shows(&d, "ls-marks-toggle") && !shows(&d, "ls-mk") && !shows(&d, "ls-marked"));
}
