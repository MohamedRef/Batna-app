//! Lab values in lesson text and cards, with their normal range: "Na 128
//! mmol/L" reads as sodium, and the app puts the adult range beside it
//! (Mohamed, 2026-11-04). The numbers are [`crate::calc::normals`], the same
//! ones the calculator boxes use, so a range lives in one place; the few
//! units and tests the calculators don't take are added here, each with its
//! source. A course book's range wins where it gives one, then the NBME
//! Laboratory Values table (April 2026, nbme.org/laboratory-values), then a
//! named guideline.
//!
//! [`find`] is cautious: it wants the test's name, then at most a comparator
//! or a linking word, then a number and (for most tests) its unit. A change
//! ("a rise in creatinine of 0.3 mg/dL"), another fluid ("ascitic albumin",
//! "urine sodium") or a multiple ("ALT 3× ULN") is left alone.

use crate::calc::normals as n;
use crate::calc::{Normal, Place};

/// One test in one unit.
#[derive(Debug)]
pub struct Lab {
    pub id: &'static str,
    /// Spellings matched with their case (`Na`, `HCO3`).
    pub abbr: &'static [&'static str],
    /// Words matched ignoring case (`sodium`, `serum sodium`).
    pub words: &'static [&'static str],
    /// The unit as shown, then other spellings. Empty: the test has no
    /// unit (pH, INR).
    pub units: &'static [&'static str],
    /// A number written without a unit still counts when it falls in this
    /// span ("Na 128", "eGFR <30"); `None`: the unit must be written.
    pub bare: Option<(f64, f64)>,
    pub normal: Normal,
    /// Which value the range is for, when it matters ("fasting").
    pub note: &'static str,
    pub source: &'static str,
}
mod data;
pub use data::*;

mod find;
pub use find::*;

#[cfg(test)]
mod tests;
