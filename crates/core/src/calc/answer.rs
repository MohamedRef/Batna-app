//! Reading a written answer against a computed outcome.

use super::*;

pub(super) const UNIT_SUFFIXES: &[&str] = &[
    "_10e9_l", "_per_mm3", "_mm_hg", "_meq_l", "_mmol_l", "_mg_dl", "_g_dl", "_iu_l", "_u_l", "_years",
    "_points", "_days", "_pct", "_kpa", "_cm", "_kg", "_s", "_l",
];

/// A field key without its unit suffix: `alt_uln_u_l` → `alt_uln`.
pub(super) fn base_key(key: &str) -> &str {
    UNIT_SUFFIXES.iter().find_map(|s| key.strip_suffix(s)).unwrap_or(key)
}

/// Input keys in `inputs` that the calculator does not read: almost always a
/// typo in content.
pub fn unknown_inputs<'a>(spec: &CalcSpec, inputs: &'a Inputs) -> Vec<&'a str> {
    inputs.0.keys().map(String::as_str).filter(|k| !spec.fields.iter().any(|f| f.key == *k)).collect()
}

/// Whether an answer Mohamed typed for a `calc` item is right: anything
/// [`expect_matches`] accepts, or a text that names only the right band
/// ("Child-Pugh C", "class c", "met"). The validator keeps the stricter
/// [`expect_matches`], since content should show the number.
pub fn answer_matches(outcome: &Outcome, typed: &str) -> bool {
    if expect_matches(outcome, typed) {
        return true;
    }
    let named = named_bands(typed, outcome.band);
    !named.is_empty() && named.iter().all(|b| *b == outcome.band)
}

/// Whether a content item's `expect` agrees with the computed outcome. It
/// may be the band id (`child_pugh.b`) or free text that shows the result,
/// such as "Child-Pugh B (8 points)" (contains "8") or "Stage C" for BCLC.
pub fn expect_matches(outcome: &Outcome, expect: &str) -> bool {
    let expect = expect.trim();
    if expect == outcome.band {
        return true;
    }
    // A text that names another band ("Below 9", "Child-Pugh C") is wrong
    // even when one of its numbers equals the score.
    if named_bands(expect, outcome.band).iter().any(|b| *b != outcome.band) {
        return false;
    }
    if let Some(score) = outcome.score {
        // A number written in `expect` equal to the score at the precision it
        // is written with: "0.9" and "0.90" both match 0.898.
        let matches = |n: &String| {
            let dp = n.split_once('.').map_or(0, |(_, f)| f.len()) as i32;
            let scale = 10f64.powi(dp);
            n.parse::<f64>().is_ok_and(|v| ((score * scale).round() - v * scale).abs() < 0.5)
        };
        if numbers(expect).iter().any(matches) {
            return true;
        }
    }
    expect.eq_ignore_ascii_case(&outcome.display) || expect.contains(outcome.label)
}

