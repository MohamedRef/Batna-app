//! Diabetes calculators.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("corrected_sodium", &["corrected_sodium.low", "corrected_sodium.normal", "corrected_sodium.high"])
            .paper(
                "book p. 189 (Katz 1973 factor 1.6 / 100 mg/dL): glucose 500, Na 132 → 138.4",
                ins![na_meq_l: 132, glucose_mg_dl: 500],
                "corrected_sodium.normal",
            )
            .score(138.4, 1e-9)
            .edge("glucose 90: no correction below 100", ins![na_meq_l: 140, glucose_mg_dl: 90], "corrected_sodium.normal")
            .score(140.0, 1e-9)
            .edge("glucose 200: + 1.6", ins![na_meq_l: 140, glucose_mg_dl: 200], "corrected_sodium.normal")
            .score(141.6, 1e-9)
            .cut("134 / 135 (Nephrology p. 127: 135–145)", ins![glucose_mg_dl: 100], "na_meq_l", (134.0, "corrected_sodium.low"), (135.0, "corrected_sodium.normal"))
            .cut("145.0 / 146.0 (above 145 high)", ins![glucose_mg_dl: 100], "na_meq_l", (145.0, "corrected_sodium.normal"), (146.0, "corrected_sodium.high")),
        Table::new("effective_osmolality", &["effective_osmolality.hhs", "effective_osmolality.below"])
            .paper(
                "book p. 189: 2 × 150 + 900/18 = 350, HHS",
                ins![na_meq_l: 150, glucose_mg_dl: 900],
                "effective_osmolality.hhs",
            )
            .score(350.0, 1e-9)
            .cut("effective 300.0 / 300.1 (above 300)", ins![na_meq_l: 145], "glucose_mg_dl", (180.0, "effective_osmolality.below"), (181.8, "effective_osmolality.hhs"))
            .cut(
                "total 320.0 / 320.1 with urea in mmol/L (above 320)",
                ins![na_meq_l: 140, glucose_mg_dl: 180],
                "urea_mmol_l",
                (30.0, "effective_osmolality.below"),
                (30.1, "effective_osmolality.hhs"),
            )
            .unit(
                "glucose 360 mg/dL = 20 mmol/L: 2 × 140 + 20 = 300, and urea 10 gives the total note",
                ins![na_meq_l: 140, glucose_mg_dl: 360, urea_mmol_l: 10.0],
                "effective_osmolality.below",
            )
            .score(300.0, 1e-9)
            .noted("Total osmolality 310.0"),
        Table::new("dka_severity", &["dka_severity.not_met", "dka_severity.mild", "dka_severity.moderate", "dka_severity.severe"])
            .paper(
                "ADA 2024 (Umpierrez): glucose 450, β-OHB 4, pH 7.15, HCO3 12, alert → moderate",
                ins![ph: 7.15, hco3_mmol_l: 12, bhb_mmol_l: 4.0, glucose_mg_dl: 450, mental: "alert"],
                "dka_severity.moderate",
            )
            .paper(
                "euglycaemic DKA on an SGLT2 inhibitor: glucose 180 with known diabetes",
                ins![ph: 7.2, hco3_mmol_l: 14, bhb_mmol_l: 5.0, glucose_mg_dl: 180, known_diabetes: true, mental: "alert"],
                "dka_severity.moderate",
            )
            .noted("euglycaemic")
            .paper(
                "ketoacidosis without diabetes: think of another cause",
                ins![ph: 7.2, hco3_mmol_l: 14, bhb_mmol_l: 5.0, glucose_mg_dl: 110, mental: "alert"],
                "dka_severity.not_met",
            )
            .noted("another ketoacidosis")
            .cut("pH 7.30 / 7.29 (acidosis < 7.3)", ins![hco3_mmol_l: 20, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "ph", (7.3, "dka_severity.not_met"), (7.29, "dka_severity.mild"))
            .cut("pH 7.26 / 7.25 (7.0–7.25 moderate)", ins![hco3_mmol_l: 20, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "ph", (7.26, "dka_severity.mild"), (7.25, "dka_severity.moderate"))
            .cut("pH 7.0 / 6.99 (< 7.0 severe)", ins![hco3_mmol_l: 20, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "ph", (7.0, "dka_severity.moderate"), (6.99, "dka_severity.severe"))
            .cut("HCO3 18 / 17.9 (< 18)", ins![ph: 7.35, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "hco3_mmol_l", (18.0, "dka_severity.not_met"), (17.9, "dka_severity.mild"))
            .cut("HCO3 15 / 14.9 (10 to < 15 moderate)", ins![ph: 7.35, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "hco3_mmol_l", (15.0, "dka_severity.mild"), (14.9, "dka_severity.moderate"))
            .cut("HCO3 10 / 9.9 (< 10 severe)", ins![ph: 7.35, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "hco3_mmol_l", (10.0, "dka_severity.moderate"), (9.9, "dka_severity.severe"))
            .cut("β-OHB 2.9 / 3.0 as the only ketone test (≥ 3.0)", ins![ph: 7.28, hco3_mmol_l: 17, glucose_mg_dl: 300, mental: "alert"], "bhb_mmol_l", (2.9, "dka_severity.not_met"), (3.0, "dka_severity.mild"))
            .cut("β-OHB 6.0 / 6.1 (above 6 severe)", ins![ph: 7.28, hco3_mmol_l: 17, glucose_mg_dl: 300, mental: "alert"], "bhb_mmol_l", (6.0, "dka_severity.mild"), (6.1, "dka_severity.severe"))
            .cut("glucose 199 / 200 without known diabetes", ins![ph: 7.28, hco3_mmol_l: 17, urine_ketones: true, mental: "alert"], "glucose_mg_dl", (199.0, "dka_severity.not_met"), (200.0, "dka_severity.mild"))
            .edge("alert: mild", ins![ph: 7.28, hco3_mmol_l: 17, urine_ketones: true, glucose_mg_dl: 300, mental: "alert"], "dka_severity.mild")
            .edge("drowsy: moderate", ins![ph: 7.28, hco3_mmol_l: 17, urine_ketones: true, glucose_mg_dl: 300, mental: "drowsy"], "dka_severity.moderate")
            .edge("drowsy: moderate", ins![ph: 7.28, hco3_mmol_l: 17, urine_ketones: true, glucose_mg_dl: 300, mental: "drowsy"], "dka_severity.moderate")
            .edge("stupor or coma: severe", ins![ph: 7.28, hco3_mmol_l: 17, urine_ketones: true, glucose_mg_dl: 300, mental: "coma"], "dka_severity.severe"),
        Table::new("premixed_insulin", &["premixed_insulin.dose"])
            .paper("book p. 128: 80 kg × 0.3 = 24 U/day (16 before breakfast, 8 before dinner)", ins![weight_kg: 80], "premixed_insulin.dose")
            .score(24.0, 0.0)
            .noted("whole units")
            .edge("74 kg: 22.2 → 22 U", ins![weight_kg: 74], "premixed_insulin.dose")
            .score(22.0, 0.0)
            .edge("75 kg: 22.5 → 23 U", ins![weight_kg: 75], "premixed_insulin.dose")
            .score(23.0, 0.0),
        Table::new("bmi", &["bmi.class3", "bmi.class2", "bmi.class1", "bmi.overweight", "bmi.normal", "bmi.under"])
            .paper("WHO 2000: 70 kg, 175 cm → 22.9", ins![weight_kg: 70, height_cm: 175], "bmi.normal")
            .score(22.9, 1e-9)
            .cut("18.4 / 18.5 (WHO: under 18.5)", ins![height_cm: 200], "weight_kg", (73.6, "bmi.under"), (74.0, "bmi.normal"))
            .cut("24.9 / 25.0 (book p. 18: ≥ 25 overweight)", ins![height_cm: 200], "weight_kg", (99.6, "bmi.normal"), (100.0, "bmi.overweight"))
            .cut("29.9 / 30.0", ins![height_cm: 200], "weight_kg", (119.6, "bmi.overweight"), (120.0, "bmi.class1"))
            .cut("34.9 / 35.0", ins![height_cm: 200], "weight_kg", (139.6, "bmi.class1"), (140.0, "bmi.class2"))
            .cut("39.9 / 40.0", ins![height_cm: 200], "weight_kg", (159.6, "bmi.class2"), (160.0, "bmi.class3"))
            .paper("26.9: no drug note", ins![weight_kg: 107.6, height_cm: 200], "bmi.overweight")
            .paper("27.0: obesity drugs may be added (book p. 22)", ins![weight_kg: 108, height_cm: 200], "bmi.overweight")
            .noted("pharmacotherapy"),
    ]
}
