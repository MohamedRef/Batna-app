//! The `calc` study item: a vignette with numbers. Mohamed computes the score
//! himself (or opens the calculator, pre-filled from the vignette), sees the
//! worked result, then says what it means before the interpretation opens.

use batna_core::calc::{self, Inputs, Outcome};
use batna_core::content::Body;
use dioxus::prelude::*;

use super::{CalcForm, OutcomeCard};
use crate::case::{ItemMeta, effective, find_item, record, use_case_store};
use crate::hover::BookPage;
use crate::i18n::{calc as tx, case as cx};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Computed {
    /// Not yet checked.
    Open,
    /// He typed a result: right or not.
    Typed(bool),
    /// He used the calculator instead of typing.
    Calculator,
}

/// Whether the result he typed is right: anything [`calc::answer_matches`]
/// accepts for the computed outcome, or the item's own `expect` text.
fn typed_right(outcome: Option<&Outcome>, typed: &str, expect: &str) -> bool {
    let t = typed.trim();
    outcome.is_some_and(|o| calc::answer_matches(o, t)) || t.eq_ignore_ascii_case(expect.trim())
}

/// (right, out of) for the review record: two decisions when he typed a
/// result (the number, then what it means), one when he used the calculator.
fn decisions(computed: Computed, meaning_right: bool) -> (u32, u32) {
    match computed {
        Computed::Typed(ok) => (u32::from(ok) + u32::from(meaning_right), 2),
        _ => (u32::from(meaning_right), 1),
    }
}

