//! Marks on a figure's picture (#984): numbered places on a book crop or an
//! open-licence image that the lesson points at ("1: the Mallory body"), so
//! the step doesn't have to say "the arrow at the top right" in words.
//!
//! ```yaml
//! - kind: figure
//!   image: { file: hb/L05-biopsy.webp, book: true }
//!   marks:
//!     - { at: [0.62, 0.30], label: "Mallory-Denk body" }
//!     - { at: [0.20, 0.75], label: "Neutrophils", kind: arrow }
//!     - { at: [0.50, 0.50], size: [0.30, 0.20], label: "Portal tract", kind: box }
//! ```
//!
//! `at` is a fraction of the picture's width and height from its top left,
//! so a mark lands on the same spot at every screen size and zoom. The app
//! draws them over the picture, numbered, and lists the labels under it in
//! the same numbers (`crates/app/src/learn/steps/marks.rs`).

use serde::{Deserialize, Serialize};

/// At most this many marks on one picture: more and they hide it.
pub const MAX_MARKS: usize = 8;

/// One numbered place on a figure's picture. Its number is its place in
/// the list, from 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FigureMark {
    /// `[x, y]`: where, as fractions (0–1) of the picture's width and height
    /// from its top left. For a box or a bracket, the middle of the area.
    pub at: [f64; 2],
    /// What is there, in English (shown left to right, in the list under
    /// the picture). Plain text; abbreviations need a glossary entry.
    pub label: String,
    #[serde(default, skip_serializing_if = "MarkKind::is_point")]
    pub kind: MarkKind,
    /// `[width, height]` of the area a box or a bracket marks, as fractions
    /// of the picture's. Only on a box or a bracket, and required there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[f64; 2]>,
}

/// How a mark is drawn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    /// A ring around the spot, its number beside it.
    #[default]
    Point,
    /// An arrow whose tip is on the spot; its number sits at the tail.
    Arrow,
    /// An outline around an area (`size`).
    Box,
    /// A bracket along an area (`size`): down its left side when it is
    /// taller than wide (a layer's thickness), else under it (a length).
    Bracket,
}

impl MarkKind {
    fn is_point(&self) -> bool {
        *self == Self::Point
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Point => "point",
            Self::Arrow => "arrow",
            Self::Box => "box",
            Self::Bracket => "bracket",
        }
    }

    /// Box and bracket mark an area and need a `size`.
    pub fn has_size(self) -> bool {
        matches!(self, Self::Box | Self::Bracket)
    }
}

/// Closer than this to the left or right edge, a mark's number goes on the
/// inner side so the picture's edge never cuts it. Pictures are at least
/// about 300 px wide on a phone, so a fifth leaves room for an arrow.
const EDGE: f64 = 0.2;

impl FigureMark {
    /// Which way the mark's number (and an arrow's tail) sits from the spot,
    /// as `(x, y)` signs. Sideways: away from the middle, so it covers as
    /// little as it can, unless that runs off the nearest edge. Up or down:
    /// towards the middle, since a wide picture can be short on a phone and
    /// only that side always has room.
    pub fn outward(&self) -> (i8, i8) {
        let [x, y] = self.at;
        let out = if x < 0.5 { -1 } else { 1 };
        let dx = if !(EDGE..=1.0 - EDGE).contains(&x) { -out } else { out };
        let dy = if y > 0.5 { -1 } else { 1 };
        match (self.kind, self.size) {
            // A tall bracket's number sits beside it, a wide one's under
            // (or, low on the picture, over) it.
            (MarkKind::Bracket, Some([w, h])) if h >= w => {
                let left = x - w / 2.0;
                (if left < EDGE { 1 } else { -1 }, 0)
            }
            (MarkKind::Bracket, Some([_, h])) => {
                let bottom = y + h / 2.0;
                (0, if bottom >= 0.6 { -1 } else { 1 })
            }
            _ => (dx, dy),
        }
    }

    /// A bracket taller than it is wide runs down the area's left side.
    pub fn upright(&self) -> bool {
        self.size.is_some_and(|[w, h]| h >= w)
    }
}

/// Problems with a figure's marks. `on_image` is whether the figure is a
/// picture: a drawn figure labels its own parts.
pub fn mark_problems(marks: &[FigureMark], on_image: bool) -> Vec<String> {
    let mut out = Vec::new();
    if marks.is_empty() {
        return out;
    }
    if !on_image {
        out.push("marks go on an image figure; a drawn (svg) figure labels its own parts".into());
    }
    if marks.len() > MAX_MARKS {
        out.push(format!("{} marks: keep it to {MAX_MARKS} or fewer on one picture", marks.len()));
    }
    let unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
    for (i, m) in marks.iter().enumerate() {
        let n = i + 1;
        let [x, y] = m.at;
        if !unit(x) || !unit(y) {
            out.push(format!("mark {n}: at [{x}, {y}] must be fractions of the picture, from 0 to 1"));
        }
        let label = m.label.trim();
        if label.is_empty() {
            out.push(format!("mark {n} needs a label"));
        } else if label.contains("**") || label.contains("==") || label.contains('\n') {
            out.push(format!("mark {n}: the label is one plain line (no ** or ==)"));
        }
        match (m.kind.has_size(), m.size) {
            (true, None) => out.push(format!(
                "mark {n}: a {} needs size: [width, height] (fractions of the picture)",
                m.kind.as_str()
            )),
            (false, Some(_)) => out.push(format!("mark {n}: only a box or a bracket takes a size")),
            (true, Some([w, h])) => {
                let part = |v: f64| v.is_finite() && v > 0.0 && v <= 1.0;
                if !part(w) || !part(h) {
                    out.push(format!("mark {n}: size [{w}, {h}] must be fractions above 0, up to 1"));
                } else if x - w / 2.0 < -1e-9
                    || x + w / 2.0 > 1.0 + 1e-9
                    || y - h / 2.0 < -1e-9
                    || y + h / 2.0 > 1.0 + 1e-9
                {
                    out.push(format!("mark {n}: the {} runs past the picture's edge", m.kind.as_str()));
                }
            }
            (false, None) => {}
        }
    }
    out
}
