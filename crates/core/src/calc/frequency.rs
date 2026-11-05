//! Basics of Research, Lecture 2 (Measures of disease frequency, Prof. Nadia
//! Farghaly): the measures the first calculators in `research.rs` left out.
//!
//! - Disease duration, D = P / I (slides 37, 42 and 70): the slides' P = I × D
//!   solved for D. It holds in a steady state for a rare disease (Freeman
//!   and Hutchison, "Prevalence, incidence and duration", Am J Epidemiol
//!   1980;112:707–723; Gordis, Epidemiology, 6th ed. 2019).
//! - Death rate (slides 53–58, 65, 67): deaths over the midyear population of
//!   the same group, × 1000 on the slides (crude, age-, sex- and
//!   cause-specific) and × 100,000 in the 15–24 examples (CDC, Principles of
//!   Epidemiology in Public Health Practice, 3rd ed. 2012, Lesson 3).
//! - Perinatal mortality rate, period I (slide 66): fetal deaths of 28 weeks
//!   or more + deaths in the first 7 days, over fetal deaths + live births,
//!   × 1000 (Park's Textbook of Preventive and Social Medicine, 26th ed.
//!   2021; WHO, ICD-10 vol. 2, perinatal definitions).
//! - Maternal mortality ratio (slide 68): maternal deaths per 100,000 live
//!   births in the same year and area (WHO, Trends in maternal mortality
//!   2000 to 2020, 2023).
//!
//! The lecture's recording (2026-11-05 upload) works the same examples:
//! 80 / 20 = 4 years, 12,000 / 2,000,000 → 6 per 1000. `page` stays `None`:
//! these are slides, not book pages.

use super::{Area, CalcError, CalcSpec, Guide, Inputs, Outcome, Part, choice, num, show};

const COUNT_MAX: f64 = 1.0e9;

const fn count(key: &'static str, label: &'static str) -> super::Field {
    num(key, label, "", 0.0, COUNT_MAX, 1.0)
}

