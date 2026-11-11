//! Tests of significance (Research L17). The paper cases are the slides'
//! own worked examples: the coffee and hypertension 2×2 table (slides
//! 24–26) and the haemoglobin in pregnancy one-sample t-test (slides
//! 30–33), with the formulas cited in `calc/significance.rs`.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("chi_square", &["chi_square.significant", "chi_square.not_significant"])
            .paper(
                "Slides 24–26: >8 cups 30 HTN / 20 normal, no coffee 10 / 35 → X² 13.84 on the slide (13.87 unrounded) > 3.84",
                ins![a: 30, b: 20, c: 10, d: 35, alpha: "0.05"],
                "chi_square.significant",
            )
            .score(13.87, 0.005)
            .noted("critical X² at 5% = 3.84")
            .cut(
                "a 31 (31/20 vs 21/29) gives X² 3.57 (< 3.84); a 32 gives 3.90 (≥ 3.84)",
                ins![b: 20, c: 21, d: 29, alpha: "0.05"],
                "a",
                (31.0, "chi_square.not_significant"),
                (32.0, "chi_square.significant"),
            )
            .edge(
                "X² 4.00 is significant at 5% but not at 1% (critical 6.63)",
                ins![a: 30, b: 20, c: 20, d: 30, alpha: "0.01"],
                "chi_square.not_significant",
            )
            .edge(
                "The same table at 5%",
                ins![a: 30, b: 20, c: 20, d: 30, alpha: "0.05"],
                "chi_square.significant",
            ),
        Table::new("t_test", &["t_test.significant", "t_test.not_significant"])
            .paper(
                "Slides 30–33: Hb x̄ 86%, μ 89%, S 7%, n 25 → |t| 2.14 > critical 2.064 (df 24)",
                ins![test: "one_sample", mean: 86.0, sd: 7.0, n: 25, mu: 89.0, alpha: "0.05"],
                "t_test.significant",
            )
            .score(2.14, 0.0)
            .noted("critical t (two-sided, 5%) = 2.064")
            .cut(
                "x̄ 86.2 gives |t| 2.00 (< 2.064); x̄ 86.1 gives 2.07 (≥ 2.064)",
                ins![test: "one_sample", sd: 7.0, n: 25, mu: 89.0, alpha: "0.05"],
                "mean",
                (86.2, "t_test.not_significant"),
                (86.1, "t_test.significant"),
            )
            .edge(
                "Paired: mean difference 2, Sd 4, n 16 → t 2.00 < 2.131 (df 15)",
                ins![test: "paired", mean: 2.0, sd: 4.0, n: 16, alpha: "0.05"],
                "t_test.not_significant",
            )
            .edge(
                "Paired: mean difference 2.5 → t 2.50 ≥ 2.131",
                ins![test: "paired", mean: 2.5, sd: 4.0, n: 16, alpha: "0.05"],
                "t_test.significant",
            )
            .edge(
                "Two groups 10 vs 8, S 3 and 3, n 10 each → t 1.49 < 2.101 (df 18)",
                ins![test: "independent", mean: 10.0, sd: 3.0, n: 10, mean2: 8.0, sd2: 3.0, n2: 10, alpha: "0.05"],
                "t_test.not_significant",
            )
            .edge(
                "Two groups 10 vs 7 → t 2.24 ≥ 2.101",
                ins![test: "independent", mean: 10.0, sd: 3.0, n: 10, mean2: 7.0, sd2: 3.0, n2: 10, alpha: "0.05"],
                "t_test.significant",
            ),
    ]
}
