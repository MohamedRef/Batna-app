//! Alveolar–arterial (A-a) oxygen gradient.
//!
//! Source: the Nephrology book (Chapter 8, Lecture 1), the arterial blood
//! gas table on p. 14: PAO2 = (713 × FiO2) − (pCO2 × 1.25); A-a gradient =
//! PAO2 − PaO2, expected (age / 4) + 4; pO2 80–100 normal, < 80 hypoxia.

use super::{Area, CalcError, CalcSpec, Inputs, Outcome, Part, num, show};

pub const SPEC: CalcSpec = CalcSpec {
    id: "aa_gradient",
    name: "A-a gradient",
    area: Area::Renal,
    fields: &[
        num("fio2", "FiO2", "fraction", 0.21, 1.0, 0.01).hint("Room air = 0.21."),
        num("pao2_mm_hg", "PaO2", "mmHg", 10.0, 700.0, 1.0),
        num("pco2_mm_hg", "PaCO2", "mmHg", 5.0, 150.0, 1.0),
        num("age_years", "Age", "years", 1.0, 120.0, 1.0),
    ],
    source: "Book Nephrology Lecture 1, ABG table (p. 14): alveolar gas equation, expected gradient \
             age/4 + 4 (room air)",
    compute,
};

fn compute(i: &Inputs) -> Result<Outcome, CalcError> {
    let mut fio2 = i.get_pos("fio2")?;
    let mut notes = Vec::new();
    // Written as a percentage (21, 40): read it as one.
    if fio2 > 1.0 && fio2 <= 100.0 {
        fio2 /= 100.0;
        notes.push(format!("FiO2 read as {}.", show(fio2, 2)));
    }
    if !(0.21..=1.0).contains(&fio2) {
        return Err(CalcError::Invalid { field: "fio2", reason: "must be 0.21–1.0" });
    }
    let pao2 = i.get_num("pao2_mm_hg")?;
    let pco2 = i.get_num("pco2_mm_hg")?;
    let age = i.get_num("age_years")?;
    let alveolar = 713.0 * fio2 - 1.25 * pco2;
    let gradient = ((alveolar - pao2) * 10.0).round() / 10.0;
    let expected = ((age / 4.0 + 4.0) * 10.0).round() / 10.0;
    let (band, label, meaning) = if gradient > expected {
        (
            "aa_gradient.high",
            "Raised for age",
            "A raised gradient: the hypoxia comes from the lung itself. Give oxygen and see: V/Q mismatch \
             corrects with oxygen, a shunt barely does.",
        )
    } else {
        (
            "aa_gradient.normal",
            "Normal for age",
            "A normal gradient: if PaO2 is low, the lungs are fine and the cause is hypoventilation (high \
             pCO2) or a low inspired oxygen.",
        )
    };
    notes.push(
        if gradient > expected {
            "DD: V/Q mismatch (pulmonary embolism, pneumonia, asthma, COPD); shunt (ARDS, pulmonary \
             oedema, lobar collapse, intracardiac right-to-left shunt); diffusion defect (interstitial \
             lung disease)."
        } else {
            "DD of hypoxia with a normal gradient: hypoventilation (opioids or sedatives, neuromuscular \
             weakness, chest-wall disease, obesity hypoventilation) or high altitude."
        }
        .to_owned(),
    );
    if fio2 > 0.21 {
        notes.push("The age rule is for room air; on extra oxygen the normal gradient is higher.".to_owned());
    }
    Ok(Outcome {
        score: Some(gradient),
        display: format!("{} mmHg (expected ≤ {})", show(gradient, 1), show(expected, 1)),
        band,
        label,
        meaning: meaning.to_owned(),
        page: Some(14),
        parts: vec![
            Part::new("PAO2 = 713 × FiO2 − 1.25 × pCO2", (alveolar * 10.0).round() / 10.0),
            Part::new("− PaO2", -pao2),
            Part::new("Expected: age / 4 + 4", expected),
        ],
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(fio2: f64, pao2: f64, pco2: f64, age: f64) -> Outcome {
        compute(
            &Inputs::new()
                .num("fio2", fio2)
                .num("pao2_mm_hg", pao2)
                .num("pco2_mm_hg", pco2)
                .num("age_years", age),
        )
        .unwrap()
    }

    #[test]
    fn room_air() {
        // PAO2 = 713 × 0.21 − 1.25 × 40 = 149.73 − 50 = 99.73; − 90 = 9.7.
        let o = run(0.21, 90.0, 40.0, 40.0);
        assert_eq!(o.score, Some(9.7));
        assert_eq!(o.band, "aa_gradient.normal"); // expected 40/4 + 4 = 14
        // Pulmonary embolism style: low PaO2 and low pCO2.
        let o = run(0.21, 60.0, 30.0, 40.0);
        assert_eq!(o.score, Some(52.2));
        assert_eq!(o.band, "aa_gradient.high");
    }

    #[test]
    fn percent_fio2_is_read_as_a_fraction() {
        assert_eq!(run(21.0, 90.0, 40.0, 40.0).score, Some(9.7));
    }
}
