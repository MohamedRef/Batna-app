//! Adult normal ranges shared by calculator inputs, one per test, each with
//! its source. A course book's range wins where it gives one; the rest come
//! from the NBME Laboratory Values table (April 2026,
//! nbme.org/laboratory-values), the reference table of the USMLE. Labs
//! differ a little, so these colour an input box; they diagnose nothing.
//!
//! A field takes one with `.normal_as(normals::ALBUMIN_G_DL)`; a range used
//! by one calculator only can stay on its field as `.normal(lo, hi)`.

use super::Normal;

const fn r(low: f64, high: f64) -> Normal {
    Normal { low, high, female: None, below: false }
}

/// Men's and women's ranges.
const fn sexes(male: (f64, f64), female: (f64, f64)) -> Normal {
    Normal { low: male.0, high: male.1, female: Some(female), below: false }
}

/// Normal under `high`, which is itself high ("< 150").
pub(super) const fn under(high: f64) -> Normal {
    Normal { low: f64::NEG_INFINITY, high, female: None, below: true }
}

// Acid–base and electrolytes.

/// Nephrology book, Lecture 6, p. 127: "Normal SNa: 135-145 mEq/L".
pub const SODIUM: Normal = r(135.0, 145.0);
/// NBME.
pub const CHLORIDE: Normal = r(95.0, 105.0);
/// Nephrology book, Lecture 1, p. 14–15: 24 ± 2 (22–26 mEq/L).
pub const HCO3: Normal = r(22.0, 26.0);
/// Nephrology book, Lecture 1, p. 14: 12 ± 4.
pub const ANION_GAP: Normal = r(8.0, 16.0);
/// Nephrology book, Lecture 1, p. 14–15.
pub const PCO2_MM_HG: Normal = r(35.0, 45.0);
/// Nephrology book, Lecture 1, p. 15.
pub const PH: Normal = r(7.35, 7.45);
/// Nephrology book, Lecture 1, p. 15.
pub const PAO2_MM_HG: Normal = r(80.0, 100.0);
/// Base excess −2 to +2 (Nephrology book, Lecture 1, p. 15); a base deficit
/// is the same number with the sign turned.
pub const BASE_DEFICIT: Normal = r(-2.0, 2.0);
/// NBME.
pub const OSMOLALITY: Normal = r(275.0, 295.0);
/// Above 2 mmol/L is raised (Sepsis-3, Singer et al., JAMA 2016).
pub const LACTATE_MMOL_L: Normal = r(f64::NEG_INFINITY, 2.0);

// Kidney.

/// NBME (urea nitrogen).
pub const BUN_MG_DL: Normal = r(7.0, 18.0);
/// NBME's urea nitrogen in SI units, which is urea in mmol/L.
pub const UREA_MMOL_L: Normal = r(2.5, 6.4);
/// Urea itself, as Egyptian labs report it: NBME's urea nitrogen 7–18
/// mg/dL × 2.14 (urea weighs 60, its two nitrogens 28).
pub const UREA_MG_DL: Normal = r(15.0, 38.5);
/// NBME.
pub const CREATININE_MG_DL: Normal = r(0.6, 1.2);
/// KDIGO 2024 CKD guideline: G1 is 90 and above ("normal or high").
pub const EGFR: Normal = r(90.0, f64::INFINITY);
/// KDIGO 2024 CKD guideline: A1, below 30 mg/g ("normal to mildly
/// increased").
pub const ACR_MG_G: Normal = under(30.0);

// Chemistry.

/// NBME: fasting 70–100, random (non-fasting) below 140 mg/dL. The
/// calculators take a random glucose.
pub const GLUCOSE_MG_DL: Normal = r(70.0, 140.0);
/// NBME (total calcium).
pub const CALCIUM_MG_DL: Normal = r(8.4, 10.2);
/// NBME.
pub const LDH_U_L: Normal = r(45.0, 200.0);
/// NBME: normal below 150 mg/dL.
pub const TRIGLYCERIDES_MG_DL: Normal = under(150.0);
/// β-hydroxybutyrate: DKA counts as resolved below 0.6 mmol/L
/// (ADA/EASD/JBDS hyperglycaemic crises consensus, Umpierrez et al.,
/// Diabetes Care 2024).
pub const BHB_MMOL_L: Normal = under(0.6);

// Liver.

/// NBME.
pub const ALBUMIN_G_DL: Normal = r(3.5, 5.5);
/// NBME (total bilirubin).
pub const BILIRUBIN_MG_DL: Normal = r(0.1, 1.0);
/// NBME.
pub const AST_U_L: Normal = r(12.0, 38.0);
/// NBME.
pub const ALT_U_L: Normal = r(10.0, 40.0);
/// NBME.
pub const ALP_U_L: Normal = r(25.0, 100.0);
/// GGT, adults: men 8–61, women 5–36 U/L (Mayo Clinic Laboratories). It
/// varies more between labs than most.
pub const GGT_U_L: Normal = sexes((8.0, 61.0), (5.0, 36.0));

// Blood and clotting.

/// NBME: men 13.5–17.5, women 12.0–16.0 g/dL.
pub const HB_G_DL: Normal = sexes((13.5, 17.5), (12.0, 16.0));
/// NBME: men 41–53 %, women 36–46 %.
pub const HCT_PCT: Normal = sexes((41.0, 53.0), (36.0, 46.0));
/// NBME: men 4.3–5.9, women 3.5–5.5 million/mm³ (× 10¹²/L).
pub const RBC_10E12_L: Normal = sexes((4.3, 5.9), (3.5, 5.5));
/// NBME: 0.5–1.5 % of red cells.
pub const RETIC_PCT: Normal = r(0.5, 1.5);
/// NBME: 80–100 fL.
pub const MCV_FL: Normal = r(80.0, 100.0);
/// NBME (Westergren): men 0–15, women 0–20 mm/h.
pub const ESR_MM_H: Normal = sexes((0.0, 15.0), (0.0, 20.0));
/// NBME.
pub const WBC_10E9_L: Normal = r(4.5, 11.0);
/// NBME, per mm³.
pub const WBC_PER_MM3: Normal = r(4500.0, 11000.0);
/// Haematology book, Lecture 4, p. 62: 150,000–400,000/mm³.
pub const PLATELETS_10E9_L: Normal = r(150.0, 400.0);
/// Blasts are not seen in normal blood (NBME's differential has none).
pub const BLASTS_PCT: Normal = r(0.0, 0.0);
/// NBME.
pub const PT_S: Normal = r(11.0, 15.0);
/// Medscape, "Lab Values, Normal Adult" (2025): 0.8–1.1.
pub const INR: Normal = r(0.8, 1.1);

// Vital signs.

/// Normal resting heart rate (American Heart Association).
pub const HEART_RATE: Normal = r(60.0, 100.0);
/// Normal is below 120 mmHg (ACC/AHA 2017, Whelton et al.); below 90 is
/// hypotension.
pub const SBP_MM_HG: Normal = r(90.0, 119.0);

// Body.

/// WHO (2000), as the BMI calculator: 18.5–24.9 kg/m².
pub const BMI: Normal = r(18.5, 24.9);
/// Waist under 94 cm in men and 80 cm in women: the IDF cut-offs for
/// Europid and Middle Eastern (Arab) adults (Alberti et al., Circulation
/// 2009, the harmonised metabolic syndrome definition).
pub const WAIST_CM: Normal =
    Normal { low: f64::NEG_INFINITY, high: 94.0, female: Some((f64::NEG_INFINITY, 80.0)), below: true };
