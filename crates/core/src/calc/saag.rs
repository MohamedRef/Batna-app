//! Serum-ascites albumin gradient (SAAG).
//!
//! Source: the book, Lecture 9, p. 150–151: SAAG = serum albumin − ascitic
//! albumin (g/dL); ≥ 1.1 portal hypertension, < 1.1 non-portal-hypertensive
//! ascites. Ascitic total protein (p. 150): low < 1.5 g/dL in cirrhosis,
//! high > 2.5 g/dL in heart failure and malignant ascites.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, normals, num, show};

/// Ascitic albumin above serum albumin.
pub const ABOVE_SERUM: &str = "higher than the serum albumin";
/// Ascitic total protein below its own albumin.
pub const BELOW_ALBUMIN: &str = "lower than the ascitic albumin";

pub const SPEC: CalcSpec = CalcSpec {
    id: "saag",
    name: "SAAG",
    area: Area::Ascites,
    fields: &[
        num("serum_albumin_g_dl", "Serum albumin", "g/dL", 0.5, 6.0, 0.1).normal_as(normals::ALBUMIN_G_DL),
        num("ascitic_albumin_g_dl", "Ascitic albumin", "g/dL", 0.0, 6.0, 0.1),
        num("ascitic_protein_g_dl", "Ascitic total protein", "g/dL", 0.0, 8.0, 0.1).optional(),
    ],
    source: "Book Lecture 9, p. 150–151",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let serum = i.get_num("serum_albumin_g_dl")?;
    let ascitic = i.get_num("ascitic_albumin_g_dl")?;
    // Ascitic fluid comes from plasma: its albumin can't pass the serum's,
    // and albumin is part of its total protein. Either way round is a slip
    // (serum albumin 12 and ascitic 23 gave SAAG −11 before this).
    if ascitic > serum {
        return Err(CalcError::Invalid { field: "ascitic_albumin_g_dl", reason: ABOVE_SERUM });
    }
    if i.opt_num("ascitic_protein_g_dl")?.is_some_and(|p| p < ascitic) {
        return Err(CalcError::Invalid { field: "ascitic_protein_g_dl", reason: BELOW_ALBUMIN });
    }
    // Rounded to one decimal, as labs report, so 1.1 lands on the cut-off.
    let saag = ((serum - ascitic) * 10.0).round() / 10.0;
    let high = saag >= 1.1;
    let (band, label, meaning) = if high {
        ("saag.high", "High SAAG (≥ 1.1 g/dL)", "Ascites most likely due to portal hypertension.")
    } else {
        ("saag.low", "Low SAAG (< 1.1 g/dL)", "Suggests non-portal-hypertensive ascites.")
    };
    let mut notes = Vec::new();
    let mut parts = vec![Part::new("Serum albumin", serum), Part::new("Ascitic albumin", ascitic)];
    if let Some(p) = i.opt_num("ascitic_protein_g_dl")? {
        parts.push(Part::new("Ascitic protein", p));
        if p < 1.5 {
            notes.push("Protein < 1.5 g/dL: low, as in cirrhosis.".to_owned());
        } else if p > 2.5 {
            notes.push("Protein > 2.5 g/dL: high, as in heart failure or malignant ascites.".to_owned());
        }
    }
    if high {
        notes.push("Portal hypertension plus a second cause also gives SAAG ≥ 1.1.".to_owned());
    }
    Ok(Outcome {
        score: Some(saag),
        display: format!("{} g/dL", show(saag, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(150),
        parts,
        notes,
    })
}

/// Next steps and differential per band (book Lecture 9, pp. 145–153).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "saag.high",
        Guide {
            next: &[
                "Total protein sorts the cause: below 1.5 g/dL in cirrhosis, above 2.5 g/dL in heart failure (p. 150)",
                "PMN count and culture to rule out SBP (pp. 150, 167)",
                "Cirrhosis, grade 2: sodium 2 g a day and spironolactone from 100 mg; grade 3: large-volume paracentesis first (pp. 152–153)",
            ],
            dd: &[
                "Cirrhosis or other portal hypertension, before or after the sinusoids (p. 145)",
                "Right heart failure or constrictive pericarditis: high protein (pp. 145, 150)",
                "Budd-Chiari syndrome or a narrowed inferior vena cava (p. 146)",
                "Massive liver metastases",
            ],
        },
    ),
    (
        "saag.low",
        Guide {
            next: &[
                "Look at the peritoneum: cytology, LDH, glucose, TB PCR and lymphocytes, amylase, triglycerides (pp. 150–151)",
                "CT when the cause is still unclear (p. 149)",
            ],
            dd: &[
                "Peritoneal malignancy: carcinomatosis, mesothelioma, pseudomyxoma (p. 146)",
                "Tuberculous peritonitis: lymphocytes, low glucose, PCR (pp. 146, 150–151)",
                "Pancreatic ascites: high amylase (pp. 145, 151)",
                "Nephrotic syndrome or protein-losing enteropathy (p. 145)",
                "Serositis: SLE and other collagen diseases, familial Mediterranean fever (p. 146)",
                "Chylous (triglycerides) or bile ascites (pp. 145, 151)",
            ],
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: f64, a: f64) -> Outcome {
        compute(&Inputs::new().num("serum_albumin_g_dl", s).num("ascitic_albumin_g_dl", a)).unwrap()
    }

    #[test]
    fn cut_off() {
        assert_eq!(run(3.0, 1.0).band, "saag.high");
        assert_eq!(run(3.2, 2.1).band, "saag.high"); // exactly 1.1
        assert_eq!(run(3.5, 2.5).band, "saag.low");
    }

    #[test]
    fn protein_note() {
        let o = compute(
            &Inputs::new()
                .num("serum_albumin_g_dl", 3.0)
                .num("ascitic_albumin_g_dl", 1.2)
                .num("ascitic_protein_g_dl", 3.0),
        )
        .unwrap();
        assert!(o.notes.iter().any(|n| n.contains("heart failure")));
    }

    #[test]
    fn impossible_pairs_are_refused() {
        // Mohamed's slip: serum 12 and ascitic 23 gave −11.0 before.
        let i = Inputs::new().num("serum_albumin_g_dl", 3.0).num("ascitic_albumin_g_dl", 3.5);
        assert_eq!(
            compute(&i),
            Err(CalcError::Invalid { field: "ascitic_albumin_g_dl", reason: ABOVE_SERUM })
        );
        let i = Inputs::new()
            .num("serum_albumin_g_dl", 3.0)
            .num("ascitic_albumin_g_dl", 1.0)
            .num("ascitic_protein_g_dl", 0.8);
        assert_eq!(
            compute(&i),
            Err(CalcError::Invalid { field: "ascitic_protein_g_dl", reason: BELOW_ALBUMIN })
        );
        // Serum albumin 12 g/dL is outside what a body shows.
        let i = Inputs::new().num("serum_albumin_g_dl", 12.0).num("ascitic_albumin_g_dl", 2.3);
        assert_eq!(
            SPEC.run(&i),
            Err(CalcError::Invalid { field: "serum_albumin_g_dl", reason: super::super::IMPOSSIBLE })
        );
    }
}
