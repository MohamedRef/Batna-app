//! The Geriatrics book's scales.

use super::{Table, V, ins};

type In = Vec<(&'static str, V)>;

/// Katz with `dependent` activities dependent and the rest independent.
fn katz(dependent: &[&str]) -> In {
    ["bathing", "dressing", "toileting", "transferring", "continence", "feeding"]
        .into_iter()
        .map(|k| (k, V::C(if dependent.contains(&k) { "dependent" } else { "independent" })))
        .collect()
}

/// A woman fully independent in all eight Lawton domains, with `over` swaps.
fn lawton_woman(over: In) -> In {
    let mut i = ins![
        sex: "female", telephone: "own_initiative", shopping: "all", food_preparation: "independent",
        housekeeping: "alone", laundry: "all", transport: "independent", medication: "responsible",
        finances: "independent",
    ];
    for (k, v) in over {
        i.retain(|(key, _)| *key != k);
        i.push((k, v));
    }
    i
}

/// GDS-15 with the first `n` items answered in the depressive direction.
fn gds(n: usize) -> In {
    const ITEMS: [(&str, &str); 15] = [
        ("satisfied", "no"),
        ("dropped_activities", "yes"),
        ("life_empty", "yes"),
        ("bored", "yes"),
        ("good_spirits", "no"),
        ("afraid", "yes"),
        ("happy", "no"),
        ("helpless", "yes"),
        ("stay_home", "yes"),
        ("memory", "yes"),
        ("wonderful_alive", "no"),
        ("worthless", "yes"),
        ("full_of_energy", "no"),
        ("hopeless", "yes"),
        ("others_better_off", "yes"),
    ];
    ITEMS
        .iter()
        .enumerate()
        .map(|(k, (key, dep))| {
            let other = if *dep == "yes" { "no" } else { "yes" };
            (*key, V::C(if k < n { dep } else { other }))
        })
        .collect()
}

/// MMSE with every section full except `attention` and `recall`.
fn mmse(attention: i32, recall: i32, orientation_time: i32) -> In {
    ins![
        orientation_time: orientation_time, orientation_place: 5, registration: 3, attention: attention,
        recall: recall, naming: 2, repetition: 1, command: 3, read_write_copy: 3,
    ]
}

fn sarc(
    strength: &'static str,
    walking: &'static str,
    rise: &'static str,
    stairs: &'static str,
    falls: &'static str,
) -> In {
    ins![strength: strength, walking: walking, rise: rise, stairs: stairs, falls: falls]
}

fn mna(
    a: &'static str,
    b: &'static str,
    c: &'static str,
    d: &'static str,
    e: &'static str,
    f: &'static str,
) -> In {
    ins![food_intake: a, weight_loss: b, mobility: c, stress: d, neuropsych: e, bmi: f]
}

pub fn tables() -> Vec<Table> {
    let mut cfs = Table::new(
        "clinical_frailty",
        &["cfs.1", "cfs.2", "cfs.3", "cfs.4", "cfs.5", "cfs.6", "cfs.7", "cfs.8", "cfs.9"],
    )
    .paper("Rockwood 2005 / book p. 121: level 5, mildly frail", ins![level: "5"], "cfs.5")
    .score(5.0, 0.0)
    .noted("In dementia")
    .edge("level 4: vulnerable, not yet frail", ins![level: "4"], "cfs.4")
    .edge("level 5: the first frail level", ins![level: "5"], "cfs.5");
    for level in ["1", "2", "3", "6", "7", "8", "9"] {
        let band = match level {
            "1" => "cfs.1",
            "2" => "cfs.2",
            "3" => "cfs.3",
            "6" => "cfs.6",
            "7" => "cfs.7",
            "8" => "cfs.8",
            _ => "cfs.9",
        };
        cfs = cfs.paper("each level is its own band (book p. 121)", vec![("level", V::C(level))], band);
    }

    vec![
        Table::new("katz_adl", &["katz_adl.independent", "katz_adl.partial", "katz_adl.very_dependent"])
            .paper("Katz 1963 / book p. 111: dependent in bathing and dressing → 4 of 6", katz(&["bathing", "dressing"]), "katz_adl.partial")
            .score(4.0, 0.0)
            .paper(
                "dependent in all six → 0",
                katz(&["bathing", "dressing", "toileting", "transferring", "continence", "feeding"]),
                "katz_adl.very_dependent",
            )
            .score(0.0, 0.0)
            .edge("6: independent", katz(&[]), "katz_adl.independent")
            .edge("5: one activity dependent", katz(&["feeding"]), "katz_adl.partial")
            .edge("1: one activity left", katz(&["bathing", "dressing", "toileting", "transferring", "continence"]), "katz_adl.partial")
            .edge("0", katz(&["bathing", "dressing", "toileting", "transferring", "continence", "feeding"]), "katz_adl.very_dependent"),
        Table::new("lawton_iadl", &["lawton_iadl.independent", "lawton_iadl.partial", "lawton_iadl.dependent"])
            .paper(
                "Lawton & Brody 1969: woman who dials a few numbers (1), shops for small items (0), the rest independent → 7 / 8",
                lawton_woman(ins![telephone: "few_numbers", shopping: "small"]),
                "lawton_iadl.partial",
            )
            .score(7.0, 0.0)
            .paper(
                "a man is scored out of 5 without food, housekeeping and laundry",
                ins![sex: "male", telephone: "own_initiative", shopping: "all", transport: "taxi", medication: "responsible", finances: "day_to_day"],
                "lawton_iadl.independent",
            )
            .score(5.0, 0.0)
            .paper(
                "a man's food, housekeeping and laundry answers are not scored, and a note says so",
                ins![
                    sex: "male", telephone: "own_initiative", shopping: "all", transport: "independent",
                    medication: "responsible", finances: "independent", food_preparation: "needs_served",
                ],
                "lawton_iadl.independent",
            )
            .noted("not scored for men")
            .edge("woman 8 / 8", lawton_woman(vec![]), "lawton_iadl.independent")
            .edge("woman 7: housekeeping none", lawton_woman(ins![housekeeping: "none"]), "lawton_iadl.partial")
            .edge("housekeeping light_poor still scores 1", lawton_woman(ins![housekeeping: "light_poor"]), "lawton_iadl.independent")
            .edge("laundry by others scores 0", lawton_woman(ins![laundry: "others"]), "lawton_iadl.partial")
            .edge("transport accompanied still scores 1", lawton_woman(ins![transport: "accompanied"]), "lawton_iadl.independent")
            .edge("transport assisted scores 0", lawton_woman(ins![transport: "assisted"]), "lawton_iadl.partial")
            .paper(
                "woman dependent in every domain → 0",
                ins![
                    sex: "female", telephone: "none", shopping: "unable", food_preparation: "needs_served",
                    housekeeping: "none", laundry: "others", transport: "none", medication: "unable", finances: "unable",
                ],
                "lawton_iadl.dependent",
            )
            .score(0.0, 0.0),
        Table::new("gds15", &["gds15.depression", "gds15.normal"])
            .paper("Sheikh & Yesavage 1986: 8 depressive answers → depression", gds(8), "gds15.depression")
            .score(8.0, 0.0)
            .paper("no depressive answer → 0", gds(0), "gds15.normal")
            .score(0.0, 0.0)
            .edge("5: normal (book p. 119: 0–5)", gds(5), "gds15.normal")
            .edge("6: above 5", gds(6), "gds15.depression"),
        Table::new("mna_sf", &["mna_sf.normal", "mna_sf.at_risk", "mna_sf.malnourished"])
            .paper(
                "Kaiser 2009: moderate intake fall (1), 1–3 kg loss (2), goes out (2), no stress (2), no problems (2), BMI 21–23 (2) = 11",
                mna("moderate", "1_3kg", "goes_out", "no", "none", "21_23"),
                "mna_sf.at_risk",
            )
            .score(11.0, 0.0)
            .paper("best on every item → 14", mna("none", "none", "goes_out", "no", "none", "23_plus"), "mna_sf.normal")
            .score(14.0, 0.0)
            .edge("12: normal", mna("none", "none", "goes_out", "no", "mild_dementia", "21_23"), "mna_sf.normal")
            .edge("11: at risk", mna("moderate", "none", "goes_out", "no", "mild_dementia", "21_23"), "mna_sf.at_risk")
            .edge("8: at risk", mna("moderate", "unknown", "indoors", "no", "mild_dementia", "21_23"), "mna_sf.at_risk")
            .edge("7: malnourished", mna("moderate", "unknown", "indoors", "no", "mild_dementia", "19_21"), "mna_sf.malnourished"),
        cfs,
        Table::new("cam", &["cam.positive", "cam.negative"])
            .paper(
                "Inouye 1990: acute fluctuating course, inattention and disorganised thinking → delirium",
                ins![acute_fluctuating: true, inattention: true, disorganised_thinking: true],
                "cam.positive",
            )
            .edge("features 1 and 2 only", ins![acute_fluctuating: true, inattention: true], "cam.negative")
            .edge("1 and 2 with 4", ins![acute_fluctuating: true, inattention: true, altered_consciousness: true], "cam.positive")
            .edge("2, 3 and 4 without 1", ins![inattention: true, disorganised_thinking: true, altered_consciousness: true], "cam.negative")
            .edge("all four", ins![acute_fluctuating: true, inattention: true, disorganised_thinking: true, altered_consciousness: true], "cam.positive"),
        Table::new("sarc_f", &["sarc_f.positive", "sarc_f.negative"])
            .paper(
                "Malmstrom & Morley 2013: some difficulty with strength, rising and stairs (1 each), 1–3 falls (1) = 4",
                sarc("some", "none", "some", "some", "1_3"),
                "sarc_f.positive",
            )
            .score(4.0, 0.0)
            .paper("maximum 10", sarc("a_lot", "a_lot", "a_lot", "a_lot", "4_plus"), "sarc_f.positive")
            .score(10.0, 0.0)
            .edge("3: negative", sarc("some", "none", "some", "some", "none"), "sarc_f.negative")
            .edge("4: positive (book p. 134: 4 or more)", sarc("some", "none", "some", "a_lot", "none"), "sarc_f.positive"),
        Table::new("mmse", &["mmse.normal", "mmse.borderline", "mmse.marked", "mmse.severe"])
            .paper("Folstein 1975: full marks 30", mmse(5, 3, 5), "mmse.normal")
            .score(30.0, 0.0)
            .paper("serial 7s 2 / 5 and recall 1 / 3 → 25, marked", mmse(2, 1, 5), "mmse.marked")
            .score(25.0, 0.0)
            .edge("29: normal (book p. 63: 29–30)", mmse(4, 3, 5), "mmse.normal")
            .edge("28: borderline", mmse(3, 3, 5), "mmse.borderline")
            .edge("26: borderline (26–28)", mmse(1, 3, 5), "mmse.borderline")
            .edge("25: marked", mmse(0, 3, 5), "mmse.marked")
            .edge("18: marked (18–25)", mmse(0, 0, 1), "mmse.marked")
            .edge("17: severe, read from the book's \"below 17\"", mmse(0, 0, 0), "mmse.severe")
            .paper("17 notes the book's gap", mmse(0, 0, 0), "mmse.severe")
            .score(17.0, 0.0)
            .noted("jumps from 18"),
        Table::new("abi", &["abi.normal", "abi.pad", "abi.critical"])
            .paper(
                "AHA/ACC 2016: higher ankle 100 (PT) over higher arm 140 (left) = 0.71, PAD",
                ins![dp_mm_hg: 80, pt_mm_hg: 100, brachial_right_mm_hg: 120, brachial_left_mm_hg: 140],
                "abi.pad",
            )
            .score(0.71, 1e-9)
            .paper(
                "0.91–0.99 is normal by the book, borderline by AHA/ACC",
                ins![dp_mm_hg: 95, brachial_right_mm_hg: 100],
                "abi.normal",
            )
            .noted("borderline")
            .cut("0.90 / 0.91 (book: above 0.9 normal)", ins![brachial_right_mm_hg: 100], "dp_mm_hg", (90.0, "abi.pad"), (91.0, "abi.normal"))
            .cut("0.40 / 0.39 (below 0.4 critical)", ins![brachial_right_mm_hg: 100], "dp_mm_hg", (40.0, "abi.pad"), (39.0, "abi.critical"))
            .cut("1.40 / 1.41: non-compressible note above 1.40", ins![brachial_right_mm_hg: 100], "dp_mm_hg", (140.0, "abi.normal"), (141.0, "abi.normal"))
            .paper("1.41 says non-compressible", ins![dp_mm_hg: 141, brachial_right_mm_hg: 100], "abi.normal")
            .noted("non-compressible"),
    ]
}
