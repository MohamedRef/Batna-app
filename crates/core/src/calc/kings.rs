//! King's College criteria for emergent liver transplantation in acute liver
//! failure.
//!
//! Source: the book's table, Lecture 10, p. 187 (O'Grady et al.,
//! Gastroenterology 1989, with the lactate criterion of Bernal 2002).
//! Wilson disease or Budd-Chiari presenting as ALF qualifies by itself.
//! Acetaminophen: arterial pH < 7.3 after resuscitation and > 24 h after
//! ingestion, or lactate > 3 mmol/L, or all of INR > 6.5, creatinine >
//! 3.4 mg/dL and encephalopathy stage 3–4. Non-acetaminophen: INR > 6.5, or
//! any 3 of INR > 3.5, bilirubin > 17.5 mg/dL, jaundice-to-encephalopathy >
//! 7 days, age < 10 or > 40 years, indeterminate or drug-induced cause.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, flag, normals, num};

const CAUSE: &[(&str, &str)] = &[
    ("acetaminophen", "Acetaminophen (paracetamol)"),
    ("wilson_budd_chiari", "Wilson disease or Budd-Chiari"),
    ("indeterminate_or_drug", "Indeterminate or other drug"),
    ("other", "Other cause (viral, autoimmune…)"),
];

pub const SPEC: CalcSpec = CalcSpec {
    id: "kings_college",
    name: "King's College criteria",
    area: Area::AcuteFailure,
    fields: &[
        choice("cause", "Cause of ALF", CAUSE),
        num("inr", "INR", "", 0.5, 20.0, 0.1).normal_as(normals::INR),
        num("ph", "Arterial pH (after resuscitation)", "", 6.5, 7.8, 0.01).normal_as(normals::PH).optional(),
        num("lactate_mmol_l", "Lactate", "mmol/L", 0.0, 30.0, 0.1)
            .normal_as(normals::LACTATE_MMOL_L)
            .optional(),
        num("creatinine_mg_dl", "Creatinine", "mg/dL", 0.1, 20.0, 0.1)
            .normal_as(normals::CREATININE_MG_DL)
            .optional(),
        num("he_stage", "Encephalopathy stage", "", 0.0, 4.0, 1.0).optional(),
        num("bilirubin_mg_dl", "Bilirubin", "mg/dL", 0.1, 60.0, 0.1)
            .normal_as(normals::BILIRUBIN_MG_DL)
            .optional(),
        num("age_years", "Age", "years", 0.0, 110.0, 1.0).optional(),
        num("jaundice_to_he_days", "Days from jaundice to encephalopathy", "days", 0.0, 180.0, 1.0)
            .optional(),
        flag("over_24h", "More than 24 h since ingestion"),
    ],
    source: "Book Lecture 10, p. 187 (O'Grady 1989; Bernal 2002)",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let cause =
        i.get_choice("cause", &["acetaminophen", "wilson_budd_chiari", "indeterminate_or_drug", "other"])?;
    let inr = i.get_num("inr")?;
    let mut parts = Vec::new();
    let meets = match cause {
        "wilson_budd_chiari" => {
            parts.push(Part::met("Wilson disease or Budd-Chiari presenting as ALF", true));
            true
        }
        "acetaminophen" => {
            let ph = i.opt_num("ph")?.is_some_and(|v| v < 7.3) && i.get_flag("over_24h")?;
            let lactate = i.opt_num("lactate_mmol_l")?.is_some_and(|v| v > 3.0);
            let triad = inr > 6.5
                && i.opt_num("creatinine_mg_dl")?.is_some_and(|v| v > 3.4)
                && i.opt_num("he_stage")?.is_some_and(|v| v >= 3.0);
            parts.push(Part::met("pH < 7.3 after resuscitation, > 24 h after ingestion", ph));
            parts.push(Part::met("Lactate > 3 mmol/L", lactate));
            parts.push(Part::met("INR > 6.5 + creatinine > 3.4 mg/dL + encephalopathy stage 3–4", triad));
            ph || lactate || triad
        }
        _ => {
            let minor = [
                Part::met("INR > 3.5", inr > 3.5),
                Part::met("Bilirubin > 17.5 mg/dL", i.opt_num("bilirubin_mg_dl")?.is_some_and(|v| v > 17.5)),
                Part::met(
                    "Jaundice to encephalopathy > 7 days",
                    i.opt_num("jaundice_to_he_days")?.is_some_and(|v| v > 7.0),
                ),
                Part::met(
                    "Age < 10 or > 40 years",
                    i.opt_num("age_years")?.is_some_and(|v| !(10.0..=40.0).contains(&v)),
                ),
                Part::met("Indeterminate or drug-induced cause", cause == "indeterminate_or_drug"),
            ];
            let count = minor.iter().filter(|p| p.points > 0.0).count();
            parts.push(Part::met("INR > 6.5", inr > 6.5));
            parts.extend(minor);
            inr > 6.5 || count >= 3
        }
    };
    let (band, label, meaning) = if meets {
        (
            "kings.met",
            "Criteria met",
            "Poor prognosis without transplant: list for emergent liver transplantation.",
        )
    } else {
        (
            "kings.not_met",
            "Criteria not met",
            "Does not meet King's criteria now; reassess as the course evolves.",
        )
    };
    Ok(Outcome {
        score: None,
        display: label.to_owned(),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(187),
        parts,
        notes: Vec::new(),
    })
}

