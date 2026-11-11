//! Basics of Research measures. The worked examples are the standard
//! textbook ones (Gordis, Epidemiology; the course slides use the same
//! definitions).

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("incidence", &["incidence.rate"])
            .paper(
                "50 new cases in 10,000 at risk = 5 per 1,000",
                ins![new_cases: 50, population_at_risk: 10000, per: "1000"],
                "incidence.rate",
            )
            .score(5.0, 1e-9)
            .edge(
                "per 100: 0.5%",
                ins![new_cases: 50, population_at_risk: 10000, per: "100"],
                "incidence.rate",
            )
            .score(0.5, 1e-9)
            .edge(
                "per 100,000: 500",
                ins![new_cases: 50, population_at_risk: 10000, per: "100000"],
                "incidence.rate",
            )
            .score(500.0, 1e-9),
        Table::new("prevalence", &["prevalence.rate"])
            .paper(
                "200 existing cases in 5,000 = 4%",
                ins![existing_cases: 200, population: 5000, per: "100"],
                "prevalence.rate",
            )
            .score(4.0, 1e-9)
            .edge(
                "per 1,000: 40",
                ins![existing_cases: 200, population: 5000, per: "1000"],
                "prevalence.rate",
            )
            .score(40.0, 1e-9)
            .edge(
                "per 10,000: 400",
                ins![existing_cases: 200, population: 5000, per: "10000"],
                "prevalence.rate",
            )
            .score(400.0, 1e-9),
        Table::new("prevalence_id", &["prevalence_id.prevalence"])
            .paper(
                "P = I × D: 2 per 1,000 a year for 10 years = 20 per 1,000",
                ins![incidence: 2.0, duration_years: 10.0],
                "prevalence_id.prevalence",
            )
            .score(20.0, 1e-9)
            .noted("steady state")
            .edge("duration 5 years", ins![incidence: 2.0, duration_years: 5.0], "prevalence_id.prevalence")
            .score(10.0, 1e-9)
            .edge("duration 0.5 years", ins![incidence: 2.0, duration_years: 0.5], "prevalence_id.prevalence")
            .score(1.0, 1e-9),
        Table::new("attack_rate", &["attack_rate.rate"])
            .paper(
                "outbreak: 30 ill of 75 who ate the meal = 40%",
                ins![cases: 30, at_risk: 75],
                "attack_rate.rate",
            )
            .score(40.0, 1e-9)
            .edge("1 of 3 = 33.33%", ins![cases: 1, at_risk: 3], "attack_rate.rate")
            .score(33.33, 1e-9)
            .edge("2 of 3 = 66.67%", ins![cases: 2, at_risk: 3], "attack_rate.rate")
            .score(66.67, 1e-9),
        Table::new("sar", &["sar.rate"])
            .paper(
                "4 secondary cases among 10 household contacts with 2 primary cases: 4 / 8 = 50%",
                ins![secondary_cases: 4, contacts: 10, primary_cases: 2],
                "sar.rate",
            )
            .score(50.0, 1e-9)
            .noted("incubation period")
            .edge(
                "1 primary case: 4 / 9",
                ins![secondary_cases: 4, contacts: 10, primary_cases: 1],
                "sar.rate",
            )
            .score(44.44, 1e-9)
            .edge(
                "2 primary cases: 4 / 8",
                ins![secondary_cases: 4, contacts: 10, primary_cases: 2],
                "sar.rate",
            )
            .score(50.0, 1e-9),
        Table::new("cfr", &["cfr.rate"])
            .paper("10 deaths among 200 cases = 5%", ins![deaths: 10, cases: 200], "cfr.rate")
            .score(5.0, 1e-9)
            .edge("no deaths: 0%", ins![deaths: 0, cases: 200], "cfr.rate")
            .score(0.0, 1e-9)
            .edge("all died: 100%", ins![deaths: 200, cases: 200], "cfr.rate")
            .score(100.0, 1e-9),
        Table::new("pmr", &["pmr.ratio"])
            .paper(
                "300 cardiovascular deaths of 1,500 = 20%",
                ins![cause_deaths: 300, all_deaths: 1500],
                "pmr.ratio",
            )
            .score(20.0, 1e-9)
            .edge("1 of 7 = 14.29%", ins![cause_deaths: 1, all_deaths: 7], "pmr.ratio")
            .score(14.29, 1e-9)
            .edge("2 of 7 = 28.57%", ins![cause_deaths: 2, all_deaths: 7], "pmr.ratio")
            .score(28.57, 1e-9),
        Table::new("imr", &["imr.rate"])
            .paper(
                "30 infant deaths per 1,000 live births",
                ins![infant_deaths: 60, live_births: 2000],
                "imr.rate",
            )
            .score(30.0, 1e-9)
            .edge("15 per 1,000", ins![infant_deaths: 30, live_births: 2000], "imr.rate")
            .score(15.0, 1e-9)
            .edge("1 of 3,000 = 0.33 per 1,000", ins![infant_deaths: 1, live_births: 3000], "imr.rate")
            .score(0.33, 1e-9),
        Table::new("nmr", &["nmr.rate"])
            .paper(
                "40 neonatal deaths in 2,000 live births = 20 per 1,000",
                ins![neonatal_deaths: 40, live_births: 2000],
                "nmr.rate",
            )
            .score(20.0, 1e-9)
            .edge("10 per 1,000", ins![neonatal_deaths: 20, live_births: 2000], "nmr.rate")
            .score(10.0, 1e-9)
            .edge("2 of 3,000 = 0.67 per 1,000", ins![neonatal_deaths: 2, live_births: 3000], "nmr.rate")
            .score(0.67, 1e-9),
        Table::new(
            "relative_risk",
            &["relative_risk.risk", "relative_risk.protective", "relative_risk.none"],
        )
        .paper(
            "cohort: 84 of 3,000 smokers vs 87 of 5,000 non-smokers: 2.8% / 1.74% = 1.61",
            ins![a: 84, b: 2916, c: 87, d: 4913],
            "relative_risk.risk",
        )
        .score(1.61, 1e-9)
        .noted("Attributable risk percent (Ie − Io)/Ie = 37.86%")
        .edge("RR 1.0: no association", ins![a: 10, b: 90, c: 10, d: 90], "relative_risk.none")
        .edge("RR 1.1: risk factor", ins![a: 11, b: 89, c: 10, d: 90], "relative_risk.risk")
        .edge("RR 1.0", ins![a: 10, b: 90, c: 10, d: 90], "relative_risk.none")
        .edge("RR 0.9: protective", ins![a: 9, b: 91, c: 10, d: 90], "relative_risk.protective"),
        Table::new("odds_ratio", &["odds_ratio.risk", "odds_ratio.protective", "odds_ratio.none"])
            .paper(
                "case-control: (90 × 40) / (10 × 60) = 6",
                ins![a: 90, b: 10, c: 60, d: 40],
                "odds_ratio.risk",
            )
            .score(6.0, 1e-9)
            .noted("ad / bc")
            .edge("OR 1", ins![a: 50, b: 50, c: 50, d: 50], "odds_ratio.none")
            .edge("OR 1.08: risk", ins![a: 52, b: 48, c: 50, d: 50], "odds_ratio.risk")
            .edge("OR 1", ins![a: 50, b: 50, c: 50, d: 50], "odds_ratio.none")
            .edge("OR 0.92: protective", ins![a: 48, b: 52, c: 50, d: 50], "odds_ratio.protective"),
        Table::new("attributable_risk", &["attributable_risk.ar"])
            .paper(
                "Ie 28.0 − Io 17.4 per 1,000 = 10.6; ARP 37.86%",
                ins![incidence_exposed: 28.0, incidence_unexposed: 17.4],
                "attributable_risk.ar",
            )
            .score(10.6, 1e-9)
            .noted("37.86%")
            .edge(
                "equal incidence: 0",
                ins![incidence_exposed: 10.0, incidence_unexposed: 10.0],
                "attributable_risk.ar",
            )
            .score(0.0, 1e-9)
            .edge(
                "Ie above Io: 5",
                ins![incidence_exposed: 15.0, incidence_unexposed: 10.0],
                "attributable_risk.ar",
            )
            .score(5.0, 1e-9),
        Table::new("nnt", &["nnt.benefit", "nnt.harm"])
            .paper(
                "CER 20%, EER 15%: ARR 5%, NNT 100/5 = 20; RRR 25%",
                ins![control_event_rate: 20.0, experimental_event_rate: 15.0],
                "nnt.benefit",
            )
            .score(20.0, 0.0)
            .noted("RRR = ARR / CER = 25%")
            .edge(
                "ARR 3%: 33.3 rounds up to 34",
                ins![control_event_rate: 20.0, experimental_event_rate: 17.0],
                "nnt.benefit",
            )
            .score(34.0, 0.0)
            .edge(
                "treatment raises events by 3%: NNH 34",
                ins![control_event_rate: 10.0, experimental_event_rate: 13.0],
                "nnt.harm",
            )
            .score(34.0, 0.0),
        Table::new(
            "diagnostic_accuracy",
            &[
                "diagnostic_accuracy.sensitivity",
                "diagnostic_accuracy.specificity",
                "diagnostic_accuracy.ppv",
                "diagnostic_accuracy.npv",
                "diagnostic_accuracy.accuracy",
                "diagnostic_accuracy.lr_positive",
                "diagnostic_accuracy.lr_negative",
            ],
        )
        .paper(
            "a 90, b 20, c 10, d 80: sensitivity 90/100 = 90%",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "sensitivity"],
            "diagnostic_accuracy.sensitivity",
        )
        .score(90.0, 1e-9)
        .noted("Prevalence in this table 50%")
        .paper(
            "specificity 80/100 = 80%",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "specificity"],
            "diagnostic_accuracy.specificity",
        )
        .score(80.0, 1e-9)
        .paper(
            "PPV 90/110 = 81.82%",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "ppv"],
            "diagnostic_accuracy.ppv",
        )
        .score(81.82, 1e-9)
        .paper(
            "NPV 80/90 = 88.89%",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "npv"],
            "diagnostic_accuracy.npv",
        )
        .score(88.89, 1e-9)
        .paper(
            "accuracy 170/200 = 85%",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "accuracy"],
            "diagnostic_accuracy.accuracy",
        )
        .score(85.0, 1e-9)
        .paper(
            "LR+ 0.9 / 0.2 = 4.5",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "lr_positive"],
            "diagnostic_accuracy.lr_positive",
        )
        .score(4.5, 1e-9)
        .paper(
            "LR− 0.1 / 0.8 = 0.125 → 0.13",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "lr_negative"],
            "diagnostic_accuracy.lr_negative",
        )
        .score(0.13, 1e-9)
        .edge(
            "PPV at 50% prevalence",
            ins![tp: 90, fp: 20, fn: 10, tn: 80, measure: "ppv"],
            "diagnostic_accuracy.ppv",
        )
        .score(81.82, 1e-9)
        .edge(
            "PPV falls at 10% prevalence (same test: 9/(9 + 36) = 20%)",
            ins![tp: 9, fp: 36, fn: 1, tn: 144, measure: "ppv"],
            "diagnostic_accuracy.ppv",
        )
        .score(20.0, 1e-9),
        Table::new("z_score", &["z_score.ordinary", "z_score.unusual", "z_score.extreme"])
            .paper(
                "L16 slide 17: Mohamed's 87 with mean 75 and SD 10 = +1.2, well above average but not unusual",
                ins![value: 87.0, mean: 75.0, sd: 10.0],
                "z_score.ordinary",
            )
            .score(1.2, 1e-9)
            .noted("68%")
            .edge(
                "L16 slide 20: Sarah's 99 = +2.4, unusually above average",
                ins![value: 99.0, mean: 75.0, sd: 10.0],
                "z_score.unusual",
            )
            .score(2.4, 1e-9)
            .edge(
                "L16 slide 20: Fares's 39 = −3.6, extremely below average",
                ins![value: 39.0, mean: 75.0, sd: 10.0],
                "z_score.extreme",
            )
            .score(-3.6, 1e-9)
            .cut(
                "z 2.0 / 2.01 (over 2 is unusual)",
                ins![mean: 100.0, sd: 10.0],
                "value",
                (120.0, "z_score.ordinary"),
                (120.1, "z_score.unusual"),
            )
            .cut(
                "z −2.0 / −2.01",
                ins![mean: 100.0, sd: 10.0],
                "value",
                (80.0, "z_score.ordinary"),
                (79.9, "z_score.unusual"),
            )
            .cut(
                "z 3.0 / 3.01 (over 3 is extreme)",
                ins![mean: 100.0, sd: 10.0],
                "value",
                (130.0, "z_score.unusual"),
                (130.1, "z_score.extreme"),
            )
            .cut(
                "z −3.0 / −3.01",
                ins![mean: 100.0, sd: 10.0],
                "value",
                (70.0, "z_score.unusual"),
                (69.9, "z_score.extreme"),
            ),
    ]
}