/// A whole number, 0 or more.
fn get_count(i: &Inputs, key: &'static str) -> Result<f64, CalcError> {
    let v = i.get_num(key)?;
    if v < 0.0 || v.fract() != 0.0 {
        return Err(CalcError::Invalid { field: key, reason: "must be a whole number, 0 or more" });
    }
    Ok(v)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Two decimals without trailing zeros: 6, 0.95, 133.33.
fn show2(v: f64) -> String {
    let s = show(v, 2);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
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

pub const DISEASE_DURATION: CalcSpec = CalcSpec {
    id: "disease_duration",
    name: "Disease duration (D = P / I)",
    area: Area::Research,
    fields: &[
        num("prevalence", "Point prevalence", "per population base", 0.0, 1.0e9, 0.01)
            .hint("Same base as the incidence, e.g. both per 1,000,000"),
        num("incidence", "Annual incidence", "per population base per year", 0.0, 1.0e9, 0.01),
    ],
    source: "Research L02: measures of disease frequency, slides 37, 42, 70 (Freeman and Hutchison 1980)",
    compute: |i| {
        let p = i.get_num("prevalence")?;
        let inc = i.get_num("incidence")?;
        if p < 0.0 {
            return Err(CalcError::Invalid { field: "prevalence", reason: "can't be below zero" });
        }
        if inc <= 0.0 {
            return Err(CalcError::Invalid { field: "incidence", reason: "must be above zero" });
        }
        let d = round2(p / inc);
        Ok(Outcome {
            score: Some(d),
            display: format!("{} years", show2(d)),
            band: "disease_duration.years",
            label: "Average duration of the disease (D = P / I)",
            meaning: "How long, on average, a case stays a case before cure or death.".to_owned(),
            page: None,
            parts: vec![Part::new("Prevalence", p), Part::new("Incidence per year", inc)],
            notes: vec![
                "The mortality rate is not part of it: P = I × D needs only prevalence and incidence."
                    .to_owned(),
                "Holds in a steady state, when incidence and duration stay stable.".to_owned(),
            ],
        })
    },
};

const DEATH_PER: &[(&str, &str)] = &[("1000", "per 1,000"), ("100000", "per 100,000")];

const DEATH_KIND: &[(&str, &str)] = &[
    ("crude", "Crude: all causes, whole population"),
    ("cause", "Cause-specific: one cause, whole population"),
    ("age", "Age-specific: one age group"),
    ("sex", "Sex-specific: one sex"),
];

pub const DEATH_RATE: CalcSpec = CalcSpec {
    id: "death_rate",
    name: "Death rate (crude, cause-, age- or sex-specific)",
    area: Area::Research,
    fields: &[
        choice("kind", "Which rate", DEATH_KIND),
        count("deaths", "Deaths in the year"),
        count("population", "Midyear population")
            .hint("The whole population, or only the age group or sex you count deaths in"),
        choice("per", "Expressed", DEATH_PER),
    ],
    source: "Research L02: measures of disease frequency, slides 53–58, 65, 67 (CDC, Principles of Epidemiology, 3rd ed. 2012)",
    compute: |i| {
        let kind = i.get_choice("kind", &["crude", "cause", "age", "sex"])?;
        let per: f64 = i.get_choice("per", &["1000", "100000"])?.parse().unwrap_or(1000.0);
        let deaths = get_count(i, "deaths")?;
        let pop = get_count(i, "population")?;
        if pop <= 0.0 {
            return Err(CalcError::Invalid { field: "population", reason: "must be above zero" });
        }
        if deaths > pop {
            return Err(CalcError::Invalid { field: "deaths", reason: "can't be more than the population" });
        }
        let v = round2(deaths / pop * per);
        let (band, label, meaning) = match kind {
            "cause" => (
                "death_rate.cause",
                "Cause-specific death rate",
                "Deaths from one cause in the whole population: how much that cause kills the community.",
            ),
            "age" => (
                "death_rate.age",
                "Age-specific death rate",
                "Deaths in one age group over that group's own population.",
            ),
            "sex" => (
                "death_rate.sex",
                "Sex-specific death rate",
                "Deaths in one sex over that sex's own population.",
            ),
            _ => (
                "death_rate.crude",
                "Crude death rate",
                "Deaths from all causes over the whole midyear population.",
            ),
        };
        Ok(Outcome {
            score: Some(v),
            display: format!("{} per {}", show2(v), group(per)),
            band,
            label,
            meaning: meaning.to_owned(),
            page: None,
            parts: vec![Part::new("Deaths", deaths), Part::new("Midyear population", pop)],
            notes: vec!["Per year: deaths in one year over the population at midyear.".to_owned()],
        })
    },
};

pub const PERINATAL: CalcSpec = CalcSpec {
    id: "perinatal_mr",
    name: "Perinatal mortality rate (period I)",
    area: Area::Research,
    fields: &[
        count("late_fetal_deaths", "Fetal deaths, 28 weeks or more"),
        count("early_neonatal_deaths", "Deaths in the first 7 days"),
        count("live_births", "Live births, same period"),
    ],
    source: "Research L02: measures of disease frequency, slide 66 (Park 2021; WHO ICD-10)",
    compute: |i| {
        let fetal = get_count(i, "late_fetal_deaths")?;
        let early = get_count(i, "early_neonatal_deaths")?;
        let live = get_count(i, "live_births")?;
        if live <= 0.0 {
            return Err(CalcError::Invalid { field: "live_births", reason: "must be above zero" });
        }
        if early > live {
            return Err(CalcError::Invalid {
                field: "early_neonatal_deaths",
                reason: "can't be more than the live births",
            });
        }
        let births = fetal + live;
        let v = round2((fetal + early) / births * 1000.0);
        Ok(Outcome {
            score: Some(v),
            display: format!("{} per 1,000 total births", show2(v)),
            band: "perinatal_mr.rate",
            label: "Perinatal mortality rate",
            meaning: "Late fetal and first-week deaths per 1,000 total births (fetal deaths + live births)."
                .to_owned(),
            page: None,
            parts: vec![
                Part::new("Fetal deaths + first-week deaths", fetal + early),
                Part::new("Total births (fetal deaths + live births)", births),
            ],
            notes: vec![
                "The denominator is total births, not live births: stillbirths are in both parts.".to_owned(),
            ],
        })
    },
};

pub const MMR: CalcSpec = CalcSpec {
    id: "mmr",
    name: "Maternal mortality ratio",
    area: Area::Research,
    fields: &[
        count("maternal_deaths", "Maternal deaths in the year"),
        count("live_births", "Live births, same year and area"),
    ],
    source: "Research L02: measures of disease frequency, slide 68 (WHO 2023)",
    compute: |i| {
        let deaths = get_count(i, "maternal_deaths")?;
        let live = get_count(i, "live_births")?;
        if live <= 0.0 {
            return Err(CalcError::Invalid { field: "live_births", reason: "must be above zero" });
        }
        let v = round2(deaths / live * 100_000.0);
        Ok(Outcome {
            score: Some(v),
            display: format!("{} per 100,000 live births", show2(v)),
            band: "mmr.ratio",
            label: "Maternal mortality ratio",
            meaning: "Maternal deaths per 100,000 live births in the same year and area.".to_owned(),
            page: None,
            parts: vec![Part::new("Maternal deaths", deaths), Part::new("Live births", live)],
            notes: vec![
                "A ratio, not a rate: live births stand in for the women at risk (L02 slide 68).".to_owned(),
            ],
        })
    },
};

/// How to read each result. Sources as in the module comment.
pub const GUIDES: &[(&str, Guide)] = &[
    (
        "disease_duration.years",
        Guide {
            next: &[
                "Divide prevalence by incidence only: the mortality rate is a distractor (L02 slides 42, 70)",
                "Check that both are per the same population base before dividing (Gordis 2019)",
                "A longer duration with the same incidence means more prevalence: survival without cure fills the tub (L02 slides 40–41)",
            ],
            dd: &[],
        },
    ),
    (
        "death_rate.crude",
        Guide {
            next: &[
                "A crude rate mixes age structures: compare two populations only after age adjustment (CDC 2012)",
                "Break it down by age, sex or cause to see who is dying and from what (L02 slides 55–58)",
            ],
            dd: &[],
        },
    ),
    (
        "death_rate.cause",
        Guide {
            next: &[
                "The denominator is the whole population; deaths from the cause over all deaths is the PMR instead (L02 slides 53, 59)",
                "Over the people who have the disease, it would be the case fatality rate (L02 slides 53, 60)",
            ],
            dd: &[],
        },
    ),
    (
        "death_rate.age",
        Guide {
            next: &[
                "The denominator is that age group's own midyear population (L02 slide 55)",
                "The group with the most deaths is not always the one with the highest rate (L02 slide 56)",
            ],
            dd: &[],
        },
    ),
    (
        "death_rate.sex",
        Guide {
            next: &[
                "The denominator is that sex's own midyear population (L02 slide 68)",
                "Compare men and women by rates, never by raw death counts (L02 slides 21–22)",
            ],
            dd: &[],
        },
    ),
    (
        "perinatal_mr.rate",
        Guide {
            next: &[
                "It reflects care in late pregnancy and at birth (Park 2021)",
                "Period I runs from 28 weeks of gestation to the 7th day after birth (L02 slide 66)",
            ],
            dd: &[],
        },
    ),
    (
        "mmr.ratio",
        Guide {
            next: &[
                "Per 100,000 live births, unlike infant and neonatal rates per 1,000 (L02 slides 63, 68)",
                "The Sustainable Development Goal target is below 70 per 100,000 live births by 2030 (WHO 2023)",
            ],
            dd: &[],
        },
    ),
];

#[cfg(test)]
mod tests;
