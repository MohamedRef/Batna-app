//! Basics of Research: measures of disease frequency, association, effect
//! and diagnostic accuracy.
//!
//! Source: the course slides (medstudy/courses/res/). Lecture 2, Measures of
//! disease frequency: incidence (slide 25), attack rate ×100 (26–29),
//! point/period prevalence (31), P = I × D (37), secondary attack rate
//! = secondary cases / susceptible contacts ×100 (46–49), PMR ×100 (59),
//! CFR ×100 (60), IMR and neonatal mortality per 1000 live births (63–64).
//! Lecture 5, Analytic studies: the 2×2 table a/b exposed, c/d not exposed;
//! OR = ad/bc (32), RR = Ie/Io and ARP = (Ie − Io)/Ie ×100 (50–51), RR > 1
//! risk factor, = 1 no association, < 1 protective (53). NNT = 1/ARR and the
//! accuracy measures follow the standard definitions taught in Lectures 6
//! and 14. `page` stays `None`: these pages are slides, not book pages.

use super::{Area, CalcError, CalcSpec, Field, Guide, Inputs, Outcome, Part, choice, num, show};

const COUNT_MAX: f64 = 1.0e9;

const fn count(key: &'static str, label: &'static str) -> Field {
    num(key, label, "", 0.0, COUNT_MAX, 1.0)
}

/// A count, which must be a whole number and not below zero.
fn get_count(i: &Inputs, key: &'static str) -> Result<f64, CalcError> {
    let v = i.get_num(key)?;
    if v < 0.0 || v.fract() != 0.0 {
        return Err(CalcError::Invalid { field: key, reason: "must be a whole number, 0 or more" });
    }
    Ok(v)
}

/// A denominator count: above zero.
fn get_denominator(i: &Inputs, key: &'static str) -> Result<f64, CalcError> {
    let v = get_count(i, key)?;
    if v > 0.0 { Ok(v) } else { Err(CalcError::Invalid { field: key, reason: "must be above zero" }) }
}

const PER: &[(&str, &str)] =
    &[("100", "per 100 (%)"), ("1000", "per 1,000"), ("10000", "per 10,000"), ("100000", "per 100,000")];

fn get_per(i: &Inputs) -> Result<f64, CalcError> {
    let p =
        i.get_option("per", &[("100", &["percent", "%"]), ("1000", &[]), ("10000", &[]), ("100000", &[])])?;
    Ok(p.parse().unwrap_or(1000.0))
}

fn per_label(per: f64) -> String {
    if per == 100.0 { "%".to_owned() } else { format!("per {}", group(per)) }
}

