//! Predictive values and post-test probability (Research L14). The paper
//! cases are the slides' own 2×2 table (slides 14 and 17: a 80, b 100,
//! c 20, d 800), run through the slide 25 formula and through likelihood
//! ratios (Deeks and Altman, BMJ 2004); both must give the table's PPV
//! 44.44% and NPV 97.56%. The LR cut-offs are Jaeschke et al., JAMA 1994.

use super::{Table, ins};

/// Specificity 800/900 as a percentage, as typed from the slide table.
const SP: f64 = 88.888_888_888_9;

pub fn tables() -> Vec<Table> {
    let pt = |lr: f64| ins![pretest: 10.0, lr: lr];
    vec![
        Table::new("predictive_values", &["predictive_values.ppv", "predictive_values.npv"])
            .paper(
                "Slides 14, 17, 25: Se 80%, Sp 800/900, prevalence 100/1000 → PPV 80/180 = 44.44%",
                ins![sensitivity: 80.0, specificity: SP, prevalence: 10.0, measure: "ppv"],
                "predictive_values.ppv",
            )
            .score(44.44, 1e-9)
            .noted("80 true +, 100 false +, 20 false −, 800 true −")
            .paper(
                "Same table: NPV 800/820 = 97.56%",
                ins![sensitivity: 80.0, specificity: SP, prevalence: 10.0, measure: "npv"],
                "predictive_values.npv",
            )
            .score(97.56, 1e-9)
            .edge(
                "Rarer disease, prevalence 1%: PPV 0.008 / (0.008 + 0.11) = 6.78%",
                ins![sensitivity: 80.0, specificity: SP, prevalence: 1.0, measure: "ppv"],
                "predictive_values.ppv",
            )
            .score(6.78, 1e-9)
            .edge(
                "Commoner disease, prevalence 30%: NPV 0.6222 / (0.6222 + 0.06) = 91.21%",
                ins![sensitivity: 80.0, specificity: SP, prevalence: 30.0, measure: "npv"],
                "predictive_values.npv",
            )
            .score(91.21, 1e-9),
        Table::new(
            "post_test_probability",
            &[
                "post_test_probability.large_rise",
                "post_test_probability.moderate_rise",
                "post_test_probability.small_rise",
                "post_test_probability.little_change",
                "post_test_probability.small_fall",
                "post_test_probability.moderate_fall",
                "post_test_probability.large_fall",
            ],
        )
        .paper(
            "Slide table, positive result: pre-test 10%, LR+ 0.8 / (1 − 0.8889) = 7.2 → odds 0.111 × 7.2 = 0.8 → 44.44% (= PPV)",
            pt(7.2),
            "post_test_probability.moderate_rise",
        )
        .score(44.44, 1e-9)
        .noted("Post-test odds = 0.111 × 7.2 = 0.8")
        .paper(
            "Slide table, negative result: LR− 0.2 / 0.8889 = 0.225 → 2.44% (= 1 − NPV 97.56%)",
            pt(0.225),
            "post_test_probability.small_fall",
        )
        .score(2.44, 1e-9)
        .cut(
            "Jaeschke: above 10 is large",
            pt(10.0),
            "lr",
            (10.0, "post_test_probability.moderate_rise"),
            (10.01, "post_test_probability.large_rise"),
        )
        .cut(
            "Above 5 is moderate",
            pt(5.0),
            "lr",
            (5.0, "post_test_probability.small_rise"),
            (5.01, "post_test_probability.moderate_rise"),
        )
        .cut(
            "Above 2 is small",
            pt(2.0),
            "lr",
            (2.0, "post_test_probability.little_change"),
            (2.01, "post_test_probability.small_rise"),
        )
        .cut(
            "0.5 and up changes little",
            pt(0.5),
            "lr",
            (0.49, "post_test_probability.small_fall"),
            (0.5, "post_test_probability.little_change"),
        )
        .cut(
            "0.2 and up is a small fall",
            pt(0.2),
            "lr",
            (0.19, "post_test_probability.moderate_fall"),
            (0.2, "post_test_probability.small_fall"),
        )
        .cut(
            "Below 0.1 is large",
            pt(0.1),
            "lr",
            (0.09, "post_test_probability.large_fall"),
            (0.1, "post_test_probability.moderate_fall"),
        )
        .paper("LR 1 leaves the probability where it was (Fagan 1975)", pt(1.0), "post_test_probability.little_change")
        .score(10.0, 1e-9),
    ]
}
