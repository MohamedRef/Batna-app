//! Ascitic fluid: SBP and its variants, and secondary bacterial peritonitis.
//!
//! Source: the book, Lecture 9, p. 167–168. SBP: ascitic PMN ≥ 250/mm³ with
//! or without a positive culture. Variants: culture-positive neutrocytic
//! ascites, culture-negative neutrocytic ascites, and bacterascites (PMN
//! < 250/mm³ with a single organism). Secondary peritonitis is suspected with
//! total protein > 1.5 g/dL, PMN > 5,000/mm³, a polymicrobial culture, glucose
//! < 50 mg/dL, or ascitic LDH above serum LDH. (The book prints "50 mg/l";
//! Runyon's criterion is 50 mg/dL; see medstudy/book-issues.md.)

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, flag, num};

const CULTURE: &[(&str, &str)] = &[
    ("pending", "Not sent or pending"),
    ("negative", "Negative"),
    ("monomicrobial", "Positive, one organism"),
    ("polymicrobial", "Positive, several organisms"),
];

pub const SPEC: CalcSpec = CalcSpec {
    id: "ascitic_pmn",
    name: "Ascitic PMN / SBP",
    area: Area::Ascites,
    fields: &[
        num("pmn_per_mm3", "Ascitic PMN (neutrophils)", "/mm³", 0.0, 100000.0, 1.0)
            .hint("Total WBC × neutrophil %."),
        choice("culture", "Ascitic culture", CULTURE),
        num("protein_g_dl", "Ascitic total protein", "g/dL", 0.0, 8.0, 0.1).optional(),
        num("glucose_mg_dl", "Ascitic glucose", "mg/dL", 0.0, 500.0, 1.0).optional(),
        flag("ldh_above_serum", "Ascitic LDH above serum LDH"),
    ],
    source: "Book Lecture 9, p. 167–168",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let pmn = i.get_num("pmn_per_mm3")?;
    let culture = i.get_choice("culture", &["pending", "negative", "monomicrobial", "polymicrobial"])?;
    let neutrocytic = pmn >= 250.0;

    let protein = i.opt_num("protein_g_dl")?;
    let glucose = i.opt_num("glucose_mg_dl")?;
    let secondary = [
        Part::met("Total protein > 1.5 g/dL", protein.is_some_and(|p| p > 1.5)),
        Part::met("PMN > 5,000/mm³", pmn > 5000.0),
        Part::met("Polymicrobial culture", culture == "polymicrobial"),
        Part::met("Glucose < 50 mg/dL", glucose.is_some_and(|g| g < 50.0)),
        Part::met("Ascitic LDH > serum LDH", i.get_flag("ldh_above_serum")?),
    ];
    let secondary_hits = secondary.iter().filter(|p| p.points > 0.0).count();

    let (band, label, meaning) = if neutrocytic && secondary_hits > 0 {
        (
            "ascitic.secondary",
            "Suspect secondary peritonitis",
            "Neutrocytic ascites with features of secondary bacterial peritonitis: look for a surgically treatable source (abscess, perforation).",
        )
    } else if neutrocytic && culture == "monomicrobial" {
        (
            "ascitic.sbp_culture_positive",
            "SBP: culture-positive neutrocytic ascites",
            "PMN ≥ 250/mm³ with a positive culture: treat as SBP.",
        )
    } else if neutrocytic && culture == "negative" {
        (
            "ascitic.sbp_culture_negative",
            "SBP: culture-negative neutrocytic ascites",
            "PMN ≥ 250/mm³, culture negative: treat as SBP; also consider TB peritonitis, carcinomatosis and fungal peritonitis.",
        )
    } else if neutrocytic {
        (
            "ascitic.sbp",
            "SBP (PMN ≥ 250/mm³)",
            "Diagnostic of SBP with or without a positive culture: start empirical antibiotics.",
        )
    } else if culture == "monomicrobial" {
        (
            "ascitic.bacterascites",
            "Bacterascites",
            "PMN < 250/mm³ with a single organism: may be resolving or evolving SBP.",
        )
    } else if culture == "polymicrobial" {
        (
            "ascitic.polymicrobial",
            "Polymicrobial, non-neutrocytic",
            "PMN < 250/mm³ with several organisms: suspect gut puncture during paracentesis or secondary peritonitis.",
        )
    } else {
        ("ascitic.no_sbp", "No SBP (PMN < 250/mm³)", "SBP is not diagnosed.")
    };

    // The PMN cut-off is on p. 167; the variants and secondary peritonitis
    // are defined on p. 168.
    let page = if matches!(band, "ascitic.sbp" | "ascitic.no_sbp") { 167 } else { 168 };
    let mut parts = vec![Part::met("PMN ≥ 250/mm³", neutrocytic)];
    parts.extend(secondary);
    Ok(Outcome {
        score: Some(pmn),
        display: format!("{pmn:.0}/mm³"),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(page),
        parts,
        notes: Vec::new(),
    })
}

