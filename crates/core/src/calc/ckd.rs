//! Chronic kidney disease: eGFR by CKD-EPI 2021 and the KDIGO G/A stage.
//!
//! Source: the Nephrology book (Chapter 8, Lecture 2). p. 29 prints worked
//! examples of the "CKD-Epi 2021 updated formula" (creatinine 1.3 and
//! 4.5 mg/dL in men and women aged 24 and 60); the tests below use all
//! eight. The book does not print the 2021 equation itself; it is the
//! race-free one from Inker et al., NEJM 2021: 142 × min(Scr/κ, 1)^α ×
//! max(Scr/κ, 1)^−1.200 × 0.9938^age × 1.012 (women), κ 0.7 (women) / 0.9
//! (men), α −0.241 / −0.302. Lecture 13 (p. 432) prints the older 2009
//! equation with a race factor; the notes say so.
//!
//! The KDIGO 2012 heat map (p. 32, "Monitoring/yr for progression of CKD";
//! also p. 31): G1 ≥ 90, G2 60–89, G3a 45–59, G3b 30–44, G4 15–29, G5 < 15;
//! A1 < 30, A2 30–299, A3 ≥ 300 mg/g; each cell has a risk colour and a
//! number of checks a year. CKD needs a marker of damage or GFR < 60 for
//! more than 3 months (p. 24).

use super::{Area, CalcError, CalcSpec, Inputs, Outcome, Part, choice, num, show};

const SEX: &[(&str, &str)] = &[("male", "Man"), ("female", "Woman")];

pub const EGFR: CalcSpec = CalcSpec {
    id: "ckd_epi_2021",
    name: "eGFR (CKD-EPI 2021, race-free)",
    area: Area::Renal,
    fields: &[
        num("creatinine_mg_dl", "Serum creatinine", "mg/dL", 0.1, 25.0, 0.01),
        num("age_years", "Age", "years", 18.0, 120.0, 1.0),
        choice("sex", "Sex", SEX),
    ],
    source: "Book Nephrology Lecture 2, p. 29 (worked examples); equation from Inker et al., \
             NEJM 2021 (the book doesn't print it)",
    compute: egfr,
};

/// CKD-EPI 2021 creatinine equation, mL/min/1.73 m².
pub(crate) fn ckd_epi_2021(cr: f64, age: f64, woman: bool) -> f64 {
    let (kappa, alpha) = if woman { (0.7, -0.241) } else { (0.9, -0.302) };
    let r = cr / kappa;
    let sex = if woman { 1.012 } else { 1.0 };
    142.0 * r.min(1.0).powf(alpha) * r.max(1.0).powf(-1.200) * 0.9938f64.powf(age) * sex
}

fn egfr(i: &Inputs) -> Result<Outcome, CalcError> {
    let cr = i.get_pos("creatinine_mg_dl")?;
    let age = i.get_num("age_years")?;
    if age < 18.0 {
        return Err(CalcError::Invalid {
            field: "age_years",
            reason: "the equation is for adults (18 and over)",
        });
    }
    let woman = i.get_choice("sex", &["male", "female"])? == "female";
    let gfr = (ckd_epi_2021(cr, age, woman) * 10.0).round() / 10.0;
    let g = g_category(gfr);
    Ok(Outcome {
        score: Some(gfr),
        display: format!("{} mL/min/1.73 m²", show(gfr, 1)),
        band: g.0,
        label: g.1,
        meaning: format!("{}. {}", g.2, egfr_next(gfr)),
        page: Some(29),
        parts: vec![
            Part::new("Creatinine (mg/dL)", cr),
            Part::new("Age (years)", age),
            Part::new(if woman { "Woman (× 1.012)" } else { "Man" }, if woman { 1.012 } else { 1.0 }),
        ],
        notes: vec![
            "DD of a low eGFR: AKI vs CKD (old results, kidney size); a creatinine raised without a GFR \
             fall (trimethoprim, cimetidine, creatine, high muscle mass); a falsely high eGFR with low \
             muscle (amputation, cachexia, cirrhosis): confirm with cystatin C."
                .to_owned(),
            "Lecture 13 (p. 432) prints the 2009 CKD-EPI equation with a race factor; the 2021 \
             one used here (and in the p. 29 examples) has none."
                .to_owned(),
        ],
    })
}

