//! Diabetic emergencies: corrected sodium and serum osmolality in HHS.
//!
//! Source: the Diabetology book (Chapter 6, Lecture 15, p. 189). Corrected
//! Na = measured Na + 0.016 × (glucose mg/dL − 100), i.e. 1.6 mEq/L per
//! 100 mg/dL above 100 (glucose 500, Na 132 → 138.4). HHS criteria:
//! effective osmolality 2 × Na + glucose (mmol/L) above 300 mOsm/kg, or
//! total osmolality 2 × Na + glucose + urea (mmol/L) above 320. HHS has
//! resolved once osmolality is below 300 (p. 192).

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num, show};

const NA: super::Field = num("na_meq_l", "Sodium", "mEq/L", 90.0, 200.0, 1.0).normal_as(normals::SODIUM);
const GLUCOSE: super::Field =
    num("glucose_mg_dl", "Glucose", "mg/dL", 10.0, 3000.0, 1.0).normal_as(normals::GLUCOSE_MG_DL);

pub const CORRECTED_NA: CalcSpec = CalcSpec {
    id: "corrected_sodium",
    name: "Corrected sodium (hyperglycaemia)",
    area: Area::Diabetes,
    fields: &[NA, GLUCOSE],
    source: "Book Diabetology Lecture 15, p. 189",
    compute: corrected_sodium,
};

