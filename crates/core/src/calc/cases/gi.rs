//! Pancreas and GI calculators.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("ranson", &["ranson.0_2", "ranson.3_4", "ranson.5_6", "ranson.7_11"])
            .paper(
                "Ranson 1974, non-gallstone: age 60, WBC 18,000, glucose 250, LDH 400, AST 300 = 5 at admission",
                ins![age_years: 60, wbc_per_mm3: 18000, glucose_mg_dl: 250, ldh_iu_l: 400, ast_iu_l: 300],
                "ranson.5_6",
            )
            .score(5.0, 0.0)
            .noted("6 of the 48-hour criteria not entered")
            .paper(
                "all 11 non-gallstone criteria met",
                ins![
                    age_years: 60, wbc_per_mm3: 18000, glucose_mg_dl: 250, ldh_iu_l: 400, ast_iu_l: 300,
                    hct_fall_points: 12, bun_rise_mg_dl: 6, calcium_mg_dl: 7.5, pao2_mm_hg: 55,
                    base_deficit_meq_l: 5, fluid_sequestration_l: 7.0,
                ],
                "ranson.7_11",
            )
            .score(11.0, 0.0)
            .paper(
                "Ranson 1979, gallstone: no PaO₂ criterion, so PaO₂ 50 adds nothing",
                ins![gallstone: true, age_years: 50, wbc_per_mm3: 10000, glucose_mg_dl: 120, ldh_iu_l: 200, ast_iu_l: 40, pao2_mm_hg: 50],
                "ranson.0_2",
            )
            .score(0.0, 0.0)
            .cut(
                "age 55 / 56 as the third point (non-gallstone: > 55)",
                ins![wbc_per_mm3: 17000, glucose_mg_dl: 210, ldh_iu_l: 200, ast_iu_l: 40],
                "age_years",
                (55.0, "ranson.0_2"),
                (56.0, "ranson.3_4"),
            )
            .cut(
                "WBC 16,000 / 16,100 as the fifth point (> 16,000)",
                ins![age_years: 60, glucose_mg_dl: 250, ldh_iu_l: 400, ast_iu_l: 300],
                "wbc_per_mm3",
                (16000.0, "ranson.3_4"),
                (16100.0, "ranson.5_6"),
            )
            .cut(
                "glucose 200 / 201 mg/dL (> 200)",
                ins![age_years: 40, wbc_per_mm3: 8000, ldh_iu_l: 200, ast_iu_l: 40],
                "glucose_mg_dl",
                (200.0, "ranson.0_2"),
                (201.0, "ranson.0_2"),
            )
            .cut(
                "LDH 350 / 351 IU/L (> 350)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ast_iu_l: 40],
                "ldh_iu_l",
                (350.0, "ranson.0_2"),
                (351.0, "ranson.0_2"),
            )
            .cut(
                "AST 250 / 251 IU/L (> 250)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200],
                "ast_iu_l",
                (250.0, "ranson.0_2"),
                (251.0, "ranson.0_2"),
            )
            .cut(
                "calcium 8.0 / 7.9 mg/dL at 48 h as the seventh point (< 8)",
                ins![
                    age_years: 60, wbc_per_mm3: 18000, glucose_mg_dl: 250, ldh_iu_l: 400, ast_iu_l: 300,
                    hct_fall_points: 12,
                ],
                "calcium_mg_dl",
                (8.0, "ranson.5_6"),
                (7.9, "ranson.7_11"),
            )
            .cut(
                "PaO₂ 60 / 59 mmHg (< 60, non-gallstone only)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "pao2_mm_hg",
                (60.0, "ranson.0_2"),
                (59.0, "ranson.0_2"),
            )
            .cut(
                "haematocrit fall 10 / 11 points (> 10)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "hct_fall_points",
                (10.0, "ranson.0_2"),
                (11.0, "ranson.0_2"),
            )
            .cut(
                "BUN rise 5 / 6 mg/dL non-gallstone (> 5)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "bun_rise_mg_dl",
                (5.0, "ranson.0_2"),
                (6.0, "ranson.0_2"),
            )
            .cut(
                "base deficit 4 / 5 non-gallstone (> 4)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "base_deficit_meq_l",
                (4.0, "ranson.0_2"),
                (5.0, "ranson.0_2"),
            )
            .cut(
                "sequestration 6 / 6.5 L non-gallstone (> 6)",
                ins![age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "fluid_sequestration_l",
                (6.0, "ranson.0_2"),
                (6.5, "ranson.0_2"),
            )
            .cut(
                "gallstone age 70 / 71 (> 70)",
                ins![gallstone: true, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "age_years",
                (70.0, "ranson.0_2"),
                (71.0, "ranson.0_2"),
            )
            .cut(
                "gallstone WBC 18,000 / 18,100 (> 18,000)",
                ins![gallstone: true, age_years: 40, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "wbc_per_mm3",
                (18000.0, "ranson.0_2"),
                (18100.0, "ranson.0_2"),
            )
            .cut(
                "gallstone glucose 220 / 221 (> 220)",
                ins![gallstone: true, age_years: 40, wbc_per_mm3: 8000, ldh_iu_l: 200, ast_iu_l: 40],
                "glucose_mg_dl",
                (220.0, "ranson.0_2"),
                (221.0, "ranson.0_2"),
            )
            .cut(
                "gallstone LDH 400 / 401 (> 400)",
                ins![gallstone: true, age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ast_iu_l: 40],
                "ldh_iu_l",
                (400.0, "ranson.0_2"),
                (401.0, "ranson.0_2"),
            )
            .cut(
                "gallstone BUN rise 2 / 3 (> 2)",
                ins![gallstone: true, age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "bun_rise_mg_dl",
                (2.0, "ranson.0_2"),
                (3.0, "ranson.0_2"),
            )
            .cut(
                "gallstone base deficit 5 / 6 (> 5)",
                ins![gallstone: true, age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "base_deficit_meq_l",
                (5.0, "ranson.0_2"),
                (6.0, "ranson.0_2"),
            )
            .cut(
                "gallstone sequestration 4 / 4.5 L (> 4)",
                ins![gallstone: true, age_years: 40, wbc_per_mm3: 8000, glucose_mg_dl: 100, ldh_iu_l: 200, ast_iu_l: 40],
                "fluid_sequestration_l",
                (4.0, "ranson.0_2"),
                (4.5, "ranson.0_2"),
            ),
        Table::new("bisap", &["bisap.severe", "bisap.mild"])
            .paper("Wu 2008: BUN 30 and age 65 = 2", ins![bun_mg_dl: 30, age_years: 65], "bisap.severe")
            .score(2.0, 0.0)
            .paper(
                "all five: 5",
                ins![bun_mg_dl: 30, age_years: 65, impaired_mental_status: true, sirs: true, pleural_effusion: true],
                "bisap.severe",
            )
            .score(5.0, 0.0)
            .cut("BUN 25 / 26 mg/dL with age 70 (> 25)", ins![age_years: 70], "bun_mg_dl", (25.0, "bisap.mild"), (26.0, "bisap.severe"))
            .cut("age 60 / 61 with SIRS (> 60)", ins![bun_mg_dl: 10, sirs: true], "age_years", (60.0, "bisap.mild"), (61.0, "bisap.severe")),
        Table::new("ctsi", &["ctsi.0_1", "ctsi.2_3", "ctsi.4_6", "ctsi.7_10"])
            .paper(
                "Balthazar 1990 / book p. 282: grade D (3) + necrosis < 30% (2) = 5, severe",
                ins![ct_grade: "d", necrosis: "lt30"],
                "ctsi.4_6",
            )
            .score(5.0, 0.0)
            .noted("repeat contrast CT")
            .paper("grade E with > 50% necrosis = 10", ins![ct_grade: "e", necrosis: "gt50"], "ctsi.7_10")
            .score(10.0, 0.0)
            .edge("grade B, no necrosis = 1", ins![ct_grade: "b", necrosis: "none"], "ctsi.0_1")
            .edge("grade C, no necrosis = 2", ins![ct_grade: "c", necrosis: "none"], "ctsi.2_3")
            .edge("grade D, no necrosis = 3", ins![ct_grade: "d", necrosis: "none"], "ctsi.2_3")
            .edge("grade E, no necrosis = 4", ins![ct_grade: "e", necrosis: "none"], "ctsi.4_6")
            .edge("grade C with 30–50% = 6", ins![ct_grade: "c", necrosis: "30_50"], "ctsi.4_6")
            .edge("grade D with 30–50% = 7", ins![ct_grade: "d", necrosis: "30_50"], "ctsi.7_10"),
        Table::new("atlanta_2012", &["atlanta_2012.severe", "atlanta_2012.moderately_severe", "atlanta_2012.mild"])
            .paper("Banks 2013: organ failure over 48 h is severe", ins![organ_failure: "persistent"], "atlanta_2012.severe")
            .paper("Banks 2013: complications without organ failure", ins![organ_failure: "none", complications: true], "atlanta_2012.moderately_severe")
            .edge("no organ failure, no complications: mild", ins![organ_failure: "none"], "atlanta_2012.mild")
            .edge("organ failure resolving within 48 h", ins![organ_failure: "transient"], "atlanta_2012.moderately_severe")
            .edge("transient organ failure", ins![organ_failure: "transient", complications: true], "atlanta_2012.moderately_severe")
            .edge("persistent organ failure", ins![organ_failure: "persistent", complications: true], "atlanta_2012.severe"),
        Table::new("glasgow_blatchford", &["glasgow_blatchford.very_low", "glasgow_blatchford.not_low"])
            .paper(
                "Blatchford 2000: man, BUN 30 (4), Hb 11 (3), SBP 95 (2), HR 105 (1), melena (1) = 11",
                ins![bun_mg_dl: 30, sex: "male", hb_g_dl: 11.0, sbp_mmhg: 95, hr_bpm: 105, melena: true],
                "glasgow_blatchford.not_low",
            )
            .score(11.0, 0.0)
            .paper("all normal: 0", ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80], "glasgow_blatchford.very_low")
            .score(0.0, 0.0)
            .cut(
                "BUN 18.1 / 18.2 mg/dL (urea 6.5 mmol/L) with melena: 1 / 3",
                ins![sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80, melena: true],
                "bun_mg_dl",
                (18.1, "glasgow_blatchford.very_low"),
                (18.2, "glasgow_blatchford.not_low"),
            )
            .cut("BUN 22.3 / 22.4 (urea 8): 2 / 3", ins![sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80], "bun_mg_dl", (22.3, "glasgow_blatchford.not_low"), (22.4, "glasgow_blatchford.not_low"))
            .cut("BUN 27.9 / 28 (urea 10): 3 / 4", ins![sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80], "bun_mg_dl", (27.9, "glasgow_blatchford.not_low"), (28.0, "glasgow_blatchford.not_low"))
            .cut("BUN 69.9 / 70 (urea 25): 4 / 6", ins![sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80], "bun_mg_dl", (69.9, "glasgow_blatchford.not_low"), (70.0, "glasgow_blatchford.not_low"))
            .cut("man Hb 12.9 / 13.0: 1 / 0", ins![bun_mg_dl: 10, sex: "male", sbp_mmhg: 120, hr_bpm: 80], "hb_g_dl", (12.9, "glasgow_blatchford.very_low"), (13.0, "glasgow_blatchford.very_low"))
            .cut("man Hb 11.9 / 12.0: 3 / 1", ins![bun_mg_dl: 10, sex: "male", sbp_mmhg: 120, hr_bpm: 80], "hb_g_dl", (11.9, "glasgow_blatchford.not_low"), (12.0, "glasgow_blatchford.very_low"))
            .cut("Hb 9.9 / 10.0 in a man: 6 / 3", ins![bun_mg_dl: 10, sex: "male", sbp_mmhg: 120, hr_bpm: 80], "hb_g_dl", (9.9, "glasgow_blatchford.not_low"), (10.0, "glasgow_blatchford.not_low"))
            .cut("woman Hb 11.9 / 12.0: 1 / 0", ins![bun_mg_dl: 10, sex: "female", sbp_mmhg: 120, hr_bpm: 80], "hb_g_dl", (11.9, "glasgow_blatchford.very_low"), (12.0, "glasgow_blatchford.very_low"))
            .cut("woman Hb 9.9 / 10.0: 6 / 1", ins![bun_mg_dl: 10, sex: "female", sbp_mmhg: 120, hr_bpm: 80], "hb_g_dl", (9.9, "glasgow_blatchford.not_low"), (10.0, "glasgow_blatchford.very_low"))
            .cut("SBP 109 / 110: 1 / 0", ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, hr_bpm: 80], "sbp_mmhg", (109.0, "glasgow_blatchford.very_low"), (110.0, "glasgow_blatchford.very_low"))
            .cut("SBP 99 / 100: 2 / 1", ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, hr_bpm: 80], "sbp_mmhg", (99.0, "glasgow_blatchford.not_low"), (100.0, "glasgow_blatchford.very_low"))
            .cut("SBP 89 / 90: 3 / 2", ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, hr_bpm: 80], "sbp_mmhg", (89.0, "glasgow_blatchford.not_low"), (90.0, "glasgow_blatchford.not_low"))
            .cut("HR 99 / 100: 0 / 1", ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120], "hr_bpm", (99.0, "glasgow_blatchford.very_low"), (100.0, "glasgow_blatchford.very_low"))
            .paper(
                "syncope, hepatic disease and cardiac failure are 2 each",
                ins![bun_mg_dl: 10, sex: "male", hb_g_dl: 14.0, sbp_mmhg: 120, hr_bpm: 80, syncope: true, hepatic_disease: true, cardiac_failure: true],
                "glasgow_blatchford.not_low",
            )
            .score(6.0, 0.0),
        Table::new("shock_index", &["shock_index.unstable", "shock_index.stable"])
            .paper("book p. 227: HR 120 / SBP 90 = 1.33, unstable", ins![hr_bpm: 120, sbp_mmhg: 90], "shock_index.unstable")
            .score(1.33, 1e-9)
            .cut("HR 100 / 101 over SBP 100 (above 1 is unstable)", ins![sbp_mmhg: 100], "hr_bpm", (100.0, "shock_index.stable"), (101.0, "shock_index.unstable")),
        Table::new("oakland", &["oakland.discharge", "oakland.admit"])
            .paper(
                "Oakland 2017: man 75 (2 + 1), previous LGIB (1), DRE blood (1), HR 95 (2), SBP 110 (4), Hb 100 g/L (13) = 24",
                ins![age_years: 75, sex: "male", previous_lgib: true, dre_blood: true, hr_bpm: 95, sbp_mmhg: 110, hb_g_dl: 10.0],
                "oakland.admit",
            )
            .score(24.0, 0.0)
            .noted("footnote")
            .cut(
                "HR 69 / 70 moves 8 to 9 (book table: 8 or less discharge)",
                ins![age_years: 45, sex: "male", previous_lgib: true, dre_blood: true, sbp_mmhg: 170, hb_g_dl: 14.0],
                "hr_bpm",
                (69.0, "oakland.discharge"),
                (70.0, "oakland.admit"),
            )
            .cut("age 39 / 40: 0 / 1", ins![sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 17.0], "age_years", (39.0, "oakland.discharge"), (40.0, "oakland.discharge"))
            .cut("age 69 / 70: 1 / 2", ins![sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 17.0], "age_years", (69.0, "oakland.discharge"), (70.0, "oakland.discharge"))
            .cut("HR 89 / 90: 1 / 2", ins![age_years: 30, sex: "female", sbp_mmhg: 170, hb_g_dl: 17.0], "hr_bpm", (89.0, "oakland.discharge"), (90.0, "oakland.discharge"))
            .cut("HR 109 / 110: 2 / 3", ins![age_years: 30, sex: "female", sbp_mmhg: 170, hb_g_dl: 17.0], "hr_bpm", (109.0, "oakland.discharge"), (110.0, "oakland.discharge"))
            .cut("SBP 89 / 90: 5 / 4", ins![age_years: 30, sex: "female", hr_bpm: 60, hb_g_dl: 17.0], "sbp_mmhg", (89.0, "oakland.discharge"), (90.0, "oakland.discharge"))
            .cut("SBP 119 / 120: 4 / 3", ins![age_years: 30, sex: "female", hr_bpm: 60, hb_g_dl: 17.0], "sbp_mmhg", (119.0, "oakland.discharge"), (120.0, "oakland.discharge"))
            .cut("SBP 129 / 130: 3 / 2", ins![age_years: 30, sex: "female", hr_bpm: 60, hb_g_dl: 17.0], "sbp_mmhg", (129.0, "oakland.discharge"), (130.0, "oakland.discharge"))
            .cut("SBP 159 / 160: 2 / 0", ins![age_years: 30, sex: "female", hr_bpm: 60, hb_g_dl: 17.0], "sbp_mmhg", (159.0, "oakland.discharge"), (160.0, "oakland.discharge"))
            .unit("Hb 6.9 g/dL = 69 g/L: 22", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 6.9], "oakland.admit")
            .score(22.0, 0.0)
            .unit("Hb 7.0 g/dL = 70 g/L: 17", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 7.0], "oakland.admit")
            .score(17.0, 0.0)
            .unit("Hb 8.9 g/dL = 89 g/L: 17", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 8.9], "oakland.admit")
            .score(17.0, 0.0)
            .unit("Hb 9.0 g/dL = 90 g/L: 13", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 9.0], "oakland.admit")
            .score(13.0, 0.0)
            .unit("Hb 10.9 g/dL = 109 g/L: 13", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 10.9], "oakland.admit")
            .score(13.0, 0.0)
            .unit("Hb 11.0 g/dL = 110 g/L: 8", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 11.0], "oakland.discharge")
            .score(8.0, 0.0)
            .unit("Hb 12.9 g/dL = 129 g/L: 8", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 12.9], "oakland.discharge")
            .score(8.0, 0.0)
            .unit("Hb 13.0 g/dL = 130 g/L: 4", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 13.0], "oakland.discharge")
            .score(4.0, 0.0)
            .unit("Hb 15.9 g/dL = 159 g/L: 4", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 15.9], "oakland.discharge")
            .score(4.0, 0.0)
            .unit("Hb 16.0 g/dL = 160 g/L: 0", ins![age_years: 30, sex: "female", hr_bpm: 60, sbp_mmhg: 170, hb_g_dl: 16.0], "oakland.discharge")
            .score(0.0, 0.0),
        Table::new("uc_activity", &["uc_activity.severe", "uc_activity.mild", "uc_activity.moderate"])
            .paper(
                "Truelove and Witts 1955: 8 stools, visible blood, pulse 95 and fever → severe",
                ins![stools_per_day: 8, blood: "visible", pulse_90: true, fever: true],
                "uc_activity.severe",
            )
            .paper("3 stools, small blood, well → mild", ins![stools_per_day: 3, blood: "small"], "uc_activity.mild")
            .paper(
                "7 stools with no systemic feature is not severe",
                ins![stools_per_day: 7, blood: "visible"],
                "uc_activity.moderate",
            )
            .noted("only with a systemic feature")
            .paper(
                "severe without visible blood says the book's column has it",
                ins![stools_per_day: 7, blood: "between", anaemia: true],
                "uc_activity.severe",
            )
            .noted("visible blood")
            .cut("stools 4 / 5 (mild ≤ 4)", ins![blood: "small"], "stools_per_day", (4.0, "uc_activity.mild"), (5.0, "uc_activity.moderate"))
            .cut("stools 5 / 6 with fever (severe ≥ 6)", ins![blood: "visible", fever: true], "stools_per_day", (5.0, "uc_activity.moderate"), (6.0, "uc_activity.severe"))
            .cut("ESR 30 / 31 with 6 stools (> 30)", ins![stools_per_day: 6, blood: "visible"], "esr_mm_h", (30.0, "uc_activity.moderate"), (31.0, "uc_activity.severe")),
        Table::new(
            "kyoto_gastritis",
            &["kyoto_gastritis.cancer_risk", "kyoto_gastritis.current", "kyoto_gastritis.indeterminate", "kyoto_gastritis.none"],
        )
            .paper(
                "Toyoshima 2020: atrophy O-type (2), IM in the antrum (1), mild redness (1) = 4",
                ins![atrophy: "severe", intestinal_metaplasia: "mild", diffuse_redness: "mild"],
                "kyoto_gastritis.cancer_risk",
            )
            .score(4.0, 0.0)
            .paper(
                "maximum: 2 + 2 + 1 + 1 + 2 = 8",
                ins![atrophy: "severe", intestinal_metaplasia: "severe", enlarged_folds: true, nodularity: true, diffuse_redness: "severe"],
                "kyoto_gastritis.cancer_risk",
            )
            .score(8.0, 0.0)
            .edge("0", ins![atrophy: "none", intestinal_metaplasia: "none", diffuse_redness: "none"], "kyoto_gastritis.none")
            .edge("1", ins![atrophy: "none", intestinal_metaplasia: "none", diffuse_redness: "none", nodularity: true], "kyoto_gastritis.indeterminate")
            .edge("1", ins![atrophy: "mild", intestinal_metaplasia: "none", diffuse_redness: "none"], "kyoto_gastritis.indeterminate")
            .edge("2", ins![atrophy: "mild", intestinal_metaplasia: "none", diffuse_redness: "mild"], "kyoto_gastritis.current")
            .edge("3", ins![atrophy: "mild", intestinal_metaplasia: "mild", diffuse_redness: "mild"], "kyoto_gastritis.current")
            .edge("4", ins![atrophy: "mild", intestinal_metaplasia: "mild", diffuse_redness: "mild", enlarged_folds: true], "kyoto_gastritis.cancer_risk"),
        Table::new("rome4_ibs", &["rome4_ibs.not_met", "rome4_ibs.c", "rome4_ibs.d", "rome4_ibs.m", "rome4_ibs.u"])
            .paper(
                "Lacy 2016: weekly pain for 3 months, onset 6 months ago, with defecation and form change, hard stools → IBS-C",
                ins![pain_weekly: true, onset_6_months: true, defecation: true, form: true, hard_25: true],
                "rome4_ibs.c",
            )
            .noted("off drugs")
            .paper("loose stools → IBS-D", ins![pain_weekly: true, onset_6_months: true, defecation: true, frequency: true, loose_25: true], "rome4_ibs.d")
            .paper("both → IBS-M", ins![pain_weekly: true, onset_6_months: true, frequency: true, form: true, hard_25: true, loose_25: true], "rome4_ibs.m")
            .edge("one feature only", ins![pain_weekly: true, onset_6_months: true, defecation: true], "rome4_ibs.not_met")
            .edge("two features, neither stool type → IBS-U", ins![pain_weekly: true, onset_6_months: true, defecation: true, form: true], "rome4_ibs.u")
            .edge("onset under 6 months", ins![pain_weekly: true, defecation: true, form: true, frequency: true], "rome4_ibs.not_met")
            .edge("onset 6 months", ins![pain_weekly: true, onset_6_months: true, defecation: true, form: true, frequency: true], "rome4_ibs.u")
            .edge("pain less than weekly", ins![onset_6_months: true, defecation: true, form: true], "rome4_ibs.not_met")
            .edge("weekly pain", ins![pain_weekly: true, onset_6_months: true, defecation: true, form: true], "rome4_ibs.u"),
    ]
}
