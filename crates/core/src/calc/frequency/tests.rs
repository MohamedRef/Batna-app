use super::*;

fn run(spec: &CalcSpec, i: Inputs) -> Outcome {
    (spec.compute)(&i).unwrap()
}

#[test]
fn the_slides_worked_examples() {
    // Slide 42: P 80, I 20 per 10,000,000 → 4 years (mortality 4 ignored).
    let o = run(&DISEASE_DURATION, Inputs::new().num("prevalence", 80.0).num("incidence", 20.0));
    assert_eq!((o.score, o.display.as_str()), (Some(4.0), "4 years"));
    // Slide 70: melanoma, 90 / 30 → 3 years.
    let o = run(&DISEASE_DURATION, Inputs::new().num("prevalence", 90.0).num("incidence", 30.0));
    assert_eq!(o.score, Some(3.0));
    // Slide 54: 12,000 deaths in 2 million → 6 per 1000.
    let crude = Inputs::new().choice("kind", "crude").num("deaths", 12000.0).num("population", 2.0e6);
    let o = run(&DEATH_RATE, crude.choice("per", "1000"));
    assert_eq!((o.score, o.band, o.display.as_str()), (Some(6.0), "death_rate.crude", "6 per 1,000"));
    // Slide 56: PCP age 28–47, 750 / 150,000 → 5 per 1000.
    let age = Inputs::new().choice("kind", "age").num("deaths", 750.0).num("population", 150_000.0);
    assert_eq!(run(&DEATH_RATE, age.choice("per", "1000")).score, Some(5.0));
    // Slide 58: acute MI 2500 / 2.5 million → 1 per 1000.
    let cause = Inputs::new().choice("kind", "cause").num("deaths", 2500.0).num("population", 2.5e6);
    let o = run(&DEATH_RATE, cause.choice("per", "1000"));
    assert_eq!((o.score, o.band), (Some(1.0), "death_rate.cause"));
}

#[test]
fn per_100000_and_the_other_measures() {
    let i = Inputs::new().choice("kind", "sex").num("deaths", 30.0).num("population", 200_000.0);
    let o = run(&DEATH_RATE, i.choice("per", "100000"));
    assert_eq!((o.display.as_str(), o.band), ("15 per 100,000", "death_rate.sex"));
    // 40 stillbirths + 20 first-week deaths, 3960 live births → 60 / 4000 = 15.
    let o = run(
        &PERINATAL,
        Inputs::new()
            .num("late_fetal_deaths", 40.0)
            .num("early_neonatal_deaths", 20.0)
            .num("live_births", 3960.0),
    );
    assert_eq!(o.score, Some(15.0));
    let o = run(&MMR, Inputs::new().num("maternal_deaths", 15.0).num("live_births", 30_000.0));
    assert_eq!(o.display, "50 per 100,000 live births");
}

#[test]
fn bad_inputs_are_refused() {
    let zero_i = Inputs::new().num("prevalence", 80.0).num("incidence", 0.0);
    assert!((DISEASE_DURATION.compute)(&zero_i).is_err());
    let too_many = Inputs::new()
        .choice("kind", "crude")
        .num("deaths", 10.0)
        .num("population", 5.0)
        .choice("per", "1000");
    assert!((DEATH_RATE.compute)(&too_many).is_err());
    let no_births = Inputs::new().num("maternal_deaths", 1.0).num("live_births", 0.0);
    assert!((MMR.compute)(&no_births).is_err());
    let half = Inputs::new()
        .num("late_fetal_deaths", 1.5)
        .num("early_neonatal_deaths", 0.0)
        .num("live_births", 10.0);
    assert!((PERINATAL.compute)(&half).is_err());
}
