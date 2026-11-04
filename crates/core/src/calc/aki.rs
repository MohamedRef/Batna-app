//! Acute kidney injury: KDIGO stage, FENa and FEurea.
//!
//! Source: the Nephrology book (Chapter 8, Lecture 5). KDIGO (p. 100,
//! Table 28-1): stage 1 creatinine 1.5–1.9 × baseline or a rise of
//! ≥ 0.3 mg/dL, urine < 0.5 mL/kg/h for 6–12 h; stage 2 creatinine
//! 2.0–2.9 ×, urine < 0.5 mL/kg/h for ≥ 12 h; stage 3 creatinine 3 ×, or
//! ≥ 4.0 mg/dL, or renal replacement therapy, urine < 0.3 mL/kg/h for ≥ 24 h
//! or anuria for ≥ 12 h. Prerenal azotemia vs ATN (p. 109): FENa below 1%
//! vs above 2%, FEurea below 35% vs above 50%. FE = (urine X × plasma Cr)
//! / (plasma X × urine Cr) × 100.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, flag, normals, num, show};

const URINE_OUTPUT: &[(&str, &str)] = &[
    ("normal", "Not reduced"),
    ("lt05_6h", "< 0.5 mL/kg/h for 6–12 h"),
    ("lt05_12h", "< 0.5 mL/kg/h for ≥ 12 h"),
    ("lt03_24h_or_anuria", "< 0.3 mL/kg/h for ≥ 24 h, or anuria ≥ 12 h"),
];

pub const KDIGO: CalcSpec = CalcSpec {
    id: "kdigo_aki",
    name: "KDIGO AKI stage",
    area: Area::Renal,
    fields: &[
        num("baseline_cr_mg_dl", "Baseline creatinine", "mg/dL", 0.1, 20.0, 0.01)
            .normal_as(normals::CREATININE_MG_DL),
        num("current_cr_mg_dl", "Current creatinine", "mg/dL", 0.1, 30.0, 0.01)
            .normal_as(normals::CREATININE_MG_DL),
        choice("urine_output", "Urine output", URINE_OUTPUT),
        flag("rrt", "Renal replacement therapy started"),
    ],
    source: "KDIGO 2012; book Nephrology Lecture 5, p. 100",
    compute: kdigo,
};

fn kdigo(i: &Inputs) -> Result<Outcome, CalcError> {
    let base = i.get_pos("baseline_cr_mg_dl")?;
    let now = i.get_pos("current_cr_mg_dl")?;
    let uo = i.get_choice("urine_output", &["normal", "lt05_6h", "lt05_12h", "lt03_24h_or_anuria"])?;
    let rrt = i.get_flag("rrt")?;
    let ratio = now / base;
    let cr_stage = if ratio >= 3.0 || now >= 4.0 || rrt {
        3.0
    } else if ratio >= 2.0 {
        2.0
    } else if ratio >= 1.5 || now - base >= 0.3 {
        1.0
    } else {
        0.0
    };
    let uo_stage = match uo {
        "lt03_24h_or_anuria" => 3.0,
        "lt05_12h" => 2.0,
        "lt05_6h" => 1.0,
        _ => 0.0,
    };
    let stage = f64::max(cr_stage, uo_stage);
    let (band, label, meaning) = match stage as u8 {
        0 => (
            "kdigo_aki.none",
            "No AKI",
            "Neither the creatinine nor the urine output meets the AKI criteria.",
        ),
        1 => ("kdigo_aki.1", "Stage 1", "AKI stage 1."),
        2 => ("kdigo_aki.2", "Stage 2", "AKI stage 2."),
        _ => ("kdigo_aki.3", "Stage 3", "AKI stage 3."),
    };
    Ok(Outcome {
        score: Some(stage),
        display: label.to_owned(),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(100),
        parts: vec![
            Part::new(format!("Creatinine × {} of baseline", show(ratio, 2)), cr_stage),
            Part::new("Urine output", uo_stage),
        ],
        notes: vec![
            "A rise of ≥ 0.3 mg/dL counts within 48 hours; a 1.5-fold rise within 7 days.".to_owned(),
            "The stage is the higher of the creatinine and urine-output stages.".to_owned(),
        ],
    })
}

