//! The calculator form and result card as rendered (Dioxus SSR), and the
//! small helpers they show text with.

use super::*;

fn inputs(pairs: &[(&str, CalcValue)]) -> Inputs {
    Inputs(pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect())
}

fn n(v: f64) -> CalcValue {
    CalcValue::Number(v)
}

fn t(v: &str) -> CalcValue {
    CalcValue::Text(v.to_owned())
}

fn form(id: &'static str, initial: Inputs) -> String {
    without_style(dioxus::ssr::render_element(rsx! { CalcForm { id, initial } }))
}

/// The page without its stylesheet, whose selectors name every state.
fn without_style(html: String) -> String {
    match (html.find("<style"), html.find("</style>")) {
        (Some(a), Some(b)) => format!("{}{}", &html[..a], &html[b + "</style>".len()..]),
        _ => html,
    }
}

/// The part of the page for one field, from its row to the next row.
fn field_html<'a>(html: &'a str, key: &str) -> &'a str {
    let start = html.find(&format!("id=\"l-{key}\"")).unwrap_or_else(|| panic!("no field {key}"));
    let rest = &html[start..];
    let end = rest[1..].find("calc-field\"").map_or(rest.len(), |e| e + 1);
    &rest[..end]
}

/// Child-Pugh B, 8 points: the book's vignette hb.L01.200.
fn child_pugh_b() -> Inputs {
    inputs(&[
        ("encephalopathy", t("grade_1_2")),
        ("ascites", t("absent")),
        ("bilirubin_mg_dl", n(1.8)),
        ("albumin_g_dl", n(2.9)),
        ("pt_prolonged_s", n(5.0)),
    ])
}

#[test]
fn an_empty_form_names_the_calculator_and_counts_what_is_left() {
    let html = form("child_pugh", Inputs::new());
    assert!(html.contains("Child-Pugh"), "{html}");
    assert!(html.contains(tx::blurb("child_pugh").unwrap()));
    assert!(html.contains("data-state=\"waiting\""));
    assert!(!html.contains("data-state=\"ready\""));
    // Encephalopathy, ascites, bilirubin and albumin; PT and INR are optional.
    assert!(html.contains(&tx::left(4)), "{html}");
    assert!(html.contains(&format!("{}: ", tx::MISSING)));
    // Both optional fields carry the tag, and the Arabic hints replace the English.
    assert_eq!(html.matches(tx::OPTIONAL).count(), 2);
    assert!(html.contains(tx::hint("child_pugh", "pt_prolonged_s").unwrap()));
    assert!(!html.contains("Leave empty to use INR"));
    // A number box carries its hard limits and step.
    let bili = field_html(&html, "bilirubin_mg_dl");
    for attr in ["min=\"0.1\"", "max=\"50\"", "step=\"0.1\"", "inputmode=\"decimal\"", "mg/dL"] {
        assert!(bili.contains(attr), "missing {attr}: {bili}");
    }
    assert!(html.contains(tx::RESET));
}

#[test]
fn a_filled_form_shows_the_result_bar_and_what_it_means() {
    let html = form("child_pugh", child_pugh_b());
    assert!(html.contains("data-state=\"ready\""), "{html}");
    assert!(html.contains("<bdi class=\"calc-display\">8</bdi>"), "{html}");
    assert!(html.contains("Class B (7–9 points)"));
    assert!(html.contains(tx::MEANS));
    assert!(html.contains("1- and 2-year survival about 80% and 60%."));
    assert!(html.contains(tx::BREAKDOWN));
    assert!(html.contains("Prothrombin time prolonged"));
    assert!(html.contains("Book Lecture 1, Table 2 (p. 14)"));
    // The bar shows the number, so the card under it doesn't repeat it.
    assert!(!html.contains("calc-headline"));
    assert!(!html.contains(&tx::left(1)));
}

#[test]
fn an_impossible_number_is_marked_on_its_field_with_the_possible_range() {
    let mut i = child_pugh_b();
    i.0.insert("albumin_g_dl".into(), n(29.0));
    let html = form("child_pugh", i);
    let albumin = field_html(&html, "albumin_g_dl");
    assert!(albumin.contains("data-invalid=\"true\"") || html.contains("data-invalid=\"true\""), "{html}");
    assert!(albumin.contains("aria-invalid=\"true\""), "{albumin}");
    assert!(albumin.contains(&tx::impossible("0.5–6", "g/dL")), "{albumin}");
    assert_eq!(html.matches("aria-invalid=\"true\"").count(), 1);
    assert!(html.contains("data-state=\"waiting\""));
    assert!(html.contains(&format!("{}: ", tx::INVALID)));
    assert!(html.contains(tx::FILL), "nothing required is missing: {html}");
}

#[test]
fn a_refusal_between_two_fields_is_said_in_arabic() {
    let html = form("saag", inputs(&[("serum_albumin_g_dl", n(2.0)), ("ascitic_albumin_g_dl", n(3.0))]));
    let ascitic = field_html(&html, "ascitic_albumin_g_dl");
    let reason = tx::reason("higher than the serum albumin").unwrap();
    assert!(ascitic.contains(reason), "{ascitic}");
}

