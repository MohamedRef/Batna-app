//! Plasma BUN : creatinine ratio, prerenal azotemia vs ATN.
//!
//! Source: the Nephrology book (Chapter 8, Lecture 5), "Distinguishing
//! prerenal azotemia and ATN" (p. 109): plasma BUN-creatinine ratio > 20 in
//! prerenal azotemia, < 15 in ATN. The book gives no word for 15–20.

use super::{Area, CalcError, CalcSpec, Inputs, Outcome, Part, num, show};

pub const SPEC: CalcSpec = CalcSpec {
    id: "bun_creatinine",
    name: "BUN : creatinine ratio",
    area: Area::Renal,
    fields: &[
        num("bun_mg_dl", "BUN", "mg/dL", 1.0, 300.0, 0.1).hint("Urea ÷ 2.14."),
        num("creatinine_mg_dl", "Serum creatinine", "mg/dL", 0.1, 30.0, 0.01),
    ],
    source: "Book Nephrology Lecture 5, p. 109 (prerenal vs ATN table); no validated cut-off \
             study, so read it with the FENa",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let bun = i.get_pos("bun_mg_dl")?;
    let cr = i.get_pos("creatinine_mg_dl")?;
    let ratio = ((bun / cr) * 10.0).round() / 10.0;
    let (band, label, meaning) = if ratio > 20.0 {
        ("bun_creatinine.prerenal", "Above 20", "Suggests prerenal azotemia.")
    } else if ratio < 15.0 {
        ("bun_creatinine.atn", "Below 15", "Suggests intrinsic AKI (ATN).")
    } else {
        (
            "bun_creatinine.indeterminate",
            "15–20",
            "In between: read it with the FENa, urine sodium and sediment.",
        )
    };
    Ok(Outcome {
        score: Some(ratio),
        display: show(ratio, 1),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(109),
        parts: vec![Part::new("BUN", bun), Part::new("÷ creatinine", cr)],
        notes: vec![
            "GI bleeding, steroids and a high protein intake raise BUN without prerenal AKI.".to_owned(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(bun: f64, cr: f64) -> Outcome {
        compute(&Inputs::new().num("bun_mg_dl", bun).num("creatinine_mg_dl", cr)).unwrap()
    }

    #[test]
    fn bands() {
        assert_eq!((run(60.0, 2.0).score, run(60.0, 2.0).band), (Some(30.0), "bun_creatinine.prerenal"));
        assert_eq!(run(40.0, 4.0).band, "bun_creatinine.atn");
        assert_eq!(run(36.0, 2.0).band, "bun_creatinine.indeterminate");
        assert_eq!(run(40.0, 2.0).band, "bun_creatinine.indeterminate"); // exactly 20
        assert_eq!(run(30.0, 2.0).band, "bun_creatinine.indeterminate"); // exactly 15
    }
}