pub const FENA: CalcSpec = CalcSpec {
    id: "fena",
    name: "FENa (fractional excretion of sodium)",
    area: Area::Renal,
    fields: &[
        num("urine_na_meq_l", "Urine sodium", "mEq/L", 1.0, 300.0, 1.0),
        num("plasma_na_meq_l", "Plasma sodium", "mEq/L", 90.0, 200.0, 1.0).normal_as(normals::SODIUM),
        num("urine_cr_mg_dl", "Urine creatinine", "mg/dL", 1.0, 1000.0, 0.1),
        num("plasma_cr_mg_dl", "Plasma creatinine", "mg/dL", 0.1, 30.0, 0.01)
            .normal_as(normals::CREATININE_MG_DL),
    ],
    source: "Book Nephrology Lecture 5, p. 109",
    compute: fena,
};

pub const FEUREA: CalcSpec = CalcSpec {
    id: "feurea",
    name: "FEurea (fractional excretion of urea)",
    area: Area::Renal,
    fields: &[
        num("urine_urea_mg_dl", "Urine urea", "mg/dL", 1.0, 5000.0, 1.0)
            .hint("Urea or BUN, the same unit as the plasma value."),
        num("plasma_urea_mg_dl", "Plasma urea", "mg/dL", 1.0, 500.0, 0.1).normal_as(normals::UREA_MG_DL),
        num("urine_cr_mg_dl", "Urine creatinine", "mg/dL", 1.0, 1000.0, 0.1),
        num("plasma_cr_mg_dl", "Plasma creatinine", "mg/dL", 0.1, 30.0, 0.01)
            .normal_as(normals::CREATININE_MG_DL),
    ],
    source: "Book Nephrology Lecture 5, p. 109",
    compute: feurea,
};

