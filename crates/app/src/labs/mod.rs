//! Normal ranges beside lab values in lessons and cards (Mohamed,
//! 2026-11-04): "Na 128 mmol/L" gets a small "طبيعي 135–145" chip, and the
//! value turns blue when low and red when high, as in the calculator boxes.
//! Hover or tap the chip for the unit and the source. The finding and the
//! numbers are `batna_core::labs`.

use batna_core::calc::Place;
use batna_core::labs::{self, Lab, Piece};
use dioxus::prelude::*;

use crate::i18n::labs as tx;

/// The `data-place` value for a low or high value.
pub fn place(p: Option<Place>) -> Option<&'static str> {
    match p {
        Some(Place::Low) => Some("low"),
        Some(Place::High) => Some("high"),
        _ => None,
    }
}

/// The range chip after a value. `bare`: the value had no unit, so the
/// chip names it.
#[component]
pub fn RangeChip(lab: &'static str, bare: bool, place: Option<Place>) -> Element {
    let Some(lab) = labs::get(lab) else { return rsx! {} };
    let text = chip_text(lab, bare);
    let arrow = match place {
        Some(Place::Low) => "↓ ",
        Some(Place::High) => "↑ ",
        _ => "",
    };
    let label = format!("{} {text}", tx::NORMAL);
    rsx! {
        span {
            class: "lab-n",
            dir: "rtl",
            "data-place": self::place(place),
            "data-peek": lab.peek(),
            aria_label: label,
            title: lab.peek(),
            if !arrow.is_empty() {
                span { class: "lab-arrow", aria_hidden: "true", {arrow} }
            }
            span { class: "lab-k", lang: "ar", {tx::NORMAL} }
            " "
            bdi { dir: "ltr", lang: "en", {text} }
        }
    }
}

fn chip_text(lab: &Lab, bare: bool) -> String {
    let mut s = String::new();
    if !lab.note.is_empty() && lab.note != "total" {
        s.push_str(lab.note);
        s.push(' ');
    }
    s.push_str(&lab.range());
    if bare && !lab.unit().is_empty() {
        s.push(' ');
        s.push_str(lab.unit());
    }
    s
}

/// Plain English text (a card's front or back, a stem) with its lab values
/// marked and their chips. Each test's chip shows once.
#[component]
pub fn LabText(text: String) -> Element {
    let pieces = labs::pieces(&text);
    if !pieces.iter().any(|p| matches!(p, Piece::Value { .. })) {
        return rsx! { "{text}" };
    }
    // The comma or full stop right after a value goes before its chip, so a
    // wrapped line never starts with it.
    let mut parts: Vec<(&str, Option<(labs::Hit, bool, &str)>)> = Vec::new();
    let mut carry = "";
    for p in pieces {
        match p {
            Piece::Text(t) => {
                let t = &t[carry.len()..];
                carry = "";
                parts.push((t, None));
            }
            Piece::Value { text: v, hit, chip } => {
                let after = &text[hit.end..];
                let n: usize = if chip {
                    after.chars().take_while(|c| ",.;:)".contains(*c)).map(char::len_utf8).sum()
                } else {
                    0
                };
                carry = &after[..n];
                parts.push((v, Some((hit, chip, carry))));
            }
        }
    }
    rsx! {
        for (i , (t , v)) in parts.into_iter().enumerate() {
            match v {
                None => rsx! { span { key: "{i}", "{t}" } },
                Some((hit, chip, punct)) => rsx! {
                    span { key: "{i}", class: "lab-v", "data-place": place(hit.place), "{t}" }
                    "{punct}"
                    if chip {
                        RangeChip { lab: hit.lab.id, bare: hit.bare, place: hit.place }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(text: &str) -> String {
        let text = text.to_string();
        dioxus::ssr::render_element(rsx! { LabText { text } })
    }

    #[test]
    fn values_get_a_chip_once() {
        let h = html("Na 120 mmol/L, then Na 126 mmol/L.");
        assert_eq!(h.matches("class=\"lab-n\"").count(), 1, "{h}");
        assert_eq!(h.matches("class=\"lab-v\"").count(), 2, "{h}");
        assert!(h.contains("135–145") && h.contains("data-place=\"low\""), "{h}");
        assert!(h.contains("Nephrology book"), "the source is in the peek: {h}");
        // A bare value: the chip names the unit.
        assert!(html("eGFR <30").contains("≥90 mL/min/1.73 m²"));
        assert!(html("fasting glucose 130 mg/dL").contains("fasting 70–100"));
        let h = html("Hb 5.8, WBC 2.6.");
        assert!(h.contains(r#">5.8</span>,<span class="lab-n""#), "the comma comes before the chip: {h}");
        assert!(h.ends_with(".</span>") || h.contains(r#">2.6</span>.<span"#), "{h}");
    }

    #[test]
    fn text_without_values_is_unchanged() {
        assert_eq!(html("Portal hypertension"), "Portal hypertension");
    }
}
