//! Disease duration and the mortality measures of Research L02: the slides'
//! own worked examples, plus edges.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("disease_duration", &["disease_duration.years"])
            .paper(
                "Slide 42: P 80 / I 20 = 4 years",
                ins![prevalence: 80.0, incidence: 20.0],
                "disease_duration.years",
            )
            .score(4.0, 1e-9)
            .noted("mortality")
            .edge(
                "Slide 70: melanoma 90 / 30 = 3 years",
                ins![prevalence: 90.0, incidence: 30.0],
                "disease_duration.years",
            )
            .score(3.0, 1e-9)
            .edge(
                "short disease: 5 / 20 = 0.25 years",
                ins![prevalence: 5.0, incidence: 20.0],
                "disease_duration.years",
            )
            .score(0.25, 1e-9),
        Table::new(
            "death_rate",
            &["death_rate.crude", "death_rate.cause", "death_rate.age", "death_rate.sex"],
        )
        .paper(
            "Slide 54: 12,000 deaths in 2 million = 6 per 1,000",
            ins![kind: "crude", deaths: 12000, population: 2000000, per: "1000"],
            "death_rate.crude",
        )
        .score(6.0, 1e-9)
        .edge(
            "Slide 58: acute MI 2500 in 2.5 million = 1 per 1,000",
            ins![kind: "cause", deaths: 2500, population: 2500000, per: "1000"],
            "death_rate.cause",
        )
        .score(1.0, 1e-9)
        .edge(
            "Slide 56: PCP age 28-47, 750 in 150,000 = 5 per 1,000",
            ins![kind: "age", deaths: 750, population: 150000, per: "1000"],
            "death_rate.age",
        )
        .score(5.0, 1e-9)
        .edge(
            "30 deaths in 200,000 women = 15 per 100,000",
            ins![kind: "sex", deaths: 30, population: 200000, per: "100000"],
            "death_rate.sex",
        )
        .score(15.0, 1e-9)
        .edge(
            "Slide 54 per 100,000: 600",
            ins![kind: "crude", deaths: 12000, population: 2000000, per: "100000"],
            "death_rate.crude",
        )
        .score(600.0, 1e-9),
        Table::new("perinatal_mr", &["perinatal_mr.rate"])
            .paper(
                "40 stillbirths + 20 first-week deaths, 3960 live births = 15 per 1,000",
                ins![late_fetal_deaths: 40, early_neonatal_deaths: 20, live_births: 3960],
                "perinatal_mr.rate",
            )
            .score(15.0, 1e-9)
            .noted("total births")
            .edge(
                "no stillbirths: 20 / 4000 = 5",
                ins![late_fetal_deaths: 0, early_neonatal_deaths: 20, live_births: 4000],
                "perinatal_mr.rate",
            )
            .score(5.0, 1e-9)
            .edge(
                "stillbirths only: 40 / 4000 = 10",
                ins![late_fetal_deaths: 40, early_neonatal_deaths: 0, live_births: 3960],
                "perinatal_mr.rate",
            )
            .score(10.0, 1e-9),
        Table::new("mmr", &["mmr.ratio"])
            .paper(
                "15 maternal deaths, 30,000 live births = 50 per 100,000",
                ins![maternal_deaths: 15, live_births: 30000],
                "mmr.ratio",
            )
            .score(50.0, 1e-9)
            .noted("ratio")
            .edge("1 in 1,000 = 100", ins![maternal_deaths: 1, live_births: 1000], "mmr.ratio")
            .score(100.0, 1e-9)
            .edge("none: 0", ins![maternal_deaths: 0, live_births: 1000], "mmr.ratio")
            .score(0.0, 1e-9),
    ]
}
