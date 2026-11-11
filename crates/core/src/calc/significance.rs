//! Basics of Research: tests of significance (chi-square and Student's t).
//!
//! Source: Research L17, Hypothesis testing and tests of significance
//! (Nahla Ahmed Gamaleldin), slides 12–33. The decision rule is the
//! slides': a calculated value ≥ the critical value (from the level of
//! significance and the degrees of freedom) rejects H0 (slides 18–19).
//!
//! - Chi-square for a 2×2 table (slides 23–26): expected = row total ×
//!   column total / grand total, X² = Σ (O − E)² / E, df = (r − 1)(c − 1)
//!   = 1, critical 3.84 at 5% (Pearson 1900; Kirkwood and Sterne, Essential
//!   Medical Statistics, 2nd ed. 2003, ch. 17). No continuity correction,
//!   as the slides compute it; Yates's value is shown in the notes. The
//!   expected-count rule (every expected count at least 5, slide 28) is
//!   Cochran's (Biometrics 1954).
//! - Student's t (slides 12, 30–33): one sample against a known population
//!   mean, t = (x̄ − μ) / (S / √n), df = n − 1; paired, the same on the
//!   before–after differences with μ = 0; two independent groups with the
//!   pooled SD, df = n1 + n2 − 2 (slide 14; Student, Biometrika 1908;
//!   Kirkwood and Sterne 2003, ch. 7; Altman, Practical Statistics for
//!   Medical Research, 1991, ch. 9). Two-sided, as the slides' t table.
//!
//! P values come from the t distribution's regularized incomplete beta
//! function (Numerical Recipes, 3rd ed. 2007, §6.4) and, for X² with 1 df,
//! from the normal distribution (X² = z²).

use super::sample_size::inverse_normal;
use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, num};

const ALPHA: &[(&str, &str)] = &[("0.05", "5% (α = 0.05)"), ("0.01", "1% (α = 0.01)")];

fn get_alpha(i: &Inputs) -> Result<f64, CalcError> {
    let ids: Vec<&str> = ALPHA.iter().map(|(id, _)| *id).collect();
    Ok(i.get_choice("alpha", &ids)?.parse().unwrap_or(0.05))
}

/// ln Γ(x), x > 0 (Lanczos, g = 7, n = 9).
fn ln_gamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = G[0];
    let t = x + 7.5;
    for (k, g) in G.iter().enumerate().skip(1) {
        a += g / (x + k as f64);
    }
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

/// The continued fraction of the incomplete beta function (Lentz).
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    const TINY: f64 = 1e-300;
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..300 {
        let m = m as f64;
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    h
}

/// The regularized incomplete beta function I_x(a, b).
fn inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_cf(a, b, x) / a
    } else {
        1.0 - front * beta_cf(b, a, 1.0 - x) / b
    }
}

/// Two-sided P for Student's t with `df` degrees of freedom.
fn t_p_two_sided(t: f64, df: f64) -> f64 {
    inc_beta(df / 2.0, 0.5, df / (df + t * t))
}

