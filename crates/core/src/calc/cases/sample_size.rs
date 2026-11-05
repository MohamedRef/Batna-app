//! Sample size (Research L11). The paper cases are the slides' own Epi Info
//! StatCalc screens (slides 75, 81 and 85), which use the formulas cited in
//! `calc/sample_size.rs`.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("sample_size_survey", &["sample_size_survey.n"])
            .paper(
                "Slide 75: N 100,000, expected 30%, margin 5%, 95% → 322",
                ins![population: 100000, expected_prevalence: 30.0, margin: 5.0, confidence: "95", design_effect: 1.0],
                "sample_size_survey.n",
            )
            .score(322.0, 0.0)
            .noted("1.960")
            .edge(
                "Slide 76: margin 3% → 888 (888.4 rounds down, as StatCalc)",
                ins![population: 100000, expected_prevalence: 30.0, margin: 3.0, confidence: "95"],
                "sample_size_survey.n",
            )
            .score(888.0, 0.0)
            .edge(
                "Design effect 2 doubles it: 2 × 888.4 = 1,777",
                ins![population: 100000, expected_prevalence: 30.0, margin: 3.0, confidence: "95", design_effect: 2.0],
                "sample_size_survey.n",
            )
            .score(1777.0, 0.0),
        Table::new(
            "sample_size_two_proportions",
            &["sample_size_two_proportions.case_control", "sample_size_two_proportions.cohort"],
        )
        .paper(
            "Slide 81: controls 20% exposed, cases 30%, 95%, 80% power → Kelsey 295 + 295",
            ins![design: "case_control", confidence: "95", power: 80.0, ratio: 1.0, p_group2: 20.0, p_group1: 30.0],
            "sample_size_two_proportions.case_control",
        )
        .score(295.0, 0.0)
        .noted("Fleiss: 294 cases + 294 controls = 588")
        .paper(
            "Slide 85: unexposed 25%, exposed 38% → Kelsey 201 + 201",
            ins![design: "cohort", confidence: "95", power: 80.0, ratio: 1.0, p_group2: 25.0, p_group1: 38.0],
            "sample_size_two_proportions.cohort",
        )
        .score(201.0, 0.0)
        .noted("215 exposed + 215 unexposed = 430")
        .edge(
            "Slide 87: CMV 4.9% vs 18% → 93",
            ins![design: "cohort", confidence: "95", power: 80.0, p_group2: 4.9, p_group1: 18.0],
            "sample_size_two_proportions.cohort",
        )
        .score(93.0, 0.0)
        .edge(
            "Power 90% needs more: 125",
            ins![design: "cohort", confidence: "95", power: 90.0, p_group2: 4.9, p_group1: 18.0],
            "sample_size_two_proportions.cohort",
        )
        .score(125.0, 0.0),
    ]
}
