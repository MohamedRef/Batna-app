//! Haematology calculators.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("ipss_mf", &["ipss_mf.low", "ipss_mf.int1", "ipss_mf.int2", "ipss_mf.high"])
            .paper(
                "Cervantes 2009: age 70, Hb 9 g/dL and night sweats = 3, high risk",
                ins![age_years: 70, wbc_10e9_l: 12.0, hb_g_dl: 9.0, blasts_pct: 0.0, constitutional: true],
                "ipss_mf.high",
            )
            .score(3.0, 0.0)
            .noted("1% or more")
            .cut("age 65 / 66 (> 65)", ins![wbc_10e9_l: 8.0, hb_g_dl: 13.0, blasts_pct: 0.0], "age_years", (65.0, "ipss_mf.low"), (66.0, "ipss_mf.int1"))
            .cut("WBC 25 / 25.1 (> 25)", ins![age_years: 70, hb_g_dl: 13.0, blasts_pct: 0.0], "wbc_10e9_l", (25.0, "ipss_mf.int1"), (25.1, "ipss_mf.int2"))
            .cut("Hb 10.0 / 9.9 (< 10)", ins![age_years: 50, wbc_10e9_l: 8.0, blasts_pct: 0.0], "hb_g_dl", (10.0, "ipss_mf.low"), (9.9, "ipss_mf.int1"))
            .cut(
                "blasts 0.9 / 1.0% (Cervantes: ≥ 1)",
                ins![age_years: 70, wbc_10e9_l: 30.0, hb_g_dl: 13.0],
                "blasts_pct",
                (0.9, "ipss_mf.int2"),
                (1.0, "ipss_mf.high"),
            ),
        Table::new("rpi", &["rpi.low", "rpi.adequate"])
            .paper(
                "Hillman & Finch: retics 6%, Hct 30 → 6 × 30/45 = 4.0, ÷ 2 days = 2.0, adequate",
                ins![retic_pct: 6.0, hct_pct: 30.0],
                "rpi.adequate",
            )
            .score(2.0, 1e-9)
            .noted("Hillman and Finch")
            .cut("RPI 1.9 / 2.0 at Hct 45 (book p. 14: RI < 2)", ins![hct_pct: 45.0], "retic_pct", (1.9, "rpi.low"), (2.0, "rpi.adequate"))
            .cut("Hct 39.9 / 40: maturation 1.5 → 1 day", ins![retic_pct: 4.0], "hct_pct", (39.9, "rpi.adequate"), (40.0, "rpi.adequate"))
            .cut("Hct 34.9 / 35: 2 → 1.5 days", ins![retic_pct: 4.0], "hct_pct", (34.9, "rpi.low"), (35.0, "rpi.adequate"))
            .cut("Hct 24.9 / 25: 2.5 → 2 days", ins![retic_pct: 10.0], "hct_pct", (24.9, "rpi.adequate"), (25.0, "rpi.adequate"))
            .cut("Hct 14.9 / 15: 3 → 2.5 days", ins![retic_pct: 20.0], "hct_pct", (14.9, "rpi.adequate"), (15.0, "rpi.adequate"))
            .unit("absolute count: 2% × 4.0 × 10¹²/L = 80 × 10⁹/L", ins![retic_pct: 2.0, hct_pct: 45.0, rbc_10e12_l: 4.0], "rpi.adequate")
            .noted("80 × 10⁹/L"),
        Table::new("rai_cll", &["rai_cll.0", "rai_cll.1", "rai_cll.2", "rai_cll.3", "rai_cll.4"])
            .paper("Rai 1975: lymphocytosis with nodes → stage I", ins![lymphadenopathy: true, hb_g_dl: 13.0, platelets_10e9_l: 200], "rai_cll.1")
            .noted("modified Rai")
            .paper("splenomegaly → stage II", ins![lymphadenopathy: true, organomegaly: true, hb_g_dl: 13.0, platelets_10e9_l: 200], "rai_cll.2")
            .cut("Hb 11.0 / 10.9 (< 11 is III)", ins![platelets_10e9_l: 200], "hb_g_dl", (11.0, "rai_cll.0"), (10.9, "rai_cll.3"))
            .cut("platelets 100 / 99 (< 100 is IV)", ins![hb_g_dl: 13.0], "platelets_10e9_l", (100.0, "rai_cll.0"), (99.0, "rai_cll.4")),
        Table::new("binet_cll", &["binet_cll.a", "binet_cll.b", "binet_cll.c"])
            .paper("Binet 1981: 2 areas, Hb 13, platelets 200 → A", ins![areas: 2, hb_g_dl: 13.0, platelets_10e9_l: 200], "binet_cll.a")
            .noted("exactly 100")
            .cut("2 / 3 areas", ins![hb_g_dl: 13.0, platelets_10e9_l: 200], "areas", (2.0, "binet_cll.a"), (3.0, "binet_cll.b"))
            .cut("Hb 10.0 / 9.9 (< 10 is C)", ins![areas: 1, platelets_10e9_l: 200], "hb_g_dl", (10.0, "binet_cll.a"), (9.9, "binet_cll.c"))
            .cut("platelets 100 / 99 (Binet 1981: < 100 is C)", ins![areas: 4, hb_g_dl: 13.0], "platelets_10e9_l", (100.0, "binet_cll.b"), (99.0, "binet_cll.c")),
        Table::new(
            "ann_arbor",
            &["ann_arbor.1a", "ann_arbor.1b", "ann_arbor.2a", "ann_arbor.2b", "ann_arbor.3a", "ann_arbor.3b", "ann_arbor.4a", "ann_arbor.4b"],
        )
            .paper("Carbone 1971: nodes both sides of the diaphragm with night sweats → IIIB", ins![spread: "both_sides", night_sweats: true], "ann_arbor.3b")
            .score(3.0, 0.0)
            .noted("E suffix")
            .paper("single group, no symptoms → IA", ins![spread: "single_group"], "ann_arbor.1a")
            .paper("single group with fever → IB", ins![spread: "single_group", fever: true], "ann_arbor.1b")
            .paper("one side → IIA", ins![spread: "one_side"], "ann_arbor.2a")
            .paper("both sides → IIIA", ins![spread: "both_sides"], "ann_arbor.3a")
            .paper("marrow → IVA", ins![spread: "extralymphatic"], "ann_arbor.4a")
            .paper("marrow with weight loss → IVB", ins![spread: "extralymphatic", weight_loss: true], "ann_arbor.4b")
            .edge("one side, no B symptom", ins![spread: "one_side"], "ann_arbor.2a")
            .edge("one side with weight loss > 10%", ins![spread: "one_side", weight_loss: true], "ann_arbor.2b"),
        Table::new("ipi_nhl", &["ipi_nhl.low", "ipi_nhl.low_int", "ipi_nhl.high_int", "ipi_nhl.high"])
            .paper("Shipp 1993: age 65, raised LDH, stage III = 3", ins![age_years: 65, ldh_high: true, stage_3_4: true], "ipi_nhl.high_int")
            .score(3.0, 0.0)
            .noted("Shipp 1993")
            .cut("age 60 / 61 with LDH (> 60)", ins![ldh_high: true], "age_years", (60.0, "ipi_nhl.low"), (61.0, "ipi_nhl.low_int"))
            .edge("3", ins![age_years: 50, ldh_high: true, stage_3_4: true, ecog_2_plus: true], "ipi_nhl.high_int")
            .edge("4", ins![age_years: 50, ldh_high: true, stage_3_4: true, ecog_2_plus: true, extranodal_2_plus: true], "ipi_nhl.high"),
        Table::new("who_pv", &["who_pv.met", "who_pv.not_met"])
            .paper("Arber 2016: man, Hb 17, panmyelosis, JAK2 V617F", ins![sex: "male", hb_g_dl: 17.0, bm_panmyelosis: true, jak2: true], "who_pv.met")
            .noted("all 3 major")
            .paper("first 2 majors with low EPO instead of JAK2", ins![sex: "female", hb_g_dl: 16.5, bm_panmyelosis: true, low_epo: true], "who_pv.met")
            .paper("no JAK2 and normal EPO", ins![sex: "male", hb_g_dl: 18.0, bm_panmyelosis: true], "who_pv.not_met")
            .cut("man Hb 16.5 / 16.6 (> 16.5)", ins![sex: "male", bm_panmyelosis: true, jak2: true], "hb_g_dl", (16.5, "who_pv.not_met"), (16.6, "who_pv.met"))
            .cut("woman Hb 16.0 / 16.1 (> 16.0)", ins![sex: "female", bm_panmyelosis: true, jak2: true], "hb_g_dl", (16.0, "who_pv.not_met"), (16.1, "who_pv.met"))
            .cut("man Hct 49 / 49.1 (> 49%)", ins![sex: "male", hb_g_dl: 15.0, bm_panmyelosis: true, jak2: true], "hct_pct", (49.0, "who_pv.not_met"), (49.1, "who_pv.met"))
            .cut("woman Hct 48 / 48.1 (> 48%)", ins![sex: "female", hb_g_dl: 15.0, bm_panmyelosis: true, jak2: true], "hct_pct", (48.0, "who_pv.not_met"), (48.1, "who_pv.met"))
            .edge("raised red cell mass alone counts for major 1: without it", ins![sex: "male", hb_g_dl: 15.0, bm_panmyelosis: true, jak2: true], "who_pv.not_met")
            .edge("with it", ins![sex: "male", hb_g_dl: 15.0, rcm_increased: true, bm_panmyelosis: true, jak2: true], "who_pv.met"),
        Table::new("who_et", &["who_et.met", "who_et.not_met"])
            .paper(
                "Arber 2016: platelets 700, megakaryocytic marrow, not another MPN, CALR",
                ins![platelets_10e9_l: 700, bm_megakaryocytes: true, not_other_myeloid: true, driver_mutation: true],
                "who_et.met",
            )
            .noted("all 4 major")
            .paper(
                "triple-negative with no reactive cause: first 3 majors + the minor",
                ins![platelets_10e9_l: 700, bm_megakaryocytes: true, not_other_myeloid: true, clonal_or_not_reactive: true],
                "who_et.met",
            )
            .paper("without the marrow major", ins![platelets_10e9_l: 700, not_other_myeloid: true, driver_mutation: true], "who_et.not_met")
            .cut(
                "platelets 449 / 450 (≥ 450)",
                ins![bm_megakaryocytes: true, not_other_myeloid: true, driver_mutation: true],
                "platelets_10e9_l",
                (449.0, "who_et.not_met"),
                (450.0, "who_et.met"),
            ),
        Table::new("who_pmf", &["who_pmf.met", "who_pmf.not_met"])
            .paper(
                "Arber 2016: grade 2 fibrosis, not another neoplasm, JAK2, with splenomegaly",
                ins![bm_megakaryocytes_fibrosis: true, not_other_myeloid: true, clonal: true, splenomegaly: true],
                "who_pmf.met",
            )
            .noted("two consecutive")
            .edge("3 majors, no minor", ins![bm_megakaryocytes_fibrosis: true, not_other_myeloid: true, clonal: true], "who_pmf.not_met")
            .edge("3 majors + LDH", ins![bm_megakaryocytes_fibrosis: true, not_other_myeloid: true, clonal: true, ldh_high: true], "who_pmf.met")
            .edge("2 majors + all minors", ins![bm_megakaryocytes_fibrosis: true, not_other_myeloid: true, anaemia: true, leukocytosis: true, splenomegaly: true, ldh_high: true, leukoerythroblastosis: true], "who_pmf.not_met")
            .edge("3 majors + anaemia", ins![bm_megakaryocytes_fibrosis: true, not_other_myeloid: true, clonal: true, anaemia: true], "who_pmf.met"),
    ]
}
