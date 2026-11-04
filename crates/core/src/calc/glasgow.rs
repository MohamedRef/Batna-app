//! Glasgow alcoholic hepatitis score (GAHS).
//!
//! Source: Forrest et al., Gut 2005. Age < 50 = 1, ≥ 50 = 2; WBC < 15 = 1,
//! ≥ 15 = 2 (×10⁹/L); urea < 5 = 1, ≥ 5 = 2 (mmol/L); PT ratio or INR < 1.5 = 1,
//! 1.5–2.0 = 2, > 2.0 = 3; bilirubin < 125 = 1, 125–250 = 2, > 250 = 3 (µmol/L).
//! Score 5–12; ≥ 9 predicts poor 28-day survival, and in Forrest 2007 these
//! patients (with DF ≥ 32) benefited from corticosteroids. Not in the book.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num};

pub const SPEC: CalcSpec = CalcSpec {
    id: "glasgow_ah",
    name: "Glasgow alcoholic hepatitis score",
    area: Area::Liver,
    fields: &[
        num("age_years", "Age", "years", 1.0, 110.0, 1.0),
        num("wbc_10e9_l", "White cells", "×10⁹/L", 0.1, 100.0, 0.1).normal_as(normals::WBC_10E9_L),
        num("urea_mmol_l", "Urea", "mmol/L", 0.5, 80.0, 0.1)
            .normal_as(normals::UREA_MMOL_L)
            .hint("Urea in mg/dL ÷ 6 = mmol/L."),
        num("inr", "INR or PT ratio", "", 0.5, 15.0, 0.1).normal_as(normals::INR),
        num("bilirubin_mg_dl", "Bilirubin", "mg/dL", 0.1, 50.0, 0.1).normal_as(normals::BILIRUBIN_MG_DL),
    ],
    source: "Forrest 2005 (Gut)",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let age = i.get_num("age_years")?;
    let wbc = i.get_num("wbc_10e9_l")?;
    let urea = i.get_num("urea_mmol_l")?;
    let inr = i.get_num("inr")?;
    let bili = i.get_num("bilirubin_mg_dl")? * 17.1;
    let pts = [
        ("Age", if age < 50.0 { 1 } else { 2 }),
        ("White cells", if wbc < 15.0 { 1 } else { 2 }),
        ("Urea", if urea < 5.0 { 1 } else { 2 }),
        (
            "INR",
            if inr < 1.5 {
                1
            } else if inr <= 2.0 {
                2
            } else {
                3
            },
        ),
        (
            "Bilirubin",
            if bili < 125.0 {
                1
            } else if bili <= 250.0 {
                2
            } else {
                3
            },
        ),
    ];
    let total: u32 = pts.iter().map(|p| p.1).sum();
    let (band, label, meaning) = if total >= 9 {
        ("glasgow.high", "9 or more", "Poor 28-day survival; with DF ≥ 32 corticosteroids improved survival.")
    } else {
        (
            "glasgow.low",
            "Below 9",
            "Better 28-day survival; no survival benefit from corticosteroids was shown.",
        )
    };
    Ok(Outcome {
        score: Some(total as f64),
        display: total.to_string(),
        band,
        label,
        meaning: meaning.to_owned(),
        page: None,
        parts: pts.iter().map(|(l, p)| Part::new(*l, *p as f64)).collect(),
        notes: Vec::new(),
    })
}

/// Next steps per band (Forrest et al., Gut 2007, for the steroid benefit at
/// 9 or more with DF ≥ 32; Louvet 2007 for the Lille rule; book Lecture 12,
/// p. 196). The differential is shared with the Maddrey DF.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "glasgow.high",
        Guide {
            next: &[
                "With a Maddrey DF of 32 or more, prednisolone 40 mg a day for 28 days improved survival (Forrest 2007)",
                "Screen for infection and GI bleeding before the steroid (EASL 2018)",
                "Lille score at day 4–7: 0.45 or more is no response, so stop the steroid (Louvet 2007)",
                "Abstinence and nutrition support (p. 196)",
            ],
            dd: super::maddrey::AH_DD,
        },
    ),
    (
        "glasgow.low",
        Guide {
            next: &[
                "Steroids showed no survival benefit below 9 (Forrest 2007): supportive care, abstinence and nutrition",
                "Rescore if the patient worsens",
            ],
            dd: super::maddrey::AH_DD,
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worked_example() {
        // 55 y (2), WBC 18 (2), urea 7 (2), INR 1.8 (2), bilirubin 15 mg/dL = 256 µmol/L (3) → 11.
        let o = compute(
            &Inputs::new()
                .num("age_years", 55.0)
                .num("wbc_10e9_l", 18.0)
                .num("urea_mmol_l", 7.0)
                .num("inr", 1.8)
                .num("bilirubin_mg_dl", 15.0),
        )
        .unwrap();
        assert_eq!(o.score, Some(11.0));
        assert_eq!(o.band, "glasgow.high");
    }

    #[test]
    fn minimum_is_five() {
        let o = compute(
            &Inputs::new()
                .num("age_years", 35.0)
                .num("wbc_10e9_l", 8.0)
                .num("urea_mmol_l", 3.0)
                .num("inr", 1.1)
                .num("bilirubin_mg_dl", 3.0),
        )
        .unwrap();
        assert_eq!(o.score, Some(5.0));
        assert_eq!(o.band, "glasgow.low");
    }
}
