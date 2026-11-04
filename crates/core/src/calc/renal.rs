//! Acid–base and osmolality: anion gap, delta ratio, expected compensation
//! and osmolal gap.
//!
//! Source: the Nephrology book (Chapter 8, Lecture 1, Evaluation of an
//! acid–base disorder). AG = Na − (Cl + HCO3) (p. 5), normal 12 ± 4, above
//! 16 high, below 8 low (p. 14); each 1 g/dL fall of albumin below 4.0
//! lowers the AG by 2.5 (p. 9). Delta ratio = (AG − 12) / (24 − HCO3) with
//! the bands < 0.4, 0.4–0.8, 0.8–2.0 and > 2.0 (pp. 12, 14). Expected
//! compensation as the p. 14 table writes it: pCO2 = 1.5 × HCO3 + 8 (± 2)
//! in metabolic acidosis and 0.7 × HCO3 + 20 (± 5) in metabolic alkalosis;
//! HCO3 = 24 ± (pCO2 − 40)/10 × 1 (acute) or × 4 (chronic) in respiratory
//! acidosis, × 2 or × 5 in respiratory alkalosis; above or below the
//! expected value means a second disorder (p. 13). Calculated osmolality
//! = 2 × Na + glucose/18 + BUN/2.8 (mg/dL); a gap above 10 mOsm is an
//! osmolal gap (p. 20).

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, normals, num, show};

const NA: super::Field = num("na_meq_l", "Sodium", "mEq/L", 90.0, 200.0, 1.0).normal_as(normals::SODIUM);
const CL: super::Field = num("cl_meq_l", "Chloride", "mEq/L", 50.0, 160.0, 1.0).normal_as(normals::CHLORIDE);
const HCO3: super::Field =
    num("hco3_meq_l", "Bicarbonate (HCO3)", "mEq/L", 1.0, 60.0, 0.1).normal_as(normals::HCO3);

pub const ANION_GAP: CalcSpec = CalcSpec {
    id: "anion_gap",
    name: "Anion gap",
    area: Area::Renal,
    fields: &[
        NA,
        CL,
        HCO3,
        num("albumin_g_dl", "Albumin", "g/dL", 0.5, 6.0, 0.1)
            .normal_as(normals::ALBUMIN_G_DL)
            .optional()
            .hint("Corrects the gap for a low albumin."),
    ],
    source: "Book Nephrology Lecture 1, pp. 5, 9, 14",
    compute: anion_gap,
};