/// The two-sided critical t: P(|T| ≥ t) = α (the slides' t table).
fn t_critical(alpha: f64, df: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 1.0e4_f64);
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if t_p_two_sided(mid, df) > alpha {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// The standard normal upper tail Q(z) = 1 − Φ(z), from the complementary
/// error function (Numerical Recipes erfc, fractional error < 1.2 × 10⁻⁷).
fn normal_upper(z: f64) -> f64 {
    let x = z / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * x.abs());
    let y = t
        * (-x * x - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    let erfc = if x >= 0.0 { y } else { 2.0 - y };
    erfc / 2.0
}

/// "p = 0.043", or "p < 0.001".
fn show_p(p: f64) -> String {
    if p < 0.001 { "p < 0.001".to_owned() } else { format!("p = {p:.3}") }
}

fn r2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn r3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

fn pct(alpha: f64) -> String {
    format!("{}", alpha * 100.0)
}

/// A count: a whole number, 0 or more.
fn get_count(i: &Inputs, key: &'static str) -> Result<f64, CalcError> {
    let v = i.get_num(key)?;
    if v < 0.0 || v.fract() != 0.0 {
        return Err(CalcError::Invalid { field: key, reason: "must be a whole number, 0 or more" });
    }
    Ok(v)
}

pub const CHI_SQUARE: CalcSpec = CalcSpec {
    id: "chi_square",
    name: "Chi-square test (2×2 table)",
    area: Area::Research,
    fields: &[
        num("a", "Row 1, column 1 (a)", "", 0.0, 1.0e9, 1.0),
        num("b", "Row 1, column 2 (b)", "", 0.0, 1.0e9, 1.0),
        num("c", "Row 2, column 1 (c)", "", 0.0, 1.0e9, 1.0),
        num("d", "Row 2, column 2 (d)", "", 0.0, 1.0e9, 1.0),
        choice("alpha", "Level of significance", ALPHA),
    ],
    source: "Research L17: hypothesis testing, slides 23–28 (Pearson 1900; Cochran 1954; Kirkwood and Sterne 2003)",
    compute: |i| {
        let obs = [get_count(i, "a")?, get_count(i, "b")?, get_count(i, "c")?, get_count(i, "d")?];
        let [a, b, c, d] = obs;
        let rows = [a + b, c + d];
        let cols = [a + c, b + d];
        let n = a + b + c + d;
        if rows.contains(&0.0) || cols.contains(&0.0) {
            return Err(CalcError::Invalid {
                field: "a",
                reason: "every row and column needs at least one count",
            });
        }
        let alpha = get_alpha(i)?;
        let exp =
            [rows[0] * cols[0] / n, rows[0] * cols[1] / n, rows[1] * cols[0] / n, rows[1] * cols[1] / n];
        let x2: f64 = obs.iter().zip(exp).map(|(o, e)| (o - e).powi(2) / e).sum();
        let yates: f64 = obs.iter().zip(exp).map(|(o, e)| ((o - e).abs() - 0.5).max(0.0).powi(2) / e).sum();
        let critical = inverse_normal(1.0 - alpha / 2.0).powi(2);
        let p = 2.0 * normal_upper(x2.sqrt());
        let x2_shown = r2(x2);
        let significant = x2_shown >= r2(critical);
        let names = ["a", "b", "c", "d"];
        let mut notes = vec![
            format!(
                "Expected = row total × column total / grand total: {}",
                names.iter().zip(exp).map(|(k, e)| format!("{k} {:.2}", e)).collect::<Vec<_>>().join(", ")
            ),
            format!(
                "X² = Σ (O − E)² / E = {}",
                obs.iter()
                    .zip(exp)
                    .map(|(o, e)| format!("{:.2}", (o - e).powi(2) / e))
                    .collect::<Vec<_>>()
                    .join(" + ")
            ),
            format!("df = (r − 1)(c − 1) = 1; critical X² at {}% = {:.2}", pct(alpha), critical),
            format!("{} (two-sided)", show_p(p)),
            format!("With Yates's continuity correction: X² = {:.2}", yates),
        ];
        let small = exp.iter().filter(|e| **e < 5.0).count();
        if small > 0 {
            notes.push(format!(
                "{small} expected count(s) below 5: the chi-square test needs every expected count to be at least 5; use Fisher's exact test"
            ));
        }
        let (band, label, meaning) = if significant {
            (
                "chi_square.significant",
                "X² ≥ critical value: reject H0",
                format!(
                    "X² {x2_shown:.2} ≥ {critical:.2}: reject H0 and accept H1. There is a statistically significant association between the two variables at the {}% level.",
                    pct(alpha)
                ),
            )
        } else {
            (
                "chi_square.not_significant",
                "X² < critical value: do not reject H0",
                format!(
                    "X² {x2_shown:.2} < {critical:.2}: H0 stands. No statistically significant association at the {}% level.",
                    pct(alpha)
                ),
            )
        };
        Ok(Outcome {
            score: Some(x2_shown),
            display: format!("X² = {x2_shown:.2}"),
            band,
            label,
            meaning,
            page: None,
            parts: vec![
                Part::new("a", a),
                Part::new("b", b),
                Part::new("c", c),
                Part::new("d", d),
                Part::new("Critical value", r2(critical)),
            ],
            notes,
        })
    },
};

const T_TESTS: &[(&str, &str)] = &[
    ("one_sample", "One sample vs a known population mean"),
    ("paired", "Paired: the same people before and after"),
    ("independent", "Two independent groups (Student's t)"),
];

pub const T_TEST: CalcSpec = CalcSpec {
    id: "t_test",
    name: "Student's t-test (one sample, paired, two groups)",
    area: Area::Research,
    fields: &[
        choice("test", "Which t-test", T_TESTS),
        num("mean", "Mean (x̄): the sample, the differences (paired) or group 1", "", -1.0e6, 1.0e6, 0.1),
        num("sd", "Standard deviation (S) of the same", "", 0.0, 1.0e6, 0.1),
        num("n", "Number (n) of the same", "", 2.0, 1.0e7, 1.0),
        num("mu", "Population mean (μ), one sample only", "", -1.0e6, 1.0e6, 0.1).optional(),
        num("mean2", "Group 2 mean", "", -1.0e6, 1.0e6, 0.1).optional(),
        num("sd2", "Group 2 standard deviation", "", 0.0, 1.0e6, 0.1).optional(),
        num("n2", "Group 2 number", "", 2.0, 1.0e7, 1.0).optional(),
        choice("alpha", "Level of significance", ALPHA),
    ],
    source: "Research L17: hypothesis testing, slides 12–14 and 29–33 (Student 1908; Kirkwood and Sterne 2003; Altman 1991)",
    compute: |i| {
        let ids: Vec<&str> = T_TESTS.iter().map(|(id, _)| *id).collect();
        let test = i.get_choice("test", &ids)?;
        let alpha = get_alpha(i)?;
        let mean = i.get_num("mean")?;
        let sd = i.get_pos("sd")?;
        let n = i.get_num("n")?;
        if n < 2.0 || n.fract() != 0.0 {
            return Err(CalcError::Invalid { field: "n", reason: "must be a whole number, 2 or more" });
        }
        let (t, df, formula, mut parts) = match test {
            "independent" => {
                let missing =
                    |f| CalcError::Invalid { field: f, reason: "needed for two independent groups" };
                let mean2 = i.opt_num("mean2")?.ok_or(missing("mean2"))?;
                let sd2 = i.opt_num("sd2")?.ok_or(missing("sd2"))?;
                if sd2 <= 0.0 {
                    return Err(CalcError::Invalid { field: "sd2", reason: "must be above zero" });
                }
                let n2 = i.opt_num("n2")?.ok_or(missing("n2"))?;
                if n2 < 2.0 || n2.fract() != 0.0 {
                    return Err(CalcError::Invalid {
                        field: "n2",
                        reason: "must be a whole number, 2 or more",
                    });
                }
                let df = n + n2 - 2.0;
                let sp = (((n - 1.0) * sd * sd + (n2 - 1.0) * sd2 * sd2) / df).sqrt();
                let se = sp * (1.0 / n + 1.0 / n2).sqrt();
                (
                    (mean - mean2) / se,
                    df,
                    format!(
                        "Pooled SD = √{{[(n1 − 1)S1² + (n2 − 1)S2²] / (n1 + n2 − 2)}} = {sp:.3}; t = (x̄1 − x̄2) / [Sp √(1/n1 + 1/n2)] = ({mean} − {mean2}) / {se:.3}"
                    ),
                    vec![
                        Part::new("Group 1 mean", mean),
                        Part::new("Group 2 mean", mean2),
                        Part::new("df = n1 + n2 − 2", df),
                    ],
                )
            }
            "paired" => {
                let se = sd / n.sqrt();
                (
                    mean / se,
                    n - 1.0,
                    format!("t = d̄ / (Sd / √n) = {mean} / ({sd} / √{n})"),
                    vec![Part::new("Mean difference", mean), Part::new("df = n − 1", n - 1.0)],
                )
            }
            _ => {
                let mu = i
                    .opt_num("mu")?
                    .ok_or(CalcError::Invalid { field: "mu", reason: "needed for a one-sample test" })?;
                let se = sd / n.sqrt();
                (
                    (mean - mu) / se,
                    n - 1.0,
                    format!("t = (x̄ − μ) / (S / √n) = ({mean} − {mu}) / ({sd} / √{n})"),
                    vec![
                        Part::new("Sample mean", mean),
                        Part::new("Population mean", mu),
                        Part::new("df = n − 1", n - 1.0),
                    ],
                )
            }
        };
        let t_abs = r2(t.abs());
        let critical = t_critical(alpha, df);
        let p = t_p_two_sided(t, df);
        parts.push(Part::new("Critical t", r3(critical)));
        let significant = t_abs >= r3(critical);
        let notes = vec![
            format!("{formula} = {:.2}", t),
            format!("df = {df}; critical t (two-sided, {}%) = {:.3} from the t table", pct(alpha), critical),
            format!("{} (two-sided)", show_p(p)),
            "Compare the size of t (ignore its sign) with the critical value".to_owned(),
        ];
        let (band, label, meaning) = if significant {
            (
                "t_test.significant",
                "|t| ≥ critical value: reject H0",
                format!(
                    "|t| {t_abs:.2} ≥ {critical:.3}: reject H0 and accept H1. The means differ significantly at the {}% level.",
                    pct(alpha)
                ),
            )
        } else {
            (
                "t_test.not_significant",
                "|t| < critical value: do not reject H0",
                format!(
                    "|t| {t_abs:.2} < {critical:.3}: H0 stands. No statistically significant difference between the means at the {}% level.",
                    pct(alpha)
                ),
            )
        };
        Ok(Outcome {
            score: Some(t_abs),
            display: format!("t = {:.2}", t),
            band,
            label,
            meaning,
            page: None,
            parts,
            notes,
        })
    },
};

const NOT_PROOF: &str = "Not significant is not proof of no difference: the evidence was too weak to reject H0; check the sample size and power (L17 notes; Altman and Bland, BMJ 1995)";
const REPORT_P: &str =
    "Report the exact P value rather than only 'significant' or 'not significant' (L17 slide 17)";
const CHANCE_ONLY: &str = "A significant test rules out chance only, not bias or confounding, and an association is not causation (L17 slide 2; L12)";

/// How to read each result: the slides (L17) and Kirkwood and Sterne,
/// Essential Medical Statistics, 2nd ed. 2003; Altman and Bland, "Absence
/// of evidence is not evidence of absence", BMJ 1995; Cochran, Biometrics
/// 1954. A test statistic has no differential.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "chi_square.significant",
        Guide {
            next: &[
                "State it: reject H0 and accept H1, there is a significant association between the condition and the characteristic (L17 slides 23, 26)",
                "Check every expected count is at least 5; if not, use Fisher's exact test (L17 slide 28; Cochran 1954)",
                "Measure how strong the association is with the relative risk or odds ratio (L05)",
                CHANCE_ONLY,
                REPORT_P,
            ],
            dd: &[],
        },
    ),
    (
        "chi_square.not_significant",
        Guide {
            next: &[
                "State it: H0 stands, no significant association at this level (L17 slide 19)",
                "Check every expected count is at least 5; if not, use Fisher's exact test (L17 slide 28; Cochran 1954)",
                NOT_PROOF,
                REPORT_P,
            ],
            dd: &[],
        },
    ),
    (
        "t_test.significant",
        Guide {
            next: &[
                "State it: reject H0 and accept H1, the means differ significantly; say which mean is higher (L17 slides 29, 33)",
                "A t-test needs a quantitative outcome that is roughly normal; if it is skewed, use a non-parametric test (L17 slide 12; L18)",
                "Pick the right t-test: same people before and after → paired; two separate groups → Student's; more than two groups → ANOVA (L17 slide 12)",
                CHANCE_ONLY,
                REPORT_P,
            ],
            dd: &[],
        },
    ),
    (
        "t_test.not_significant",
        Guide {
            next: &[
                "State it: H0 stands, no significant difference between the means at this level (L17 slide 19)",
                "A t-test needs a quantitative outcome that is roughly normal; if it is skewed, use a non-parametric test (L17 slide 12; L18)",
                NOT_PROOF,
                REPORT_P,
            ],
            dd: &[],
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn chi(a: f64, b: f64, c: f64, d: f64) -> Outcome {
        let i = Inputs::new().num("a", a).num("b", b).num("c", c).num("d", d).choice("alpha", "0.05");
        (CHI_SQUARE.compute)(&i).unwrap()
    }

    #[test]
    fn the_slides_t_table() {
        // Slide 16: two-sided critical t.
        for (df, alpha, t) in [
            (1.0, 0.05, 12.706),
            (5.0, 0.05, 2.571),
            (24.0, 0.05, 2.064),
            (30.0, 0.05, 2.042),
            (10.0, 0.01, 3.169),
            (24.0, 0.01, 2.797),
            (120.0, 0.05, 1.980),
        ] {
            assert!((t_critical(alpha, df) - t).abs() < 0.0006, "df {df} α {alpha}");
        }
    }

    #[test]
    fn the_slides_chi_square_critical() {
        assert!((inverse_normal(0.975).powi(2) - 3.841).abs() < 0.001);
        assert!((inverse_normal(0.995).powi(2) - 6.635).abs() < 0.001);
        // P of X² = 3.841 with 1 df is 0.05.
        assert!((2.0 * normal_upper(3.841_f64.sqrt()) - 0.05).abs() < 1e-4);
    }

    #[test]
    fn the_coffee_example() {
        // Slides 24–26: 30/20, 10/35 → X² 13.84 on the slide (13.87 unrounded).
        let o = chi(30.0, 20.0, 10.0, 35.0);
        assert_eq!(o.band, "chi_square.significant");
        assert!((o.score.unwrap() - 13.87).abs() < 0.01);
        assert!(o.notes[0].contains("a 21.05"));
    }

    #[test]
    fn the_haemoglobin_example() {
        // Slides 30–33: x̄ 86, μ 89, S 7, n 25 → |t| 2.14 > 2.064.
        let i = Inputs::new()
            .choice("test", "one_sample")
            .num("mean", 86.0)
            .num("sd", 7.0)
            .num("n", 25.0)
            .num("mu", 89.0)
            .choice("alpha", "0.05");
        let o = (T_TEST.compute)(&i).unwrap();
        assert_eq!((o.score, o.band), (Some(2.14), "t_test.significant"));
        assert_eq!(o.display, "t = -2.14");
        // At 1% the critical t is 2.797: not significant.
        let i = i.choice("alpha", "0.01");
        assert_eq!((T_TEST.compute)(&i).unwrap().band, "t_test.not_significant");
    }

    #[test]
    fn independent_groups_need_group_two() {
        let i = Inputs::new()
            .choice("test", "independent")
            .num("mean", 271.0)
            .num("sd", 40.0)
            .num("n", 20.0)
            .choice("alpha", "0.05");
        assert!((T_TEST.compute)(&i).is_err());
        let i = i.num("mean2", 184.0).num("sd2", 40.0).num("n2", 20.0);
        let o = (T_TEST.compute)(&i).unwrap();
        // Pooled SD 40, SE 12.649 → t = 87 / 12.649 = 6.88, df 38.
        assert!((o.score.unwrap() - 6.88).abs() < 0.01);
        assert_eq!(o.band, "t_test.significant");
    }

    #[test]
    fn small_expected_counts_are_flagged() {
        let o = chi(3.0, 1.0, 1.0, 3.0);
        assert!(o.notes.iter().any(|n| n.contains("Fisher")));
    }

    #[test]
    fn empty_row_is_an_error() {
        let i = Inputs::new().num("a", 0.0).num("b", 0.0).num("c", 3.0).num("d", 4.0).choice("alpha", "0.05");
        assert!((CHI_SQUARE.compute)(&i).is_err());
    }
}