/// Next steps and differential per result (book Lecture 9, pp. 151,
/// 166–171; AASLD 2021, Biggins et al., for the CT, surgical review and
/// anaerobic cover in secondary peritonitis and for polymicrobial
/// bacterascites).
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "ascitic.sbp",
        Guide {
            next: &[
                "Start empirical antibiotics now: cefotaxime 2 g IV every 8–12 h or ceftriaxone 2 g a day, for at least 5–7 days (p. 169)",
                ALBUMIN,
                REPEAT_TAP,
                AFTER_SBP,
            ],
            dd: &[SECONDARY, NEGATIVE_CULTURE],
        },
    ),
    (
        "ascitic.sbp_culture_positive",
        Guide {
            next: &[
                "Antibiotics as for SBP, then adjust to the organism's sensitivities (p. 169)",
                ALBUMIN,
                REPEAT_TAP,
                AFTER_SBP,
            ],
            dd: &[
                SECONDARY,
                "Spontaneous fungal peritonitis if fungi grow: an echinocandin when critically ill, otherwise fluconazole (pp. 170–171)",
            ],
        },
    ),
    (
        "ascitic.sbp_culture_negative",
        Guide {
            next: &[
                "Continue the empirical antibiotic for at least 5–7 days (p. 169)",
                ALBUMIN,
                REPEAT_TAP,
                AFTER_SBP,
            ],
            dd: &[NEGATIVE_CULTURE, SECONDARY],
        },
    ),
    (
        "ascitic.bacterascites",
        Guide {
            next: &[
                "Treat by the organism's sensitivities (p. 169)",
                "Repeat the tap: it may be resolving SBP or the first step of SBP (p. 168)",
            ],
            dd: &[
                "Resolving or evolving SBP (p. 168)",
                "Fungal ascites if the culture grows fungi with PMN below 250 (p. 170)",
                "A contaminated sample (skin organisms)",
            ],
        },
    ),
    (
        "ascitic.polymicrobial",
        Guide {
            next: &[
                "Repeat the tap: the needle may have entered the bowel",
                "CT of the abdomen if the PMN rises or the patient is unwell, to look for a perforation or abscess",
            ],
            dd: &[
                "Bowel punctured by the paracentesis needle",
                "Early secondary bacterial peritonitis: polymicrobial culture (p. 168)",
            ],
        },
    ),
    (
        "ascitic.secondary",
        Guide {
            next: &[
                "Urgent CT of the abdomen to find the source: abscess or perforation (p. 168)",
                "Surgical review; antibiotics that cover anaerobes, such as cefotaxime with metronidazole",
            ],
            dd: &[
                "Bowel perforation or an abdominal abscess (p. 168)",
                "SBP not responding: PMN not falling after 48 h of antibiotics (p. 168)",
            ],
        },
    ),
    (
        "ascitic.no_sbp",
        Guide {
            next: &[
                "No SBP treatment; tap again if the patient worsens: fever, pain, encephalopathy, AKI (p. 167)",
                "Primary prophylaxis (norfloxacin 400 mg or ciprofloxacin 500 mg a day) if ascitic protein is below 1.5 g/dL with Child-Pugh 9 or more, bilirubin 3 or more, renal impairment or low sodium (p. 170)",
                "During a GI bleed: ceftriaxone 1 g a day for up to 7 days (p. 170)",
            ],
            dd: &[
                "Another source of fever or confusion: urine, chest, skin, GI bleeding, electrolytes or drugs",
            ],
        },
    ),
];

const ALBUMIN: &str =
    "Albumin 1.5 g/kg at diagnosis and 1 g/kg on day 3, above all with AKI or jaundice (p. 169)";
const REPEAT_TAP: &str = "Tap again at 48 h: a PMN fall of more than 25% is a response; if not, broaden cover and look for secondary peritonitis (p. 169)";
const AFTER_SBP: &str = "After recovery: norfloxacin 400 mg or ciprofloxacin 500 mg a day as secondary prophylaxis, and consider transplant (pp. 169–170)";
const SECONDARY: &str = "Secondary bacterial peritonitis: protein above 1.5 g/dL, PMN above 5,000, several organisms, glucose below 50 mg/dL, LDH above serum (p. 168)";
const NEGATIVE_CULTURE: &str = "If the culture stays negative: TB peritonitis, peritoneal carcinomatosis, fungal peritonitis (pp. 168, 170)";

#[cfg(test)]
mod tests {
    use super::*;

    fn run(pmn: f64, culture: &str) -> Outcome {
        compute(&Inputs::new().num("pmn_per_mm3", pmn).choice("culture", culture)).unwrap()
    }

    #[test]
    fn variants() {
        assert_eq!(run(250.0, "pending").band, "ascitic.sbp");
        assert_eq!(run(600.0, "monomicrobial").band, "ascitic.sbp_culture_positive");
        assert_eq!(run(600.0, "negative").band, "ascitic.sbp_culture_negative");
        assert_eq!(run(120.0, "monomicrobial").band, "ascitic.bacterascites");
        assert_eq!(run(249.0, "negative").band, "ascitic.no_sbp");
        assert_eq!(run(250.0, "pending").page, Some(167));
        assert_eq!(run(120.0, "monomicrobial").page, Some(168));
    }

    #[test]
    fn secondary_peritonitis() {
        assert_eq!(run(800.0, "polymicrobial").band, "ascitic.secondary");
        assert_eq!(run(6000.0, "monomicrobial").band, "ascitic.secondary");
        let o = compute(
            &Inputs::new()
                .num("pmn_per_mm3", 900.0)
                .choice("culture", "pending")
                .num("protein_g_dl", 2.0)
                .num("glucose_mg_dl", 30.0),
        )
        .unwrap();
        assert_eq!(o.band, "ascitic.secondary");
        assert_eq!(o.parts.iter().filter(|p| p.points > 0.0).count(), 3);
    }
}