/// The next step for an eGFR (KDIGO 2012 / 2024).
fn egfr_next(gfr: f64) -> &'static str {
    if gfr >= 60.0 {
        "On its own this is not CKD: check the ACR and urine; CKD needs a marker of damage for more \
         than 3 months (p. 24)."
    } else if gfr >= 30.0 {
        "CKD if it lasts more than 3 months (p. 24): check the ACR to stage it (G and A), review drug \
         doses, and treat BP and proteinuria."
    } else {
        "Severely reduced: rule out AKI, refer to nephrology, adjust every renal drug, and start \
         planning kidney replacement if it is falling (p. 36)."
    }
}

/// (band, label, meaning) of a GFR category.
type Band = (&'static str, &'static str, &'static str);

fn g_category(gfr: f64) -> Band {
    if gfr >= 90.0 {
        ("ckd_epi_2021.g1", "G1 (≥ 90)", "Normal or high")
    } else if gfr >= 60.0 {
        ("ckd_epi_2021.g2", "G2 (60–89)", "Mildly decreased")
    } else if gfr >= 45.0 {
        ("ckd_epi_2021.g3a", "G3a (45–59)", "Mildly to moderately decreased")
    } else if gfr >= 30.0 {
        ("ckd_epi_2021.g3b", "G3b (30–44)", "Moderately to severely decreased")
    } else if gfr >= 15.0 {
        ("ckd_epi_2021.g4", "G4 (15–29)", "Severely decreased")
    } else {
        ("ckd_epi_2021.g5", "G5 (< 15)", "Kidney failure")
    }
}

pub const STAGE: CalcSpec = CalcSpec {
    id: "kdigo_ckd",
    name: "KDIGO CKD stage (G and A)",
    area: Area::Renal,
    fields: &[
        num("egfr", "eGFR", "mL/min/1.73 m²", 0.0, 200.0, 0.1),
        num("acr_mg_g", "Urine albumin : creatinine ratio (ACR)", "mg/g", 0.0, 20000.0, 1.0)
            .hint("The book's table: 3 mg/mmol = 30 mg/g."),
    ],
    source: "KDIGO 2012 CKD guideline (Kidney Int Suppl 2013); book Nephrology Lecture 2, pp. 24, 31–32",
    compute: stage,
};

fn stage(i: &Inputs) -> Result<Outcome, CalcError> {
    let gfr = i.get_num("egfr")?;
    let acr = i.get_num("acr_mg_g")?;
    if gfr < 0.0 {
        return Err(CalcError::Invalid { field: "egfr", reason: "can't be negative" });
    }
    if acr < 0.0 {
        return Err(CalcError::Invalid { field: "acr_mg_g", reason: "can't be negative" });
    }
    // Row of the p. 32 heat map: 0 = G1 … 5 = G5.
    let (row, g) = match gfr {
        x if x >= 90.0 => (0, "G1"),
        x if x >= 60.0 => (1, "G2"),
        x if x >= 45.0 => (2, "G3a"),
        x if x >= 30.0 => (3, "G3b"),
        x if x >= 15.0 => (4, "G4"),
        _ => (5, "G5"),
    };
    let (col, a) = match acr {
        x if x < 30.0 => (0, "A1"),
        x if x < 300.0 => (1, "A2"),
        _ => (2, "A3"),
    };
    // Risk (0 green … 3 red) and checks a year, as printed on p. 32.
    const RISK: [[u8; 3]; 6] = [[0, 1, 2], [0, 1, 2], [1, 2, 3], [2, 3, 3], [3, 3, 3], [3, 3, 3]];
    const VISITS: [[&str; 3]; 6] = [
        ["1", "1", "3"],
        ["1", "1", "3"],
        ["1", "2", "3"],
        ["2", "3", "3"],
        ["3", "3", "4+"],
        ["4+", "4+", "4+"],
    ];
    let (band, label, meaning) = match RISK[row][col] {
        0 => (
            "kdigo_ckd.low",
            "Low risk (green)",
            "Low risk: with no other marker of kidney damage this is not CKD; screen once a year.",
        ),
        1 => (
            "kdigo_ckd.moderate",
            "Moderately increased risk (yellow)",
            "Moderately increased risk: treat the cause, control BP, start an ACE inhibitor or ARB if there \
             is albuminuria, and an SGLT2 inhibitor where indicated.",
        ),
        2 => (
            "kdigo_ckd.high",
            "High risk (orange)",
            "High risk of progression and of cardiovascular events: ACE inhibitor or ARB at the highest \
             tolerated dose, SGLT2 inhibitor, statin; refer if ACR ≥ 300 or eGFR is falling fast.",
        ),
        _ => (
            "kdigo_ckd.very_high",
            "Very high risk (red)",
            "Very high risk: nephrology care, check for CKD complications (anaemia, mineral bone \
             disease, acidosis, potassium), and plan kidney replacement as eGFR nears 20.",
        ),
    };
    let visits = VISITS[row][col];
    let what = if RISK[row][col] == 0 { "Screen" } else { "Treat" };
    let mut notes = vec![format!("{what}, and check eGFR and ACR {visits} time(s) a year (p. 32).")];
    if gfr < 30.0 {
        notes.push(
            "Below 30: plan nephrology care; below 20, access and transplant planning (p. 36).".to_owned(),
        );
    }
    notes.push("CKD needs the abnormality for more than 3 months (p. 24).".to_owned());
    if col > 0 {
        notes.push(
            "DD of albuminuria: diabetic or hypertensive kidney disease, glomerulonephritis; a transient \
             rise with exercise, fever, urinary infection, menstruation or haematuria (Table 17, p. 30): \
             repeat it."
                .to_owned(),
        );
    }
    Ok(Outcome {
        score: None,
        display: format!("{g} {a}"),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(32),
        parts: vec![Part::new(format!("GFR category {g}"), gfr), Part::new(format!("Albuminuria {a}"), acr)],
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(cr: f64, age: f64, sex: &str) -> f64 {
        egfr(&Inputs::new().num("creatinine_mg_dl", cr).num("age_years", age).choice("sex", sex))
            .unwrap()
            .score
            .unwrap()
    }

    // The book's own table, p. 29.
    #[test]
    fn the_books_worked_examples() {
        for (cr, age, sex, want) in [
            (1.3, 24.0, "male", 78.7),
            (4.5, 24.0, "male", 17.7),
            (1.3, 60.0, "male", 62.9),
            (4.5, 60.0, "male", 14.2),
            (1.3, 24.0, "female", 58.9),
            (4.5, 24.0, "female", 13.3),
            (1.3, 60.0, "female", 47.1),
            (4.5, 60.0, "female", 10.6),
        ] {
            assert_eq!(run(cr, age, sex), want, "{cr} {age} {sex}");
        }
    }

    #[test]
    fn egfr_band_is_the_g_category() {
        let o =
            egfr(&Inputs::new().num("creatinine_mg_dl", 1.3).num("age_years", 60.0).choice("sex", "male"))
                .unwrap();
        assert_eq!(o.band, "ckd_epi_2021.g2");
        let o =
            egfr(&Inputs::new().num("creatinine_mg_dl", 4.5).num("age_years", 60.0).choice("sex", "male"))
                .unwrap();
        assert_eq!(o.band, "ckd_epi_2021.g5");
    }

    fn st(gfr: f64, acr: f64) -> Outcome {
        stage(&Inputs::new().num("egfr", gfr).num("acr_mg_g", acr)).unwrap()
    }

    #[test]
    fn the_heat_map() {
        assert_eq!((st(95.0, 10.0).display.as_str(), st(95.0, 10.0).band), ("G1 A1", "kdigo_ckd.low"));
        assert_eq!(st(70.0, 30.0).band, "kdigo_ckd.moderate");
        assert_eq!(st(50.0, 10.0).band, "kdigo_ckd.moderate");
        assert_eq!(st(50.0, 100.0).band, "kdigo_ckd.high");
        assert_eq!(st(40.0, 10.0).band, "kdigo_ckd.high");
        assert_eq!(st(60.0, 300.0).band, "kdigo_ckd.high");
        assert_eq!(st(59.9, 300.0).band, "kdigo_ckd.very_high");
        assert_eq!(st(40.0, 299.0).band, "kdigo_ckd.very_high");
        assert_eq!(st(20.0, 5.0).band, "kdigo_ckd.very_high");
        assert_eq!(st(10.0, 5.0).display, "G5 A1");
        assert!(st(20.0, 400.0).notes[0].contains("4+"));
        assert!(st(50.0, 100.0).notes[0].contains("2 time"));
    }
}
