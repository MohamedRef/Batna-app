//! Kidney, acid–base and gas exchange calculators.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("anion_gap", &["anion_gap.high", "anion_gap.low", "anion_gap.normal"])
            .paper(
                "book p. 5: Na 140 − (Cl 100 + HCO3 10) = 30, high; check the delta ratio",
                ins![na_meq_l: 140, cl_meq_l: 100, hco3_meq_l: 10],
                "anion_gap.high",
            )
            .score(30.0, 1e-9)
            .noted("delta ratio")
            .paper(
                "Figge 1998 / book p. 9: gap 12 with albumin 2.0 → 12 + 2.5 × 2 = 17, high",
                ins![na_meq_l: 140, cl_meq_l: 110, hco3_meq_l: 18, albumin_g_dl: 2.0],
                "anion_gap.high",
            )
            .score(17.0, 1e-9)
            .noted("Measured AG 12.0")
            .cut("16.0 / 16.1 (book p. 14: above 16 is high)", ins![na_meq_l: 140, cl_meq_l: 100], "hco3_meq_l", (24.0, "anion_gap.normal"), (23.9, "anion_gap.high"))
            .cut("8.0 / 7.9 (below 8 is low)", ins![na_meq_l: 140, cl_meq_l: 108], "hco3_meq_l", (24.0, "anion_gap.normal"), (24.1, "anion_gap.low"))
            .cut(
                "albumin 4.0 / 3.9: no correction, then + 0.25",
                ins![na_meq_l: 140, cl_meq_l: 104, hco3_meq_l: 24],
                "albumin_g_dl",
                (4.0, "anion_gap.normal"),
                (3.9, "anion_gap.normal"),
            ),
        Table::new(
            "delta_ratio",
            &["delta_ratio.normal_ag", "delta_ratio.mixed", "delta_ratio.pure_high", "delta_ratio.with_alkalosis"],
        )
            .paper("book p. 12: (24 − 12) / (24 − 12) = 1.0, pure high-gap acidosis", ins![anion_gap_meq_l: 24, hco3_meq_l: 12], "delta_ratio.pure_high")
            .score(1.0, 1e-9)
            .cut("0.39 / 0.40 (book: < 0.4 normal-gap)", ins![hco3_meq_l: 12], "anion_gap_meq_l", (16.7, "delta_ratio.normal_ag"), (16.8, "delta_ratio.mixed"))
            .cut("0.79 / 0.80 (0.8–2.0 pure)", ins![hco3_meq_l: 12], "anion_gap_meq_l", (21.5, "delta_ratio.mixed"), (21.6, "delta_ratio.pure_high"))
            .cut("2.00 / 2.02 (> 2 adds an alkalosis)", ins![hco3_meq_l: 12], "anion_gap_meq_l", (36.0, "delta_ratio.pure_high"), (36.2, "delta_ratio.with_alkalosis")),
        Table::new(
            "expected_compensation",
            &[
                "expected_compensation.appropriate",
                "expected_compensation.added_resp_acidosis",
                "expected_compensation.added_resp_alkalosis",
                "expected_compensation.added_met_alkalosis",
                "expected_compensation.added_met_acidosis",
            ],
        )
            .paper(
                "Winter's formula (Albert 1967, book p. 14): HCO3 12 → pCO2 1.5 × 12 + 8 = 26 ± 2",
                ins![disorder: "metabolic_acidosis", hco3_meq_l: 12, pco2_mm_hg: 26],
                "expected_compensation.appropriate",
            )
            .score(26.0, 1e-9)
            .cut("pCO2 28 / 29 (above 26 + 2)", ins![disorder: "metabolic_acidosis", hco3_meq_l: 12], "pco2_mm_hg", (28.0, "expected_compensation.appropriate"), (29.0, "expected_compensation.added_resp_acidosis"))
            .cut("pCO2 24 / 23 (below 26 − 2)", ins![disorder: "metabolic_acidosis", hco3_meq_l: 12], "pco2_mm_hg", (24.0, "expected_compensation.appropriate"), (23.0, "expected_compensation.added_resp_alkalosis"))
            .cut("metabolic alkalosis, HCO3 40: pCO2 53 / 54 (above 48 + 5)", ins![disorder: "metabolic_alkalosis", hco3_meq_l: 40], "pco2_mm_hg", (53.0, "expected_compensation.appropriate"), (54.0, "expected_compensation.added_resp_acidosis"))
            .paper(
                "book p. 14: metabolic alkalosis, HCO3 40 → pCO2 0.7 × 40 + 20 = 48 ± 5",
                ins![disorder: "metabolic_alkalosis", hco3_meq_l: 40, pco2_mm_hg: 48],
                "expected_compensation.appropriate",
            )
            .score(48.0, 1e-9)
            .cut("pCO2 53 / 54 (above 48 + 5)", ins![disorder: "metabolic_alkalosis", hco3_meq_l: 40], "pco2_mm_hg", (53.0, "expected_compensation.appropriate"), (54.0, "expected_compensation.added_resp_acidosis"))
            .paper(
                "acute respiratory acidosis, pCO2 60 → HCO3 24 + 2 × 1 = 26",
                ins![disorder: "respiratory_acidosis_acute", hco3_meq_l: 26, pco2_mm_hg: 60],
                "expected_compensation.appropriate",
            )
            .score(26.0, 1e-9)
            .noted("no range")
            .cut("HCO3 28 / 28.1 (above 26 + 2)", ins![disorder: "respiratory_acidosis_acute", pco2_mm_hg: 60], "hco3_meq_l", (28.0, "expected_compensation.appropriate"), (28.1, "expected_compensation.added_met_alkalosis"))
            .paper(
                "chronic respiratory acidosis, pCO2 60 → 24 + 2 × 4 = 32 (p. 14 summary)",
                ins![disorder: "respiratory_acidosis_chronic", hco3_meq_l: 32, pco2_mm_hg: 60],
                "expected_compensation.appropriate",
            )
            .score(32.0, 1e-9)
            .noted("3.5 per 10 mmHg")
            .cut("HCO3 30 / 29.9 (below 32 − 2)", ins![disorder: "respiratory_acidosis_chronic", pco2_mm_hg: 60], "hco3_meq_l", (30.0, "expected_compensation.appropriate"), (29.9, "expected_compensation.added_met_acidosis"))
            .paper(
                "acute respiratory alkalosis, pCO2 30 → 24 − 1 × 2 = 22",
                ins![disorder: "respiratory_alkalosis_acute", hco3_meq_l: 22, pco2_mm_hg: 30],
                "expected_compensation.appropriate",
            )
            .score(22.0, 1e-9)
            .cut("HCO3 20 / 19.9", ins![disorder: "respiratory_alkalosis_acute", pco2_mm_hg: 30], "hco3_meq_l", (20.0, "expected_compensation.appropriate"), (19.9, "expected_compensation.added_met_acidosis"))
            .paper(
                "chronic respiratory alkalosis, pCO2 30 → 24 − 1 × 5 = 19",
                ins![disorder: "respiratory_alkalosis_chronic", hco3_meq_l: 19, pco2_mm_hg: 30],
                "expected_compensation.appropriate",
            )
            .score(19.0, 1e-9)
            .noted("4–5 per 10 mmHg")
            .cut("HCO3 21 / 21.1", ins![disorder: "respiratory_alkalosis_chronic", pco2_mm_hg: 30], "hco3_meq_l", (21.0, "expected_compensation.appropriate"), (21.1, "expected_compensation.added_met_alkalosis")),
        Table::new("osmolal_gap", &["osmolal_gap.present", "osmolal_gap.normal"])
            .paper(
                "book p. 20: 2 × 140 + 90/18 + 14/2.8 = 290; measured 310 → gap 20",
                ins![measured_osm: 310, na_meq_l: 140, glucose_mg_dl: 90, bun_mg_dl: 14],
                "osmolal_gap.present",
            )
            .score(20.0, 1e-9)
            .noted("Calculated osmolality 290.0")
            .cut("gap 10 / 11 (above 10 is a gap)", ins![na_meq_l: 140, glucose_mg_dl: 90, bun_mg_dl: 14], "measured_osm", (300.0, "osmolal_gap.normal"), (301.0, "osmolal_gap.present"))
            .unit(
                "glucose 180 mg/dL = 10 mmol/L and BUN 28 mg/dL = 10 mmol/L: 280 + 10 + 10 = 300",
                ins![measured_osm: 300, na_meq_l: 140, glucose_mg_dl: 180, bun_mg_dl: 28],
                "osmolal_gap.normal",
            )
            .score(0.0, 1e-9),
        Table::new("kdigo_aki", &["kdigo_aki.none", "kdigo_aki.1", "kdigo_aki.2", "kdigo_aki.3"])
            .paper("KDIGO 2012: creatinine 1.0 → 2.0 is 2 × baseline, stage 2", ins![baseline_cr_mg_dl: 1.0, current_cr_mg_dl: 2.0, urine_output: "normal"], "kdigo_aki.2")
            .score(2.0, 0.0)
            .paper("RRT is stage 3 whatever the creatinine", ins![baseline_cr_mg_dl: 1.0, current_cr_mg_dl: 1.0, urine_output: "normal", rrt: true], "kdigo_aki.3")
            .paper("anuria 12 h is stage 3 by urine output", ins![baseline_cr_mg_dl: 1.0, current_cr_mg_dl: 1.0, urine_output: "lt03_24h_or_anuria"], "kdigo_aki.3")
            .paper("urine < 0.5 for 12 h is stage 2", ins![baseline_cr_mg_dl: 1.0, current_cr_mg_dl: 1.0, urine_output: "lt05_12h"], "kdigo_aki.2")
            .paper("urine < 0.5 for 6–12 h is stage 1; the stage is the worse of the two", ins![baseline_cr_mg_dl: 1.0, current_cr_mg_dl: 1.1, urine_output: "lt05_6h"], "kdigo_aki.1")
            .cut("ratio 1.48 / 1.50 with a rise under 0.3", ins![baseline_cr_mg_dl: 0.5, urine_output: "normal"], "current_cr_mg_dl", (0.74, "kdigo_aki.none"), (0.75, "kdigo_aki.1"))
            .cut("rise 0.29 / 0.30 mg/dL (≥ 0.3)", ins![baseline_cr_mg_dl: 1.0, urine_output: "normal"], "current_cr_mg_dl", (1.29, "kdigo_aki.none"), (1.3, "kdigo_aki.1"))
            .cut("ratio 1.99 / 2.0", ins![baseline_cr_mg_dl: 1.0, urine_output: "normal"], "current_cr_mg_dl", (1.99, "kdigo_aki.1"), (2.0, "kdigo_aki.2"))
            .cut("ratio 2.99 / 3.0", ins![baseline_cr_mg_dl: 1.0, urine_output: "normal"], "current_cr_mg_dl", (2.99, "kdigo_aki.2"), (3.0, "kdigo_aki.3"))
            .cut("creatinine 3.99 / 4.0 mg/dL (≥ 4 is stage 3)", ins![baseline_cr_mg_dl: 2.0, urine_output: "normal"], "current_cr_mg_dl", (3.99, "kdigo_aki.1"), (4.0, "kdigo_aki.3"))
            .paper("KDIGO 2012: an increase *to* ≥ 4.0; a stable 4.5 is no AKI", ins![baseline_cr_mg_dl: 4.5, current_cr_mg_dl: 4.5, urine_output: "normal"], "kdigo_aki.none")
            .paper("a rise of 0.3 that reaches 4.1 is stage 3", ins![baseline_cr_mg_dl: 3.8, current_cr_mg_dl: 4.1, urine_output: "normal"], "kdigo_aki.3"),
        Table::new("fena", &["fena.prerenal", "fena.indeterminate", "fena.atn"])
            .paper(
                "Espinel 1976 / book p. 109: (10 × 2) / (140 × 100) × 100 = 0.14%, prerenal",
                ins![urine_na_meq_l: 10, plasma_na_meq_l: 140, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 2.0],
                "fena.prerenal",
            )
            .score(0.14, 1e-9)
            .paper(
                "ATN: (60 × 4) / (135 × 40) × 100 = 4.44%",
                ins![urine_na_meq_l: 60, plasma_na_meq_l: 135, urine_cr_mg_dl: 40, plasma_cr_mg_dl: 4.0],
                "fena.atn",
            )
            .score(4.44, 1e-9)
            .cut("0.99 / 1.00% (< 1 prerenal)", ins![plasma_na_meq_l: 100, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 1.0], "urine_na_meq_l", (99.0, "fena.prerenal"), (100.0, "fena.indeterminate"))
            .cut("2.00 / 2.01% (> 2 ATN)", ins![plasma_na_meq_l: 100, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 1.0], "urine_na_meq_l", (200.0, "fena.indeterminate"), (201.0, "fena.atn")),
        Table::new("feurea", &["feurea.prerenal", "feurea.indeterminate", "feurea.atn"])
            .paper(
                "Carvounis 2002: (300 × 2) / (60 × 100) × 100 = 10%, prerenal even on diuretics",
                ins![urine_urea_mg_dl: 300, plasma_urea_mg_dl: 60, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 2.0],
                "feurea.prerenal",
            )
            .score(10.0, 1e-9)
            .cut("34.98 / 35.00% (< 35 prerenal)", ins![plasma_urea_mg_dl: 50, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 1.0], "urine_urea_mg_dl", (1749.0, "feurea.prerenal"), (1750.0, "feurea.indeterminate"))
            .cut("50.00 / 50.02% (> 50 ATN)", ins![plasma_urea_mg_dl: 50, urine_cr_mg_dl: 100, plasma_cr_mg_dl: 1.0], "urine_urea_mg_dl", (2500.0, "feurea.indeterminate"), (2501.0, "feurea.atn")),
        Table::new("bun_creatinine", &["bun_creatinine.prerenal", "bun_creatinine.atn", "bun_creatinine.indeterminate"])
            .paper("book p. 109: BUN 60 / creatinine 2 = 30, prerenal", ins![bun_mg_dl: 60, creatinine_mg_dl: 2.0], "bun_creatinine.prerenal")
            .score(30.0, 1e-9)
            .cut("20.0 / 20.1 (> 20 prerenal)", ins![creatinine_mg_dl: 1.0], "bun_mg_dl", (20.0, "bun_creatinine.indeterminate"), (20.1, "bun_creatinine.prerenal"))
            .cut("15.0 / 14.9 (< 15 ATN)", ins![creatinine_mg_dl: 1.0], "bun_mg_dl", (15.0, "bun_creatinine.indeterminate"), (14.9, "bun_creatinine.atn")),
        Table::new(
            "ckd_epi_2021",
            &["ckd_epi_2021.g1", "ckd_epi_2021.g2", "ckd_epi_2021.g3a", "ckd_epi_2021.g3b", "ckd_epi_2021.g4", "ckd_epi_2021.g5"],
        )
            .paper("book p. 29 (Inker 2021): man 24, creatinine 1.3 → 78.7", ins![creatinine_mg_dl: 1.3, age_years: 24, sex: "male"], "ckd_epi_2021.g2")
            .score(78.7, 1e-9)
            .paper("book p. 29: man 24, creatinine 4.5 → 17.7", ins![creatinine_mg_dl: 4.5, age_years: 24, sex: "male"], "ckd_epi_2021.g4")
            .score(17.7, 1e-9)
            .paper("book p. 29: man 60, creatinine 1.3 → 62.9", ins![creatinine_mg_dl: 1.3, age_years: 60, sex: "male"], "ckd_epi_2021.g2")
            .score(62.9, 1e-9)
            .paper("book p. 29: man 60, creatinine 4.5 → 14.2", ins![creatinine_mg_dl: 4.5, age_years: 60, sex: "male"], "ckd_epi_2021.g5")
            .score(14.2, 1e-9)
            .paper("book p. 29: woman 24, creatinine 1.3 → 58.9", ins![creatinine_mg_dl: 1.3, age_years: 24, sex: "female"], "ckd_epi_2021.g3a")
            .score(58.9, 1e-9)
            .paper("book p. 29: woman 24, creatinine 4.5 → 13.3", ins![creatinine_mg_dl: 4.5, age_years: 24, sex: "female"], "ckd_epi_2021.g5")
            .score(13.3, 1e-9)
            .paper("book p. 29: woman 60, creatinine 1.3 → 47.1", ins![creatinine_mg_dl: 1.3, age_years: 60, sex: "female"], "ckd_epi_2021.g3a")
            .score(47.1, 1e-9)
            .paper("book p. 29: woman 60, creatinine 4.5 → 10.6", ins![creatinine_mg_dl: 4.5, age_years: 60, sex: "female"], "ckd_epi_2021.g5")
            .score(10.6, 1e-9)
            .paper(
                "Inker 2021, below κ (α −0.241): woman 40, creatinine 0.6 → 142 × (0.6/0.7)^−0.241 × 0.9938^40 × 1.012 = 116.3",
                ins![creatinine_mg_dl: 0.6, age_years: 40, sex: "female"],
                "ckd_epi_2021.g1",
            )
            .score(116.3, 1e-9)
            .noted("2009 CKD-EPI")
            .cut("90.0 / 89.1 (G1 from 90)", ins![age_years: 21, sex: "male"], "creatinine_mg_dl", (1.18, "ckd_epi_2021.g1"), (1.19, "ckd_epi_2021.g2"))
            .cut("60.0 / 59.6 (G2 from 60)", ins![age_years: 18, sex: "male"], "creatinine_mg_dl", (1.68, "ckd_epi_2021.g2"), (1.69, "ckd_epi_2021.g3a"))
            .cut("45.0 / 44.8 (G3a from 45)", ins![age_years: 23, sex: "male"], "creatinine_mg_dl", (2.08, "ckd_epi_2021.g3a"), (2.09, "ckd_epi_2021.g3b"))
            .cut("30.0 / 29.9 (G3b from 30)", ins![age_years: 19, sex: "male"], "creatinine_mg_dl", (2.98, "ckd_epi_2021.g3b"), (2.99, "ckd_epi_2021.g4"))
            .cut("15.0 / 14.9 (G4 from 15)", ins![age_years: 18, sex: "male"], "creatinine_mg_dl", (5.35, "ckd_epi_2021.g4"), (5.36, "ckd_epi_2021.g5")),
        Table::new("kdigo_ckd", &["kdigo_ckd.low", "kdigo_ckd.moderate", "kdigo_ckd.high", "kdigo_ckd.very_high"])
            .paper("KDIGO 2012 heat map (book p. 32): G3a A2 is orange, checked twice a year", ins![egfr: 50, acr_mg_g: 100], "kdigo_ckd.high")
            .noted("2 time(s) a year")
            .paper("G4 A3: red, 4+ checks and nephrology care", ins![egfr: 20, acr_mg_g: 400], "kdigo_ckd.very_high")
            .noted("4+")
            .cut("eGFR 60 / 59.9 with ACR 10", ins![acr_mg_g: 10], "egfr", (60.0, "kdigo_ckd.low"), (59.9, "kdigo_ckd.moderate"))
            .cut("eGFR 45 / 44.9 with ACR 10", ins![acr_mg_g: 10], "egfr", (45.0, "kdigo_ckd.moderate"), (44.9, "kdigo_ckd.high"))
            .cut("eGFR 30 / 29.9 with ACR 10", ins![acr_mg_g: 10], "egfr", (30.0, "kdigo_ckd.high"), (29.9, "kdigo_ckd.very_high"))
            .cut("eGFR 45 / 44.9 with ACR 100 (G3a A2 orange, G3b A2 red)", ins![acr_mg_g: 100], "egfr", (45.0, "kdigo_ckd.high"), (44.9, "kdigo_ckd.very_high"))
            .cut("ACR 29 / 30 mg/g with eGFR 70 (A2 from 30)", ins![egfr: 70], "acr_mg_g", (29.0, "kdigo_ckd.low"), (30.0, "kdigo_ckd.moderate"))
            .cut("ACR 299 / 300 mg/g with eGFR 70 (A3 from 300)", ins![egfr: 70], "acr_mg_g", (299.0, "kdigo_ckd.moderate"), (300.0, "kdigo_ckd.high")),
        Table::new("aa_gradient", &["aa_gradient.high", "aa_gradient.normal"])
            .paper(
                "book p. 14: room air, pCO2 40, PaO2 90, age 40: 713 × 0.21 − 50 − 90 = 9.7 (expected 14)",
                ins![fio2: 0.21, pao2_mm_hg: 90, pco2_mm_hg: 40, age_years: 40],
                "aa_gradient.normal",
            )
            .score(9.7, 1e-9)
            .paper(
                "PaO2 60, pCO2 30: 149.7 − 37.5 − 60 = 52.2, raised",
                ins![fio2: 0.21, pao2_mm_hg: 60, pco2_mm_hg: 30, age_years: 40],
                "aa_gradient.high",
            )
            .score(52.2, 1e-9)
            .paper(
                "on 40% oxygen the age rule is for room air",
                ins![fio2: 0.4, pao2_mm_hg: 200, pco2_mm_hg: 40, age_years: 40],
                "aa_gradient.high",
            )
            .noted("room air")
            .cut("gradient 14.0 / 14.1 at age 40 (expected 40/4 + 4)", ins![fio2: 0.21, pco2_mm_hg: 40, age_years: 40], "pao2_mm_hg", (85.73, "aa_gradient.normal"), (85.6, "aa_gradient.high")),
        Table::new("free_water_deficit", &["free_water_deficit.volume"])
            .paper(
                "Adrogué & Madias 2000: man 70 kg, Na 160: 42 × (160/140 − 1) = 6.0 L of water",
                ins![na_meq_l: 160, weight_kg: 70, sex: "male", fluid: "water"],
                "free_water_deficit.volume",
            )
            .score(6.0, 1e-9)
            .unit(
                "half-normal saline is half free water: 6.0 / (1 − 77/154) = 12.0 L",
                ins![na_meq_l: 160, weight_kg: 70, sex: "male", fluid: "saline_0_45"],
                "free_water_deficit.volume",
            )
            .score(12.0, 1e-9)
            .noted("deficit ÷ (1 − Na/154)")
            .unit(
                "0.2% saline: 6.0 / (1 − 34/154) = 7.7 L",
                ins![na_meq_l: 160, weight_kg: 70, sex: "male", fluid: "saline_0_2"],
                "free_water_deficit.volume",
            )
            .score(7.7, 1e-9)
            .edge("man 70 kg: TBW 0.6", ins![na_meq_l: 154, weight_kg: 70, sex: "male", fluid: "water"], "free_water_deficit.volume")
            .score(4.2, 1e-9)
            .edge("elderly man: TBW 0.5", ins![na_meq_l: 154, weight_kg: 70, sex: "male", elderly: true, fluid: "water"], "free_water_deficit.volume")
            .score(3.5, 1e-9)
            .edge("woman: TBW 0.5", ins![na_meq_l: 154, weight_kg: 50, sex: "female", fluid: "water"], "free_water_deficit.volume")
            .score(2.5, 1e-9)
            .edge("elderly woman: TBW 0.45", ins![na_meq_l: 154, weight_kg: 50, sex: "female", elderly: true, fluid: "water"], "free_water_deficit.volume")
            .score(2.3, 1e-9),
        Table::new("plasmic", &["plasmic.high", "plasmic.intermediate", "plasmic.low"])
            .paper(
                "Bendapudi 2017: platelets 15, haemolysis, no cancer, no transplant, MCV 85, INR 1.1, creatinine 1.2 = 7",
                ins![platelets_10e9_l: 15, hemolysis: true, mcv_fl: 85, inr: 1.1, creatinine_mg_dl: 1.2],
                "plasmic.high",
            )
            .score(7.0, 0.0)
            .noted("schistocytes")
            .cut(
                "creatinine 2.0 / 1.99 moves 4 to 5 (< 2.0 scores)",
                ins![platelets_10e9_l: 50, mcv_fl: 85, inr: 1.2],
                "creatinine_mg_dl",
                (2.0, "plasmic.low"),
                (1.99, "plasmic.intermediate"),
            )
            .cut(
                "platelets 30 / 29 moves 5 to 6 (< 30 scores)",
                ins![mcv_fl: 85, inr: 1.2, creatinine_mg_dl: 1.0],
                "platelets_10e9_l",
                (30.0, "plasmic.intermediate"),
                (29.0, "plasmic.high"),
            )
            .cut("INR 1.5 / 1.4 (< 1.5 scores)", ins![platelets_10e9_l: 50, mcv_fl: 85, creatinine_mg_dl: 1.0], "inr", (1.5, "plasmic.low"), (1.4, "plasmic.intermediate"))
            .cut("MCV 90 / 89 (< 90 scores)", ins![platelets_10e9_l: 50, inr: 1.2, creatinine_mg_dl: 1.0], "mcv_fl", (90.0, "plasmic.low"), (89.0, "plasmic.intermediate"))
            .paper(
                "active cancer and a transplant take 2 points off",
                ins![platelets_10e9_l: 15, hemolysis: true, mcv_fl: 85, inr: 1.1, creatinine_mg_dl: 1.2, active_cancer: true, transplant: true],
                "plasmic.intermediate",
            )
            .score(5.0, 0.0),
        Table::new(
            "cockcroft_gault",
            &["cockcroft_gault.normal", "cockcroft_gault.mild", "cockcroft_gault.moderate", "cockcroft_gault.severe", "cockcroft_gault.failure"],
        )
            .paper(
                "Cockcroft & Gault 1976 / book p. 30: man 60, 72 kg, creatinine 1.0: 80 × 72 / 72 = 80 mL/min",
                ins![age_years: 60, weight_kg: 72, creatinine_mg_dl: 1.0, sex: "male"],
                "cockcroft_gault.mild",
            )
            .score(80.0, 1e-9)
            .edge("man: no factor", ins![age_years: 80, weight_kg: 60, creatinine_mg_dl: 1.5, sex: "male"], "cockcroft_gault.moderate")
            .score(33.3, 1e-9)
            .edge("woman: × 0.85", ins![age_years: 80, weight_kg: 60, creatinine_mg_dl: 1.5, sex: "female"], "cockcroft_gault.severe")
            .score(28.3, 1e-9)
            .edge("FDA 2024: 90 or more normal", ins![age_years: 50, weight_kg: 72, creatinine_mg_dl: 1.0, sex: "male"], "cockcroft_gault.normal")
            .score(90.0, 1e-9)
            .edge("FDA 2024: below 15 kidney failure", ins![age_years: 80, weight_kg: 60, creatinine_mg_dl: 5.0, sex: "male"], "cockcroft_gault.failure")
            .score(10.0, 1e-9),
    ]
}
