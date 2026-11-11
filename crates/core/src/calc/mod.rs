//! Clinical calculators for hepatology and the pancreas, and the research
//! measures of Basics of Research (rates, RR/OR, NNT, diagnostic accuracy).
//!
//! Every calculator is a pure function from named [`Inputs`] to an
//! [`Outcome`]. The [`CalcSpec`] registry describes each calculator's fields so
//! the app can render one generic form, and so a content item of kind `calc`
//! (`calculator: child_pugh`, `inputs: {…}`) can be computed and checked by
//! the validator. Where the book gives the cut-offs, the book wins and the
//! outcome carries its printed page; other sources are cited in each module.
//!
//! Medical text here is English (like the book and the exam). Units follow
//! the book and Egyptian lab reports: bilirubin mg/dL, albumin g/dL,
//! creatinine mg/dL. Calculators whose published formula uses SI units
//! convert internally.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

pub use crate::content::CalcValue;

// The calculator machinery, one file per job; each calculator is its own
// module below.
mod answer;
mod guide;
mod inputs;
mod reference;
mod registry;
mod spec;

pub use answer::*;
pub use guide::*;
pub use inputs::*;
pub use reference::*;
pub use registry::*;
pub use spec::*;

/// Adult normal ranges shared by inputs (`.normal_as(normals::SODIUM)`).
pub mod normals;

mod aa_gradient;
mod aav;
mod aki;
mod albi;
mod apri;
mod ascitic;
mod atlanta;
mod baveno;
mod bclc;
mod bisap;
mod bmi;
mod bun_cr;
#[cfg(test)]
mod cases;
mod cha2ds2_vasc;
mod child_pugh;
mod ckd;
mod clichy;
mod cll;
mod cockcroft;
mod ctsi;
mod diabetes;
mod diagnostic;
mod dka;
mod fib4;
mod fli;
mod free_water;
mod frequency;
mod gbs;
mod geri;
mod glasgow;
mod insulin;
mod ipss_mf;
mod kings;
mod kyoto;
mod lgib;
mod lymphoma;
mod maddrey;
mod meld;
mod milan;
mod mpn;
mod okuda;
mod page_b;
mod plasmic;
mod r_ratio;
mod ra2010;
mod ranson;
mod renal;
mod research;
mod rome_ibs;
mod rpi;
mod saag;
mod sample_size;
mod sle;
mod sos;
mod spa;
mod uc_activity;

/// Round to `dp` decimal places for display.
fn show(v: f64, dp: usize) -> String {
    format!("{v:.dp$}")
}

/// `v` as [`show`] prints it, so the band always agrees with the number on
/// screen (FLI 59.6 shows "60" and is banded 60 or above).
fn shown(v: f64, dp: usize) -> f64 {
    show(v, dp).parse().unwrap_or(v)
}

#[cfg(test)]
mod tests;
