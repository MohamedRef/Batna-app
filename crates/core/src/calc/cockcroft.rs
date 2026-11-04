//! Cockcroft-Gault creatinine clearance.
//!
//! Source: the Geriatrics book (Chapter 7). Lecture 2, Renal diseases in the
//! elderly, p. 30: CrCl = (140 − age) × weight (kg) / (72 × serum
//! creatinine mg/dL), × 0.85 for women. Lecture 3, p. 42, repeats it for
//! drug dosing and warns that serum creatinine is not reliable in the
//! elderly and the result is only an estimate. The book gives no bands;
//! the bands are the renal-function groups for drug dosing in the FDA
//! guidance "Pharmacokinetics in Patients with Impaired Renal Function"
//! (2024): 90 or more normal, 60–89 mild, 30–59 moderate, 15–29 severe,
//! below 15 kidney failure.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, normals, num, show};

const SEX: &[(&str, &str)] = &[("male", "Man"), ("female", "Woman")];

pub const SPEC: CalcSpec = CalcSpec {
    id: "cockcroft_gault",
    name: "Cockcroft-Gault creatinine clearance",
    area: Area::Elderly,
    fields: &[
        num("age_years", "Age", "years", 18.0, 120.0, 1.0),
        num("weight_kg", "Weight", "kg", 20.0, 300.0, 0.1),
        num("creatinine_mg_dl", "Serum creatinine", "mg/dL", 0.1, 20.0, 0.01)
            .normal_as(normals::CREATININE_MG_DL),
        choice("sex", "Sex", SEX),
    ],
    source: "Book Geriatrics Lecture 2, p. 30; Lecture 3, p. 42",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let age = i.get_num("age_years")?;
    let weight = i.get_pos("weight_kg")?;
    let cr = i.get_pos("creatinine_mg_dl")?;
    let woman = i.get_choice("sex", &["male", "female"])? == "female";
    if age >= 140.0 {
        return Err(CalcError::Invalid { field: "age_years", reason: "must be below 140" });
    }
    let base = (140.0 - age) * weight / (72.0 * cr);
    let crcl = ((if woman { base * 0.85 } else { base }) * 10.0).round() / 10.0;
    let (band, label, meaning) = band(crcl);
    let mut parts = vec![
        Part::new("140 − age", 140.0 - age),
        Part::new("× weight (kg)", weight),
        Part::new("÷ (72 × creatinine)", 72.0 * cr),
    ];
    if woman {
        parts.push(Part::new("× 0.85 (woman)", 0.85));
    }
    Ok(Outcome {
        score: Some(crcl),
        display: format!("{} mL/min", show(crcl, 1)),
        band,
        label,
        meaning: format!(
            "{meaning} In the elderly serum creatinine is not reliable (less muscle), so read it as an \
             estimate."
        ),
        page: Some(30),
        parts,
        notes: Vec::new(),
    })
}

/// The FDA (2024) renal-function group for dosing.
fn band(crcl: f64) -> (&'static str, &'static str, &'static str) {
    if crcl >= 90.0 {
        ("cockcroft_gault.normal", "90 or more: normal", "Normal clearance: usual doses.")
    } else if crcl >= 60.0 {
        ("cockcroft_gault.mild", "60–89: mild", "Mildly reduced: most drugs at the usual dose.")
    } else if crcl >= 30.0 {
        (
            "cockcroft_gault.moderate",
            "30–59: moderate",
            "Moderately reduced: check each renally cleared drug.",
        )
    } else if crcl >= 15.0 {
        (
            "cockcroft_gault.severe",
            "15–29: severe",
            "Severely reduced: lower the dose or lengthen the interval of renally cleared drugs.",
        )
    } else {
        (
            "cockcroft_gault.failure",
            "Below 15: kidney failure",
            "Kidney failure: many drugs are avoided or need dialysis dosing.",
        )
    }
}

/// Next steps and the pitfalls of the estimate (book Geriatrics Lectures 2
/// and 3, pp. 28–30 and 42–43, and the Nephrology book p. 101). Which weight
/// to use when overweight: Winter et al., Pharmacotherapy 2012.
const PITFALLS: &[&str] = &[
    "Little muscle (age, wasting) keeps the creatinine low, so the clearance reads too high (pp. 29, 42)",
    "Overweight: the total weight overstates it; use the adjusted weight, ideal + 0.4 × the excess (Winter 2012)",
    "A creatinine that is still changing, as in AKI, does not yet reflect the GFR (Nephrology p. 101)",
];

const DOSE: &str = "Dose renally cleared drugs by this clearance, above all digoxin and gentamicin (p. 43)";
const AGEING: &str = "Kidney function falls by about a third with normal ageing; a normal creatinine does not mean normal function in the elderly (p. 43)";
const POTASSIUM: &str = "With a low clearance, take care with potassium-sparing diuretics, RAAS blockers, NSAIDs and beta-blockers: high potassium (p. 28)";

pub const GUIDES: &[(&str, Guide)] = &[
    ("cockcroft_gault.normal", Guide { next: &[AGEING], dd: PITFALLS }),
    ("cockcroft_gault.mild", Guide { next: &[DOSE, AGEING], dd: PITFALLS }),
    ("cockcroft_gault.moderate", Guide { next: &[DOSE, POTASSIUM, AGEING], dd: PITFALLS }),
    (
        "cockcroft_gault.severe",
        Guide { next: &[DOSE, POTASSIUM, "Below 30: plan nephrology care (Nephrology p. 36)"], dd: PITFALLS },
    ),
    (
        "cockcroft_gault.failure",
        Guide {
            next: &[
                DOSE,
                POTASSIUM,
                "Below 15 is kidney failure: nephrology care, dialysis or transplant planning (Nephrology p. 36)",
            ],
            dd: PITFALLS,
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(sex: &str) -> Outcome {
        compute(
            &Inputs::new()
                .num("age_years", 80.0)
                .num("weight_kg", 60.0)
                .num("creatinine_mg_dl", 1.0)
                .choice("sex", sex),
        )
        .unwrap()
    }

    #[test]
    fn the_formula() {
        // (140 − 80) × 60 / 72 = 50; × 0.85 = 42.5.
        assert_eq!(run("male").score, Some(50.0));
        assert_eq!(run("female").score, Some(42.5));
        assert!(
            compute(
                &Inputs::new()
                    .num("age_years", 80.0)
                    .num("weight_kg", 60.0)
                    .num("creatinine_mg_dl", 0.0)
                    .choice("sex", "male")
            )
            .is_err()
        );
    }
}