fn anion_gap(i: &Inputs) -> Result<Outcome, CalcError> {
    let (na, cl, hco3) = (i.get_num("na_meq_l")?, i.get_num("cl_meq_l")?, i.get_num("hco3_meq_l")?);
    let ag = na - (cl + hco3);
    let mut parts = vec![Part::new("Na", na), Part::new("− Cl", -cl), Part::new("− HCO3", -hco3)];
    let mut notes = Vec::new();
    let value = match i.opt_num("albumin_g_dl")? {
        Some(alb) if alb < 4.0 => {
            let add = 2.5 * (4.0 - alb);
            parts.push(Part::new("Albumin correction", add));
            notes.push(format!("Measured AG {}; corrected for albumin {} g/dL.", show(ag, 1), show(alb, 1)));
            ag + add
        }
        _ => ag,
    };
    let value = (value * 10.0).round() / 10.0;
    let (band, label, meaning) = if value > 16.0 {
        ("anion_gap.high", "High (> 16)", "High anion gap: look for added unmeasured anions (Box 2).")
    } else if value < 8.0 {
        ("anion_gap.low", "Low (< 8)", "Low anion gap: the commonest cause in hospital is a low albumin.")
    } else {
        ("anion_gap.normal", "Normal (12 ± 4)", "Normal anion gap.")
    };
    if value > 16.0 {
        notes.push("Check the delta ratio next.".to_owned());
    }
    Ok(Outcome {
        score: Some(value),
        display: format!("{} mEq/L", show(value, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(14),
        parts,
        notes,
    })
}

pub const DELTA_RATIO: CalcSpec = CalcSpec {
    id: "delta_ratio",
    name: "Delta ratio (gap : gap)",
    area: Area::Renal,
    fields: &[
        num("anion_gap_meq_l", "Anion gap", "mEq/L", 0.0, 60.0, 0.1).normal_as(normals::ANION_GAP),
        HCO3,
    ],
    source: "Book Nephrology Lecture 1, pp. 12, 14",
    compute: delta_ratio,
};

fn delta_ratio(i: &Inputs) -> Result<Outcome, CalcError> {
    let ag = i.get_num("anion_gap_meq_l")?;
    let hco3 = i.get_num("hco3_meq_l")?;
    if hco3 >= 24.0 {
        return Err(CalcError::Invalid {
            field: "hco3_meq_l",
            reason: "must be below 24 (a metabolic acidosis)",
        });
    }
    let ratio = ((ag - 12.0) / (24.0 - hco3) * 100.0).round() / 100.0;
    let (band, label, meaning) = match ratio {
        r if r < 0.4 => ("delta_ratio.normal_ag", "< 0.4", "Normal anion gap metabolic acidosis."),
        r if r < 0.8 => (
            "delta_ratio.mixed",
            "0.4–0.8",
            "Mixed: normal anion gap plus high anion gap metabolic acidosis.",
        ),
        r if r <= 2.0 => ("delta_ratio.pure_high", "0.8–2.0", "Pure high anion gap metabolic acidosis."),
        _ => (
            "delta_ratio.with_alkalosis",
            "> 2.0",
            "High anion gap metabolic acidosis plus metabolic alkalosis (or a pre-existing compensated respiratory acidosis).",
        ),
    };
    Ok(Outcome {
        score: Some(ratio),
        display: show(ratio, 2),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(12),
        parts: vec![Part::new("Δ AG = AG − 12", ag - 12.0), Part::new("Δ HCO3 = 24 − HCO3", 24.0 - hco3)],
        notes: Vec::new(),
    })
}

const DISORDERS: &[(&str, &str)] = &[
    ("metabolic_acidosis", "Metabolic acidosis"),
    ("metabolic_alkalosis", "Metabolic alkalosis"),
    ("respiratory_acidosis_acute", "Respiratory acidosis, acute"),
    ("respiratory_acidosis_chronic", "Respiratory acidosis, chronic"),
    ("respiratory_alkalosis_acute", "Respiratory alkalosis, acute"),
    ("respiratory_alkalosis_chronic", "Respiratory alkalosis, chronic"),
];

pub const COMPENSATION: CalcSpec = CalcSpec {
    id: "expected_compensation",
    name: "Expected compensation",
    area: Area::Renal,
    fields: &[
        choice("disorder", "Primary disorder", DISORDERS),
        HCO3,
        num("pco2_mm_hg", "pCO2", "mmHg", 5.0, 150.0, 1.0).normal_as(normals::PCO2_MM_HG),
    ],
    source: "Book Nephrology Lecture 1, pp. 13–14",
    compute: compensation,
};

/// The book gives no range for the respiratory rules; ± 2 mEq/L is the
/// usual one.
const RESP_RANGE: f64 = 2.0;

fn compensation(i: &Inputs) -> Result<Outcome, CalcError> {
    let ids: Vec<&str> = DISORDERS.iter().map(|(id, _)| *id).collect();
    let disorder = i.get_choice("disorder", &ids)?;
    let hco3 = i.get_num("hco3_meq_l")?;
    let pco2 = i.get_num("pco2_mm_hg")?;
    let mut notes = Vec::new();
    // (expected, range, measured, what is measured, too-high band, too-low band)
    let (expected, range, measured, what, high, low) = match disorder {
        "metabolic_acidosis" if hco3 >= 24.0 => {
            return Err(CalcError::Invalid {
                field: "hco3_meq_l",
                reason: "must be below 24 in metabolic acidosis",
            });
        }
        "metabolic_alkalosis" if hco3 <= 24.0 => {
            return Err(CalcError::Invalid {
                field: "hco3_meq_l",
                reason: "must be above 24 in metabolic alkalosis",
            });
        }
        "metabolic_acidosis" => (1.5 * hco3 + 8.0, 2.0, pco2, "pCO2", RESP_ACIDOSIS, RESP_ALKALOSIS),
        "metabolic_alkalosis" => (0.7 * hco3 + 20.0, 5.0, pco2, "pCO2", RESP_ACIDOSIS, RESP_ALKALOSIS),
        resp => {
            let acidosis = resp.starts_with("respiratory_acidosis");
            if acidosis && pco2 <= 40.0 {
                return Err(CalcError::Invalid {
                    field: "pco2_mm_hg",
                    reason: "must be above 40 in respiratory acidosis",
                });
            }
            if !acidosis && pco2 >= 40.0 {
                return Err(CalcError::Invalid {
                    field: "pco2_mm_hg",
                    reason: "must be below 40 in respiratory alkalosis",
                });
            }
            let per_10 = match resp {
                "respiratory_acidosis_acute" => 1.0,
                "respiratory_acidosis_chronic" => {
                    notes.push("The p. 13 table says 3.5 per 10 mmHg; the p. 14 summary uses 4.".to_owned());
                    4.0
                }
                "respiratory_alkalosis_acute" => 2.0,
                _ => {
                    notes.push("The p. 13 table says 4–5 per 10 mmHg; the p. 14 summary uses 5.".to_owned());
                    5.0
                }
            };
            notes.push("The book gives no range here; ± 2 mEq/L is used.".to_owned());
            (24.0 + (pco2 - 40.0) / 10.0 * per_10, RESP_RANGE, hco3, "HCO3", MET_ALKALOSIS, MET_ACIDOSIS)
        }
    };
    let expected = (expected * 10.0).round() / 10.0;
    let unit = if what == "pCO2" { "mmHg" } else { "mEq/L" };
    let (band, label, meaning) = if measured > expected + range {
        high
    } else if measured < expected - range {
        low
    } else {
        APPROPRIATE
    };
    Ok(Outcome {
        score: Some(expected),
        display: format!(
            "Expected {what} {} ({}–{} {unit})",
            show(expected, 1),
            show(expected - range, 1),
            show(expected + range, 1)
        ),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(14),
        parts: vec![
            Part::new(format!("Expected {what}"), expected),
            Part::new(format!("Measured {what}"), measured),
        ],
        notes,
    })
}

type Band = (&'static str, &'static str, &'static str);

const APPROPRIATE: Band =
    ("expected_compensation.appropriate", "Appropriate", "Compensation is as expected: a simple disorder.");
const RESP_ACIDOSIS: Band = (
    "expected_compensation.added_resp_acidosis",
    "pCO2 above expected",
    "pCO2 is higher than compensation explains: a complicating respiratory acidosis.",
);
const RESP_ALKALOSIS: Band = (
    "expected_compensation.added_resp_alkalosis",
    "pCO2 below expected",
    "pCO2 is lower than compensation explains: a complicating respiratory alkalosis.",
);
const MET_ALKALOSIS: Band = (
    "expected_compensation.added_met_alkalosis",
    "HCO3 above expected",
    "HCO3 is above the expected value: a complicating metabolic alkalosis.",
);
const MET_ACIDOSIS: Band = (
    "expected_compensation.added_met_acidosis",
    "HCO3 below expected",
    "HCO3 is below the expected value: a complicating metabolic acidosis.",
);

pub const OSMOLAL_GAP: CalcSpec = CalcSpec {
    id: "osmolal_gap",
    name: "Osmolal gap",
    area: Area::Renal,
    fields: &[
        num("measured_osm", "Measured osmolality", "mOsm/kg", 200.0, 500.0, 1.0)
            .normal_as(normals::OSMOLALITY),
        NA,
        num("glucose_mg_dl", "Glucose", "mg/dL", 10.0, 2000.0, 1.0).normal_as(normals::GLUCOSE_MG_DL),
        num("bun_mg_dl", "BUN", "mg/dL", 1.0, 300.0, 0.1).normal_as(normals::BUN_MG_DL).hint("Urea ÷ 2.14."),
    ],
    source: "Book Nephrology Lecture 1, p. 20",
    compute: osmolal_gap,
};

fn osmolal_gap(i: &Inputs) -> Result<Outcome, CalcError> {
    let measured = i.get_num("measured_osm")?;
    let na = i.get_num("na_meq_l")?;
    let glucose = i.get_num("glucose_mg_dl")?;
    let bun = i.get_num("bun_mg_dl")?;
    let calculated = 2.0 * na + glucose / 18.0 + bun / 2.8;
    let gap = ((measured - calculated) * 10.0).round() / 10.0;
    let (band, label, meaning) = if gap > 10.0 {
        (
            "osmolal_gap.present",
            "Gap present (> 10)",
            "An osmolal gap: unmeasured osmoles such as methanol, ethylene glycol, ethanol or isopropyl alcohol.",
        )
    } else {
        ("osmolal_gap.normal", "No gap (≤ 10)", "No osmolal gap.")
    };
    Ok(Outcome {
        score: Some(gap),
        display: format!("{} mOsm/kg", show(gap, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(20),
        parts: vec![
            Part::new("2 × Na", 2.0 * na),
            Part::new("Glucose ÷ 18", (glucose / 18.0 * 10.0).round() / 10.0),
            Part::new("BUN ÷ 2.8", (bun / 2.8 * 10.0).round() / 10.0),
        ],
        notes: vec![format!("Calculated osmolality {} mOsm/kg.", show(calculated, 1))],
    })
}

/// Next steps and differential per band (book Nephrology Lecture 1,
/// pp. 7–22, and the Diabetology book p. 193). Where the book is silent: the
/// urine anion gap from Batlle et al., N Engl J Med 1988; a normal osmolal
/// gap late after a toxic alcohol from Kraut and Kurtz, Clin J Am Soc
/// Nephrol 2008.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "anion_gap.high",
        Guide {
            next: &[
                "Work out the delta ratio: a ratio far from 1 means a second disorder (pp. 10–12)",
                "Name the anion: lactate, ketones, urea and creatinine, a salicylate level (pp. 8–9)",
                "If an alcohol or glycol is possible, measure the osmolality for an osmolal gap (p. 20)",
                "In hyperglycaemia, use the measured sodium, not the corrected one (pp. 7–8)",
                "Check the expected pCO2 (1.5 × HCO3 + 8 ± 2) for a respiratory disorder as well (p. 14)",
            ],
            dd: HIGH_AG_DD,
        },
    ),
    (
        "anion_gap.normal",
        Guide {
            next: &[
                "If the HCO3 is low, this is a normal anion gap (hyperchloraemic) metabolic acidosis (pp. 8–9)",
                "Enter the albumin: each 1 g/dL below 4.0 hides 2.5 of gap, so a normal gap can be a high one (p. 9)",
                "Labs that use ion-selective electrodes read lower (3–11): use your lab's range (p. 7)",
                "Urine anion gap (Na + K − Cl): negative with loss in the stool, positive in distal renal tubular acidosis (Batlle 1988)",
            ],
            dd: NORMAL_AG_DD,
        },
    ),
    (
        "anion_gap.low",
        Guide {
            next: &[
                "Check the albumin first: after a lab error, the commonest cause in hospital (p. 9)",
                "Repeat the electrolytes to rule out a lab error (p. 9)",
            ],
            dd: &[
                "Low albumin (p. 10)",
                "Added cations: IgG myeloma, high calcium or magnesium, lithium toxicity (p. 10)",
                "Bromide or salicylate read as chloride (p. 10)",
                "Very high triglycerides upsetting the assay (p. 10)",
            ],
        },
    ),
    (
        "delta_ratio.normal_ag",
        Guide {
            next: &[
                "The acidosis is a normal anion gap one: look for bicarbonate lost in stool or urine (p. 9)",
                "In hospital, think of saline or the recovery phase of DKA first (p. 9; Diabetology p. 193)",
                "Urine anion gap (Na + K − Cl): negative with loss in the stool, positive in distal renal tubular acidosis (Batlle 1988)",
            ],
            dd: NORMAL_AG_DD,
        },
    ),
    (
        "delta_ratio.mixed",
        Guide {
            next: &[
                "Two acidoses together: a high anion gap and a normal anion gap acidosis (pp. 11–12)",
                "Find and treat the cause of each part (pp. 8–9)",
                "In DKA the ratio falls from 2 early to 0 late, and saline adds chloride: time the sample (p. 11; Diabetology p. 193)",
            ],
            dd: &[
                "DKA in the recovery phase or after saline (p. 9; Diabetology p. 193)",
                "Renal failure: retained anions plus less ammonium excretion (pp. 8–9)",
                "Diarrhoea together with a lactic acidosis or ketoacidosis (pp. 8–9)",
            ],
        },
    ),
    (
        "delta_ratio.pure_high",
        Guide {
            next: &[
                "A simple high anion gap acidosis: find the anion with MUDPILES (p. 9)",
                "Lactic acidosis averages 1.6, because cells buffer up to half the acid (p. 11)",
                "Check the expected pCO2 (1.5 × HCO3 + 8 ± 2) for a respiratory disorder as well (p. 14)",
            ],
            dd: HIGH_AG_DD,
        },
    ),
    (
        "delta_ratio.with_alkalosis",
        Guide {
            next: &[
                "The HCO3 is higher than the gap explains: a metabolic alkalosis on top (p. 11)",
                "Or a compensated chronic respiratory acidosis that raised the HCO3 beforehand (p. 12)",
                "Early DKA can reach 2 before the ketones are excreted (p. 11)",
                "Urine chloride sorts the cause of the alkalosis (p. 17)",
            ],
            dd: &[
                "Vomiting, nasogastric suction or diuretics with a high anion gap acidosis (pp. 16–17)",
                "Chronic respiratory acidosis, such as COPD, with a raised HCO3 (pp. 12, 19)",
            ],
        },
    ),
    (
        "expected_compensation.appropriate",
        Guide {
            next: &[
                "A simple disorder: in a metabolic acidosis, go on to the anion gap (p. 20)",
                "If the gap is high, then the delta ratio, then the differential (p. 20)",
                "In a metabolic alkalosis, urine chloride sorts the cause (p. 17)",
            ],
            dd: &[
                "The causes of the primary disorder: metabolic acidosis (pp. 8–9), metabolic alkalosis (p. 16), respiratory acidosis (pp. 18–19), respiratory alkalosis (p. 19)",
            ],
        },
    ),
    (
        "expected_compensation.added_resp_acidosis",
        Guide {
            next: &[
                "The pCO2 is above the expected value: ventilation is failing (p. 13)",
                "Find the cause: drugs that depress breathing, weak muscles, a blocked airway or lung disease (pp. 18–19)",
            ],
            dd: &[
                "Depressed breathing: sedatives, opiates, anaesthetics, brain injury or stroke (p. 18)",
                "Weak muscles: myasthenia crisis, Guillain-Barré syndrome, low potassium or phosphate (p. 18)",
                "Airway or lung: aspiration, severe asthma or pneumonia, pulmonary oedema, COPD (pp. 18–19)",
            ],
        },
    ),
    (
        "expected_compensation.added_resp_alkalosis",
        Guide {
            next: &[
                "The pCO2 is below the expected value: something is driving the breathing (p. 13)",
                "Salicylates give a respiratory alkalosis first, then a high anion gap acidosis: check a level (p. 22)",
            ],
            dd: &[
                "Gram-negative sepsis, liver failure, pregnancy, fever or thyrotoxicosis (p. 19)",
                "Drugs: salicylates, progesterone, xanthines, catecholamines (pp. 19, 22)",
                "Lung disease (pneumonia, pulmonary embolus, pulmonary oedema) or anxiety (p. 19)",
            ],
        },
    ),
    (
        "expected_compensation.added_met_alkalosis",
        Guide {
            next: &[
                "The HCO3 is higher than expected: a metabolic alkalosis on top (p. 13)",
                "Urine chloride below 10–20 points to vomiting or diuretics given earlier; above it, to diuretics now, alkali or mineralocorticoid excess (p. 17)",
                "A chronic high pCO2 corrected quickly leaves the HCO3 high: posthypercapnic alkalosis (p. 16)",
            ],
            dd: &[
                "Vomiting or nasogastric suction (p. 16)",
                "Loop or thiazide diuretics (p. 16)",
                "Alkali: bicarbonate, citrate in transfusions, antacids (p. 16)",
                "Mineralocorticoid excess: primary aldosteronism, Cushing syndrome, liquorice (p. 16)",
                "Severe potassium deficiency; Bartter or Gitelman syndrome (p. 16)",
            ],
        },
    ),
    (
        "expected_compensation.added_met_acidosis",
        Guide {
            next: &[
                "The HCO3 is lower than expected: a metabolic acidosis on top (p. 13)",
                "Work out the anion gap, then the delta ratio (p. 20)",
            ],
            dd: &[
                "High anion gap: lactate, ketones, uraemia, toxins (pp. 8–9)",
                "Normal anion gap: diarrhoea, renal tubular acidosis, saline (p. 9)",
            ],
        },
    ),
    (
        "osmolal_gap.present",
        Guide {
            next: &[
                "With a high anion gap acidosis, suspect methanol or ethylene glycol: fomepizole or ethanol, with dialysis as needed (p. 22)",
                "Clues: visual loss in methanol; AKI and calcium oxalate crystals in the urine in ethylene glycol (p. 22)",
                "A gap with no acidosis fits isopropanol (acetone, fruity breath) or ethanol (p. 22)",
            ],
            dd: &[
                "Ethylene glycol, methanol, ethanol or isopropyl alcohol (p. 21)",
                "Ketoacidosis or lactic acidosis; kidney failure without dialysis (p. 21)",
                "Infused mannitol, glycine or sorbitol (p. 21)",
                "Very high protein or lipids; a lab error (p. 21)",
                "Formaldehyde, paraldehyde or diethyl ether (p. 21)",
            ],
        },
    ),
    (
        "osmolal_gap.normal",
        Guide {
            next: &[
                "A normal gap does not rule out a toxic alcohol taken hours ago, once it has turned into its acid (Kraut 2008)",
                "So if one is suspected, go by the history and the anion gap: all but isopropanol raise it (p. 22)",
            ],
            dd: &[],
        },
    ),
];

const HIGH_AG_DD: &[&str] = &[
    "Lactic acidosis; D-lactate after small-bowel resection (p. 8)",
    "Ketoacidosis: diabetic, alcoholic or starvation (pp. 8–9)",
    "Uraemia: renal failure (pp. 8–9)",
    "Methanol or ethylene glycol: look for an osmolal gap (pp. 9, 20)",
    "Salicylates, paracetamol, propylene glycol, iron, isoniazid, toluene (pp. 8–9)",
];

const NORMAL_AG_DD: &[&str] = &[
    "Bicarbonate lost in stool: diarrhoea, ureterosigmoidostomy or an ileal conduit (p. 9)",
    "Renal tubular acidosis: proximal (type II), distal (type I) or type IV (p. 9)",
    "Acetazolamide, spironolactone or cholestyramine (pp. 9, 15)",
    "Saline infusion or parenteral nutrition (pp. 9, 15)",
    "Recovery from ketoacidosis; CKD stage 4–5; Addison disease (pp. 9, 15)",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn ag(na: f64, cl: f64, hco3: f64) -> Inputs {
        Inputs::new().num("na_meq_l", na).num("cl_meq_l", cl).num("hco3_meq_l", hco3)
    }

    #[test]
    fn anion_gap_bands() {
        let o = anion_gap(&ag(140.0, 104.0, 24.0)).unwrap();
        assert_eq!((o.score, o.band), (Some(12.0), "anion_gap.normal"));
        assert_eq!(anion_gap(&ag(140.0, 100.0, 10.0)).unwrap().band, "anion_gap.high");
        assert_eq!(anion_gap(&ag(140.0, 110.0, 24.0)).unwrap().band, "anion_gap.low");
        // 16 is still normal; only above 16 is high.
        assert_eq!(anion_gap(&ag(140.0, 100.0, 24.0)).unwrap().band, "anion_gap.normal");
    }

    #[test]
    fn albumin_correction() {
        // Albumin 2.0: + 5 (the book's p. 9 example).
        let o = anion_gap(&ag(140.0, 108.0, 22.0).num("albumin_g_dl", 2.0)).unwrap();
        assert_eq!((o.score, o.band), (Some(15.0), "anion_gap.normal"));
        let o = anion_gap(&ag(140.0, 104.0, 24.0).num("albumin_g_dl", 4.5)).unwrap();
        assert_eq!(o.score, Some(12.0));
    }

    #[test]
    fn delta_ratio_bands() {
        let run = |ag: f64, hco3: f64| {
            delta_ratio(&Inputs::new().num("anion_gap_meq_l", ag).num("hco3_meq_l", hco3)).unwrap()
        };
        assert_eq!(run(12.0, 14.0).band, "delta_ratio.normal_ag");
        assert_eq!(run(18.0, 12.0).band, "delta_ratio.mixed");
        let o = run(24.0, 12.0);
        assert_eq!((o.score, o.band), (Some(1.0), "delta_ratio.pure_high"));
        assert_eq!(run(32.0, 18.0).band, "delta_ratio.with_alkalosis");
        assert!(delta_ratio(&Inputs::new().num("anion_gap_meq_l", 20.0).num("hco3_meq_l", 24.0)).is_err());
    }

    fn comp(disorder: &str, hco3: f64, pco2: f64) -> Outcome {
        compensation(
            &Inputs::new().choice("disorder", disorder).num("hco3_meq_l", hco3).num("pco2_mm_hg", pco2),
        )
        .unwrap()
    }

    #[test]
    fn winters_formula() {
        // HCO3 12: expected pCO2 26 ± 2.
        let o = comp("metabolic_acidosis", 12.0, 27.0);
        assert_eq!((o.score, o.band), (Some(26.0), "expected_compensation.appropriate"));
        assert_eq!(comp("metabolic_acidosis", 12.0, 35.0).band, "expected_compensation.added_resp_acidosis");
        assert_eq!(comp("metabolic_acidosis", 12.0, 20.0).band, "expected_compensation.added_resp_alkalosis");
        // Metabolic alkalosis: pCO2 should rise to 0.7 × 40 + 20 = 48 ± 5.
        assert_eq!(comp("metabolic_alkalosis", 40.0, 70.0).band, "expected_compensation.added_resp_acidosis");
        assert_eq!(
            comp("metabolic_alkalosis", 40.0, 38.0).band,
            "expected_compensation.added_resp_alkalosis"
        );
        // HCO3 40: expected pCO2 48 ± 5.
        assert_eq!(comp("metabolic_alkalosis", 40.0, 52.0).band, "expected_compensation.appropriate");
    }

    #[test]
    fn respiratory_rules() {
        // pCO2 60: acute HCO3 26, chronic 32.
        assert_eq!(comp("respiratory_acidosis_acute", 26.0, 60.0).score, Some(26.0));
        assert_eq!(
            comp("respiratory_acidosis_chronic", 32.0, 60.0).band,
            "expected_compensation.appropriate"
        );
        assert_eq!(
            comp("respiratory_acidosis_acute", 32.0, 60.0).band,
            "expected_compensation.added_met_alkalosis"
        );
        // pCO2 20: acute HCO3 20, chronic 14.
        assert_eq!(comp("respiratory_alkalosis_acute", 20.0, 20.0).score, Some(20.0));
        assert_eq!(comp("respiratory_alkalosis_chronic", 14.0, 20.0).score, Some(14.0));
        assert_eq!(
            comp("respiratory_alkalosis_acute", 15.0, 20.0).band,
            "expected_compensation.added_met_acidosis"
        );
        let wrong = Inputs::new()
            .choice("disorder", "respiratory_acidosis_acute")
            .num("hco3_meq_l", 24.0)
            .num("pco2_mm_hg", 30.0);
        assert!(compensation(&wrong).is_err());
    }

    #[test]
    fn osmolal_gap_cut_off() {
        // Calculated 2×140 + 90/18 + 14/2.8 = 290.
        let run = |m: f64| {
            osmolal_gap(
                &Inputs::new()
                    .num("measured_osm", m)
                    .num("na_meq_l", 140.0)
                    .num("glucose_mg_dl", 90.0)
                    .num("bun_mg_dl", 14.0),
            )
            .unwrap()
        };
        assert_eq!((run(300.0).score, run(300.0).band), (Some(10.0), "osmolal_gap.normal"));
        assert_eq!(run(320.0).band, "osmolal_gap.present");
    }
}