fn corrected_sodium(i: &Inputs) -> Result<Outcome, CalcError> {
    let na = i.get_num("na_meq_l")?;
    let glucose = i.get_num("glucose_mg_dl")?;
    let add = 0.016 * (glucose - 100.0).max(0.0);
    let value = ((na + add) * 10.0).round() / 10.0;
    // Sodium 135–145 mEq/L (Nephrology book, Lecture 6, p. 127).
    let (band, label, meaning) = if value < 135.0 {
        (
            "corrected_sodium.low",
            "Low (below 135)",
            "Still low once the glucose is allowed for: a true hyponatraemia on top of the \
             hyperglycaemia.",
        )
    } else if value > 145.0 {
        (
            "corrected_sodium.high",
            "High (above 145)",
            "High once the glucose is allowed for: a profound free-water loss (p. 189).",
        )
    } else {
        (
            "corrected_sodium.normal",
            "Normal (135–145)",
            "Normal once the glucose is allowed for: the low measured sodium is from the glucose, \
             and a normal corrected sodium still means a large free-water loss (p. 189).",
        )
    };
    Ok(Outcome {
        score: Some(value),
        display: format!("{} mEq/L", show(value, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(189),
        parts: vec![
            Part::new("Measured Na", na),
            Part::new("+ 0.016 × (glucose − 100)", (add * 10.0).round() / 10.0),
        ],
        notes: Vec::new(),
    })
}

pub const OSMOLALITY: CalcSpec = CalcSpec {
    id: "effective_osmolality",
    name: "Serum osmolality (HHS)",
    area: Area::Diabetes,
    fields: &[
        NA,
        GLUCOSE,
        num("urea_mmol_l", "Urea", "mmol/L", 0.5, 100.0, 0.1)
            .normal_as(normals::UREA_MMOL_L)
            .optional()
            .hint("For the total osmolality. Urea mg/dL ÷ 6."),
    ],
    source: "Book Diabetology Lecture 15, pp. 189, 192",
    compute: osmolality,
};

fn osmolality(i: &Inputs) -> Result<Outcome, CalcError> {
    let na = i.get_num("na_meq_l")?;
    let glucose = i.get_num("glucose_mg_dl")? / 18.0;
    let effective = ((2.0 * na + glucose) * 10.0).round() / 10.0;
    let mut parts =
        vec![Part::new("2 × Na", 2.0 * na), Part::new("Glucose (mmol/L)", (glucose * 10.0).round() / 10.0)];
    let mut notes = Vec::new();
    let mut met = effective > 300.0;
    if let Some(urea) = i.opt_num("urea_mmol_l")? {
        let total = effective + urea;
        parts.push(Part::new("Urea (total only)", urea));
        notes.push(format!("Total osmolality {} mOsm/kg (HHS criterion: above 320).", show(total, 1)));
        met |= total > 320.0;
    }
    let (band, label, meaning) = if met {
        (
            "effective_osmolality.hhs",
            "Meets the HHS criterion",
            "Hyperosmolality as in HHS: effective osmolality above 300 or total above 320 mOsm/kg.",
        )
    } else {
        (
            "effective_osmolality.below",
            "Below the HHS cut-off",
            "Not hyperosmolar by the HHS criteria. Below 300 is also the resolution target.",
        )
    };
    Ok(Outcome {
        score: Some(effective),
        display: format!("{} mOsm/kg", show(effective, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(189),
        parts,
        notes,
    })
}

/// Next steps and differential (book Diabetology Lecture 15, pp. 187–192,
/// and the Nephrology book pp. 7–8, 128–129 and 133). Where the book is
/// silent: the rate limits of the JBDS HHS guideline (Mustafa et al.,
/// Diabet Med 2023).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "corrected_sodium.low",
        Guide {
            next: &[
                "A true hyponatraemia as well: check the serum osmolality and the volume status (Nephrology pp. 127–129)",
                "Replace about half the fluid deficit in the first 8–12 h with 0.9% saline (p. 190)",
                "Recheck the sodium as the glucose falls: the measured value should rise (p. 189)",
                "For the anion gap use the measured sodium, not this corrected one (Nephrology pp. 7–8)",
            ],
            dd: &[
                "Hypovolaemic: vomiting, diarrhoea, osmotic diuresis with salt loss, diuretics (Nephrology p. 129)",
                "Euvolaemic or hypervolaemic: SIADH, hypothyroidism, heart, liver or kidney failure (Nephrology pp. 129–130)",
                "Pseudohyponatraemia from very high lipids or proteins, as in myeloma (Nephrology p. 129)",
            ],
        },
    ),
    (
        "corrected_sodium.normal",
        Guide {
            next: &[
                "A normal corrected sodium still means a profound free-water loss (p. 189)",
                "Replace about half the fluid deficit in the first 8–12 h with 0.9% saline or another crystalloid (p. 190)",
                "In HHS the sodium often rises at first as the glucose falls; after that, no more than a 10 mmol/L fall in 24 h (JBDS 2023)",
                "For the anion gap use the measured sodium, not this corrected one (Nephrology pp. 7–8)",
            ],
            dd: &[
                "A low measured sodium from glucose drawing water out of the cells: translocational, not a true hyponatraemia (p. 189; Nephrology p. 128)",
            ],
        },
    ),
    (
        "corrected_sodium.high",
        Guide {
            next: &[
                "A high corrected sodium means a profound free-water loss (p. 189): work out the water deficit",
                "Replace about half the fluid deficit in the first 8–12 h with 0.9% saline or another crystalloid (p. 190)",
                "Let the sodium fall no more than 10 mmol/L in 24 h (JBDS 2023); in HHS it often rises at first as the glucose falls",
            ],
            dd: &[
                "Osmotic diuresis from the glucose losing more water than salt (p. 189)",
                "Too little water: elderly, immobile, no access to drink (Nephrology p. 133)",
                "Diabetes insipidus or other water loss as well (Nephrology p. 133)",
            ],
        },
    ),
    (
        "effective_osmolality.hhs",
        Guide {
            next: &[
                "HHS also needs glucose 600 mg/dL or more, β-hydroxybutyrate below 3 mmol/L, pH 7.3 or more and HCO3 above 15 (p. 189)",
                "Fluids first: 0.9% saline or another crystalloid, replacing half the deficit in 8–12 h; no insulin until the potassium is known and above 3.5 (pp. 190–191)",
                "Then insulin 0.05 units/kg/h, keeping glucose at 200–250 mg/dL until resolution (p. 190)",
                "Let the osmolality fall only 3–8 mOsm/kg/h (JBDS 2023); a sudden coma suggests brain oedema: mannitol 1 g/kg IV over 30 min (p. 192)",
                "Prophylactic LMWH for the whole admission: the clot risk is higher than in DKA (p. 192)",
                "Resolved when glucose is below 250 mg/dL, osmolality below 300 and urine output above 0.5 mL/kg/h (p. 192)",
            ],
            dd: &[
                "Mixed DKA and HHS: check the ketones and the pH (pp. 188–189)",
                "The trigger: infection most often; poor water intake in the elderly; diabetes not yet known in 20% (p. 187)",
                "Hypernatraemia from a water deficit, with no high glucose (Nephrology p. 133)",
            ],
        },
    ),
    (
        "effective_osmolality.below",
        Guide {
            next: &[
                "Not hyperosmolar: with a high glucose, check the ketones and pH for DKA (p. 188)",
                "Under treatment, below 300 is one of the HHS resolution criteria, with glucose below 250 mg/dL and urine above 0.5 mL/kg/h (p. 192)",
            ],
            dd: &[
                "DKA: glucose 200 mg/dL or more (or known diabetes), β-hydroxybutyrate 3 mmol/L or more, pH below 7.3 or HCO3 below 18 (p. 188)",
                "Euglycaemic DKA, glucose below 200: SGLT2 inhibitors, pregnancy, fasting, insulin given on the way in (p. 188)",
            ],
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_books_sodium_example() {
        let o = corrected_sodium(&Inputs::new().num("na_meq_l", 132.0).num("glucose_mg_dl", 500.0)).unwrap();
        assert_eq!(o.score, Some(138.4));
        let o = corrected_sodium(&Inputs::new().num("na_meq_l", 140.0).num("glucose_mg_dl", 90.0)).unwrap();
        assert_eq!(o.score, Some(140.0));
    }

    #[test]
    fn hhs_cut_offs() {
        // 2 × 145 + 900/18 = 340.
        let i = Inputs::new().num("na_meq_l", 145.0).num("glucose_mg_dl", 900.0);
        let o = osmolality(&i).unwrap();
        assert_eq!((o.score, o.band), (Some(340.0), "effective_osmolality.hhs"));
        // 2 × 140 + 180/18 = 290; with urea 35 the total is 325.
        let i = Inputs::new().num("na_meq_l", 140.0).num("glucose_mg_dl", 180.0);
        assert_eq!(osmolality(&i).unwrap().band, "effective_osmolality.below");
        assert_eq!(osmolality(&i.num("urea_mmol_l", 35.0)).unwrap().band, "effective_osmolality.hhs");
    }
}