/// Next steps and differential, shared with the Clichy criteria (book
/// Lecture 10, pp. 178–187).
pub const GUIDES: &[(&str, Guide)] = &[
    ("kings.met", Guide { next: LIST_NOW, dd: ALF_DD }),
    ("kings.not_met", Guide { next: REASSESS, dd: ALF_DD }),
];

pub(super) const LIST_NOW: &[&str] = &[
    "Transfer to a transplant centre and list for emergent liver transplantation (pp. 183, 187)",
    "Intensive care: from encephalopathy stage 3, intubation and ICP monitoring; hypertonic saline to sodium 145–155 (p. 179)",
    "Dextrose to keep glucose 100–200 mg/dL; norepinephrine if needed; blood cultures and a low threshold for antibiotics (pp. 178, 180)",
    "N-acetylcysteine for paracetamol (up to 72 h) and in early encephalopathy of any cause; it lengthens the PT by itself (pp. 180–181)",
    "By cause: a nucleoside analogue for hepatitis B, acyclovir for herpes simplex, plasmapheresis with penicillamine for Wilson disease (p. 181)",
];

pub(super) const REASSESS: &[&str] = &[
    "Reassess every day: falling transaminases with rising bilirubin and INR and a shrinking liver are poor signs (p. 186)",
    "Any encephalopathy: intensive care review and a call to a transplant centre (p. 184)",
    "A subacute course or an unknown cause does badly without transplant: discuss early (p. 184)",
    "MELD above 32 also predicts poor transplant-free survival (p. 186)",
    "Treat the cause: N-acetylcysteine for paracetamol (up to 72 h), antivirals for hepatitis B or herpes simplex (pp. 180–181)",
];

pub(super) const ALF_DD: &[&str] = &[
    "Secondary liver failure, where emergent transplant is not a treatment: ischaemic hepatitis; lymphoma, infiltration or haemophagocytic syndrome; malaria, leptospirosis or rickettsia; extensive liver resection (p. 185)",
    "Severe alcoholic hepatitis: chronic disease with an acute picture, also not for emergent transplant (p. 185)",
    "Chronic disease that may still qualify: Wilson disease, autoimmune hepatitis, Budd-Chiari, hepatitis B reactivation (p. 185)",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acetaminophen() {
        let i = Inputs::new().choice("cause", "acetaminophen").num("inr", 3.0);
        assert_eq!(compute(&i.clone().num("ph", 7.2).flag("over_24h", true)).unwrap().band, "kings.met");
        // pH before 24 h does not count.
        assert_eq!(compute(&i.clone().num("ph", 7.2)).unwrap().band, "kings.not_met");
        assert_eq!(compute(&i.clone().num("lactate_mmol_l", 3.5)).unwrap().band, "kings.met");
        let triad = Inputs::new()
            .choice("cause", "acetaminophen")
            .num("inr", 7.0)
            .num("creatinine_mg_dl", 3.6)
            .num("he_stage", 3.0);
        assert_eq!(compute(&triad).unwrap().band, "kings.met");
        assert_eq!(compute(&triad.num("he_stage", 2.0)).unwrap().band, "kings.not_met");
    }

    #[test]
    fn non_acetaminophen() {
        assert_eq!(
            compute(&Inputs::new().choice("cause", "other").num("inr", 6.6)).unwrap().band,
            "kings.met"
        );
        // Three minor criteria: INR 4, age 45, drug-induced.
        let i = Inputs::new().choice("cause", "indeterminate_or_drug").num("inr", 4.0).num("age_years", 45.0);
        assert_eq!(compute(&i).unwrap().band, "kings.met");
        // Only two.
        let i = Inputs::new().choice("cause", "other").num("inr", 4.0).num("age_years", 45.0);
        assert_eq!(compute(&i).unwrap().band, "kings.not_met");
    }

    #[test]
    fn wilson_qualifies_alone() {
        assert_eq!(
            compute(&Inputs::new().choice("cause", "wilson_budd_chiari").num("inr", 1.5)).unwrap().band,
            "kings.met"
        );
    }
}
