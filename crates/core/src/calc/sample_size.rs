//! Basics of Research: sample size for a prevalence survey and for comparing
//! two proportions (unmatched case-control and cohort studies).
//!
//! Source: Research L11, Sampling methods and sample size determination
//! (Prof. Heba El-Weshahi), slides 72–87, which run Epi Info 7 StatCalc.
//! The formulas are the ones StatCalc and OpenEpi document:
//!
//! - Population survey (slides 72–76): n = DEFF·N·p(1−p) /
//!   [(d²/z²)(N−1) + p(1−p)], the normal approximation with the finite
//!   population correction (Lwanga and Lemeshow, Sample size determination
//!   in health studies, WHO 1991; Dean, Sullivan and Soe, OpenEpi,
//!   "Sample size for a proportion or descriptive study"). StatCalc rounds
//!   to the nearest whole person: 321.7 → 322 and 888.4 → 888 on the slides.
//! - Two proportions (slides 77–87): Kelsey et al., Methods in Observational
//!   Epidemiology, 2nd ed. 1996; Fleiss, Statistical Methods for Rates and
//!   Proportions, 3rd ed. 2003 (with and without continuity correction).
//!   Each group is rounded up. The slides circle the Kelsey column.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, num};

/// The two-sided confidence levels StatCalc offers, with z(1 − α/2).
const CONFIDENCE: &[(&str, &str)] = &[
    ("80", "80%"),
    ("90", "90%"),
    ("95", "95%"),
    ("97", "97%"),
    ("99", "99%"),
    ("99.9", "99.9%"),
    ("99.99", "99.99%"),
];

fn z_two_sided(i: &Inputs) -> Result<(f64, f64), CalcError> {
    let ids: Vec<&str> = CONFIDENCE.iter().map(|(id, _)| *id).collect();
    let level: f64 = i.get_choice("confidence", &ids)?.parse().unwrap_or(95.0);
    let alpha = 1.0 - level / 100.0;
    Ok((level, inverse_normal(1.0 - alpha / 2.0)))
}