/// 100000 → "100,000".
fn group(v: f64) -> String {
    let s = format!("{v:.0}");
    let mut out = String::new();
    for (k, c) in s.chars().enumerate() {
        if k > 0 && (s.len() - k) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A rate: `n / d × per`, rounded to two decimals.
fn rate(
    i: &Inputs,
    (num_key, num_label): (&'static str, &'static str),
    (den_key, den_label): (&'static str, &'static str),
    per: f64,
    band: &'static str,
    label: &'static str,
    meaning: &str,
) -> Result<Outcome, CalcError> {
    let n = get_count(i, num_key)?;
    let d = get_denominator(i, den_key)?;
    if n > d {
        return Err(CalcError::Invalid { field: num_key, reason: "can't be more than the denominator" });
    }
    let v = round2(n / d * per);
    let unit = per_label(per);
    let display = if per == 100.0 { format!("{}%", show2(v)) } else { format!("{} {unit}", show2(v)) };
    Ok(Outcome {
        score: Some(v),
        display,
        band,
        label,
        meaning: meaning.to_owned(),
        page: None,
        parts: vec![Part::new(num_label, n), Part::new(den_label, d)],
        notes: Vec::new(),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Two decimals, without trailing zeros: 61.33, 10, 4.9.
fn show2(v: f64) -> String {
    let s = show(v, 2);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub const INCIDENCE: CalcSpec = CalcSpec {
    id: "incidence",
    name: "Incidence rate",
    area: Area::Research,
    fields: &[
        count("new_cases", "New cases during the period"),
        count("population_at_risk", "Population at risk (mid-period)"),
        choice("per", "Expressed", PER),
    ],
    source: "Research L02: measures of disease frequency, slide 25",
    compute: |i| {
        let per = get_per(i)?;
        rate(
            i,
            ("new_cases", "New cases"),
            ("population_at_risk", "Population at risk"),
            per,
            "incidence.rate",
            "Incidence rate",
            "New cases in a population at risk over a period: the risk of getting the disease.",
        )
    },
};

pub const PREVALENCE: CalcSpec = CalcSpec {
    id: "prevalence",
    name: "Prevalence",
    area: Area::Research,
    fields: &[
        count("existing_cases", "Existing (old + new) cases"),
        count("population", "Total population (mid-point or mid-period)"),
        choice("per", "Expressed", PER),
    ],
    source: "Research L02: measures of disease frequency, slide 31",
    compute: |i| {
        let per = get_per(i)?;
        rate(
            i,
            ("existing_cases", "Existing cases"),
            ("population", "Population"),
            per,
            "prevalence.rate",
            "Prevalence",
            "All existing cases at a point or over a period: the burden of illness needing care.",
        )
    },
};

pub const PREVALENCE_ID: CalcSpec = CalcSpec {
    id: "prevalence_id",
    name: "Prevalence = Incidence × Duration",
    area: Area::Research,
    fields: &[
        num("incidence", "Incidence", "per 1,000 per year", 0.0, 1000.0, 0.1),
        num("duration_years", "Mean duration of disease", "years", 0.0, 100.0, 0.1),
    ],
    source: "Research L02: measures of disease frequency, slide 37",
    compute: |i| {
        let inc = i.get_num("incidence")?;
        let dur = i.get_num("duration_years")?;
        if inc < 0.0 || dur < 0.0 {
            return Err(CalcError::Invalid { field: "incidence", reason: "can't be below zero" });
        }
        let p = round2(inc * dur);
        Ok(Outcome {
            score: Some(p),
            display: format!("{} per 1,000", show2(p)),
            band: "prevalence_id.prevalence",
            label: "Prevalence (P = I × D)",
            meaning: "A long-lasting disease piles up: low incidence can still give a high prevalence."
                .to_owned(),
            page: None,
            parts: vec![Part::new("Incidence", inc), Part::new("Duration (years)", dur)],
            notes: vec!["Holds in a steady state, when incidence and duration stay stable.".to_owned()],
        })
    },
};

pub const ATTACK_RATE: CalcSpec = CalcSpec {
    id: "attack_rate",
    name: "Attack rate",
    area: Area::Research,
    fields: &[count("cases", "Cases during the outbreak"), count("at_risk", "Population at risk")],
    source: "Research L02: measures of disease frequency, slides 26–29",
    compute: |i| {
        rate(
            i,
            ("cases", "Cases"),
            ("at_risk", "At risk"),
            100.0,
            "attack_rate.rate",
            "Attack rate (%)",
            "An incidence rate for a narrow population over a short time, as in an outbreak.",
        )
    },
};

pub const SAR: CalcSpec = CalcSpec {
    id: "sar",
    name: "Secondary attack rate",
    area: Area::Research,
    fields: &[
        count("secondary_cases", "Secondary cases (within one incubation period)"),
        count("contacts", "Contacts (household members, all)"),
        count("primary_cases", "Primary cases among them").hint("Left out of the susceptibles"),
    ],
    source: "Research L02: measures of disease frequency, slides 46–49",
    compute: |i| {
        let secondary = get_count(i, "secondary_cases")?;
        let contacts = get_count(i, "contacts")?;
        let primary = get_count(i, "primary_cases")?;
        let susceptible = contacts - primary;
        if susceptible <= 0.0 {
            return Err(CalcError::Invalid {
                field: "contacts",
                reason: "must be more than the primary cases",
            });
        }
        if secondary > susceptible {
            return Err(CalcError::Invalid {
                field: "secondary_cases",
                reason: "can't be more than the susceptibles",
            });
        }
        let v = round2(secondary / susceptible * 100.0);
        Ok(Outcome {
            score: Some(v),
            display: format!("{}%", show2(v)),
            band: "sar.rate",
            label: "Secondary attack rate (%)",
            meaning: "How easily the disease spreads from primary cases to their susceptible contacts."
                .to_owned(),
            page: None,
            parts: vec![
                Part::new("Secondary cases", secondary),
                Part::new("Susceptibles (contacts − primary cases)", susceptible),
            ],
            notes: vec![
                "Count within one incubation period; immune contacts are not susceptible.".to_owned(),
            ],
        })
    },
};

pub const CFR: CalcSpec = CalcSpec {
    id: "cfr",
    name: "Case fatality rate",
    area: Area::Research,
    fields: &[count("deaths", "Deaths from the disease"), count("cases", "People with the disease")],
    source: "Research L02: measures of disease frequency, slide 60",
    compute: |i| {
        rate(
            i,
            ("deaths", "Deaths"),
            ("cases", "Cases"),
            100.0,
            "cfr.rate",
            "Case fatality rate (%)",
            "The share of cases who die: a measure of how severe (killing) an acute disease is.",
        )
    },
};

pub const PMR: CalcSpec = CalcSpec {
    id: "pmr",
    name: "Proportionate mortality ratio",
    area: Area::Research,
    fields: &[count("cause_deaths", "Deaths from the cause"), count("all_deaths", "Deaths from all causes")],
    source: "Research L02: measures of disease frequency, slide 59",
    compute: |i| {
        rate(
            i,
            ("cause_deaths", "Deaths from the cause"),
            ("all_deaths", "All deaths"),
            100.0,
            "pmr.ratio",
            "Proportionate mortality (%)",
            "The cause's share of all deaths. It needs no population count.",
        )
    },
};

pub const IMR: CalcSpec = CalcSpec {
    id: "imr",
    name: "Infant mortality rate",
    area: Area::Research,
    fields: &[
        count("infant_deaths", "Deaths under one year"),
        count("live_births", "Live births, same year"),
    ],
    source: "Research L02: measures of disease frequency, slide 63",
    compute: |i| {
        rate(
            i,
            ("infant_deaths", "Infant deaths"),
            ("live_births", "Live births"),
            1000.0,
            "imr.rate",
            "Infant mortality rate (per 1,000 live births)",
            "Infant deaths per 1,000 live births in a year.",
        )
    },
};

pub const NMR: CalcSpec = CalcSpec {
    id: "nmr",
    name: "Neonatal mortality rate",
    area: Area::Research,
    fields: &[
        count("neonatal_deaths", "Deaths in the first 28 days"),
        count("live_births", "Live births, same year"),
    ],
    source: "Research L02: measures of disease frequency, slide 64",
    compute: |i| {
        rate(
            i,
            ("neonatal_deaths", "Neonatal deaths"),
            ("live_births", "Live births"),
            1000.0,
            "nmr.rate",
            "Neonatal mortality rate (per 1,000 live births)",
            "Deaths from birth to 28 days per 1,000 live births in a year.",
        )
    },
};

/// The 2×2 table, as the slides draw it: a/b exposed (disease yes/no or
/// case/control), c/d not exposed.
const TABLE: &[Field] = &[
    Field { hint: "Exposed, disease (or case)", ..count("a", "a") },
    Field { hint: "Exposed, no disease (or control)", ..count("b", "b") },
    Field { hint: "Not exposed, disease (or case)", ..count("c", "c") },
    Field { hint: "Not exposed, no disease (or control)", ..count("d", "d") },
];

fn table(i: &Inputs) -> Result<[f64; 4], CalcError> {
    Ok([get_count(i, "a")?, get_count(i, "b")?, get_count(i, "c")?, get_count(i, "d")?])
}

fn table_parts([a, b, c, d]: [f64; 4]) -> Vec<Part> {
    vec![Part::new("a", a), Part::new("b", b), Part::new("c", c), Part::new("d", d)]
}

/// RR or OR read the slides' way.
fn association(v: f64, prefix: &'static str) -> (&'static str, &'static str, &'static str) {
    let (band, label, meaning) = if v > 1.0 {
        ("risk", "Above 1: risk factor", "The exposure goes with more disease.")
    } else if v < 1.0 {
        ("protective", "Below 1: protective factor", "The exposure goes with less disease.")
    } else {
        ("none", "Equal to 1: no association", "The exposure makes no difference.")
    };
    let band = match (prefix, band) {
        ("rr", "risk") => "relative_risk.risk",
        ("rr", "protective") => "relative_risk.protective",
        ("rr", _) => "relative_risk.none",
        (_, "risk") => "odds_ratio.risk",
        (_, "protective") => "odds_ratio.protective",
        _ => "odds_ratio.none",
    };
    (band, label, meaning)
}

pub const RELATIVE_RISK: CalcSpec = CalcSpec {
    id: "relative_risk",
    name: "Relative risk (cohort)",
    area: Area::Research,
    fields: TABLE,
    source: "Research L05: analytic studies, slides 50–53",
    compute: |i| {
        let t @ [a, b, c, d] = table(i)?;
        if a + b == 0.0 || c + d == 0.0 {
            return Err(CalcError::Invalid { field: "a", reason: "each row needs people in it" });
        }
        let ie = a / (a + b);
        let io = c / (c + d);
        if io == 0.0 {
            return Err(CalcError::Invalid {
                field: "c",
                reason: "no disease among the unexposed: RR is undefined",
            });
        }
        let rr = round2(ie / io);
        let (band, label, meaning) = association(rr, "rr");
        let mut notes = vec![
            format!("Ie = a/(a+b) = {}%, Io = c/(c+d) = {}%", show2(ie * 100.0), show2(io * 100.0)),
            format!("Attributable risk Ie − Io = {}%", show2((ie - io) * 100.0)),
        ];
        if ie > 0.0 && ie > io {
            notes
                .push(format!("Attributable risk percent (Ie − Io)/Ie = {}%", show2((ie - io) / ie * 100.0)));
        }
        Ok(Outcome {
            score: Some(rr),
            display: format!("RR = {}", show2(rr)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: table_parts(t),
            notes,
        })
    },
};

pub const ODDS_RATIO: CalcSpec = CalcSpec {
    id: "odds_ratio",
    name: "Odds ratio (case-control)",
    area: Area::Research,
    fields: TABLE,
    source: "Research L05: analytic studies, slides 32–34",
    compute: |i| {
        let t @ [a, b, c, d] = table(i)?;
        if b * c == 0.0 {
            return Err(CalcError::Invalid { field: "b", reason: "b × c is 0: OR is undefined" });
        }
        let or = round2(a * d / (b * c));
        let (band, label, meaning) = association(or, "or");
        Ok(Outcome {
            score: Some(or),
            display: format!("OR = {}", show2(or)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: table_parts(t),
            notes: vec![format!("OR = ad / bc = ({a} × {d}) / ({b} × {c})")],
        })
    },
};

pub const ATTRIBUTABLE_RISK: CalcSpec = CalcSpec {
    id: "attributable_risk",
    name: "Attributable risk",
    area: Area::Research,
    fields: &[
        num("incidence_exposed", "Incidence among exposed (Ie)", "per 1,000", 0.0, 1000.0, 0.1),
        num("incidence_unexposed", "Incidence among non-exposed (Io)", "per 1,000", 0.0, 1000.0, 0.1),
    ],
    source: "Research L05: analytic studies, slide 51",
    compute: |i| {
        let ie = i.get_num("incidence_exposed")?;
        let io = i.get_num("incidence_unexposed")?;
        if ie < 0.0 || io < 0.0 {
            return Err(CalcError::Invalid { field: "incidence_exposed", reason: "can't be below zero" });
        }
        let ar = round2(ie - io);
        let mut notes = Vec::new();
        if ie > 0.0 {
            notes.push(format!(
                "Attributable risk percent = (Ie − Io)/Ie × 100 = {}%",
                show2((ie - io) / ie * 100.0)
            ));
        }
        Ok(Outcome {
            score: Some(ar),
            display: format!("{} per 1,000", show2(ar)),
            band: "attributable_risk.ar",
            label: "Attributable risk (Ie − Io)",
            meaning: "The extra disease among the exposed that the exposure accounts for.".to_owned(),
            page: None,
            parts: vec![Part::new("Ie", ie), Part::new("Io", io)],
            notes,
        })
    },
};

pub const NNT: CalcSpec = CalcSpec {
    id: "nnt",
    name: "NNT / NNH",
    area: Area::Research,
    fields: &[
        num("control_event_rate", "Event rate, control group (CER)", "%", 0.0, 100.0, 0.1),
        num("experimental_event_rate", "Event rate, treated group (EER)", "%", 0.0, 100.0, 0.1),
    ],
    source: "Research L06–L07: experimental studies (NNT = 1/ARR)",
    compute: |i| {
        let cer = i.get_num("control_event_rate")?;
        let eer = i.get_num("experimental_event_rate")?;
        if !(0.0..=100.0).contains(&cer) || !(0.0..=100.0).contains(&eer) {
            return Err(CalcError::Invalid { field: "control_event_rate", reason: "must be 0–100%" });
        }
        let arr = cer - eer;
        if arr == 0.0 {
            return Err(CalcError::Invalid {
                field: "experimental_event_rate",
                reason: "same rate in both groups: no NNT",
            });
        }
        let n = (100.0 / arr.abs()).ceil();
        let mut notes = vec![format!("ARR = CER − EER = {}%", show2(arr))];
        if cer > 0.0 {
            notes.push(format!("RRR = ARR / CER = {}%", show2(arr / cer * 100.0)));
        }
        notes.push("Rounded up to a whole patient.".to_owned());
        let (band, label, meaning) = if arr > 0.0 {
            ("nnt.benefit", "NNT: number needed to treat", "Treat this many patients to prevent one event.")
        } else {
            ("nnt.harm", "NNH: number needed to harm", "Treat this many patients and one extra is harmed.")
        };
        Ok(Outcome {
            score: Some(n),
            display: format!("{} {}", if arr > 0.0 { "NNT" } else { "NNH" }, show(n, 0)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: vec![Part::new("CER (%)", cer), Part::new("EER (%)", eer)],
            notes,
        })
    },
};

const MEASURES: &[(&str, &str)] = &[
    ("sensitivity", "Sensitivity"),
    ("specificity", "Specificity"),
    ("ppv", "Positive predictive value"),
    ("npv", "Negative predictive value"),
    ("accuracy", "Accuracy"),
    ("lr_positive", "LR+"),
    ("lr_negative", "LR−"),
];

pub const DIAGNOSTIC: CalcSpec = CalcSpec {
    id: "diagnostic_accuracy",
    name: "Diagnostic accuracy (2×2)",
    area: Area::Research,
    fields: &[
        Field { hint: "Test +, disease +", ..count("tp", "True positives (a)") },
        Field { hint: "Test +, disease −", ..count("fp", "False positives (b)") },
        Field { hint: "Test −, disease +", ..count("fn", "False negatives (c)") },
        Field { hint: "Test −, disease −", ..count("tn", "True negatives (d)") },
        choice("measure", "Answer with", MEASURES),
    ],
    source: "Research L14: measures of diagnostic accuracy",
    compute: |i| {
        let tp = get_count(i, "tp")?;
        let fp = get_count(i, "fp")?;
        let fn_ = get_count(i, "fn")?;
        let tn = get_count(i, "tn")?;
        let measure = i.get_option(
            "measure",
            &[
                ("sensitivity", &["sens"]),
                ("specificity", &["spec"]),
                ("ppv", &["positive_predictive_value"]),
                ("npv", &["negative_predictive_value"]),
                ("accuracy", &[]),
                ("lr_positive", &["lr+", "plr"]),
                ("lr_negative", &["lr-", "nlr"]),
            ],
        )?;
        let pct = |n: f64, d: f64| if d > 0.0 { Some(n / d * 100.0) } else { None };
        let sens = pct(tp, tp + fn_);
        let spec = pct(tn, tn + fp);
        let ppv = pct(tp, tp + fp);
        let npv = pct(tn, tn + fn_);
        let acc = pct(tp + tn, tp + fp + fn_ + tn);
        let lr_pos = match (sens, spec) {
            (Some(se), Some(sp)) if sp < 100.0 => Some(se / (100.0 - sp)),
            _ => None,
        };
        let lr_neg = match (sens, spec) {
            (Some(se), Some(sp)) if sp > 0.0 => Some((100.0 - se) / sp),
            _ => None,
        };
        let (value, band, label, meaning, is_pct) = match measure {
            "sensitivity" => (
                sens,
                "diagnostic_accuracy.sensitivity",
                "Sensitivity",
                "Of those with the disease, the share the test picks up: a/(a+c).",
                true,
            ),
            "specificity" => (
                spec,
                "diagnostic_accuracy.specificity",
                "Specificity",
                "Of those without the disease, the share the test clears: d/(b+d).",
                true,
            ),
            "ppv" => (
                ppv,
                "diagnostic_accuracy.ppv",
                "Positive predictive value",
                "Of the positives, the share who really have it: a/(a+b). Rises with prevalence.",
                true,
            ),
            "npv" => (
                npv,
                "diagnostic_accuracy.npv",
                "Negative predictive value",
                "Of the negatives, the share who really don't: d/(c+d). Falls as prevalence rises.",
                true,
            ),
            "accuracy" => (
                acc,
                "diagnostic_accuracy.accuracy",
                "Accuracy",
                "The share of all results that are right: (a+d)/total.",
                true,
            ),
            "lr_positive" => (
                lr_pos,
                "diagnostic_accuracy.lr_positive",
                "LR+",
                "How much a positive result raises the odds of disease: sensitivity/(1 − specificity).",
                false,
            ),
            _ => (
                lr_neg,
                "diagnostic_accuracy.lr_negative",
                "LR−",
                "How much a negative result lowers the odds of disease: (1 − sensitivity)/specificity.",
                false,
            ),
        };
        let v = round2(
            value.ok_or(CalcError::Invalid { field: "tp", reason: "a needed row or column is all zero" })?,
        );
        let all = |o: Option<f64>, unit: &str| {
            o.map_or("—".to_owned(), |v| format!("{}{unit}", show2(round2(v))))
        };
        let notes = vec![
            format!(
                "Sensitivity {} · Specificity {} · PPV {} · NPV {}",
                all(sens, "%"),
                all(spec, "%"),
                all(ppv, "%"),
                all(npv, "%")
            ),
            format!("Accuracy {} · LR+ {} · LR− {}", all(acc, "%"), all(lr_pos, ""), all(lr_neg, "")),
            format!("Prevalence in this table {}", all(pct(tp + fn_, tp + fp + fn_ + tn), "%")),
        ];
        Ok(Outcome {
            score: Some(v),
            display: if is_pct { format!("{}%", show2(v)) } else { show2(v) },
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: vec![
                Part::new("a (TP)", tp),
                Part::new("b (FP)", fp),
                Part::new("c (FN)", fn_),
                Part::new("d (TN)", tn),
            ],
            notes,
        })
    },
};

pub const Z_SCORE: CalcSpec = CalcSpec {
    id: "z_score",
    name: "Z-score",
    area: Area::Research,
    fields: &[
        num("value", "Observation (x)", "", -1.0e6, 1.0e6, 0.1),
        num("mean", "Mean (μ)", "", -1.0e6, 1.0e6, 0.1),
        num("sd", "Standard deviation (σ)", "", 0.0, 1.0e6, 0.1),
    ],
    source: "Research L16: normal distribution curve and Z-score, slides 17–23 (Ass. Prof. Noha Elshaer)",
    compute: |i| {
        let x = i.get_num("value")?;
        let mean = i.get_num("mean")?;
        let sd = i.get_pos("sd")?;
        let z = round2((x - mean) / sd);
        // L16 slides 18 and 23: within ±2 is ordinary (the middle 95%),
        // beyond ±2 unusual, beyond ±3 extreme (a potential outlier).
        let side = if z > 0.0 {
            "above the mean"
        } else if z < 0.0 {
            "below the mean"
        } else {
            "at the mean"
        };
        let (band, label, meaning) = if z.abs() > 3.0 {
            (
                "z_score.extreme",
                "Extreme (beyond ±3 SD)",
                "an extreme observation; investigate it as a potential outlier.",
            )
        } else if z.abs() > 2.0 {
            (
                "z_score.unusual",
                "Unusual (beyond ±2 SD)",
                "an unusual observation, outside the middle 95% of values.",
            )
        } else {
            (
                "z_score.ordinary",
                "Ordinary (within ±2 SD)",
                "an ordinary value, inside the middle 95% of values.",
            )
        };
        Ok(Outcome {
            score: Some(z),
            display: format!("z = {}", show2(z)),
            band,
            label,
            meaning: format!("{} SD {side}: {meaning}", show2(z.abs())),
            page: None,
            parts: vec![Part::new("x", x), Part::new("Mean", mean), Part::new("SD", sd)],
            notes: vec![
                "±1 SD holds about 68%, ±2 SD about 95%, ±3 SD about 99.7% (L16 slides 11, 15).".to_owned(),
            ],
        })
    },
};

/// How to read each result (the course slides, Lectures 2, 5 and 7). The
/// differential heading does not fit a statistic, so these guides have next
/// steps only. The Z-score bands follow Lecture 16 (slides 18–23). Diagnostic
/// accuracy (Lecture 14) is not taught yet, so it rests on standard texts: Gordis
/// Epidemiology, 6th ed. 2019; Jaeschke et al., JAMA 1994 (likelihood
/// ratios); Pewsner et al., BMJ 2004 (SpPin and SnNout); Šimundić, EJIFCC
/// 2009; Kirkwood and Sterne, Essential Medical Statistics, 2nd ed. 2003;
/// Park's Textbook of Preventive and Social Medicine, 26th ed. 2021; the
/// CDC's Principles of Epidemiology, 3rd ed. 2012; Straus et al.,
/// Evidence-Based Medicine, 5th ed. 2018; WHO, Estimating mortality from
/// COVID-19, 2020.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "incidence.rate",
        Guide {
            next: &[
                "Incidence counts new cases among people at risk: it measures risk, so it is the rate to compare when looking for causes (L02 slides 25, 33)",
                "Prevalence = incidence × mean duration: a long, non-fatal disease piles up cases (L02 slides 37–38)",
                "Compare rates only over the same period and per the same population size (L02 slide 23)",
            ],
            dd: &[],
        },
    ),
    (
        "prevalence.rate",
        Guide {
            next: &[
                "Prevalence counts all existing cases: it shows how much illness needs care, not the risk of getting it (L02 slide 32)",
                "More new cases or longer survival raise it; cure or death lowers it (L02 slides 37–40)",
                "People with the disease moving in or out change it (L02 slide 32)",
            ],
            dd: &[],
        },
    ),
    (
        "prevalence_id.prevalence",
        Guide {
            next: &[
                "P = I × D holds for a disease in a steady state (L02 slide 37; Gordis 2019)",
                "A treatment that prevents death without curing lengthens D and so raises P (L02 slides 40–41)",
                "A rare but long-lasting disease, like diabetes, becomes a big public health problem (L02 slides 38–39)",
            ],
            dd: &[],
        },
    ),
    (
        "attack_rate.rate",
        Guide {
            next: &[
                "An incidence for an outbreak: a defined group over a short time, as a percentage (L02 slides 26–28)",
                "To find the source, compare the attack rate in those who did and did not eat or meet each suspect (CDC 2012)",
            ],
            dd: &[],
        },
    ),
    (
        "sar.rate",
        Guide {
            next: &[
                "Take the primary cases and anyone immune out of the denominator (L02 slides 44, 46)",
                "Count only cases within one incubation period (L02 slides 44–45)",
                "It shows how infectious the disease is among close contacts (CDC 2012)",
            ],
            dd: &[],
        },
    ),
    (
        "cfr.rate",
        Guide {
            next: &[
                "Case fatality measures how severe an acute disease is (L02 slide 61)",
                "It is a share of the cases, not a death rate in the population (L02 slides 53, 60)",
                "Early in an outbreak it misleads: mild cases go uncounted and some deaths come later (WHO 2020)",
            ],
            dd: &[],
        },
    ),
    (
        "pmr.ratio",
        Guide {
            next: &[
                "The share of all deaths due to one cause, as a percentage (L02 slide 59)",
                "Not a risk of dying: it can rise just because other causes fall (Gordis 2019)",
                "It needs the deaths from all causes; the cases of one disease are not enough (L02 slide 62)",
            ],
            dd: &[],
        },
    ),
    (
        "imr.rate",
        Guide {
            next: &[
                "Deaths under 1 year per 1,000 live births in the same year (L02 slide 63)",
                "One of the most sensitive signs of a community's health and living standards (Park 2021)",
                "Split it into neonatal (first 28 days) and later deaths to see where to act (L02 slide 64; Park 2021)",
            ],
            dd: &[],
        },
    ),
    (
        "nmr.rate",
        Guide {
            next: &[
                "Deaths in the first 28 days per 1,000 live births (L02 slide 64)",
                "It mostly reflects care in pregnancy and at birth: low birth weight, birth injury, congenital anomalies (Park 2021)",
            ],
            dd: &[],
        },
    ),
    (
        "relative_risk.risk",
        Guide {
            next: &[
                "RR above 1: the exposure goes with more disease, a risk factor (L05 slide 53)",
                CI,
                CONFOUNDING,
                "The attributable risk percent shows how much of the disease in the exposed comes from the exposure (L05 slide 51)",
            ],
            dd: &[],
        },
    ),
    (
        "relative_risk.protective",
        Guide {
            next: &[
                "RR below 1: the exposure goes with less disease, a protective factor (L05 slide 53)",
                "1 − RR is the relative risk reduction (L07 slide 53)",
                CI,
                CONFOUNDING,
            ],
            dd: &[],
        },
    ),
    (
        "relative_risk.none",
        Guide {
            next: &[
                "RR of 1: no association between the exposure and the disease (L05 slide 53)",
                SMALL_STUDY,
            ],
            dd: &[],
        },
    ),
    (
        "odds_ratio.risk",
        Guide {
            next: &[
                "OR above 1: the exposure is commoner among cases than controls (L05 slides 32–34)",
                RARE,
                CI,
                CASE_CONTROL_BIAS,
            ],
            dd: &[],
        },
    ),
    (
        "odds_ratio.protective",
        Guide {
            next: &[
                "OR below 1: the exposure is rarer among cases than controls, a protective factor (L05 slides 32–33)",
                RARE,
                CI,
                CASE_CONTROL_BIAS,
            ],
            dd: &[],
        },
    ),
    (
        "odds_ratio.none",
        Guide {
            next: &[
                "OR of 1: the exposure is as common among cases as among controls, no association (L05 slide 33)",
                SMALL_STUDY,
                CASE_CONTROL_BIAS,
            ],
            dd: &[],
        },
    ),
    (
        "attributable_risk.ar",
        Guide {
            next: &[
                "The extra disease among the exposed that the exposure accounts for: what removing it could prevent (L05 slide 51)",
                "Divided by Ie it gives the attributable risk percent (L05 slide 51)",
                "Use it for public health impact, and the relative risk for how strong a cause is (Gordis 2019)",
            ],
            dd: &[],
        },
    ),
    (
        "nnt.benefit",
        Guide {
            next: &[
                "Treat this many to prevent one event: the smaller, the better (L07 slides 52–53)",
                "It depends on the baseline risk: the same 50% relative reduction gives NNT 4 at a 50% risk but 4,000 at 0.05% (L07 slide 56)",
                "Read it with the outcome, the follow-up time and the confidence interval (L07 slides 57, 64–65)",
            ],
            dd: &[],
        },
    ),
    (
        "nnt.harm",
        Guide {
            next: &[
                "Treat this many and one extra patient has the event: NNH = 1 / absolute risk increase (Straus 2018)",
                "Weigh it against the NNT for the benefit: are the benefits worth the harm and the cost? (L07 slide 96)",
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.sensitivity",
        Guide {
            next: &[
                "Very sensitive: a negative result helps rule the disease out, SnNout (Pewsner 2004)",
                "Screen with the most sensitive test when missing a case is costly (Gordis 2019)",
                FIXED,
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.specificity",
        Guide {
            next: &[
                "Very specific: a positive result helps rule the disease in, SpPin (Pewsner 2004)",
                "Confirm a positive screen with a more specific test (Gordis 2019)",
                FIXED,
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.ppv",
        Guide {
            next: &[
                "PPV rises with prevalence: where the disease is rare, more positives are false (Gordis 2019)",
                THIS_TABLE,
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.npv",
        Guide {
            next: &[
                "NPV falls as prevalence rises: where the disease is common, more negatives are false (Gordis 2019)",
                THIS_TABLE,
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.accuracy",
        Guide {
            next: &[
                "Accuracy changes with prevalence even when sensitivity and specificity don't: with a rare disease it can look high for a weak test (Šimundić 2009)",
                "So judge a test by its sensitivity, specificity and likelihood ratios (Šimundić 2009)",
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.lr_positive",
        Guide {
            next: &[
                "Above 10 shifts the probability a lot and often settles it; 5–10 moderately; 2–5 a little; 1–2 rarely matters (Jaeschke 1994)",
                ODDS,
            ],
            dd: &[],
        },
    ),
    (
        "diagnostic_accuracy.lr_negative",
        Guide {
            next: &[
                "Below 0.1 shifts the probability a lot and often rules it out; 0.1–0.2 moderately; 0.2–0.5 a little; 0.5–1 rarely matters (Jaeschke 1994)",
                ODDS,
            ],
            dd: &[],
        },
    ),
    (
        "z_score.ordinary",
        Guide {
            next: &[
                "Within ±2 SD sit the ordinary values, about 95% of a normal distribution (L16 slide 23)",
                "Read the sign and size: +1.2 is well above average, −0.4 slightly below (L16 slide 20)",
                NORMAL_ONLY,
            ],
            dd: &[],
        },
    ),
    (
        "z_score.unusual",
        Guide {
            next: &[
                "Over 2 or under −2 is unusual: outside the middle 95% (L16 slides 18, 23); for a two-sided test at 5% the exact cut-off is ±1.96 (Kirkwood 2003)",
                "+2.4 reads as unusually above average (L16 slide 20)",
                NORMAL_ONLY,
            ],
            dd: &[],
        },
    ),
    (
        "z_score.extreme",
        Guide {
            next: &[
                "Over 3 or under −3 is extreme: investigate it as a potential outlier (L16 slide 18)",
                "Check it is not a measuring or data-entry error before you keep or drop it (Kirkwood 2003)",
                NORMAL_ONLY,
            ],
            dd: &[],
        },
    ),
];

const CI: &str = "It is significant only if its 95% confidence interval leaves out 1, the value of no effect (L07 slides 64–66)";
const CONFOUNDING: &str = "Rule out confounding: a third factor tied to both, as age is to smoking and lung cancer (L05 slides 22–24)";
const SMALL_STUDY: &str = "A small study can miss a real effect: check the confidence interval and the study's power (L07 slides 5–8, 64–65)";
const RARE: &str = "When the disease is rare, the OR is close to the relative risk (Gordis 2019)";
const CASE_CONTROL_BIAS: &str = "Case-control studies are open to selection and recall bias: check how the controls were chosen (L05 slide 36)";
const FIXED: &str = "Sensitivity and specificity stay the same whatever the prevalence; predictive values change (Gordis 2019)";
const THIS_TABLE: &str = "It holds only at this table's prevalence (shown in the notes); recompute it for your own setting (Gordis 2019)";
const ODDS: &str = "Post-test odds = pre-test odds × LR; unlike predictive values, the LR does not change with prevalence (Jaeschke 1994)";
const NORMAL_ONLY: &str =
    "These cut-offs assume a roughly normal distribution: check the shape of the data first (Kirkwood 2003)";

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spec: &CalcSpec, i: Inputs) -> Outcome {
        (spec.compute)(&i).unwrap()
    }

    fn t(a: f64, b: f64, c: f64, d: f64) -> Inputs {
        Inputs::new().num("a", a).num("b", b).num("c", c).num("d", d)
    }

    #[test]
    fn the_slides_worked_examples() {
        // L02 slide 29: 46 of 75 dinner guests → 61%.
        assert_eq!(
            run(&ATTACK_RATE, Inputs::new().num("cases", 46.0).num("at_risk", 75.0)).display,
            "61.33%"
        );
        // Slide 37: 10 per 1000 per year × 5 years = 50 per 1000.
        let o = run(&PREVALENCE_ID, Inputs::new().num("incidence", 10.0).num("duration_years", 5.0));
        assert_eq!(o.score, Some(50.0));
        // Slide 49: 5 secondary cases, 32 family members, 7 primary → 20%.
        let o = run(
            &SAR,
            Inputs::new().num("secondary_cases", 5.0).num("contacts", 32.0).num("primary_cases", 7.0),
        );
        assert_eq!(o.score, Some(20.0));
        // Slide 60: 5 deaths among 500 → 1%.
        assert_eq!(run(&CFR, Inputs::new().num("deaths", 5.0).num("cases", 500.0)).score, Some(1.0));
        // Slide 63: 270 infant deaths / 30,000 live births → 9 per 1000.
        let o = run(&IMR, Inputs::new().num("infant_deaths", 270.0).num("live_births", 30000.0));
        assert_eq!(o.score, Some(9.0));
        // Slide 64: 150 / 30,000 → 5.
        let o = run(&NMR, Inputs::new().num("neonatal_deaths", 150.0).num("live_births", 30000.0));
        assert_eq!(o.score, Some(5.0));
        // L05 slide 34: alcohol and laryngeal cancer, OR = 160×110 / 90×40 = 4.89.
        let o = run(&ODDS_RATIO, t(160.0, 90.0, 40.0, 110.0));
        assert_eq!((o.score, o.band), (Some(4.89), "odds_ratio.risk"));
        // L05 slide 54: smoking and lung cancer, 20/200 vs 2/200 → RR 10.
        let o = run(&RELATIVE_RISK, t(20.0, 180.0, 2.0, 198.0));
        assert_eq!((o.score, o.band), (Some(10.0), "relative_risk.risk"));
        assert!(o.notes.iter().any(|n| n.contains("90%")), "{:?}", o.notes);
    }

    #[test]
    fn incidence_uses_the_chosen_multiplier() {
        let i = Inputs::new().num("new_cases", 200.0).num("population_at_risk", 210_000.0);
        assert_eq!(run(&INCIDENCE, i.clone().choice("per", "1000")).score, Some(0.95));
        assert_eq!(run(&INCIDENCE, i.choice("per", "100000")).display, "95.24 per 100,000");
        assert!(
            (INCIDENCE.compute)(
                &Inputs::new().num("new_cases", 5.0).num("population_at_risk", 2.0).choice("per", "100")
            )
            .is_err()
        );
    }

    #[test]
    fn association_bands() {
        assert_eq!(run(&RELATIVE_RISK, t(10.0, 90.0, 10.0, 90.0)).band, "relative_risk.none");
        assert_eq!(run(&ODDS_RATIO, t(10.0, 90.0, 30.0, 70.0)).band, "odds_ratio.protective");
    }

    #[test]
    fn nnt_and_nnh() {
        let o = run(&NNT, Inputs::new().num("control_event_rate", 20.0).num("experimental_event_rate", 15.0));
        assert_eq!((o.score, o.band), (Some(20.0), "nnt.benefit"));
        let o = run(&NNT, Inputs::new().num("control_event_rate", 10.0).num("experimental_event_rate", 13.0));
        assert_eq!((o.score, o.band), (Some(34.0), "nnt.harm"));
    }

    #[test]
    fn diagnostic_accuracy() {
        let base = Inputs::new().num("tp", 90.0).num("fp", 40.0).num("fn", 10.0).num("tn", 860.0);
        assert_eq!(run(&DIAGNOSTIC, base.clone().choice("measure", "sensitivity")).score, Some(90.0));
        assert_eq!(run(&DIAGNOSTIC, base.clone().choice("measure", "specificity")).score, Some(95.56));
        assert_eq!(run(&DIAGNOSTIC, base.clone().choice("measure", "ppv")).score, Some(69.23));
        assert_eq!(run(&DIAGNOSTIC, base.clone().choice("measure", "npv")).score, Some(98.85));
        assert_eq!(run(&DIAGNOSTIC, base.choice("measure", "lr+")).score, Some(20.25));
    }

    #[test]
    fn z_score() {
        let o = run(&Z_SCORE, Inputs::new().num("value", 130.0).num("mean", 100.0).num("sd", 15.0));
        assert_eq!((o.score, o.band), (Some(2.0), "z_score.ordinary"));
        // L16 slide 20: Mean 75, SD 10.
        let at = |x: f64| run(&Z_SCORE, Inputs::new().num("value", x).num("mean", 75.0).num("sd", 10.0));
        assert_eq!((at(87.0).score, at(87.0).band), (Some(1.2), "z_score.ordinary"));
        assert_eq!((at(71.0).score, at(71.0).band), (Some(-0.4), "z_score.ordinary"));
        assert_eq!((at(99.0).score, at(99.0).band), (Some(2.4), "z_score.unusual"));
        assert_eq!((at(39.0).score, at(39.0).band), (Some(-3.6), "z_score.extreme"));
        assert!(at(39.0).meaning.starts_with("3.6 SD below the mean"));
    }
}
