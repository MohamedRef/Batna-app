//! Rheumatology classification criteria.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("acr_eular_2010_ra", &["acr_eular_2010_ra.definite", "acr_eular_2010_ra.not_met"])
            .paper(
                "Aletaha 2010: 3 small joints (2), high-positive ACPA (3), raised CRP (1) = 6, definite RA",
                ins![large_joints: 0, small_joints: 3, serology: "high", abnormal_crp_esr: true],
                "acr_eular_2010_ra.definite",
            )
            .score(6.0, 0.0)
            .paper(
                "each domain scores its highest level only: 5 small + 4 large = 3, not 3 + 1",
                ins![large_joints: 4, small_joints: 5, serology: "negative"],
                "acr_eular_2010_ra.not_met",
            )
            .score(3.0, 0.0)
            .cut(
                "small joints 3 / 4 (1–3 = 2, 4–10 = 3)",
                ins![large_joints: 0, serology: "low", synovitis_6_weeks: true],
                "small_joints",
                (3.0, "acr_eular_2010_ra.not_met"),
                (4.0, "acr_eular_2010_ra.definite"),
            )
            .cut(
                "small joints 10 / 11 (> 10 = 5)",
                ins![large_joints: 0, serology: "negative", synovitis_6_weeks: true, abnormal_crp_esr: true],
                "small_joints",
                (10.0, "acr_eular_2010_ra.not_met"),
                (11.0, "acr_eular_2010_ra.definite"),
            )
            .cut("large joints 1 / 2 (2–10 = 1)", ins![small_joints: 0, serology: "high", synovitis_6_weeks: true, abnormal_crp_esr: true], "large_joints", (1.0, "acr_eular_2010_ra.not_met"), (2.0, "acr_eular_2010_ra.definite"))
            .edge("negative serology: 2 + 0 + 1 + 1 = 4", ins![large_joints: 0, small_joints: 1, serology: "negative", synovitis_6_weeks: true, abnormal_crp_esr: true], "acr_eular_2010_ra.not_met")
            .edge("low-positive serology: 2 + 2 + 1 + 1 = 6", ins![large_joints: 0, small_joints: 1, serology: "low", synovitis_6_weeks: true, abnormal_crp_esr: true], "acr_eular_2010_ra.definite")
            .paper("no swollen joint: the criteria do not apply", ins![large_joints: 0, small_joints: 0, serology: "high"], "acr_eular_2010_ra.not_met")
            .noted("at least one swollen joint")
            .paper("over 10 large joints only still scores 1", ins![large_joints: 12, small_joints: 0, serology: "negative"], "acr_eular_2010_ra.not_met")
            .score(1.0, 0.0)
            .noted("no level above 2–10"),
        Table::new("caspar", &["caspar.met", "caspar.not_met"])
            .paper(
                "Taylor 2006: inflammatory arthritis, psoriasis now (2) and RF negative (1) = 3",
                ins![inflammatory_disease: true, psoriasis: "current", rf_negative: true],
                "caspar.met",
            )
            .score(3.0, 0.0)
            .noted("Specificity 98.7%")
            .edge("2: past psoriasis (1) + nails (1)", ins![inflammatory_disease: true, psoriasis: "past", nail_lesions: true], "caspar.not_met")
            .edge("3: + dactylitis", ins![inflammatory_disease: true, psoriasis: "past", nail_lesions: true, dactylitis: true], "caspar.met")
            .edge("family history scores 1, total 2", ins![inflammatory_disease: true, psoriasis: "family", juxta_articular_bone: true], "caspar.not_met")
            .edge("family history scores 1, total 3", ins![inflammatory_disease: true, psoriasis: "family", juxta_articular_bone: true, rf_negative: true], "caspar.met")
            .paper(
                "without inflammatory articular disease CASPAR does not apply",
                ins![psoriasis: "current", nail_lesions: true, dactylitis: true, rf_negative: true],
                "caspar.not_met",
            )
            .noted("applies only"),
        Table::new("asas_axial_spa", &["asas_axial_spa.imaging", "asas_axial_spa.clinical", "asas_axial_spa.not_met"])
            .paper(
                "Rudwaleit 2009: chronic back pain from age 30, MRI sacroiliitis and uveitis → imaging arm",
                ins![back_pain_3_months_onset_45: true, sacroiliitis_imaging: true, uveitis: true],
                "asas_axial_spa.imaging",
            )
            .paper(
                "HLA-B27 with inflammatory back pain and heel enthesitis → clinical arm",
                ins![back_pain_3_months_onset_45: true, hla_b27: true, ibp: true, enthesitis: true],
                "asas_axial_spa.clinical",
            )
            .edge("HLA-B27 + 1 feature", ins![back_pain_3_months_onset_45: true, hla_b27: true, ibp: true], "asas_axial_spa.not_met")
            .edge("HLA-B27 + 2 features", ins![back_pain_3_months_onset_45: true, hla_b27: true, ibp: true, elevated_crp: true], "asas_axial_spa.clinical")
            .edge("sacroiliitis alone", ins![back_pain_3_months_onset_45: true, sacroiliitis_imaging: true], "asas_axial_spa.not_met")
            .edge("sacroiliitis + HLA-B27 (a feature in the imaging arm)", ins![back_pain_3_months_onset_45: true, sacroiliitis_imaging: true, hla_b27: true], "asas_axial_spa.imaging")
            .paper(
                "both arms met says so",
                ins![back_pain_3_months_onset_45: true, sacroiliitis_imaging: true, hla_b27: true, ibp: true, psoriasis: true],
                "asas_axial_spa.imaging",
            )
            .noted("Both arms")
            .paper(
                "no entry line (pain under 3 months or onset after 45)",
                ins![sacroiliitis_imaging: true, hla_b27: true, ibp: true, psoriasis: true],
                "asas_axial_spa.not_met",
            )
            .noted("started before age 45"),
        Table::new("asas_peripheral_spa", &["asas_peripheral_spa.met", "asas_peripheral_spa.not_met"])
            .paper("Rudwaleit 2011: arthritis with psoriasis", ins![arthritis: true, psoriasis: true], "asas_peripheral_spa.met")
            .noted("axial criteria")
            .edge("arthritis + enthesitis: the entry feature does not count again, 1 of list B", ins![arthritis: true, enthesitis: true], "asas_peripheral_spa.not_met")
            .edge("arthritis + enthesitis + family history: 2 of list B", ins![arthritis: true, enthesitis: true, family_history: true], "asas_peripheral_spa.met")
            .edge("arthritis + past back pain: 1 of list B", ins![arthritis: true, ibp_past: true], "asas_peripheral_spa.not_met")
            .edge("dactylitis + HLA-B27 (list A)", ins![dactylitis: true, hla_b27: true], "asas_peripheral_spa.met")
            .paper("no peripheral feature: no entry", ins![psoriasis: true, hla_b27: true], "asas_peripheral_spa.not_met")
            .noted("Entry needs"),
        Table::new("modified_new_york", &["modified_new_york.met", "modified_new_york.not_met"])
            .paper(
                "van der Linden 1984: inflammatory back pain with grade 2 sacroiliitis on both sides",
                ins![inflammatory_back_pain: true, si_grade_right: 2, si_grade_left: 2],
                "modified_new_york.met",
            )
            .cut("bilateral grade 1 / 2", ins![inflammatory_back_pain: true, si_grade_left: 2], "si_grade_right", (1.0, "modified_new_york.not_met"), (2.0, "modified_new_york.met"))
            .cut("unilateral grade 2 / 3", ins![chest_expansion: true, si_grade_left: 0], "si_grade_right", (2.0, "modified_new_york.not_met"), (3.0, "modified_new_york.met"))
            .paper("radiological criterion without a clinical one", ins![si_grade_right: 4, si_grade_left: 4], "modified_new_york.not_met")
            .paper("grade 1 on both sides notes the book's misprint", ins![lumbar_limitation: true, si_grade_right: 1, si_grade_left: 1], "modified_new_york.not_met")
            .noted("grade 1–2 bilaterally"),
        Table::new("sle_acr_1997", &["sle_acr_1997.met", "sle_acr_1997.not_met"])
            .paper(
                "Hochberg 1997: malar rash, arthritis, renal disorder and ANA = 4 of 11",
                ins![malar_rash: true, arthritis: true, renal: true, ana: true],
                "sle_acr_1997.met",
            )
            .score(4.0, 0.0)
            .edge("3 of 11", ins![oral_ulcers: true, haematological: true, ana: true], "sle_acr_1997.not_met")
            .edge("4 of 11 (book's \">4\" read as 4 or more)", ins![oral_ulcers: true, haematological: true, ana: true, immunological: true], "sle_acr_1997.met"),
        Table::new("sle_eular_acr_2019", &["sle_eular_acr_2019.met", "sle_eular_acr_2019.not_met"])
            .paper(
                "Aringer 2019: ANA 1:80, arthritis (6) and anti-dsDNA (6) = 12",
                ins![ana_1_80: true, arthritis: true, specific_antibody: true],
                "sle_eular_acr_2019.met",
            )
            .score(12.0, 0.0)
            .noted("nothing more likely")
            .edge("9: arthritis + low C3 or C4", ins![ana_1_80: true, arthritis: true, complement: "c3_or_c4"], "sle_eular_acr_2019.not_met")
            .edge("10: arthritis + low C3 and C4", ins![ana_1_80: true, arthritis: true, complement: "c3_and_c4"], "sle_eular_acr_2019.met")
            .paper(
                "class III/IV nephritis alone is 10 (the heaviest item)",
                ins![ana_1_80: true, renal: "class_3_4"],
                "sle_eular_acr_2019.met",
            )
            .score(10.0, 0.0)
            .paper(
                "immunological points only: needs a clinical domain",
                ins![ana_1_80: true, specific_antibody: true, complement: "c3_and_c4", antiphospholipid: true],
                "sle_eular_acr_2019.not_met",
            )
            .score(12.0, 0.0)
            .noted("at least one clinical domain")
            .paper(
                "without ANA 1:80 there is no entry",
                ins![arthritis: true, specific_antibody: true, renal: "class_3_4"],
                "sle_eular_acr_2019.not_met",
            )
            .noted("cannot be classified")
            .paper(
                "each domain choice carries its paper points: fever 2, alopecia 2, seizure 5, effusion 5, thrombocytopenia 4",
                ins![ana_1_80: true, fever: true, cutaneous: "alopecia", neurologic: "seizure", serosal: "effusion", haematologic: "thrombocytopenia"],
                "sle_eular_acr_2019.met",
            )
            .score(18.0, 0.0),
        Table::new("gpa_2022", &["gpa_2022.met", "gpa_2022.not_met"])
            .paper("Robson 2022: nasal crusting (3) and anti-PR3 (5) = 8", ins![nasal: true, c_anca_pr3: true], "gpa_2022.met")
            .score(8.0, 0.0)
            .noted("mimics are excluded")
            .edge("4: anti-PR3 (5) with anti-MPO (−1)", ins![c_anca_pr3: true, p_anca_mpo: true], "gpa_2022.not_met")
            .edge("5: + hearing loss (1)", ins![c_anca_pr3: true, p_anca_mpo: true, hearing_loss: true], "gpa_2022.met")
            .paper(
                "eosinophils ≥ 1 × 10⁹/L take 4 off: 3 + 2 + 2 − 4 = 3",
                ins![nasal: true, cartilage: true, lung_nodules: true, eosinophils: true],
                "gpa_2022.not_met",
            )
            .score(3.0, 0.0)
            .paper(
                "granuloma (2), sinus imaging (1), pauci-immune GN (1) and nodules (2) = 6",
                ins![granuloma: true, sinus_imaging: true, pauci_immune_gn: true, lung_nodules: true],
                "gpa_2022.met",
            )
            .score(6.0, 0.0),
        Table::new("mpa_2022", &["mpa_2022.met", "mpa_2022.not_met"])
            .paper("Suppiah 2022: anti-MPO (6)", ins![p_anca_mpo: true], "mpa_2022.met")
            .score(6.0, 0.0)
            .edge("3: anti-MPO (6) with nasal disease (−3)", ins![p_anca_mpo: true, nasal: true], "mpa_2022.not_met")
            .edge("5: anti-MPO (6) with anti-PR3 (−1)", ins![p_anca_mpo: true, c_anca_pr3: true], "mpa_2022.met")
            .paper("lung fibrosis (3) + pauci-immune GN (3) = 6", ins![lung_fibrosis: true, pauci_immune_gn: true], "mpa_2022.met")
            .score(6.0, 0.0)
            .paper("eosinophils take 4 off: 6 − 4 = 2", ins![p_anca_mpo: true, eosinophils: true], "mpa_2022.not_met")
            .score(2.0, 0.0),
    ]
}