/// The standard normal quantile Φ⁻¹(p), 0 < p < 1, by Acklam's rational
/// approximation (relative error under 1.2 × 10⁻⁹): 0.975 → 1.959964.
pub(super) fn inverse_normal(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383_577_518_672_69e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [7.784695709041462e-3, 3.224671290700398e-1, 2.445134137142996, 3.754408661907416];
    const LOW: f64 = 0.02425;
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if p < LOW {
        tail((-2.0 * p.ln()).sqrt())
    } else if p > 1.0 - LOW {
        -tail((-2.0 * (1.0 - p).ln()).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// A percentage strictly between 0 and 100, as a proportion.
fn proportion(i: &Inputs, key: &'static str) -> Result<f64, CalcError> {
    let v = i.get_num(key)?;
    if v > 0.0 && v < 100.0 {
        Ok(v / 100.0)
    } else {
        Err(CalcError::Invalid { field: key, reason: "must be above 0% and below 100%" })
    }
}

fn whole(v: f64) -> String {
    let s = format!("{v:.0}");
    let n = s.len();
    s.chars()
        .enumerate()
        .flat_map(|(k, c)| (k > 0 && (n - k) % 3 == 0).then_some(',').into_iter().chain([c]))
        .collect()
}

fn pct(p: f64) -> String {
    let s = format!("{:.2}", p * 100.0);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub const SURVEY: CalcSpec = CalcSpec {
    id: "sample_size_survey",
    name: "Sample size: prevalence survey",
    area: Area::Research,
    fields: &[
        num("population", "Population size (N)", "", 1.0, 1.0e10, 1.0),
        num("expected_prevalence", "Expected frequency (prevalence)", "%", 0.01, 99.99, 0.1)
            .hint("From earlier studies, a pilot or experts; 50% if nothing is known"),
        num("margin", "Acceptable margin of error (d)", "%", 0.01, 50.0, 0.1),
        choice("confidence", "Confidence level", CONFIDENCE),
        num("design_effect", "Design effect (DEFF)", "", 1.0, 20.0, 0.1)
            .optional()
            .hint("1 for simple random sampling; often 2 for cluster sampling"),
    ],
    source: "Research L11: sampling and sample size, slides 72–76 (Epi Info StatCalc, population survey; Lwanga and Lemeshow, WHO 1991)",
    compute: |i| {
        let n_pop = i.get_num("population")?;
        if n_pop < 1.0 || n_pop.fract() != 0.0 {
            return Err(CalcError::Invalid {
                field: "population",
                reason: "must be a whole number, 1 or more",
            });
        }
        let p = proportion(i, "expected_prevalence")?;
        let d = i.get_pos("margin")? / 100.0;
        if d >= 1.0 {
            return Err(CalcError::Invalid { field: "margin", reason: "must be below 100%" });
        }
        let deff = i.opt_num("design_effect")?.unwrap_or(1.0);
        if deff < 1.0 {
            return Err(CalcError::Invalid { field: "design_effect", reason: "must be 1 or more" });
        }
        let (level, z) = z_two_sided(i)?;
        let pq = p * (1.0 - p);
        let infinite = z * z * pq / (d * d);
        let raw = deff * n_pop * pq / (d * d / (z * z) * (n_pop - 1.0) + pq);
        let n = raw.round().max(1.0);
        let mut notes = vec![
            format!(
                "n = DEFF × N × p(1 − p) / [(d² / z²) × (N − 1) + p(1 − p)] = {raw:.1}, rounded to {} as Epi Info does",
                whole(n)
            ),
            format!("z for {}% confidence = {z:.3}", pct(level / 100.0)),
            format!(
                "Without the finite population correction: z² p(1 − p) / d² = {}",
                whole(infinite.ceil())
            ),
        ];
        if d >= p.min(1.0 - p) {
            notes.push(
                "The margin is as large as the expected prevalence itself: choose a smaller margin"
                    .to_owned(),
            );
        }
        Ok(Outcome {
            score: Some(n),
            display: format!("n = {}", whole(n)),
            band: "sample_size_survey.n",
            label: "Minimum sample size",
            meaning: format!(
                "Survey at least {} people to estimate a prevalence of about {}% within ±{}% at {}% confidence.",
                whole(n),
                pct(p),
                pct(d),
                pct(level / 100.0)
            ),
            page: None,
            parts: vec![
                Part::new("Population (N)", n_pop),
                Part::new("Expected prevalence (%)", p * 100.0),
                Part::new("Margin of error (%)", d * 100.0),
                Part::new("z", (z * 1000.0).round() / 1000.0),
                Part::new("Design effect", deff),
            ],
            notes,
        })
    },
};

const DESIGNS: &[(&str, &str)] =
    &[("case_control", "Unmatched case-control"), ("cohort", "Unmatched cohort or cross-sectional")];

/// One group size and its comparison group by each method.
struct Sizes {
    kelsey: f64,
    fleiss: f64,
    fleiss_cc: f64,
}

fn two_groups(z_a: f64, z_b: f64, r: f64, p1: f64, p0: f64) -> Sizes {
    let diff = (p1 - p0).abs();
    let p_bar = (p1 + r * p0) / (r + 1.0);
    let q_bar = 1.0 - p_bar;
    let kelsey = (z_a + z_b).powi(2) * p_bar * q_bar * (r + 1.0) / (r * diff * diff);
    let fleiss = (z_a * ((r + 1.0) * p_bar * q_bar).sqrt()
        + z_b * (r * p1 * (1.0 - p1) + p0 * (1.0 - p0)).sqrt())
    .powi(2)
        / (r * diff * diff);
    let fleiss_cc = fleiss / 4.0 * (1.0 + (1.0 + 2.0 * (r + 1.0) / (fleiss * r * diff)).sqrt()).powi(2);
    Sizes { kelsey, fleiss, fleiss_cc }
}

/// Rounds a group up, ignoring floating-point dust (294.0000001 → 294).
fn up(v: f64) -> f64 {
    (v - 1e-9).ceil()
}

pub const TWO_PROPORTIONS: CalcSpec = CalcSpec {
    id: "sample_size_two_proportions",
    name: "Sample size: case-control or cohort (two proportions)",
    area: Area::Research,
    fields: &[
        choice("design", "Study design", DESIGNS),
        choice("confidence", "Two-sided confidence level", CONFIDENCE),
        num("power", "Power", "%", 50.0, 99.9, 1.0).hint("80% at least; 85% or 90% is better"),
        num(
            "ratio",
            "Ratio of group 2 to group 1 (controls : cases, unexposed : exposed)",
            "",
            0.1,
            10.0,
            0.5,
        )
        .optional()
        .hint("1 when the groups are equal"),
        num(
            "p_group2",
            "% exposed among controls, or % with the outcome among the unexposed",
            "%",
            0.01,
            99.99,
            0.1,
        ),
        num(
            "p_group1",
            "% exposed among cases, or % with the outcome among the exposed",
            "%",
            0.01,
            99.99,
            0.1,
        ),
    ],
    source: "Research L11: sampling and sample size, slides 77–87 (Epi Info StatCalc; Kelsey 1996; Fleiss 2003)",
    compute: |i| {
        let ids: Vec<&str> = DESIGNS.iter().map(|(id, _)| *id).collect();
        let design = i.get_choice("design", &ids)?;
        let (level, z_a) = z_two_sided(i)?;
        let power = i.get_num("power")?;
        if !(50.0..100.0).contains(&power) {
            return Err(CalcError::Invalid { field: "power", reason: "must be 50% or more and below 100%" });
        }
        let z_b = inverse_normal(power / 100.0);
        let r = i.opt_num("ratio")?.unwrap_or(1.0);
        if r <= 0.0 {
            return Err(CalcError::Invalid { field: "ratio", reason: "must be above zero" });
        }
        let p0 = proportion(i, "p_group2")?;
        let p1 = proportion(i, "p_group1")?;
        if (p1 - p0).abs() < 1e-12 {
            return Err(CalcError::Invalid {
                field: "p_group1",
                reason: "same as group 2: there is no difference to detect",
            });
        }
        let s = two_groups(z_a, z_b, r, p1, p0);
        let pair = |n1: f64| (up(n1), up(n1 * r));
        let (k1, k2) = pair(s.kelsey);
        let (f1, f2) = pair(s.fleiss);
        let (c1, c2) = pair(s.fleiss_cc);
        let odds = p1 / (1.0 - p1) / (p0 / (1.0 - p0));
        let (band, label, g1, g2, meaning) = if design == "cohort" {
            (
                "sample_size_two_proportions.cohort",
                "Minimum size of each cohort",
                "exposed",
                "unexposed",
                format!(
                    "Follow at least {} exposed and {} unexposed people ({} in all) to detect a risk ratio of {:.2} with {}% power at {}% confidence.",
                    whole(k1),
                    whole(k2),
                    whole(k1 + k2),
                    p1 / p0,
                    pct(power / 100.0),
                    pct(level / 100.0)
                ),
            )
        } else {
            (
                "sample_size_two_proportions.case_control",
                "Minimum number of cases and controls",
                "cases",
                "controls",
                format!(
                    "Recruit at least {} cases and {} controls ({} in all) to detect an odds ratio of {:.2} with {}% power at {}% confidence.",
                    whole(k1),
                    whole(k2),
                    whole(k1 + k2),
                    odds,
                    pct(power / 100.0),
                    pct(level / 100.0)
                ),
            )
        };
        let mut notes = vec![
            format!("Kelsey: {} {g1} + {} {g2} = {}", whole(k1), whole(k2), whole(k1 + k2)),
            format!("Fleiss: {} {g1} + {} {g2} = {}", whole(f1), whole(f2), whole(f1 + f2)),
            format!(
                "Fleiss with continuity correction: {} {g1} + {} {g2} = {}",
                whole(c1),
                whole(c2),
                whole(c1 + c2)
            ),
        ];
        if design == "cohort" {
            notes.push(format!("Risk ratio = {:.2}; odds ratio = {odds:.2}", p1 / p0));
        } else {
            notes.push(format!("Odds ratio = {odds:.2}"));
        }
        notes.push(format!("z(α/2) = {z_a:.3}; z(β) = {z_b:.3}"));
        Ok(Outcome {
            score: Some(k1),
            display: format!("{} {g1} + {} {g2}", whole(k1), whole(k2)),
            band,
            label,
            meaning,
            page: None,
            parts: vec![
                Part::new(format!("Group 1, {g1} (%)"), p1 * 100.0),
                Part::new(format!("Group 2, {g2} (%)"), p0 * 100.0),
                Part::new("Ratio (group 2 : group 1)", r),
                Part::new("Power (%)", power),
            ],
            notes,
        })
    },
};

/// How to use each result: the slides (L11 slides 64–72, 88–89) and
/// Lwanga and Lemeshow (WHO 1991). A sample size has no differential.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "sample_size_survey.n",
        Guide {
            next: &[
                "Base it on the primary outcome, the study's main objective (L11 slide 65)",
                "No earlier estimate of the prevalence? Use 50%: it gives the largest, safest sample (L11 slide 72)",
                "Cluster sampling needs a design effect above 1 (often 2), because its sampling error is higher than a simple random sample's (L11 slide 43; Lwanga and Lemeshow 1991)",
                "Add for non-response: divide n by the expected response rate, since participation shapes how representative the sample is (L11 slide 12)",
                "A smaller margin of error or a higher confidence level makes the sample bigger: 5% → 3% takes 322 to 888 (L11 slides 75–76)",
            ],
            dd: &[],
        },
    ),
    (
        "sample_size_two_proportions.case_control",
        Guide {
            next: &[
                "Take the % of exposure among controls from earlier studies, a pilot or experts (L11 slides 67, 77)",
                "More controls per case (up to about 4) adds power when cases are scarce (L11 slide 77)",
                "A higher power, a smaller alpha or a smaller expected difference all need a bigger sample (L11 slide 70)",
                "Fleiss with continuity correction is the most cautious of the three columns",
                "For a mean difference (a quantitative outcome), use G*Power instead (L11 slide 88)",
            ],
            dd: &[],
        },
    ),
    (
        "sample_size_two_proportions.cohort",
        Guide {
            next: &[
                "Take the outcome rates of each group from earlier studies, a pilot or experts (L11 slides 67, 86)",
                "Add for loss to follow-up: divide each group by the share you expect to keep",
                "A higher power, a smaller alpha or a smaller expected difference all need a bigger sample (L11 slide 70)",
                "Fleiss with continuity correction is the most cautious of the three columns",
                "For a mean difference (a quantitative outcome), use G*Power instead (L11 slide 88)",
            ],
            dd: &[],
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn survey(margin: f64, confidence: &str) -> Outcome {
        let i = Inputs::new()
            .num("population", 100000.0)
            .num("expected_prevalence", 30.0)
            .num("margin", margin)
            .choice("confidence", confidence)
            .num("design_effect", 1.0);
        (SURVEY.compute)(&i).unwrap()
    }

    fn two(design: &str, p0: f64, p1: f64) -> Outcome {
        let i = Inputs::new()
            .choice("design", design)
            .choice("confidence", "95")
            .num("power", 80.0)
            .num("ratio", 1.0)
            .num("p_group2", p0)
            .num("p_group1", p1);
        (TWO_PROPORTIONS.compute)(&i).unwrap()
    }

    #[test]
    fn z_values() {
        assert!((inverse_normal(0.975) - 1.959964).abs() < 1e-6);
        assert!((inverse_normal(0.8) - 0.841621).abs() < 1e-6);
        assert!((inverse_normal(0.005) + 2.575829).abs() < 1e-6);
    }

    #[test]
    fn the_slides_statcalc_tables() {
        // Slide 75: N 100,000, 30%, margin 5%.
        for (level, n) in [("80", 138.0), ("95", 322.0), ("99", 554.0), ("99.9", 901.0), ("99.99", 1256.0)] {
            assert_eq!(survey(5.0, level).score, Some(n), "{level}%");
        }
        // Slide 76: margin 3%.
        for (level, n) in [("80", 382.0), ("90", 627.0), ("95", 888.0), ("97", 1087.0), ("99", 1525.0)] {
            assert_eq!(survey(3.0, level).score, Some(n), "{level}%");
        }
        // Slide 81: controls 20%, cases 30% → Kelsey 295, Fleiss 294, CC 313.
        let o = two("case_control", 20.0, 30.0);
        assert_eq!(o.display, "295 cases + 295 controls");
        assert!(o.notes.iter().any(|n| n == "Fleiss: 294 cases + 294 controls = 588"));
        assert!(o.notes.iter().any(|n| n.contains("313 cases + 313 controls = 626")));
        // Slide 85: 25% vs 38% → 201 / 200 / 215.
        let o = two("cohort", 25.0, 38.0);
        assert_eq!(o.score, Some(201.0));
        assert!(o.notes.iter().any(|n| n == "Fleiss: 200 exposed + 200 unexposed = 400"));
        assert!(o.notes.iter().any(|n| n.contains("215 exposed + 215 unexposed = 430")));
        // Slide 87: CMV 4.9% vs 18% → 93 / 92 / 107.
        let o = two("cohort", 4.9, 18.0);
        assert_eq!(o.score, Some(93.0));
        assert!(o.notes.iter().any(|n| n == "Fleiss: 92 exposed + 92 unexposed = 184"));
        assert!(o.notes.iter().any(|n| n.contains("107 exposed + 107 unexposed = 214")));
    }

    #[test]
    fn more_controls_fewer_cases() {
        let i = Inputs::new()
            .choice("design", "case_control")
            .choice("confidence", "95")
            .num("power", 80.0)
            .num("ratio", 2.0)
            .num("p_group2", 20.0)
            .num("p_group1", 30.0);
        let o = (TWO_PROPORTIONS.compute)(&i).unwrap();
        assert!(o.score.unwrap() < 295.0);
    }

    #[test]
    fn bad_inputs() {
        let i = Inputs::new()
            .choice("design", "cohort")
            .choice("confidence", "95")
            .num("power", 80.0)
            .num("p_group2", 20.0)
            .num("p_group1", 20.0);
        assert!((TWO_PROPORTIONS.compute)(&i).is_err());
        let i = Inputs::new()
            .num("population", 1000.0)
            .num("expected_prevalence", 100.0)
            .num("margin", 5.0)
            .choice("confidence", "95");
        assert!((SURVEY.compute)(&i).is_err());
    }
}