type Band = (&'static str, &'static str, &'static str);

fn fraction(
    i: &Inputs,
    (urine, plasma): (&'static str, &'static str),
    (low, high): (f64, f64),
    [pre, mid, atn]: [Band; 3],
) -> Result<Outcome, CalcError> {
    let ux = i.get_num(urine)?;
    let px = i.get_pos(plasma)?;
    let ucr = i.get_pos("urine_cr_mg_dl")?;
    let pcr = i.get_pos("plasma_cr_mg_dl")?;
    let fe = ((ux * pcr) / (px * ucr) * 100.0 * 100.0).round() / 100.0;
    let (band, label, meaning) = if fe < low {
        pre
    } else if fe > high {
        atn
    } else {
        mid
    };
    Ok(Outcome {
        score: Some(fe),
        display: format!("{}%", show(fe, 2)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(109),
        parts: vec![Part::new("Urine ÷ plasma", ux / px), Part::new("Plasma Cr ÷ urine Cr", pcr / ucr)],
        notes: Vec::new(),
    })
}

fn fena(i: &Inputs) -> Result<Outcome, CalcError> {
    fraction(
        i,
        ("urine_na_meq_l", "plasma_na_meq_l"),
        (1.0, 2.0),
        [
            ("fena.prerenal", "< 1%", "Favours prerenal azotemia."),
            ("fena.indeterminate", "1–2%", "Between the cut-offs: read with the other urinary indices."),
            ("fena.atn", "> 2%", "Favours acute tubular necrosis."),
        ],
    )
}

fn feurea(i: &Inputs) -> Result<Outcome, CalcError> {
    fraction(
        i,
        ("urine_urea_mg_dl", "plasma_urea_mg_dl"),
        (35.0, 50.0),
        [
            ("feurea.prerenal", "< 35%", "Favours prerenal azotemia (useful after diuretics)."),
            ("feurea.indeterminate", "35–50%", "Between the cut-offs: read with the other urinary indices."),
            ("feurea.atn", "> 50%", "Favours acute tubular necrosis."),
        ],
    )
}

/// Next steps and differential per band (book Nephrology Lecture 5,
/// pp. 100–118). Where the book is silent: the stage-based care of the
/// KDIGO AKI guideline 2012; a low FENa outside prerenal states from
/// Steiner, Am J Med 1984; FEurea after diuretics from Carvounis et al.,
/// Kidney Int 2002, with the 2024 meta-analysis by Abdelhafez et al.,
/// Am J Med Sci.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "kdigo_aki.none",
        Guide {
            next: &[
                "Not AKI by KDIGO: creatinine up 0.3 mg/dL in 48 h or 50% in 7 days, or urine below 0.5 mL/kg/h for 6 h (p. 100)",
                "While the risk lasts, repeat the creatinine and watch the urine output: creatinine rises late after an injury (pp. 116–118)",
                "Check the baseline: the first creatinine in hospital may already be raised (p. 101)",
            ],
            dd: &[],
        },
    ),
    (
        "kdigo_aki.1",
        Guide {
            next: &[
                "Find the cause: history, volume status and every drug; urine analysis, urinary indices and sediment; renal ultrasound (pp. 107–110)",
                "Restore perfusion with isotonic crystalloids rather than starches (p. 111)",
                "Stop nephrotoxins where you can: NSAIDs, ACE inhibitors, ARBs, aminoglycosides (p. 111)",
                "Adjust drug doses to the kidney function (p. 107)",
            ],
            dd: &[
                PRERENAL,
                INTRINSIC,
                POSTRENAL,
                "A rise that is not injury: a new ACE inhibitor or ARB, decongestion in heart failure, intensive blood pressure control (p. 101)",
            ],
        },
    ),
    (
        "kdigo_aki.2",
        Guide {
            next: &[
                "Watch for the complications: high potassium, fluid overload, acidosis, low sodium, high phosphate (p. 106)",
                "Diuretics only for fluid overload, never to prevent or treat AKI; no low-dose dopamine (pp. 112–113)",
                "In shock, vasopressors to a mean arterial pressure of 65–70 mmHg once filled (p. 112)",
                "Oliguria despite resuscitation: a furosemide stress test (1 mg/kg) separates prerenal from intrinsic (p. 113)",
                "Consider intensive care and plan renal replacement therapy early (KDIGO 2012)",
            ],
            dd: AKI_DD,
        },
    ),
    (
        "kdigo_aki.3",
        Guide {
            next: &[
                "Dialysis for AEIOU: acidosis, electrolytes (high potassium), intoxication, overload, uraemia (p. 115)",
                "Intermittent haemodialysis if stable; if not, continuous renal replacement (CRRT), peritoneal dialysis or slow low-efficiency dialysis (SLED) (p. 115)",
                "Treat the cause: relieve obstruction; immunosuppression for glomerulonephritis or vasculitis; stop the drug in interstitial nephritis (p. 114)",
                "Avoid subclavian lines, to keep the veins for dialysis (KDIGO 2012)",
                "Follow the kidneys after recovery: 10% of those who needed dialysis go on to CKD (p. 115)",
            ],
            dd: AKI_DD,
        },
    ),
    ("fena.prerenal", Guide { next: PRERENAL_NEXT, dd: LOW_FENA_DD }),
    (
        "fena.indeterminate",
        Guide {
            next: &[
                BY_INDICES,
                "After diuretics, use FEurea instead: below 35% still favours prerenal (Carvounis 2002)",
            ],
            dd: MIDDLE_DD,
        },
    ),
    (
        "fena.atn",
        Guide {
            next: ATN_NEXT,
            dd: &[
                ATN,
                "A high FENa with no tubular injury: diuretics, or chronic kidney disease (Steiner 1984)",
            ],
        },
    ),
    (
        "feurea.prerenal",
        Guide {
            next: &[
                PERFUSE,
                OTHER_INDICES,
                "Unlike the FENa it holds after diuretics (Carvounis 2002), though a 2024 meta-analysis found it no better (Abdelhafez 2024)",
                RECHECK,
            ],
            dd: &[PRERENAL],
        },
    ),
    ("feurea.indeterminate", Guide { next: &[BY_INDICES], dd: &[TURNING] }),
    ("feurea.atn", Guide { next: ATN_NEXT, dd: &[ATN] }),
];

const PRERENAL: &str = "Prerenal: blood or fluid loss, heart failure, liver failure, sepsis; NSAIDs, ACE inhibitors, ARBs, ciclosporin (p. 105)";
const INTRINSIC: &str = "Intrinsic: acute tubular necrosis, glomerulonephritis, interstitial nephritis, renal vessel disease (pp. 104–106)";
const POSTRENAL: &str = "Postrenal: prostate disease, a tumour or stone blocking the urinary tract (p. 106)";
const ATN: &str = "Acute tubular necrosis: ischaemia, sepsis, contrast, aminoglycosides, amphotericin, myoglobin or haemoglobin (p. 105)";