#[test]
fn the_normal_range_line_says_which_way_a_value_is_out() {
    let mut i = child_pugh_b();
    i.0.insert("bilirubin_mg_dl".into(), n(3.0));
    i.0.insert("albumin_g_dl".into(), n(2.0));
    let html = form("child_pugh", i);
    let bili = field_html(&html, "bilirubin_mg_dl");
    assert!(bili.contains("data-place=\"high\"") && bili.contains(tx::HIGH), "{bili}");
    let alb = field_html(&html, "albumin_g_dl");
    assert!(alb.contains("data-place=\"low\"") && alb.contains(tx::LOW), "{alb}");
    let empty = form("child_pugh", Inputs::new());
    let bili = field_html(&empty, "bilirubin_mg_dl");
    assert!(bili.contains("data-place=\"normal\"") && bili.contains(tx::NORMAL), "{bili}");
}

#[test]
fn sex_specific_ranges_show_both_until_the_sex_is_picked() {
    let unknown = form("glasgow_blatchford", inputs(&[("hb_g_dl", n(12.5))]));
    let hb = field_html(&unknown, "hb_g_dl");
    assert!(hb.contains(tx::MEN) && hb.contains(tx::WOMEN), "{hb}");
    let woman = form("glasgow_blatchford", inputs(&[("hb_g_dl", n(12.5)), ("sex", t("female"))]));
    let hb = field_html(&woman, "hb_g_dl");
    assert!(!hb.contains(tx::MEN) && !hb.contains(tx::WOMEN), "{hb}");
    assert!(hb.contains(tx::NORMAL), "12.5 is normal for a woman: {hb}");
    let man = form("glasgow_blatchford", inputs(&[("hb_g_dl", n(12.5)), ("sex", t("male"))]));
    let hb = field_html(&man, "hb_g_dl");
    assert!(hb.contains(tx::LOW), "12.5 is low for a man: {hb}");
}

#[test]
fn yes_no_fields_press_the_answer_given() {
    let html = form("cam", inputs(&[("acute_fluctuating", CalcValue::Bool(true)), ("inattention", t("no"))]));
    let yes_no = |key: &str| {
        let f = field_html(&html, key);
        let no = f.find(tx::NO).unwrap();
        let yes = f.find(tx::YES).unwrap();
        let pressed =
            |at: usize| f[..at].rfind("aria-pressed=\"true\"") > f[..at].rfind("aria-pressed=\"false\"");
        (pressed(no), pressed(yes))
    };
    assert_eq!(yes_no("acute_fluctuating"), (false, true));
    assert_eq!(yes_no("inattention"), (true, false));
    // Unanswered reads as no.
    assert_eq!(yes_no("disorganised_thinking"), (true, false));
}

#[test]
fn choices_stack_when_long_and_keep_a_content_alias_pressed() {
    let html = form("child_pugh", inputs(&[("ascites", t("mild"))]));
    let ascites = field_html(&html, "ascites");
    assert!(ascites.contains("data-stack=\"true\""), "{ascites}");
    // "mild" is not an option id: it shows as its own pressed button.
    assert!(ascites.contains("<bdi dir=\"ltr\">mild</bdi>"), "{ascites}");
    assert_eq!(ascites.matches("aria-pressed=\"true\"").count(), 1);
    let sex = form("page_b", Inputs::new());
    let sex = field_html(&sex, "sex");
    assert!(sex.contains("data-stack=\"false\""), "{sex}");
    assert_eq!(sex.matches("aria-pressed=\"true\"").count(), 0);
}

fn card(id: &'static str, i: Inputs) -> String {
    let outcome = calc::compute(id, &i).unwrap();
    dioxus::ssr::render_element(rsx! { OutcomeCard { id, outcome } })
}

#[test]
fn the_result_card_shows_the_band_breakdown_and_the_guide() {
    let html = card(
        "cam",
        inputs(&[
            ("acute_fluctuating", CalcValue::Bool(true)),
            ("inattention", CalcValue::Bool(true)),
            ("disorganised_thinking", CalcValue::Bool(true)),
        ]),
    );
    assert!(html.contains("calc-headline") && html.contains("CAM positive"), "{html}");
    assert!(html.contains("aria-live=\"polite\""));
    // Criteria met and not met.
    assert_eq!(html.matches("✓").count(), 3);
    assert_eq!(html.matches("—").count(), 1);
    let guide = calc::guide("cam.positive").unwrap();
    assert!(html.contains(tx::NEXT_STEP) && html.contains(guide.next[0]), "{html}");
    assert!(html.contains(tx::DD) && html.contains(guide.dd[0]));
    // The book page chip opens the geriatrics book.
    assert!(html.contains(tx::SOURCE));
}