/// Words that name a band in a written result, per band id. Calculators
/// whose bands are only numbers (MELD, Ranson) have none. Phrases are
/// lower case and match whole words; the longest phrase wins, so "not met"
/// is never read as "met".
pub(super) const BAND_WORDS: &[(&str, &[&str])] = &[
    ("child_pugh.a", &["child-pugh a", "child pugh a", "class a"]),
    ("child_pugh.b", &["child-pugh b", "child pugh b", "class b"]),
    ("child_pugh.c", &["child-pugh c", "child pugh c", "class c"]),
    ("albi.1", &["grade 1"]),
    ("albi.2", &["grade 2"]),
    ("albi.3", &["grade 3"]),
    ("fib4.low", &["low risk"]),
    ("fib4.mid", &["intermediate", "indeterminate"]),
    ("fib4.high", &["high risk"]),
    ("r_ratio.hepatocellular", &["hepatocellular"]),
    ("r_ratio.cholestatic", &["cholestatic"]),
    ("r_ratio.mixed", &["mixed"]),
    ("maddrey.severe", &["severe"]),
    ("maddrey.not_severe", &["not severe"]),
    ("glasgow.high", &["9 or more", "9 or above"]),
    ("glasgow.low", &["below 9", "less than 9"]),
    ("saag.high", &["high"]),
    ("saag.low", &["low"]),
    ("ascitic.no_sbp", &["no sbp"]),
    ("ascitic.bacterascites", &["bacterascites"]),
    ("ascitic.secondary", &["secondary"]),
    ("ascitic.polymicrobial", &["polymicrobial"]),
    ("bclc.0", &["stage 0"]),
    ("bclc.a", &["stage a"]),
    ("bclc.b", &["stage b"]),
    ("bclc.c", &["stage c"]),
    ("bclc.d", &["stage d"]),
    ("milan.within", &["within"]),
    ("milan.outside", &["outside"]),
    ("okuda.1", &["stage i", "stage 1"]),
    ("okuda.2", &["stage ii", "stage 2"]),
    ("okuda.3", &["stage iii", "stage 3"]),
    ("page_b.low", &["low risk", "low"]),
    ("page_b.mid", &["intermediate"]),
    ("page_b.high", &["high risk", "high"]),
    ("kings.met", &["met"]),
    ("kings.not_met", &["not met"]),
    ("clichy.met", &["met"]),
    ("clichy.not_met", &["not met"]),
    ("bisap.severe", &["severe", "2 or more"]),
    ("bisap.mild", &["mild", "not severe"]),
    ("atlanta_2012.mild", &["mild"]),
    ("atlanta_2012.moderately_severe", &["moderately severe", "moderate"]),
    ("atlanta_2012.severe", &["severe"]),
    ("baveno_vi.spare", &["met", "skip", "no endoscopy"]),
    ("baveno_vi.screen", &["not met", "screen"]),
    ("fli.low", &["below 30", "ruled out"]),
    ("fli.mid", &["indeterminate", "intermediate"]),
    ("fli.high", &["60 or above", "likely"]),
    ("sos_criteria.met", &["met"]),
    ("sos_criteria.not_met", &["not met"]),
    ("cha2ds2_vasc.low", &["low risk", "low"]),
    ("cha2ds2_vasc.high", &["high risk", "high"]),
    ("anion_gap.high", &["high"]),
    ("anion_gap.normal", &["normal"]),
    ("anion_gap.low", &["low"]),
    ("expected_compensation.appropriate", &["appropriate"]),
    ("osmolal_gap.present", &["gap present"]),
    ("osmolal_gap.normal", &["no gap"]),
    ("katz_adl.independent", &["independent"]),
    ("katz_adl.partial", &["partly dependent", "partially dependent"]),
    ("katz_adl.very_dependent", &["very dependent"]),
    ("lawton_iadl.independent", &["independent"]),
    ("lawton_iadl.partial", &["partly dependent", "partially dependent"]),
    ("lawton_iadl.dependent", &["dependent"]),
    ("gds15.normal", &["normal", "no depression"]),
    ("gds15.depression", &["suggests depression", "depression", "depressed"]),
    ("mna_sf.normal", &["normal"]),
    ("mna_sf.at_risk", &["at risk"]),
    ("mna_sf.malnourished", &["malnourished", "malnutrition"]),
    ("cfs.1", &["very fit"]),
    ("cfs.2", &["well"]),
    ("cfs.3", &["managing well"]),
    ("cfs.4", &["vulnerable"]),
    ("cfs.5", &["mildly frail"]),
    ("cfs.6", &["moderately frail"]),
    ("cfs.7", &["severely frail"]),
    ("cfs.8", &["very severely frail"]),
    ("cfs.9", &["terminally ill"]),
    ("cam.positive", &["positive", "delirium"]),
    ("cam.negative", &["negative", "no delirium"]),
    ("sarc_f.positive", &["positive"]),
    ("sarc_f.negative", &["negative"]),
    ("mmse.normal", &["normal"]),
    ("mmse.borderline", &["borderline"]),
    ("mmse.marked", &["marked"]),
    ("mmse.severe", &["severe"]),
    ("abi.non_compressible", &["non-compressible", "noncompressible"]),
    ("abi.normal", &["normal"]),
    ("abi.pad", &["pad", "peripheral arterial disease"]),
    ("abi.critical", &["critical"]),
    ("acr_eular_2010_ra.definite", &["definite"]),
    ("acr_eular_2010_ra.not_met", &["not met"]),
    ("caspar.met", &["met", "psoriatic arthritis"]),
    ("caspar.not_met", &["not met"]),
    ("asas_axial_spa.imaging", &["imaging arm"]),
    ("asas_axial_spa.clinical", &["clinical arm", "hla-b27 arm"]),
    ("asas_axial_spa.not_met", &["not met"]),
    ("asas_peripheral_spa.met", &["met"]),
    ("asas_peripheral_spa.not_met", &["not met"]),
    ("modified_new_york.met", &["met", "definite as", "definite"]),
    ("modified_new_york.not_met", &["not met"]),
    ("sle_acr_1997.met", &["met", "4 or more"]),
    ("sle_acr_1997.not_met", &["not met", "below 4"]),
    ("sle_eular_acr_2019.met", &["met", "10 or more"]),
    ("sle_eular_acr_2019.not_met", &["not met"]),
    ("gpa_2022.met", &["met", "5 or more"]),
    ("gpa_2022.not_met", &["not met", "below 5"]),
    ("mpa_2022.met", &["met", "5 or more"]),
    ("mpa_2022.not_met", &["not met", "below 5"]),
    ("effective_osmolality.hhs", &["hhs", "meets"]),
    ("ipss_mf.low", &["low"]),
    ("ipss_mf.int1", &["intermediate-1", "intermediate 1", "int-1"]),
    ("ipss_mf.int2", &["intermediate-2", "intermediate 2", "int-2"]),
    ("ipss_mf.high", &["high"]),
    ("rpi.low", &["below 2", "less than 2", "underproduction", "inadequate"]),
    ("rpi.adequate", &["2 or more", "adequate"]),
    ("rai_cll.0", &["stage 0"]),
    ("rai_cll.1", &["stage i", "stage 1"]),
    ("rai_cll.2", &["stage ii", "stage 2"]),
    ("rai_cll.3", &["stage iii", "stage 3"]),
    ("rai_cll.4", &["stage iv", "stage 4"]),
    ("binet_cll.a", &["stage a"]),
    ("binet_cll.b", &["stage b"]),
    ("binet_cll.c", &["stage c"]),
    ("ann_arbor.1a", &["stage ia", "stage 1a"]),
    ("ann_arbor.1b", &["stage ib", "stage 1b"]),
    ("ann_arbor.2a", &["stage iia", "stage 2a"]),
    ("ann_arbor.2b", &["stage iib", "stage 2b"]),
    ("ann_arbor.3a", &["stage iiia", "stage 3a"]),
    ("ann_arbor.3b", &["stage iiib", "stage 3b"]),
    ("ann_arbor.4a", &["stage iva", "stage 4a"]),
    ("ann_arbor.4b", &["stage ivb", "stage 4b"]),
    ("ipi_nhl.low", &["low risk", "low"]),
    ("ipi_nhl.low_int", &["low-intermediate", "low intermediate"]),
    ("ipi_nhl.high_int", &["high-intermediate", "high intermediate"]),
    ("ipi_nhl.high", &["high risk", "high"]),
    ("who_pv.met", &["met", "meets"]),
    ("who_pv.not_met", &["not met"]),
    ("who_et.met", &["met", "meets"]),
    ("who_et.not_met", &["not met"]),
    ("who_pmf.met", &["met", "meets"]),
    ("who_pmf.not_met", &["not met"]),
    ("kdigo_aki.none", &["no aki"]),
    ("kdigo_aki.1", &["stage 1"]),
    ("kdigo_aki.2", &["stage 2"]),
    ("kdigo_aki.3", &["stage 3"]),
    ("fena.prerenal", &["prerenal", "pre-renal"]),
    ("fena.atn", &["atn", "tubular necrosis"]),
    ("feurea.prerenal", &["prerenal", "pre-renal"]),
    ("feurea.atn", &["atn", "tubular necrosis"]),
    ("aa_gradient.high", &["raised", "high"]),
    ("aa_gradient.normal", &["normal"]),
    ("bmi.class1", &["class i", "class 1"]),
    ("bmi.class2", &["class ii", "class 2"]),
    ("bmi.class3", &["class iii", "class 3"]),
    ("bmi.normal", &["normal"]),
    ("bmi.overweight", &["overweight"]),
    ("bmi.under", &["underweight"]),
    ("bun_creatinine.atn", &["atn", "tubular necrosis"]),
    ("bun_creatinine.indeterminate", &["indeterminate"]),
    ("bun_creatinine.prerenal", &["prerenal", "pre-renal"]),
    ("ckd_epi_2021.g1", &["g1"]),
    ("ckd_epi_2021.g2", &["g2"]),
    ("ckd_epi_2021.g3a", &["g3a"]),
    ("ckd_epi_2021.g3b", &["g3b"]),
    ("ckd_epi_2021.g4", &["g4"]),
    ("ckd_epi_2021.g5", &["g5"]),
    ("dka_severity.mild", &["mild"]),
    ("dka_severity.moderate", &["moderate"]),
    ("dka_severity.not_met", &["not dka", "not met"]),
    ("dka_severity.severe", &["severe"]),
    ("kdigo_ckd.high", &["high risk"]),
    ("kdigo_ckd.low", &["low risk"]),
    ("kdigo_ckd.moderate", &["moderately increased"]),
    ("kdigo_ckd.very_high", &["very high risk", "very high"]),
    ("kyoto_gastritis.cancer_risk", &["cancer risk"]),
    ("kyoto_gastritis.current", &["current infection"]),
    ("kyoto_gastritis.none", &["no h. pylori", "no infection"]),
    ("oakland.admit", &["admit", "above 8"]),
    ("oakland.discharge", &["discharge", "8 or less"]),
    ("plasmic.high", &["high"]),
    ("plasmic.intermediate", &["intermediate"]),
    ("plasmic.low", &["low"]),
    ("rome4_ibs.c", &["ibs-c"]),
    ("rome4_ibs.d", &["ibs-d"]),
    ("rome4_ibs.m", &["ibs-m"]),
    ("rome4_ibs.not_met", &["not met"]),
    ("rome4_ibs.u", &["ibs-u"]),
    ("shock_index.stable", &["stable"]),
    ("shock_index.unstable", &["unstable"]),
    ("uc_activity.mild", &["mild"]),
    ("uc_activity.moderate", &["moderate"]),
    ("uc_activity.severe", &["severe"]),
];