const AKI_DD: &[&str] = &[PRERENAL, INTRINSIC, POSTRENAL];

const PERFUSE: &str =
    "Restore perfusion with isotonic crystalloids and stop NSAIDs, ACE inhibitors and ARBs (p. 111)";
const OTHER_INDICES: &str = "Back it with the other indices: urine sodium below 20, urine osmolality above 500, BUN to creatinine above 20, a bland sediment (p. 109)";
const RECHECK: &str =
    "Recheck the creatinine: prerenal AKI recovers fully when it is corrected promptly (p. 105)";
const BY_INDICES: &str =
    "Between the cut-offs: decide with the urine sodium, urine osmolality and the sediment (p. 109)";

const PRERENAL_NEXT: &[&str] = &[PERFUSE, OTHER_INDICES, RECHECK];

const ATN_NEXT: &[&str] = &[
    "Favours acute tubular necrosis: look for epithelial cell and granular casts (p. 109)",
    "Stop nephrotoxins, keep the volume right, and give diuretics only for overload (pp. 111, 113)",
    "Watch for the dialysis indications: acidosis, high potassium, intoxication, overload, uraemia (p. 115)",
];

const LOW_FENA_DD: &[&str] = &[
    PRERENAL,
    "A low FENa that is not prerenal: contrast nephropathy, myoglobin or haemoglobin, acute glomerulonephritis, early obstruction (Steiner 1984)",
];

const TURNING: &str = "Prerenal AKI turning into tubular injury when not corrected in time (p. 106)";

const MIDDLE_DD: &[&str] = &[TURNING, "Diuretics raise the FENa even in a prerenal state (Steiner 1984)"];

#[cfg(test)]
mod tests {
    use super::*;

    fn stage(base: f64, now: f64, uo: &str) -> Outcome {
        kdigo(
            &Inputs::new()
                .num("baseline_cr_mg_dl", base)
                .num("current_cr_mg_dl", now)
                .choice("urine_output", uo),
        )
        .unwrap()
    }

    #[test]
    fn kdigo_stages() {
        assert_eq!(stage(1.0, 1.2, "normal").band, "kdigo_aki.none");
        assert_eq!(stage(1.0, 1.3, "normal").band, "kdigo_aki.1"); // + 0.3
        assert_eq!(stage(1.0, 1.5, "normal").band, "kdigo_aki.1");
        assert_eq!(stage(1.0, 2.0, "normal").band, "kdigo_aki.2");
        assert_eq!(stage(1.0, 3.0, "normal").band, "kdigo_aki.3");
        assert_eq!(stage(2.5, 4.0, "normal").band, "kdigo_aki.3"); // ≥ 4.0
        assert_eq!(stage(1.0, 1.0, "lt05_12h").band, "kdigo_aki.2"); // urine wins
        let rrt = Inputs::new()
            .num("baseline_cr_mg_dl", 1.0)
            .num("current_cr_mg_dl", 1.1)
            .choice("urine_output", "normal")
            .flag("rrt", true);
        assert_eq!(kdigo(&rrt).unwrap().band, "kdigo_aki.3");
    }

    #[test]
    fn fractional_excretion() {
        // (10 × 2) / (140 × 100) × 100 = 0.14%.
        let i = Inputs::new()
            .num("urine_na_meq_l", 10.0)
            .num("plasma_na_meq_l", 140.0)
            .num("urine_cr_mg_dl", 100.0)
            .num("plasma_cr_mg_dl", 2.0);
        let o = fena(&i).unwrap();
        assert_eq!((o.score, o.band), (Some(0.14), "fena.prerenal"));
        let i = i.num("urine_na_meq_l", 80.0).num("urine_cr_mg_dl", 40.0);
        assert_eq!(fena(&i).unwrap().band, "fena.atn"); // 2.86%
        // (300 × 2) / (60 × 40) × 100 = 25%.
        let u = Inputs::new()
            .num("urine_urea_mg_dl", 300.0)
            .num("plasma_urea_mg_dl", 60.0)
            .num("urine_cr_mg_dl", 40.0)
            .num("plasma_cr_mg_dl", 2.0);
        assert_eq!(feurea(&u).unwrap().band, "feurea.prerenal");
    }
}