#[test]
fn points_print_without_trailing_zeros_and_a_card_without_a_guide_has_none() {
    let i = inputs(&[("bilirubin_mg_dl", n(1.0)), ("inr", n(1.0)), ("creatinine_mg_dl", n(6.0))]);
    let html = card("meld", i.clone());
    assert!(html.contains(">6.43<"), "constant: {html}");
    assert!(html.contains(">0<"), "ln 1 = 0: {html}");
    assert!(html.contains("Creatinine capped at 4 mg/dL."), "note: {html}");
    assert!(html.contains(tx::NEXT_STEP) && html.contains(tx::DD), "meld.15_30 has a guide: {html}");
    // Every real band has a guide (calc::tests::every_result_has_a_next_step),
    // so the empty card is checked with a band that has none.
    let mut outcome = calc::compute("meld", &i).unwrap();
    outcome.band = "meld.no_guide";
    assert!(calc::guide(outcome.band).is_none());
    let html = dioxus::ssr::render_element(rsx! { OutcomeCard { id: "meld", outcome } });
    assert!(!html.contains(tx::NEXT_STEP) && !html.contains(tx::DD), "{html}");
}

#[test]
fn a_guide_with_next_steps_only_shows_no_empty_dd() {
    let html = card("premixed_insulin", inputs(&[("weight_kg", n(80.0))]));
    assert!(html.contains(tx::NEXT_STEP), "{html}");
    assert!(!html.contains(tx::DD));
    assert!(html.contains("16 U before breakfast and 8 U before dinner"), "{html}");
}

#[test]
fn every_calculator_renders_its_empty_form() {
    for spec in calc::all() {
        let html = form(spec.id, Inputs::new());
        let shown = spec.name.replace('&', "&amp;").replace('\'', "&#39;");
        assert!(html.contains(spec.name) || html.contains(&shown), "{}", spec.id);
        for f in spec.fields {
            assert!(html.contains(&format!("id=\"l-{}\"", f.key)), "{}.{}", spec.id, f.key);
        }
        let left = missing(spec, &Inputs::new());
        if left > 0 {
            assert!(html.contains(&tx::left(left)), "{}", spec.id);
        }
    }
}

#[test]
fn ranges_read_the_way_the_line_under_a_box_prints_them() {
    assert_eq!(span(3.5, 5.5, false), "3.5–5.5");
    assert_eq!(span(f64::NEG_INFINITY, 2.0, false), "≤ 2");
    assert_eq!(span(f64::NEG_INFINITY, 150.0, true), "< 150");
    assert_eq!(span(90.0, f64::INFINITY, false), "≥ 90");
    assert_eq!(span(0.0, 0.0, false), "0");
}

#[test]
fn field_values_print_as_typed() {
    assert_eq!(trim_num(5.0), "5");
    assert_eq!(trim_num(0.25), "0.25");
    assert_eq!(trim_num(-3.0), "-3");
    assert_eq!(field_text(Some(&n(2.50))), "2.5");
    assert_eq!(field_text(Some(&t("mild"))), "mild");
    assert_eq!(field_text(Some(&CalcValue::Bool(true))), "yes");
    assert_eq!(field_text(Some(&CalcValue::Bool(false))), "no");
    assert_eq!(field_text(None), "");
    assert!(is_blank(&t("  ")) && !is_blank(&t("0")) && !is_blank(&n(0.0)));
}

#[test]
fn refusals_name_the_field_and_fall_back_to_a_plain_word() {
    let spec = calc::spec("child_pugh").unwrap();
    assert_eq!(error_text(spec, &CalcError::Missing("albumin_g_dl")), (tx::MISSING, "Albumin".to_owned()));
    assert_eq!(
        error_text(spec, &CalcError::Invalid { field: "inr", reason: "x" }),
        (tx::INVALID, "INR".to_owned())
    );
    assert_eq!(error_text(spec, &CalcError::Unknown("zz".into())), (tx::INVALID, "zz".to_owned()));
    // A key the calculator lacks prints as itself.
    assert_eq!(error_text(spec, &CalcError::Missing("other")).1, "other");
    let albumin = spec.fields.iter().find(|f| f.key == "albumin_g_dl").unwrap();
    assert_eq!(why(albumin, calc::IMPOSSIBLE), tx::impossible("0.5–6", "g/dL"));
    assert_eq!(why(albumin, "not a number"), tx::INVALID);
    let enc = spec.fields.iter().find(|f| f.key == "encephalopathy").unwrap();
    assert_eq!(why(enc, calc::IMPOSSIBLE), tx::INVALID, "a choice has no range");
}

#[test]
fn each_area_has_a_label_and_opens_its_own_book() {
    let course = |a| area_course(a);
    assert_eq!(
        AREAS.map(course),
        ["hb", "hb", "hb", "hb", "hb", "git", "neph", "geri", "rheum", "dm", "hem", "res"]
    );
    let labels: std::collections::BTreeSet<&str> = AREAS.iter().map(|a| area_label(*a)).collect();
    assert_eq!(labels.len(), AREAS.len(), "every area has its own label");
}
