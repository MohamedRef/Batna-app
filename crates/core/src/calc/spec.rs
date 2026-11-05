//! What a calculator is: its fields, normal ranges, outcome and spec.

use super::*;

/// One line of the breakdown: a criterion and what it contributed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Part {
    pub label: String,
    /// Points for scores; 1/0 for criteria met or not.
    pub points: f64,
    /// `Some` for yes/no criteria (King's, Ranson, BISAP), `None` for points.
    pub met: Option<bool>,
}

impl Part {
    pub(super) fn new(label: impl Into<String>, points: f64) -> Self {
        Self { label: label.into(), points, met: None }
    }

    pub(super) fn met(label: impl Into<String>, met: bool) -> Self {
        Self { label: label.into(), points: if met { 1.0 } else { 0.0 }, met: Some(met) }
    }
}

/// The result of one calculation.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    /// Numeric result; `None` for pure decision tables (BCLC, King's).
    pub score: Option<f64>,
    /// The result as shown, e.g. "9", "1.4 g/dL", "Stage B".
    pub display: String,
    /// Stable band id, e.g. `child_pugh.b`. Content `expect:` and
    /// interpretation MCQs key on this.
    pub band: &'static str,
    /// Short English label of the band, e.g. "Class B (7–9 points)".
    pub label: &'static str,
    /// What the band means, in the book's words where it has them.
    pub meaning: String,
    /// Printed book page of the cut-offs, when the book has them.
    pub page: Option<u32>,
    pub parts: Vec<Part>,
    /// Caveats, e.g. "48-hour criteria not entered".
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldKind {
    Number {
        unit: &'static str,
        min: f64,
        max: f64,
        step: f64,
    },
    Flag,
    /// (id, English label) pairs.
    Choice(&'static [(&'static str, &'static str)]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    pub optional: bool,
    pub hint: &'static str,
    /// The adult normal range of a number input, shown under its box; a
    /// value outside it colours the box (one colour high, another low).
    /// The hard limits are `FieldKind::Number`'s `min` and `max`.
    pub normal: Option<Normal>,
}

/// A number input's normal (reference) range for adults, ends included.
/// An open end is `f64::NEG_INFINITY` or `f64::INFINITY` ("up to 2").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal {
    /// Men's range, or everyone's when `female` is `None`.
    pub low: f64,
    pub high: f64,
    /// Women's range when it differs (haemoglobin, creatinine). The form
    /// picks it from the calculator's `sex` choice (`male` / `female`).
    pub female: Option<(f64, f64)>,
    /// Normal is *below* `high`, which is itself high: triglycerides are
    /// normal under 150 mg/dL ("< 150"). Only with an open low end.
    pub below: bool,
}

/// Where a value sits against its [`Normal`] range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Low,
    Normal,
    High,
}

impl Normal {
    /// The range for one sex; with the sex unknown, the span of both, so a
    /// value only shows as abnormal when it is abnormal for either.
    pub fn bounds(&self, female: Option<bool>) -> (f64, f64) {
        match (self.female, female) {
            (Some(f), Some(true)) => f,
            (Some((fl, fh)), None) => (self.low.min(fl), self.high.max(fh)),
            _ => (self.low, self.high),
        }
    }

    pub fn place(&self, v: f64, female: Option<bool>) -> Place {
        let (low, high) = self.bounds(female);
        if v < low {
            Place::Low
        } else if v > high || (self.below && v >= high) {
            Place::High
        } else {
            Place::Normal
        }
    }
}

pub(super) const fn num(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    min: f64,
    max: f64,
    step: f64,
) -> Field {
    Field {
        key,
        label,
        kind: FieldKind::Number { unit, min, max, step },
        optional: false,
        hint: "",
        normal: None,
    }
}

pub(super) const fn flag(key: &'static str, label: &'static str) -> Field {
    Field { key, label, kind: FieldKind::Flag, optional: true, hint: "", normal: None }
}

pub(super) const fn choice(
    key: &'static str,
    label: &'static str,
    options: &'static [(&'static str, &'static str)],
) -> Field {
    Field { key, label, kind: FieldKind::Choice(options), optional: false, hint: "", normal: None }
}

impl Field {
    pub(super) const fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub(super) const fn hint(mut self, hint: &'static str) -> Self {
        self.hint = hint;
        self
    }

    /// The normal range, both ends included: `.normal(3.5, 5.5)`.
    #[allow(dead_code)]
    pub(super) const fn normal(mut self, low: f64, high: f64) -> Self {
        self.normal = Some(Normal { low, high, female: None, below: false });
        self
    }

    /// Normal up to `high`, included: `.normal_up_to(2.0)` for lactate.
    #[allow(dead_code)]
    pub(super) const fn normal_up_to(self, high: f64) -> Self {
        self.normal(f64::NEG_INFINITY, high)
    }

    /// Normal under `high`, which is itself high: `.normal_under(150.0)`.
    #[allow(dead_code)]
    pub(super) const fn normal_under(mut self, high: f64) -> Self {
        self.normal = Some(normals::under(high));
        self
    }

    /// Normal from `low` up: `.normal_from(90.0)`.
    #[allow(dead_code)]
    pub(super) const fn normal_from(self, low: f64) -> Self {
        self.normal(low, f64::INFINITY)
    }

    /// A shared range from [`normals`]: `.normal_as(normals::ALBUMIN_G_DL)`.
    pub(super) const fn normal_as(mut self, normal: Normal) -> Self {
        self.normal = Some(normal);
        self
    }

    /// Different ranges for men and women: `.normal_sex((13.5, 17.5), (12.0, 15.5))`.
    #[allow(dead_code)]
    pub(super) const fn normal_sex(mut self, male: (f64, f64), female: (f64, f64)) -> Self {
        self.normal = Some(Normal { low: male.0, high: male.1, female: Some(female), below: false });
        self
    }

    /// The `[min, max]` a number may take at all: what a body can show, wide
    /// of the normal range. `None` for flags and choices.
    pub fn limits(&self) -> Option<(f64, f64)> {
        match self.kind {
            FieldKind::Number { min, max, .. } => Some((min, max)),
            _ => None,
        }
    }
}

/// The reason [`CalcSpec::run`] gives for a number outside its field's
/// `min`..`max`: a typo or a unit slip (albumin 35 for 3.5 g/dL).
pub const IMPOSSIBLE: &str = "outside the possible range";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    Liver,
    Ascites,
    Tumour,
    AcuteFailure,
    Pancreas,
    Gi,
    Renal,
    Elderly,
    Rheum,
    Diabetes,
    Blood,
    Research,
}

pub struct CalcSpec {
    /// The id used in content (`calculator: child_pugh`).
    pub id: &'static str,
    pub name: &'static str,
    pub area: Area,
    pub fields: &'static [Field],
    /// Formula source, shown under the result.
    pub source: &'static str,
    pub compute: fn(&Inputs) -> Result<Outcome, CalcError>,
}

impl fmt::Debug for CalcSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CalcSpec").field("id", &self.id).finish_non_exhaustive()
    }
}