/// The bands of `band`'s calculator that `text` names, longest phrase first.
pub(super) fn named_bands(text: &str, band: &str) -> Vec<&'static str> {
    let calculator = band.split_once('.').map_or(band, |(c, _)| c);
    let mut phrases: Vec<(&'static str, &'static str)> = BAND_WORDS
        .iter()
        .filter(|(b, _)| b.split_once('.').is_some_and(|(c, _)| c == calculator))
        .flat_map(|(b, ws)| ws.iter().map(move |w| (*b, *w)))
        .collect();
    phrases.sort_by_key(|(_, w)| std::cmp::Reverse(w.len()));
    let mut text: Vec<char> = text.to_lowercase().replace('–', "-").chars().collect();
    let mut found = Vec::new();
    for (b, w) in phrases {
        let w: Vec<char> = w.chars().collect();
        let mut i = 0;
        while i + w.len() <= text.len() {
            let edge = |j: Option<&char>| j.is_none_or(|c| !c.is_alphanumeric());
            if text[i..i + w.len()] == w[..]
                && edge(i.checked_sub(1).and_then(|j| text.get(j)))
                && edge(text.get(i + w.len()))
            {
                // Blank it out so a shorter phrase inside it can't match too.
                text[i..i + w.len()].fill(' ');
                found.push(b);
                i += w.len();
            } else {
                i += 1;
            }
        }
    }
    found
}

/// The numbers written in a text, as written ("8", "−2.59", "1.4").
pub(super) fn numbers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars().chain(std::iter::once(' ')) {
        let sign = (c == '-' || c == '−') && cur.is_empty();
        if c.is_ascii_digit() || (c == '.' && !cur.is_empty()) || sign {
            cur.push(if c == '−' { '-' } else { c });
        } else if !cur.is_empty() {
            let n = std::mem::take(&mut cur);
            if n.chars().any(|c| c.is_ascii_digit()) {
                out.push(n.trim_end_matches('.').to_owned());
            }
        }
    }
    out
}
