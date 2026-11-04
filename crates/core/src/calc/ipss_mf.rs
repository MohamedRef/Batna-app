//! IPSS for primary myelofibrosis (at diagnosis).
//!
//! Source: the Hematology book (Chapter 4, Lecture 9, p. 151): 1 point each
//! for age over 65, WBC over 25 × 10⁹/L, haemoglobin under 100 g/L
//! (10 g/dL), peripheral blasts 1% or more, and constitutional symptoms.
//! Low 0, intermediate-1 1, intermediate-2 2, high 3 or more; median
//! survival 135, 95, 48 and 27 months. The book prints the blast cut-off
//! as "< 1 / > 1"; the original IPSS (Cervantes 2009) counts ≥ 1%.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, flag, normals, num};

pub const SPEC: CalcSpec = CalcSpec {
    id: "ipss_mf",
    name: "IPSS (primary myelofibrosis)",
    area: Area::Blood,
    fields: &[
        num("age_years", "Age", "years", 18.0, 110.0, 1.0),
        num("wbc_10e9_l", "WBC", "× 10⁹/L", 0.1, 500.0, 0.1).normal_as(normals::WBC_10E9_L),
        num("hb_g_dl", "Haemoglobin", "g/dL", 2.0, 25.0, 0.1).normal_as(normals::HB_G_DL),
        num("blasts_pct", "Peripheral blasts", "%", 0.0, 100.0, 0.1).normal_as(normals::BLASTS_PCT),
        flag("constitutional", "Constitutional symptoms"),
    ],
    source: "Cervantes 2009; book Hematology Lecture 9, p. 151",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let parts = vec![
        Part::met("Age > 65", i.get_num("age_years")? > 65.0),
        Part::met("WBC > 25 × 10⁹/L", i.get_num("wbc_10e9_l")? > 25.0),
        Part::met("Hb < 10 g/dL", i.get_num("hb_g_dl")? < 10.0),
        Part::met("Blasts ≥ 1%", i.get_num("blasts_pct")? >= 1.0),
        Part::met("Constitutional symptoms", i.get_flag("constitutional")?),
    ];
    let total: f64 = parts.iter().map(|p| p.points).sum();
    let (band, label, meaning) = match total as u8 {
        0 => ("ipss_mf.low", "Low (0)", "Low risk: median survival 135 months."),
        1 => ("ipss_mf.int1", "Intermediate-1 (1)", "Intermediate-1 risk: median survival 95 months."),
        2 => ("ipss_mf.int2", "Intermediate-2 (2)", "Intermediate-2 risk: median survival 48 months."),
        _ => ("ipss_mf.high", "High (3 or more)", "High risk: median survival 27 months."),
    };
    Ok(Outcome {
        score: Some(total),
        display: format!("{total}"),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(151),
        parts,
        notes: vec![
            "The book prints the blast cut-off as \"< 1 / > 1\"; the original IPSS counts 1% or more."
                .to_owned(),
        ],
    })
}

/// Next steps and differential per risk group (book Hematology Lecture 9,
/// pp. 146–154). Where the book is silent: transplant candidates from the
/// EBMT/ELN consensus (Kröger et al., Leukemia 2015) and the first-line
/// spleen treatment by risk group from the ELN 2018 recommendations
/// (Barbui et al., Leukemia 2018).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "ipss_mf.low",
        Guide {
            next: &[
                "No symptoms: observe, since treatment other than a transplant is supportive (p. 151)",
                "Refine the risk with DIPSS or DIPSS-plus and the mutations: CALR is better; ASXL1, EZH2, IDH1/2 and SRSF2 are worse (p. 151)",
                "Interferon may help in early disease (p. 153)",
                "A large spleen that needs treatment: hydroxycarbamide first (ELN 2018)",
            ],
            dd: PMF_DD,
        },
    ),
    (
        "ipss_mf.int1",
        Guide {
            next: &[
                "Spleen or symptoms: ruxolitinib if the spleen is highly symptomatic, otherwise hydroxycarbamide (ELN 2018)",
                "Anaemia: androgens (danazol), erythropoietin when its level is low, steroids; most end up needing transfusion (pp. 152–153)",
                "Transplant only if under 65 with transfusion-dependent anaemia, blasts above 2% or adverse cytogenetics (EBMT/ELN 2015)",
                "Refine the risk with DIPSS-plus and the mutations (p. 151)",
            ],
            dd: PMF_DD,
        },
    ),
    (
        "ipss_mf.int2",
        Guide {
            next: &[
                "Assess for allogeneic stem cell transplant, the only cure, if under 70 and fit (p. 151; EBMT/ELN 2015)",
                "Transplant results: about 50% long-term survival at 45 or under (30% transplant deaths), 10–20% above 45 (p. 152)",
                "Ruxolitinib improves symptoms, spleen size and survival in intermediate-2 and high risk (p. 152)",
                "On ruxolitinib, expect Hb to fall 10% (worst at 12–16 weeks) and platelets 40%; watch for herpes and opportunistic infection; stopping it brings a rebound (p. 152)",
                "Anaemia: androgens, erythropoietin when its level is low, steroids, transfusion (pp. 152–153)",
            ],
            dd: PMF_DD,
        },
    ),
    (
        "ipss_mf.high",
        Guide {
            next: &[
                "Assess for allogeneic stem cell transplant now, if under 70 and fit: the only cure (p. 151; EBMT/ELN 2015)",
                "Ruxolitinib for symptoms, spleen size and survival (p. 152)",
                "Watch for transformation to AML (20% blasts): 20–30% of cases, usually rapidly fatal (p. 149)",
                "Splenectomy (about 10% mortality) or splenic irradiation only as a last resort (pp. 153–154)",
            ],
            dd: PMF_DD,
        },
    ),
];

const PMF_DD: &[&str] = &[
    "Other marrow fibrosis: CML (check BCR-ABL), myelodysplasia, AML M7, hairy-cell leukaemia, lymphoma, myeloma, mastocytosis, metastatic cancer (pp. 146, 150)",
    "Reactive fibrosis: TB, leishmaniasis, hyperparathyroidism, SLE, systemic sclerosis, radiation or benzene (p. 146)",
    "A leukoerythroblastic film also comes with marrow infiltration, severe sepsis or severe haemolysis (p. 149)",
    "More to the anaemia: hypersplenism, bleeding, iron or folate deficiency (p. 148)",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(age: f64, wbc: f64, hb: f64, blasts: f64, symptoms: bool) -> Outcome {
        compute(
            &Inputs::new()
                .num("age_years", age)
                .num("wbc_10e9_l", wbc)
                .num("hb_g_dl", hb)
                .num("blasts_pct", blasts)
                .flag("constitutional", symptoms),
        )
        .unwrap()
    }

    #[test]
    fn risk_groups() {
        assert_eq!(run(60.0, 10.0, 12.0, 0.0, false).band, "ipss_mf.low");
        assert_eq!(run(70.0, 10.0, 12.0, 0.0, false).band, "ipss_mf.int1");
        assert_eq!(run(70.0, 30.0, 12.0, 0.0, false).band, "ipss_mf.int2");
        let o = run(70.0, 30.0, 9.0, 1.0, true);
        assert_eq!((o.score, o.band), (Some(5.0), "ipss_mf.high"));
        // Exactly 65 does not count; 1% blasts does.
        assert_eq!(run(65.0, 10.0, 12.0, 1.0, false).score, Some(1.0));
    }
}
