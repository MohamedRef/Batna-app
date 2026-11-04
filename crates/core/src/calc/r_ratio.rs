//! R ratio (R value) for the pattern of drug-induced liver injury.
//!
//! Formula: (ALT / ALT ULN) ÷ (ALP / ALP ULN). Bands follow the book
//! (Lecture 8, p. 129): R > 5 hepatocellular, R < 2 cholestatic, 2–5 mixed.
//! CIOMS/RUCAM, EASL DILI 2019 and Kullak-Ublick (Gut 2017) put R = 5 in
//! hepatocellular and R = 2 in cholestatic; a note says so at those values.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num, show};

pub const SPEC: CalcSpec = CalcSpec {
    id: "r_ratio",
    name: "R ratio (DILI pattern)",
    area: Area::Liver,
    fields: &[
        num("alt_u_l", "ALT", "U/L", 1.0, 20000.0, 1.0).normal_as(normals::ALT_U_L),
        num("alt_uln_u_l", "ALT upper limit of normal", "U/L", 10.0, 100.0, 1.0),
        num("alp_u_l", "ALP", "U/L", 1.0, 5000.0, 1.0).normal_as(normals::ALP_U_L),
        num("alp_uln_u_l", "ALP upper limit of normal", "U/L", 30.0, 500.0, 1.0),
    ],
    source: "Book Lecture 8, p. 129; EASL DILI 2019; Kullak-Ublick 2017",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let alt = i.get_pos("alt_u_l")? / i.get_pos("alt_uln_u_l")?;
    let alp = i.get_pos("alp_u_l")? / i.get_pos("alp_uln_u_l")?;
    let r = alt / alp;
    // Band the value as shown (one decimal), so "5.0" is always mixed.
    let shown = (r * 10.0).round() / 10.0;
    let (band, label, meaning) = if shown > 5.0 {
        ("r_ratio.hepatocellular", "Hepatocellular (R > 5)", "Hepatocellular pattern of liver injury.")
    } else if shown < 2.0 {
        ("r_ratio.cholestatic", "Cholestatic (R < 2)", "Cholestatic pattern of liver injury.")
    } else {
        ("r_ratio.mixed", "Mixed (R 2–5)", "Mixed hepatocellular-cholestatic pattern.")
    };
    Ok(Outcome {
        score: Some(r),
        display: show(r, 1),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(129),
        parts: vec![Part::new("ALT × ULN", alt), Part::new("ALP × ULN", alp)],
        notes: if shown == 5.0 {
            vec!["Book: 2–5 is mixed. CIOMS and EASL 2019 call R = 5 hepatocellular.".to_owned()]
        } else if shown == 2.0 {
            vec!["Book: 2–5 is mixed. CIOMS and EASL 2019 call R = 2 cholestatic.".to_owned()]
        } else {
            Vec::new()
        },
    })
}

/// Next steps and differential per pattern (book Lecture 8, pp. 125–141,
/// with the Kullak-Ublick 2017 workflow on p. 129).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "r_ratio.hepatocellular",
        Guide {
            next: &[
                "Stop the suspected drug or herbal product now; most drug injury improves once it is withdrawn (p. 125)",
                "Abdominal ultrasound and hepatitis A, B, C and E serology (p. 129)",
                "Expect the enzymes to halve within 2 weeks and normalise by 4 weeks after stopping (p. 130)",
                "Check bilirubin and INR: jaundice with hepatocellular injury worsens the prognosis (Hy's law, p. 141)",
            ],
            dd: HEPATOCELLULAR_DD,
        },
    ),
    (
        "r_ratio.mixed",
        Guide {
            next: &[
                "Stop the suspected drug or herbal product (p. 125)",
                "Abdominal ultrasound and hepatitis A, B, C and E serology (p. 129)",
            ],
            dd: HEPATOCELLULAR_DD,
        },
    ),
    (
        "r_ratio.cholestatic",
        Guide {
            next: &[
                "Stop the suspected drug; cholestatic injury may take longer to settle (pp. 125, 130)",
                "Abdominal ultrasound first, then MRCP and, if still unclear, liver biopsy (p. 129)",
                "A raised GGT shows the ALP comes from the liver (pp. 134, 140)",
            ],
            dd: &[
                "Bile duct obstruction: stones, tumour, stricture",
                "Primary biliary cholangitis: anti-mitochondrial antibodies (pp. 13, 137)",
                "Primary sclerosing cholangitis (MRCP)",
                "Drug cholestasis: amoxicillin-clavulanate, erythromycin, chlorpromazine, carbamazepine, oestrogens (pp. 135–136)",
                "ALP from bone if the GGT is normal (p. 140)",
            ],
        },
    ),
];

const HEPATOCELLULAR_DD: &[&str] = &[
    "Viral hepatitis A, B, C and E; CMV, EBV, HSV (p. 129)",
    "Autoimmune hepatitis (p. 129)",
    "Ischaemic hepatitis (shock liver) (pp. 129, 133)",
    "Budd-Chiari syndrome (p. 129)",
    "Wilson disease (p. 129)",
    "ALT and AST of 5,000–10,000: paracetamol, toxins, toxic mushrooms, shock liver (p. 133)",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(alt: f64, alp: f64) -> Outcome {
        compute(
            &Inputs::new()
                .num("alt_u_l", alt)
                .num("alt_uln_u_l", 40.0)
                .num("alp_u_l", alp)
                .num("alp_uln_u_l", 120.0),
        )
        .unwrap()
    }

    #[test]
    fn patterns() {
        // ALT 10×ULN, ALP 1×ULN → R 10.
        assert_eq!(run(400.0, 120.0).band, "r_ratio.hepatocellular");
        // ALT 2×ULN, ALP 3×ULN → R 0.67.
        assert_eq!(run(80.0, 360.0).band, "r_ratio.cholestatic");
        // ALT 6×ULN, ALP 2×ULN → R 3.
        let o = run(240.0, 240.0);
        assert!((o.score.unwrap() - 3.0).abs() < 1e-9);
        assert_eq!(o.band, "r_ratio.mixed");
        // Boundaries follow the book (p. 129): exactly 5 and exactly 2 are
        // mixed, with a note on the guideline reading.
        let five = run(200.0, 120.0);
        assert_eq!(five.band, "r_ratio.mixed");
        assert_eq!(five.notes.len(), 1);
        let two = run(80.0, 120.0);
        assert_eq!(two.band, "r_ratio.mixed");
        assert_eq!(two.notes.len(), 1);
        assert_eq!(run(204.0, 120.0).band, "r_ratio.hepatocellular");
        assert_eq!(run(72.0, 120.0).band, "r_ratio.cholestatic");
        assert!(run(400.0, 120.0).notes.is_empty());
    }
}
