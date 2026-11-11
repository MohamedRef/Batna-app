//! Table tests for every calculator (test-only).
//!
//! Each calculator gets a [`Table`]: the full list of bands it can give and
//! a set of cases run through [`super::compute`] (so the field limits apply,
//! as in the app). Cases come in three kinds:
//!
//! - **paper**: a worked example from the cited paper or guideline. Where the
//!   paper prints no example, the expected value is worked out by hand from
//!   the paper's published formula and the arithmetic is written in the note.
//! - **edge**: the two sides of a cut-off (64 and 65), checking which side the
//!   book or paper puts the value itself on.
//! - **unit**: a unit conversion done inside the calculator (mg/dL bilirubin
//!   × 17.1 = µmol/L), checked at the converted cut-off.
//!
//! [`every_calculator_has_a_table`] makes sure no calculator is left out, that
//! each has a paper case and at least one cut-off pair, and that every band is
//! reached by some case, including every band with a written guide.

mod blood;
mod diabetes;
mod diagnostic;
mod frequency;
mod geri;
mod gi;
mod liver;
mod renal;
mod research;
mod rheum;
mod sample_size;

use std::collections::BTreeSet;

use super::{CalcValue, Inputs, Outcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Paper,
    Edge,
    Unit,
}

#[derive(Clone, Debug)]
pub enum V {
    N(f64),
    F(bool),
    C(&'static str),
}

impl From<f64> for V {
    fn from(v: f64) -> Self {
        V::N(v)
    }
}
impl From<i32> for V {
    fn from(v: i32) -> Self {
        V::N(v as f64)
    }
}
impl From<bool> for V {
    fn from(v: bool) -> Self {
        V::F(v)
    }
}
impl From<&'static str> for V {
    fn from(v: &'static str) -> Self {
        V::C(v)
    }
}

#[derive(Clone, Debug)]
pub struct Case {
    pub kind: Kind,
    pub note: &'static str,
    pub inputs: Vec<(&'static str, V)>,
    /// Expected score and the allowed difference.
    pub score: Option<(f64, f64)>,
    pub band: &'static str,
    /// Text the outcome's notes must contain.
    pub note_has: Option<&'static str>,
}

pub struct Table {
    pub calc: &'static str,
    pub bands: &'static [&'static str],
    pub cases: Vec<Case>,
}

impl Table {
    pub fn new(calc: &'static str, bands: &'static [&'static str]) -> Self {
        Self { calc, bands, cases: Vec::new() }
    }

    fn push(
        mut self,
        kind: Kind,
        note: &'static str,
        inputs: Vec<(&'static str, V)>,
        band: &'static str,
    ) -> Self {
        self.cases.push(Case { kind, note, inputs, score: None, band, note_has: None });
        self
    }

    /// A worked example from the paper or guideline.
    pub fn paper(self, note: &'static str, inputs: Vec<(&'static str, V)>, band: &'static str) -> Self {
        self.push(Kind::Paper, note, inputs, band)
    }

    /// One side of a cut-off. Always written in pairs.
    pub fn edge(self, note: &'static str, inputs: Vec<(&'static str, V)>, band: &'static str) -> Self {
        self.push(Kind::Edge, note, inputs, band)
    }

