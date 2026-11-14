//! Numbered marks on a figure's picture (#984, `batna_core::lesson::FigureMark`):
//! drawn over the picture in the theme's accent, numbered, with the labels
//! listed under it in the same numbers. Positions are percentages of the
//! picture's own box, so they stay on their spot at every width and when
//! the full-screen viewer zooms and pans (viewer.js scales the box).

use super::*;
use crate::i18n::marks as mx;
use batna_core::lesson::{FigureMark, MarkKind};

/// Whether a figure's marks start shown on this step: the player sets it
/// from `Lesson::marks_start_shown` (off before a question, so he looks
/// first and checks after).
#[derive(Clone, Copy)]
pub struct MarksStart(pub bool);

/// The marks over the picture. Hidden from screen readers: the list under
/// the picture reads them.
#[component]
pub(super) fn MarksLayer(marks: Vec<FigureMark>) -> Element {
    rsx! {
        span { class: "ls-marks-layer", aria_hidden: "true",
            for (i , m) in marks.into_iter().enumerate() {
                Mark { key: "{i}", n: i + 1, mark: m }
            }
        }
    }
}

#[component]
fn Mark(n: usize, mark: FigureMark) -> Element {
    let style = mark_style(&mark);
    let upright = if mark.upright() { "yes" } else { "no" };
    rsx! {
        span { class: "ls-mk", "data-kind": mark.kind.as_str(), "data-upright": upright, style: "{style}",
            match mark.kind {
                MarkKind::Point => rsx! { span { class: "ls-mk-ring" } },
                MarkKind::Arrow => rsx! { span { class: "ls-mk-shaft" } },
                MarkKind::Bracket => rsx! { span { class: "ls-mk-br" } },
                MarkKind::Box => rsx! {},
            }
            span { class: "ls-mk-n", "{n}" }
        }
    }
}

/// Where a mark sits, as percentages of the picture (the layer is left to
/// right), and which way its number and an arrow's tail point.
pub(crate) fn mark_style(m: &FigureMark) -> String {
    let pct = |v: f64| format!("{:.2}%", v.clamp(0.0, 1.0) * 100.0);
    let [x, y] = m.at;
    let (dx, dy) = m.outward();
    let angle = f64::from(dy).atan2(f64::from(dx)).to_degrees().round();
    let mut s = format!(
        "inset-inline-start: {}; inset-block-start: {}; --dx: {dx}; --dy: {dy}; --a: {angle}deg;",
        pct(x),
        pct(y)
    );
    if let (true, Some([w, h])) = (m.kind.has_size(), m.size) {
        s.push_str(&format!(" inline-size: {}; block-size: {};", pct(w), pct(h)));
    }
    s
}

/// The labels in the marks' numbers, read as a list.
#[component]
pub(super) fn MarksList(marks: Vec<FigureMark>) -> Element {
    rsx! {
        ol { class: "ls-marks-list", role: "list", aria_label: mx::LIST,
            for (i , m) in marks.into_iter().enumerate() {
                li { key: "{i}",
                    span { class: "ls-mk-n", aria_hidden: "true", "{i + 1}" }
                    bdi { dir: "ltr", lang: "en", "{m.label}" }
                }
            }
        }
    }
}

/// «ورّيني العلامات»: shows or hides the marks and their list (M).
#[component]
pub(super) fn MarksToggle(shown: Signal<bool>, #[props(default)] icon: bool) -> Element {
    let pressed = if shown() { "true" } else { "false" };
    rsx! {
        button {
            class: if icon { "ls-arrow ls-marks-toggle" } else { "btn btn-secondary ls-marks-toggle" },
            aria_pressed: pressed,
            "aria-keyshortcuts": "M",
            aria_label: if icon { Some(mx::SHOW) } else { None },
            title: mx::SHOW,
            onclick: move |_| shown.toggle(),
            span { class: "ls-marks-icon", aria_hidden: "true", "①" }
            if !icon { span { {mx::SHOW} } }
        }
    }
}

/// A picture with one mark of each kind, for the tests and the browser
/// fixtures (no lesson carries marks yet).
#[cfg(test)]
pub(crate) mod fixture {
    use super::*;

    pub fn marks() -> Vec<FigureMark> {
        serde_yaml_ng::from_str(
            r#"
- { at: [0.30, 0.25], label: "Point near the top left" }
- { at: [0.80, 0.70], label: "Arrow tip low on the right", kind: arrow }
- { at: [0.50, 0.50], size: [0.30, 0.20], label: "Box in the middle", kind: box }
- { at: [0.12, 0.60], size: [0.06, 0.40], label: "Bracket down the left", kind: bracket }
- { at: [0.65, 0.92], size: [0.40, 0.06], label: "Bracket along the bottom", kind: bracket }
"#,
        )
        .unwrap()
    }

    /// The figure as a lesson step shows it; `shown` is where the marks
    /// start (`MarksStart`), `zoomed` opens it in the full-screen viewer.
    #[component]
    pub fn MarkedFigure(image: Picture, shown: bool, #[props(default)] zoomed: bool) -> Element {
        use_context_provider(|| MarksStart(shown));
        rsx! {
            div { class: "ls-stage", "data-kind": "figure",
                Figure {
                    title: Some("Marks on a picture".to_string()),
                    svg: String::new(),
                    image: Some(image),
                    caption: "الشرح تحت الصورة.".to_string(),
                    alt: "A test picture.".to_string(),
                    marks: marks(),
                    zoomed,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
