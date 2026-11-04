//! Maddrey discriminant function (DF) for alcoholic hepatitis.
//!
//! Formula: 4.6 × (patient PT − control PT, seconds) + bilirubin (mg/dL);
//! DF ≥ 32 is severe alcoholic hepatitis with high short-term mortality, where
//! corticosteroids are considered (Maddrey 1978; Carithers 1989; AASLD 2019
//! alcohol-associated liver disease guidance). The book (Lecture 12, p. 196)
//! discusses steroids for moderate to severe disease but gives no DF.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num, show};

pub const SPEC: CalcSpec = CalcSpec {
    id: "maddrey",
    name: "Maddrey discriminant function",
    area: Area::Liver,
    fields: &[
        num("pt_s", "Prothrombin time (patient)", "s", 5.0, 120.0, 0.1).normal_as(normals::PT_S),
        num("pt_control_s", "Prothrombin time (control)", "s", 5.0, 30.0, 0.1).hint("Often 12–13 s."),
        num("bilirubin_mg_dl", "Bilirubin", "mg/dL", 0.1, 50.0, 0.1).normal_as(normals::BILIRUBIN_MG_DL),
    ],
    source: "Maddrey 1978, Carithers 1989; AASLD 2019",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let pt = i.get_pos("pt_s")?;
    let ctl = i.get_pos("pt_control_s")?;
    let bili = i.get_num("bilirubin_mg_dl")?;
    let pt_part = 4.6 * (pt - ctl);
    // Rounded to one decimal, as reported, so DF 32.0 lands on the cut-off.
    let df = ((pt_part + bili) * 10.0).round() / 10.0;
    let (band, label, meaning) = if df >= 32.0 {
        (
            "maddrey.severe",
            "Severe (DF ≥ 32)",
            "Severe alcoholic hepatitis: high 1-month mortality; consider corticosteroids if no contraindication.",
        )
    } else {
        (
            "maddrey.not_severe",
            "Not severe (DF < 32)",
            "Lower short-term mortality; corticosteroids not indicated by DF alone.",
        )
    };
    Ok(Outcome {
        score: Some(df),
        display: show(df, 1),
        band,
        label,
        meaning: meaning.to_owned(),
        page: None,
        parts: vec![Part::new("4.6 × PT prolongation", pt_part), Part::new("Bilirubin", bili)],
        notes: Vec::new(),
    })
}

/// Next steps per band. The book (Lecture 12, p. 196) advises abstinence and
/// corticosteroids for moderate to severe alcoholic hepatitis without a
/// threshold; the doses, the infection screen and the Lille stopping rule
/// are from AASLD 2019 (Crabb et al., Hepatology 2020) and EASL 2018, and
/// the Lille score from Louvet et al., Hepatology 2007.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "maddrey.severe",
        Guide {
            next: &[
                "Prednisolone 40 mg a day for 28 days if there is no contraindication: active infection, GI bleeding, AKI, pancreatitis (AASLD 2019; book p. 196)",
                "Screen for infection first: blood, urine and ascitic cultures, chest X-ray (EASL 2018)",
                "Lille score at day 4–7: 0.45 or more is no response, so stop the steroid (Louvet 2007)",
                "Abstinence and nutrition: 35–40 kcal/kg and protein 1.2–1.5 g/kg a day (pp. 15, 196)",
                "Non-responders: ask a transplant centre about early transplant (EASL 2018)",
            ],
            dd: AH_DD,
        },
    ),
    (
        "maddrey.not_severe",
        Guide {
            next: &[
                "Supportive care: abstinence, nutrition, treat any infection; no steroid on the DF alone (p. 196)",
                "Recalculate if the bilirubin or INR rises; a MELD above 20 also marks severe disease (AASLD 2019)",
            ],
            dd: AH_DD,
        },
    ),
];

/// What else gives jaundice and a long INR in a drinker (AASLD 2019 on
/// confounders; book Lecture 8, p. 133, for the AST of alcohol).
pub(super) const AH_DD: &[&str] = &[
    "Decompensated alcohol-related cirrhosis without hepatitis",
    "Drug-induced liver injury, including paracetamol taken with alcohol",
    "Viral or autoimmune hepatitis",
    "Bile duct obstruction or cholangitis (ultrasound)",
    "Sepsis or ischaemic hepatitis: an AST above 300 points away from alcohol alone (p. 133)",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(pt: f64, ctl: f64, b: f64) -> Outcome {
        compute(&Inputs::new().num("pt_s", pt).num("pt_control_s", ctl).num("bilirubin_mg_dl", b)).unwrap()
    }

    #[test]
    fn worked_examples() {
        // 4.6 × (19 − 12) + 10 = 42.2, severe.
        let o = run(19.0, 12.0, 10.0);
        assert!((o.score.unwrap() - 42.2).abs() < 1e-9);
        assert_eq!(o.band, "maddrey.severe");
        // 4.6 × (15 − 12) + 5 = 18.8.
        assert_eq!(run(15.0, 12.0, 5.0).band, "maddrey.not_severe");
        // Exactly 32: 4.6 × 5 + 9.
        assert_eq!(run(17.0, 12.0, 9.0).band, "maddrey.severe");
    }
}
