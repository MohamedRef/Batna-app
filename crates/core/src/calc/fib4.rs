//! FIB-4 index.
//!
//! Formula: age × AST / (platelets [10⁹/L] × √ALT) (Sterling et al.,
//! Hepatology 2006). Bands from the book, Lecture 6, p. 102: < 1.3 low,
//! 1.3–3.25 intermediate, > 3.25 high risk of advanced fibrosis. Lecture 2
//! (EASL 2021) uses < 1.3 to rule out and refers above 1.3 for elastography.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num, show, shown};

pub const SPEC: CalcSpec = CalcSpec {
    id: "fib4",
    name: "FIB-4",
    area: Area::Liver,
    fields: &[
        num("age_years", "Age", "years", 1.0, 110.0, 1.0),
        num("ast_u_l", "AST", "U/L", 1.0, 10000.0, 1.0).normal_as(normals::AST_U_L),
        num("alt_u_l", "ALT", "U/L", 1.0, 10000.0, 1.0).normal_as(normals::ALT_U_L),
        num("platelets_10e9_l", "Platelets", "×10⁹/L", 1.0, 1500.0, 1.0).normal_as(normals::PLATELETS_10E9_L),
    ],
    source: "Sterling 2006; book Lecture 6, p. 102",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let age = i.get_pos("age_years")?;
    let ast = i.get_pos("ast_u_l")?;
    let alt = i.get_pos("alt_u_l")?;
    let plt = i.get_pos("platelets_10e9_l")?;
    let score = age * ast / (plt * alt.sqrt());
    let at = shown(score, 2);
    let (band, label, meaning) = if at < 1.3 {
        ("fib4.low", "Low risk (< 1.3)", "Advanced fibrosis unlikely.")
    } else if at <= 3.25 {
        (
            "fib4.mid",
            "Intermediate (1.3–3.25)",
            "Indeterminate: go on to elastography (Fibroscan) or a patented test.",
        )
    } else {
        ("fib4.high", "High risk (> 3.25)", "Advanced fibrosis likely.")
    };
    let mut notes = Vec::new();
    if age < 35.0 || age > 65.0 {
        notes.push("Less accurate below 35 or above 65 years.".to_owned());
    }
    if age >= 65.0 {
        notes.push("65 or over: the low-risk cut-off is 2.0, not 1.3 (McPherson 2017).".to_owned());
    }
    Ok(Outcome {
        score: Some(score),
        display: show(score, 2),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(102),
        parts: vec![Part::new("Age × AST", age * ast), Part::new("Platelets × √ALT", plt * alt.sqrt())],
        notes,
    })
}

/// Next steps per band (book Lecture 6, p. 104: the EASL 2021 and ADA 2025
/// pathways; p. 105 for care by stage; McPherson et al., Am J Gastroenterol
/// 2017, for FIB-4 above 65 years).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "fib4.low",
        Guide {
            next: &[
                "No referral needed: lifestyle changes and repeat FIB-4 in 1–3 years (EASL 2021, ADA 2025, p. 104)",
                "Treat the metabolic risk: weight loss; in diabetes favour GLP-1 receptor agonists, SGLT2 inhibitors or pioglitazone (p. 105)",
            ],
            dd: OTHER_LIVER_DISEASE,
        },
    ),
    (
        "fib4.mid",
        Guide {
            next: &[
                "Liver stiffness by transient elastography (Fibroscan): below 8 kPa low risk; 8 kPa or more, refer to a liver specialist (p. 104)",
                "No elastography: a patented serum test such as ELF (p. 104)",
            ],
            dd: OTHER_LIVER_DISEASE,
        },
    ),
    (
        "fib4.high",
        Guide {
            next: &[
                "Refer to a liver specialist and confirm with elastography; F3–F4 is highly likely when the tests agree (p. 104)",
                "If cirrhosis: HCC ultrasound every 6 months and variceal screening (Lecture 1, p. 13)",
            ],
            dd: &[
                "Other chronic liver disease: alcohol, hepatitis B or C, a family history (p. 104)",
                "Falsely high: age over 65, an AST rise from muscle or acute hepatitis, a low platelet count from another cause",
            ],
        },
    ),
];

const OTHER_LIVER_DISEASE: &[&str] =
    &["Other chronic liver disease: alcohol, hepatitis B or C, a family history (p. 104)"];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(age: f64, ast: f64, alt: f64, plt: f64) -> Outcome {
        compute(
            &Inputs::new()
                .num("age_years", age)
                .num("ast_u_l", ast)
                .num("alt_u_l", alt)
                .num("platelets_10e9_l", plt),
        )
        .unwrap()
    }

    #[test]
    fn worked_examples() {
        // 50 × 40 / (200 × 5) = 2.0.
        let o = run(50.0, 40.0, 25.0, 200.0);
        assert!((o.score.unwrap() - 2.0).abs() < 1e-9);
        assert_eq!(o.band, "fib4.mid");
        // 40 × 20 / (250 × √36) = 0.53.
        assert_eq!(run(40.0, 20.0, 36.0, 250.0).band, "fib4.low");
        // 60 × 80 / (90 × √49) = 7.6.
        assert_eq!(run(60.0, 80.0, 49.0, 90.0).band, "fib4.high");
    }

    #[test]
    fn zero_platelets_is_an_error() {
        assert!(
            compute(
                &Inputs::new()
                    .num("age_years", 50.0)
                    .num("ast_u_l", 40.0)
                    .num("alt_u_l", 25.0)
                    .num("platelets_10e9_l", 0.0)
            )
            .is_err()
        );
    }
}