#[component]
pub fn CalcItem(item_id: String, on_back: EventHandler) -> Element {
    let mut typed = use_signal(String::new);
    let mut computed = use_signal(|| Computed::Open);
    let mut meaning_open = use_signal(|| false);
    let mut verdict = use_signal(|| None::<bool>);
    let mut save_failed = use_signal(|| false);
    let mut started = use_signal(crate::case::now_ms);
    let store = use_case_store();
    let finish = use_callback({
        let item_id = item_id.clone();
        let store = store.clone();
        move |right: bool| {
            if verdict.peek().is_some() {
                return;
            }
            let (got, total) = decisions(computed(), right);
            match record(store.as_ref(), &item_id, got, total, started()) {
                Ok(()) => {
                    verdict.set(Some(right));
                    save_failed.set(false);
                }
                Err(_) => save_failed.set(true),
            }
        }
    });

    let Some((lecture, item)) = find_item(&item_id) else { return rsx! {} };
    let item = effective(store.as_ref(), item);
    let Body::Calc { calculator, stem, question, inputs, expect, interpret } = &item.body else {
        return rsx! {};
    };
    let Some(spec) = calc::spec(calculator) else { return rsx! {} };
    let initial = Inputs::for_spec(spec, inputs);
    let outcome = spec.run(&initial).ok();
    let id = spec.id;
    let expect_text = expect.clone();

    let check = {
        let outcome = outcome.clone();
        move |_| {
            let right = typed_right(outcome.as_ref(), &typed.read(), &expect_text);
            computed.set(Computed::Typed(right));
        }
    };
    let state = computed();

    rsx! {
        style { {super::CSS.text()} }
        article { class: "player calc-item",
            ItemMeta { lecture: lecture.to_owned(), item_id: item.id.clone(), on_back }
            if save_failed() {
                p { class: "verdict bad", role: "alert", {cx::SAVE_FAILED} }
            }
            crate::ui::Card { class: "",
                bdi { class: "case-stem", crate::labs::LabText { text: stem.to_string() } }
                p { class: "case-question", dir: "auto",
                    {question.clone().unwrap_or_else(|| format!("Calculate {}.", spec.name))}
                }
            }
            if state == Computed::Open {
                crate::ui::Card { class: "",
                    label { r#for: "calc-answer", {tx::YOUR_RESULT} }
                    input {
                        id: "calc-answer",
                        dir: "ltr",
                        value: "{typed}",
                        oninput: move |e: FormEvent| typed.set(e.value()),
                    }
                    p { class: "small muted", {tx::YOUR_RESULT_HINT} }
                    div { class: "row",
                        button { class: "btn btn-primary", "aria-keyshortcuts": "Enter", disabled: typed.read().trim().is_empty(), onclick: check, {tx::CHECK} }
                        button { class: "btn btn-secondary", onclick: move |_| computed.set(Computed::Calculator), {tx::USE_CALC} }
                    }
                }
            } else {
                if let Computed::Typed(ok) = state {
                    p { class: if ok { "verdict good" } else { "verdict bad" }, if ok { {tx::MATCH} } else { {tx::NO_MATCH} } }
                }
                if state == Computed::Calculator {
                    CalcForm { id, initial: initial.clone() }
                } else if let Some(o) = outcome.clone() {
                    OutcomeCard { id, outcome: o }
                }
                crate::ui::Card { class: "",
                    h2 { {tx::MEANING} }
                    if !meaning_open() {
                        p { class: "small muted", {tx::MEANING_HINT} }
                        button { class: "btn btn-primary", "aria-keyshortcuts": "Space", onclick: move |_| meaning_open.set(true), {tx::SHOW_MEANING} }
                    } else {
                        bdi { class: "case-why", "{interpret}" }
                        BookPage { page: item.src.page, quote: item.src.quote.clone(), of: item.id.clone() }
                        if let Some(u) = &item.update {
                            p { class: "small update", "{cx::UPDATE}: " bdi { "{u.text} ({u.reference})" } }
                        }
                        match verdict() {
                            None => rsx! {
                                div { class: "row",
                                    button { class: "btn btn-good", "aria-keyshortcuts": "3", onclick: move |_| finish.call(true), {cx::GOT_IT} }
                                    button { class: "btn btn-secondary", "aria-keyshortcuts": "1", onclick: move |_| finish.call(false), {cx::MISSED} }
                                }
                            },
                            Some(_) => rsx! {
                                div { class: "row",
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| {
                                            typed.set(String::new());
                                            computed.set(Computed::Open);
                                            meaning_open.set(false);
                                            verdict.set(None);
                                            started.set(crate::case::now_ms());
                                        },
                                        {cx::AGAIN}
                                    }
                                    button { class: "btn btn-primary", "aria-keyshortcuts": "Enter", onclick: move |_| on_back.call(()), {cx::BACK} }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A content item: Child-Pugh B, 8 points.
    const ITEM: &str = "hb.L01.200";

    fn outcome() -> Outcome {
        let (_, item) = find_item(ITEM).expect("the item is in the bundled content");
        let Body::Calc { calculator, inputs, .. } = &item.body else { panic!("{ITEM} is a calc item") };
        let spec = calc::spec(calculator).unwrap();
        spec.run(&Inputs::for_spec(spec, inputs)).unwrap()
    }

    #[test]
    fn a_typed_result_is_right_by_number_band_or_the_items_own_words() {
        let o = outcome();
        assert_eq!((o.band, o.score), ("child_pugh.b", Some(8.0)));
        for typed in ["8", " 8 ", "Child-Pugh B", "class b", "Child-Pugh B (8 points)"] {
            assert!(typed_right(Some(&o), typed, "Child-Pugh B (8 points)"), "{typed}");
        }
        for typed in ["9", "Class C", "Child-Pugh A (8 points)", ""] {
            assert!(!typed_right(Some(&o), typed, "Child-Pugh B (8 points)"), "{typed}");
        }
        // Without a computed outcome only the item's own text counts.
        assert!(typed_right(None, "child-pugh b (8 points)", " Child-Pugh B (8 points) "));
        assert!(!typed_right(None, "8", "Child-Pugh B (8 points)"));
    }

    #[test]
    fn typing_the_result_counts_two_decisions_and_the_calculator_one() {
        assert_eq!(decisions(Computed::Typed(true), true), (2, 2));
        assert_eq!(decisions(Computed::Typed(true), false), (1, 2));
        assert_eq!(decisions(Computed::Typed(false), true), (1, 2));
        assert_eq!(decisions(Computed::Typed(false), false), (0, 2));
        assert_eq!(decisions(Computed::Calculator, true), (1, 1));
        assert_eq!(decisions(Computed::Calculator, false), (0, 1));
        assert_eq!(decisions(Computed::Open, true), (1, 1));
    }

    #[component]
    fn Host(item_id: String) -> Element {
        rsx! { CalcItem { item_id, on_back: move |_| {} } }
    }

    fn render(item_id: &str) -> String {
        let mut dom = VirtualDom::new_with_props(Host, HostProps { item_id: item_id.to_owned() });
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn the_item_opens_on_the_vignette_with_an_empty_answer() {
        let html = render(ITEM);
        let (_, item) = find_item(ITEM).unwrap();
        let Body::Calc { stem, question, .. } = &item.body else { unreachable!() };
        // Lab values in the stem are marked with their normal range (#623),
        // so the stem is split by those marks: its opening words and a
        // marked value show.
        let opening: String = stem.chars().take(30).collect();
        assert!(html.contains(&opening), "{html}");
        assert!(html.contains("class=\"lab-v\""), "{html}");
        assert!(html.contains(question.as_deref().unwrap()), "{html}");
        for text in [tx::YOUR_RESULT, tx::YOUR_RESULT_HINT, tx::CHECK, tx::USE_CALC] {
            assert!(html.contains(text), "missing {text}");
        }
        // Nothing typed yet: Check is off, and no result or meaning shows.
        assert!(html.contains("disabled"), "{html}");
        for text in [tx::MATCH, tx::NO_MATCH, tx::MEANING, "Class B"] {
            assert!(!html.contains(text), "{text} shows too early");
        }
    }

    #[test]
    fn an_unknown_item_or_a_non_calc_item_renders_nothing() {
        assert!(!render("hb.L99.999").contains("player"));
        assert!(!render("nope").contains("player"));
        let other = crate::content::Library::get()
            .lectures
            .values()
            .flat_map(|l| l.items.iter())
            .find(|i| !matches!(i.body, Body::Calc { .. }))
            .expect("some non-calc item");
        assert!(!render(&other.id).contains("calc-item"));
    }
}
