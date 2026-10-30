//! Lesson sorting interaction and response restoration.

use super::*;

/// Sort cards into bins: tap a card, then its bin. Before Check a placed
/// card goes back to the pool on a tap; after Check it is read-only.
#[component]
pub(super) fn Sort(
    prompt: String,
    bins: Vec<String>,
    cards: Vec<SortCard>,
    why: Option<String>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let n = cards.len();
    let order = use_hook(|| shuffled(n, &seed));
    let (saved, save) = use_draft();
    let (put0, checked0) = match saved {
        Draft::Sort { put, checked } if put.len() == n => {
            let all = put.iter().all(Option::is_some);
            (put, checked && all)
        }
        _ => (vec![None; n], false),
    };
    // Where each card was dropped.
    let mut put = use_signal(|| put0);
    let mut held = use_signal(|| None::<usize>);
    let mut checked = use_signal(|| checked0);
    let all_in = put.read().iter().all(Option::is_some);
    let wrong = put.read().iter().enumerate().filter(|(i, b)| **b != Some(cards[*i].bin)).count();
    let key: Vec<usize> = cards.iter().map(|c| c.bin).collect();
    let mut set = move |c: usize, bin: Option<usize>| {
        put.write()[c] = bin;
        save.call(Draft::Sort { put: put.peek().clone(), checked: false });
    };
    use_check_bar(
        move || put.read().iter().all(Option::is_some) && !checked(),
        move |_| {
            checked.set(true);
            let v = put.peek().clone();
            let right = v.iter().zip(&key).all(|(p, b)| *p == Some(*b));
            save.call(Draft::Sort { put: v, checked: true });
            on_answer.call(right);
            cue(right);
        },
    );
    let mut drop_in = move |bin: usize| {
        if let Some(c) = held() {
            if !checked() {
                set(c, Some(bin));
            }
            held.set(None);
        }
    };
    rsx! {
        div { class: "ls-sort",
            p { class: "ls-kicker", {tx::SORT} }
            div { class: "ls-q", Rich { text: prompt } }
            if !all_in {
                p { class: "muted small", {tx::SORT_HINT} }
                div { class: "ls-pool", dir: "ltr", lang: "en",
                    for &c in order.iter().filter(|c| put.read()[**c].is_none()) {
                        button {
                            key: "c{c}",
                            class: "ls-tile",
                            draggable: "true",
                            "data-state": if held() == Some(c) { "picked" } else { "open" },
                            "aria-pressed": if held() == Some(c) { "true" } else { "false" },
                            ondragstart: move |_| held.set(Some(c)),
                            onclick: move |_| held.set(if held() == Some(c) { None } else { Some(c) }),
                            Line { text: cards[c].text.clone() }
                        }
                    }
                }
            }
            div { class: "ls-bins", style: "--bins: {bins.len()}",
                for (b , name) in bins.iter().enumerate() {
                    div {
                        key: "b{b}",
                        class: "ls-bin",
                        "data-armed": if held().is_some() && !checked() { "yes" } else { "no" },
                        // The whole bin takes a held card, not only its title.
                        onclick: move |_| drop_in(b),
                        ondragover: move |e| e.prevent_default(),
                        ondrop: move |e| {
                            e.prevent_default();
                            drop_in(b);
                        },
                        button { class: "ls-bin-head", disabled: held().is_none() || checked(), onclick: move |_| drop_in(b),
                            Line { text: name.clone() }
                        }
                        ul { class: "ls-bin-cards",
                            for &c in order.iter().filter(|c| put.read()[**c] == Some(b)) {
                                li { key: "in{c}",
                                    "data-state": if !checked() { "open" } else if cards[c].bin == b { "right" } else { "wrong" },
                                    if checked() {
                                        span { class: "ls-chiplet-done", Line { text: cards[c].text.clone() } }
                                    } else {
                                        button {
                                            class: "ls-chiplet",
                                            aria_label: "{tx::REMOVE}: {cards[c].text}",
                                            onclick: move |e| {
                                                // Back to the pool, without dropping a held card here.
                                                e.stop_propagation();
                                                set(c, None);
                                            },
                                            Line { text: cards[c].text.clone() }
                                            span { class: "ls-x", aria_hidden: "true", "×" }
                                        }
                                    }
                                    if checked() && cards[c].bin != b {
                                        span { class: "ls-should small", "→ " Line { text: bins[cards[c].bin].clone() } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if checked() {
                p { class: if wrong == 0 { "ls-verdict good" } else { "ls-verdict bad" }, role: "status",
                    {if wrong == 0 { tx::RIGHT.to_string() } else { tx::in_place(n - wrong, n) }}
                }
                if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
            }
        }
    }
}