    /// A unit conversion done inside the calculator.
    pub fn unit(self, note: &'static str, inputs: Vec<(&'static str, V)>, band: &'static str) -> Self {
        self.push(Kind::Unit, note, inputs, band)
    }

    /// The expected score of the last case, within `tol`.
    pub fn score(mut self, v: f64, tol: f64) -> Self {
        self.cases.last_mut().expect("a case first").score = Some((v, tol));
        self
    }

    /// Text the last case's notes must contain.
    pub fn noted(mut self, text: &'static str) -> Self {
        self.cases.last_mut().expect("a case first").note_has = Some(text);
        self
    }

    /// Both sides of one cut-off on one input: `base` plus `key = below`
    /// gives `below_band`, `key = at` gives `at_band`.
    pub fn cut(
        self,
        note: &'static str,
        base: Vec<(&'static str, V)>,
        key: &'static str,
        (below, below_band): (f64, &'static str),
        (at, at_band): (f64, &'static str),
    ) -> Self {
        let with = |v: f64| {
            let mut i: Vec<(&'static str, V)> = base.iter().filter(|(k, _)| *k != key).cloned().collect();
            i.push((key, V::N(v)));
            i
        };
        let (lo, hi) = (with(below), with(at));
        self.edge(note, lo, below_band).edge(note, hi, at_band)
    }
}

/// `ins![age_years: 50, sex: "male"]` → input pairs.
macro_rules! ins {
    ($($k:ident : $v:expr),* $(,)?) => {
        vec![$((stringify!($k), $crate::calc::cases::V::from($v))),*]
    };
}
pub(crate) use ins;

fn inputs(pairs: &[(&'static str, V)]) -> Inputs {
    let mut i = Inputs::new();
    for (k, v) in pairs {
        i.0.insert(
            (*k).to_owned(),
            match v {
                V::N(n) => CalcValue::Number(*n),
                V::F(b) => CalcValue::Bool(*b),
                V::C(c) => CalcValue::Text((*c).to_owned()),
            },
        );
    }
    i
}

fn tables() -> Vec<Table> {
    [
        liver::tables(),
        gi::tables(),
        renal::tables(),
        geri::tables(),
        rheum::tables(),
        diabetes::tables(),
        blood::tables(),
        research::tables(),
        frequency::tables(),
        sample_size::tables(),
        diagnostic::tables(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn run(calc: &str, c: &Case) -> Outcome {
    super::compute(calc, &inputs(&c.inputs))
        .unwrap_or_else(|e| panic!("{calc} ({}): {e} for {:?}", c.note, c.inputs))
}

#[test]
fn every_case_gives_its_band_and_score() {
    let mut failures = Vec::new();
    for t in tables() {
        for c in &t.cases {
            let o = run(t.calc, c);
            if o.band != c.band {
                failures.push(format!(
                    "{} [{}]: band {} (score {:?}), want {}",
                    t.calc, c.note, o.band, o.score, c.band
                ));
            }
            if let Some((want, tol)) = c.score {
                match o.score {
                    Some(got) if (got - want).abs() <= tol => {}
                    got => {
                        failures.push(format!("{} [{}]: score {got:?}, want {want} ± {tol}", t.calc, c.note))
                    }
                }
            }
            if let Some(text) = c.note_has
                && !o.notes.iter().any(|n| n.contains(text))
            {
                failures.push(format!("{} [{}]: notes {:?} lack {text:?}", t.calc, c.note, o.notes));
            }
        }
    }
    assert!(failures.is_empty(), "{} failing cases:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn every_calculator_has_a_table() {
    let tables = tables();
    let mut problems = Vec::new();
    let mut reached = BTreeSet::new();
    for spec in super::all() {
        let mine: Vec<&Table> = tables.iter().filter(|t| t.calc == spec.id).collect();
        let [t] = mine.as_slice() else {
            problems.push(format!("{}: {} tables", spec.id, mine.len()));
            continue;
        };
        if !t.cases.iter().any(|c| c.kind == Kind::Paper) {
            problems.push(format!("{}: no paper case", spec.id));
        }
        let edges = t.cases.iter().filter(|c| c.kind == Kind::Edge).count();
        if edges < 2 || edges % 2 != 0 {
            problems.push(format!("{}: {edges} edge cases (cut-offs come in pairs)", spec.id));
        }
        let hit: BTreeSet<&str> = t.cases.iter().map(|c| c.band).collect();
        for b in t.bands {
            if !hit.contains(b) {
                problems.push(format!("{}: band {b} never reached", spec.id));
            }
        }
        for b in &hit {
            if !t.bands.contains(b) {
                problems.push(format!("{}: case band {b} not listed", spec.id));
            }
        }
        reached.extend(hit);
    }
    for t in &tables {
        if super::spec(t.calc).is_none() {
            problems.push(format!("table for unknown calculator {}", t.calc));
        }
    }
    for (band, _) in super::GUIDE_LISTS.iter().flat_map(|l| l.iter()) {
        if !reached.contains(band) {
            problems.push(format!("guide band {band} reached by no case"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_edge_pair_crosses_its_cut_off() {
    // A pair whose two sides give the same result tests nothing: the band
    // or the points must change across the cut-off.
    for t in tables() {
        let edges: Vec<&Case> = t.cases.iter().filter(|c| c.kind == Kind::Edge).collect();
        for pair in edges.chunks(2) {
            let (a, b) = (run(t.calc, pair[0]), run(t.calc, pair[1]));
            assert!(
                a.band != b.band || a.score != b.score,
                "{}: edge pair [{}] does not cross ({} {:?})",
                t.calc,
                pair[0].note,
                a.band,
                a.score
            );
        }
    }
}
