//! Basics of Research: predictive values in a new population, and the
//! post-test probability from a likelihood ratio.
//!
//! Source: Research L14, Measures of diagnostic accuracy (Prof. Maha
//! Moamen). Slide 25 gives the "standard formula" for PPV and NPV when the
//! sensitivity and specificity are known and the test is used in a
//! population of another prevalence:
//!
//! - PPV = Se·P / [Se·P + (1 − Sp)(1 − P)]
//! - NPV = Sp(1 − P) / [(1 − Se)·P + Sp(1 − P)]
//!
//! (Altman and Bland, "Diagnostic tests 2: predictive values", BMJ 1994;
//! 309:102). Slides 24–25: sensitivity and specificity do not change with
//! prevalence, PPV rises with it and NPV falls.
//!
//! The slides stop at predictive values; likelihood ratios come from the
//! standard texts: LR+ = Se / (1 − Sp), LR− = (1 − Se) / Sp; post-test odds
//! = pre-test odds × LR (Deeks and Altman, "Diagnostic tests 4: likelihood
//! ratios", BMJ 2004;329:168; Fagan, "Nomogram for Bayes's theorem", N Engl
//! J Med 1975;293:257). How much an LR moves the probability: Jaeschke,
//! Guyatt and Sackett, "Users' guides to the medical literature III. How to
//! use an article about a diagnostic test. B", JAMA 1994;271:703–707.
//! With the slides' own 2×2 table (slides 14 and 17: a 80, b 100, c 20,
//! d 800), pre-test 10% and LR+ 7.2 give 44.44%, the table's PPV, and LR−
//! 0.225 gives 2.44% = 1 − NPV: the two routes agree.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, num};

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Two decimals without trailing zeros: 44.44, 10, 7.2.
fn show2(v: f64) -> String {
    let s = format!("{:.2}", round2(v));
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// A percentage input as a fraction, inside `[lo, hi]` percent.
fn fraction(i: &Inputs, key: &'static str, lo: f64, hi: f64, reason: &'static str) -> Result<f64, CalcError> {
    let v = i.get_num(key)?;
    if (lo..=hi).contains(&v) { Ok(v / 100.0) } else { Err(CalcError::Invalid { field: key, reason }) }
}

const SHOW: &[(&str, &str)] = &[("ppv", "Positive predictive value"), ("npv", "Negative predictive value")];

pub const PREDICTIVE: CalcSpec = CalcSpec {
    id: "predictive_values",
    name: "Predictive values at a new prevalence",
    area: Area::Research,
    fields: &[
        num("sensitivity", "Sensitivity", "%", 0.0, 100.0, 0.1),
        num("specificity", "Specificity", "%", 0.0, 100.0, 0.1),
        num("prevalence", "Prevalence (pre-test probability)", "%", 0.01, 99.99, 0.1)
            .hint("Of the population you will test, not of the study that made the test"),
        choice("measure", "Answer with", SHOW),
    ],
    source: "Research L14: measures of diagnostic accuracy, slides 24–25 (Altman and Bland, BMJ 1994)",
    compute: |i| {
        let se = fraction(i, "sensitivity", 0.0, 100.0, "must be 0–100%")?;
        let sp = fraction(i, "specificity", 0.0, 100.0, "must be 0–100%")?;
        let p = fraction(i, "prevalence", 0.01, 99.99, "must be above 0% and below 100%")?;
        let measure = i.get_option(
            "measure",
            &[("ppv", &["positive_predictive_value"]), ("npv", &["negative_predictive_value"])],
        )?;
        // Out of 1,000 people tested: the slides' way of thinking in counts.
        let tp = se * p * 1000.0;
        let fn_ = (1.0 - se) * p * 1000.0;
        let tn = sp * (1.0 - p) * 1000.0;
        let fp = (1.0 - sp) * (1.0 - p) * 1000.0;
        let ppv = (tp + fp > 0.0).then(|| tp / (tp + fp) * 100.0);
        let npv = (tn + fn_ > 0.0).then(|| tn / (tn + fn_) * 100.0);
        let lr_pos = (sp < 1.0).then(|| se / (1.0 - sp));
        let lr_neg = (sp > 0.0).then(|| (1.0 - se) / sp);
        let (value, band, label, meaning) = if measure == "ppv" {
            (
                ppv,
                "predictive_values.ppv",
                "Positive predictive value",
                "Of those who test positive here, the share who really have the disease.",
            )
        } else {
            (
                npv,
                "predictive_values.npv",
                "Negative predictive value",
                "Of those who test negative here, the share who really are free of it.",
            )
        };
        let v = round2(value.ok_or(CalcError::Invalid {
            field: "specificity",
            reason: "no one tests positive (or negative) with these values",
        })?);
        let opt = |o: Option<f64>, unit: &str| o.map_or("—".to_owned(), |v| format!("{}{unit}", show2(v)));
        let notes = vec![
            format!("PPV {} · NPV {} at a prevalence of {}%", opt(ppv, "%"), opt(npv, "%"), show2(p * 100.0)),
            format!(
                "Of 1,000 tested: {} true +, {} false +, {} false −, {} true −",
                show2(tp),
                show2(fp),
                show2(fn_),
                show2(tn)
            ),
            "PPV = Se × P / [Se × P + (1 − Sp)(1 − P)]; NPV = Sp(1 − P) / [(1 − Se) P + Sp(1 − P)] (slide 25)"
                .to_owned(),
            format!("LR+ {} · LR− {} (these don't change with prevalence)", opt(lr_pos, ""), opt(lr_neg, "")),
        ];
        Ok(Outcome {
            score: Some(v),
            display: format!("{}%", show2(v)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: vec![
                Part::new("Sensitivity (%)", se * 100.0),
                Part::new("Specificity (%)", sp * 100.0),
                Part::new("Prevalence (%)", p * 100.0),
            ],
            notes,
        })
    },
};

pub const POST_TEST: CalcSpec = CalcSpec {
    id: "post_test_probability",
    name: "Post-test probability (likelihood ratio)",
    area: Area::Research,
    fields: &[
        num("pretest", "Pre-test probability", "%", 0.01, 99.99, 0.1)
            .hint("The prevalence in patients like this one, or your clinical estimate"),
        num("lr", "Likelihood ratio of the result", "", 0.001, 1000.0, 0.01)
            .hint("Positive result: LR+ = sensitivity / (1 − specificity). Negative: LR− = (1 − sensitivity) / specificity"),
    ],
    source: "Research L14: measures of diagnostic accuracy (Deeks and Altman, BMJ 2004; Fagan, NEJM 1975; Jaeschke, JAMA 1994)",
    compute: |i| {
        let pre = fraction(i, "pretest", 0.01, 99.99, "must be above 0% and below 100%")?;
        let lr = i.get_pos("lr")?;
        let pre_odds = pre / (1.0 - pre);
        let post_odds = pre_odds * lr;
        let post = post_odds / (1.0 + post_odds);
        let v = round2(post * 100.0);
        let (band, label, meaning) = if lr > 10.0 {
            ("post_test_probability.large_rise", "Large rise", "A result this strong often settles the diagnosis.")
        } else if lr > 5.0 {
            ("post_test_probability.moderate_rise", "Moderate rise", "The result moves the probability up a fair way.")
        } else if lr > 2.0 {
            ("post_test_probability.small_rise", "Small rise", "The result nudges the probability up a little.")
        } else if lr >= 0.5 {
            ("post_test_probability.little_change", "Little change", "The result barely moves the probability.")
        } else if lr >= 0.2 {
            ("post_test_probability.small_fall", "Small fall", "The result nudges the probability down a little.")
        } else if lr >= 0.1 {
            ("post_test_probability.moderate_fall", "Moderate fall", "The result moves the probability down a fair way.")
        } else {
            ("post_test_probability.large_fall", "Large fall", "A result this strong often rules the diagnosis out.")
        };
        let notes = vec![
            format!(
                "Pre-test odds = {} / {} = {}",
                show2(pre * 100.0),
                show2(100.0 - pre * 100.0),
                show_odds(pre_odds)
            ),
            format!("Post-test odds = {} × {} = {}", show_odds(pre_odds), show_odds(lr), show_odds(post_odds)),
            format!("Post-test probability = odds / (1 + odds) = {}%", show2(v)),
        ];
        Ok(Outcome {
            score: Some(v),
            display: format!("{}%", show2(v)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: vec![Part::new("Pre-test probability (%)", pre * 100.0), Part::new("Likelihood ratio", lr)],
            notes,
        })
    },
};

/// Odds and ratios to three significant places at most: 0.111, 7.2, 0.025.
fn show_odds(v: f64) -> String {
    let dp = if v >= 10.0 {
        1
    } else if v >= 0.1 {
        3
    } else {
        4
    };
    let s = format!("{v:.dp$}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// How to read each result. The slides (L14) teach the predictive values;
/// the likelihood-ratio steps rest on Jaeschke et al., JAMA 1994, and
/// Deeks and Altman, BMJ 2004.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "predictive_values.ppv",
        Guide {
            next: &[
                "PPV rises with prevalence: the same test gives more false positives where the disease is rare (L14 slide 24)",
                "So confirm a positive screening result with a diagnostic test before treating (L14 slides 5–6)",
                "Where false positives are costly or stigmatising, choose a high-specificity cut-point (L14 slide 21)",
            ],
            dd: &[],
        },
    ),
    (
        "predictive_values.npv",
        Guide {
            next: &[
                "NPV falls as prevalence rises: in a high-risk group a negative result reassures less (L14 slide 24)",
                "Where missing a case is costly (fatal, treatable, or spreads easily), choose a high-sensitivity cut-point (L14 slide 21)",
                "A negative result from a very sensitive test helps rule the disease out, SnNout (L14 slide 16)",
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.large_rise",
        Guide {
            next: &[
                "An LR above 10 makes a large, often conclusive change: act on it if the post-test probability passes your treatment threshold (Jaeschke 1994)",
                PRETEST,
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.moderate_rise",
        Guide {
            next: &[
                "An LR of 5–10 makes a moderate change: often enough with a fair pre-test probability, otherwise add a test (Jaeschke 1994)",
                CHAIN,
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.small_rise",
        Guide {
            next: &[
                "An LR of 2–5 makes a small, sometimes important change: usually needs another test (Jaeschke 1994)",
                CHAIN,
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.little_change",
        Guide {
            next: &[
                "An LR of 0.5–2 rarely changes the decision: this test adds little here (Jaeschke 1994)",
                "Choose a test with a stronger LR for this question (Deeks 2004)",
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.small_fall",
        Guide {
            next: &[
                "An LR of 0.2–0.5 makes a small, sometimes important fall: not enough to rule out alone (Jaeschke 1994)",
                CHAIN,
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.moderate_fall",
        Guide {
            next: &[
                "An LR of 0.1–0.2 makes a moderate fall: often enough when the pre-test probability was low (Jaeschke 1994)",
                CHAIN,
            ],
            dd: &[],
        },
    ),
    (
        "post_test_probability.large_fall",
        Guide {
            next: &[
                "An LR below 0.1 makes a large, often conclusive fall: the disease is usually ruled out (Jaeschke 1994)",
                PRETEST,
            ],
            dd: &[],
        },
    ),
];

const PRETEST: &str =
    "Even a strong LR can't rescue a wrong pre-test probability: check it fits this patient (Deeks 2004)";
const CHAIN: &str =
    "Use this post-test probability as the pre-test for the next independent test (Deeks 2004)";

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spec: &CalcSpec, i: Inputs) -> Outcome {
        (spec.compute)(&i).unwrap()
    }

    #[test]
    fn slide_table_gives_the_same_values_both_ways() {
        // Slides 14 and 17: sensitivity 80/100, specificity 800/900, prevalence 100/1000.
        let sp = 800.0 / 900.0 * 100.0;
        let base = Inputs::new().num("sensitivity", 80.0).num("specificity", sp).num("prevalence", 10.0);
        assert_eq!(run(&PREDICTIVE, base.clone().choice("measure", "ppv")).score, Some(44.44));
        assert_eq!(run(&PREDICTIVE, base.choice("measure", "npv")).score, Some(97.56));
        let lr_pos = 0.8 / (1.0 - sp / 100.0);
        let o = run(&POST_TEST, Inputs::new().num("pretest", 10.0).num("lr", lr_pos));
        assert_eq!((o.score, o.band), (Some(44.44), "post_test_probability.moderate_rise"));
        let lr_neg = 0.2 / (sp / 100.0);
        let o = run(&POST_TEST, Inputs::new().num("pretest", 10.0).num("lr", lr_neg));
        assert_eq!((o.score, o.band), (Some(2.44), "post_test_probability.small_fall"));
    }

    #[test]
    fn lower_prevalence_lowers_ppv_and_raises_npv() {
        let at = |p: f64, m: &str| {
            run(
                &PREDICTIVE,
                Inputs::new()
                    .num("sensitivity", 80.0)
                    .num("specificity", 90.0)
                    .num("prevalence", p)
                    .choice("measure", m),
            )
            .score
            .unwrap()
        };
        assert!(at(1.0, "ppv") < at(10.0, "ppv") && at(10.0, "ppv") < at(50.0, "ppv"));
        assert!(at(1.0, "npv") > at(10.0, "npv") && at(10.0, "npv") > at(50.0, "npv"));
    }

    #[test]
    fn a_perfectly_specific_test_has_no_lr_plus() {
        let o = run(
            &PREDICTIVE,
            Inputs::new()
                .num("sensitivity", 50.0)
                .num("specificity", 100.0)
                .num("prevalence", 5.0)
                .choice("measure", "ppv"),
        );
        assert_eq!(o.score, Some(100.0));
        assert!(o.notes.iter().any(|n| n.contains("LR+ —")), "{:?}", o.notes);
    }

    #[test]
    fn a_test_that_is_never_positive_has_no_ppv() {
        let i = Inputs::new().num("sensitivity", 0.0).num("specificity", 100.0).num("prevalence", 5.0);
        assert!((PREDICTIVE.compute)(&i.choice("measure", "ppv")).is_err());
    }
}
