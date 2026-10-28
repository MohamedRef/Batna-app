//! One component per lesson step kind. Each takes the step and calls
//! `on_answer(right)` once when a question step is answered (the player
//! counts first tries of question steps only), or `on_answer(true)` when a
//! reveal opens (that only unlocks Continue).

use batna_core::content::CaseStep;
use batna_core::lesson::{MnemonicPart, SortCard, StepBody, Tone};
use dioxus::prelude::*;

use super::text::{Line, Rich, is_latin};
use crate::i18n::learn as tx;
use crate::order::grade::{misplaced, shuffled};

const LETTERS: [&str; 5] = ["A", "B", "C", "D", "E"];

/// The player's footer check button. A step whose answer is complete but
/// not yet checked (order, sort) puts its check here; the footer then shows
/// its primary as "صحّح" and calls it. One primary, always in the same place.
#[derive(Clone, Copy)]
pub struct CheckBar(pub Signal<Option<Callback<()>>>);

/// Wires a step to the footer: while `can_check` holds, the footer's
/// primary runs `check`.
fn use_check_bar(can_check: impl Fn() -> bool + 'static, check: impl FnMut(()) + 'static) {
    let CheckBar(mut slot) = use_context::<CheckBar>();
    let check = use_callback(check);
    use_effect(move || slot.set(can_check().then_some(check)));
    use_drop(move || slot.set(None));
}

#[component]
pub fn StepView(body: StepBody, seed: String, on_answer: EventHandler<bool>) -> Element {
    match body {
        StepBody::Explain { title, text } => rsx! {
            if let Some(t) = title { h2 { class: "ls-h", Line { text: t } } }
            Rich { text }
        },
        StepBody::Figure { title, svg, caption, alt } => rsx! { Figure { title, svg, caption, alt } },
        StepBody::Reveal { prompt, answer } => rsx! { Reveal { prompt, answer, on_answer } },
        StepBody::Mnemonic { word, parts, hook } => rsx! { Mnemonic { word, parts, hook } },
        StepBody::Aside { tone, text } => rsx! { Aside { tone, text } },
        StepBody::Update { text, reference } => rsx! {
            div { class: "ls-aside", "data-tone": "update",
                p { class: "ls-aside-label", {tx::UPDATE} }
                Rich { text }
                p { class: "muted small", bdi { dir: "ltr", lang: "en", "{reference}" } }
                p { class: "small", {tx::BOOK_WINS} }
            }
        },
        StepBody::Check { question, options, answer, why } => rsx! {
            Check { question, options, answer, why, seed, on_answer }
        },
        StepBody::Order { prompt, steps, why } => rsx! { Order { prompt, steps, why, seed, on_answer } },
        StepBody::Match { prompt, pairs, why } => rsx! { Match { prompt, pairs, why, seed, on_answer } },
        StepBody::Sort { prompt, bins, cards, why } => {
            rsx! { Sort { prompt, bins, cards, why, seed, on_answer } }
        }
        StepBody::Case { stem, steps } => rsx! { MiniCase { stem, steps, seed, on_answer } },
        StepBody::Table { title, columns, rows, hide } => rsx! { Table { title, columns, rows, hide } },
        StepBody::Try { items } => rsx! { super::practice::TryItems { items, on_answer } },
    }
}

#[component]
fn Figure(title: Option<String>, svg: String, caption: String, alt: String) -> Element {
    let mut zoomed = use_signal(|| false);
    rsx! {
        figure { class: "ls-figure", "data-zoomed": if zoomed() { "yes" } else { "no" },
            if let Some(t) = title { h2 { class: "ls-h", Line { text: t } } }
            // Content-made SVG, checked by the validator (no scripts, links or fixed colours).
            button {
                class: "ls-svg",
                role: "img",
                aria_label: "{alt}",
                title: if zoomed() { tx::ZOOM_OUT } else { tx::ZOOM_IN },
                onclick: move |_| zoomed.toggle(),
                div { dir: "ltr", dangerous_inner_html: "{svg}" }
            }
            figcaption { Rich { text: caption } }
        }
    }
}

#[component]
fn Reveal(prompt: String, answer: String, on_answer: EventHandler<bool>) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "ls-reveal",
            p { class: "ls-kicker", {tx::THINK} }
            div { class: "ls-q", Rich { text: prompt } }
            if open() {
                div { class: "ls-answer ls-pop", Rich { text: answer } }
            } else {
                p { class: "muted small", {tx::THINK_HINT} }
                button {
                    class: "btn btn-primary",
                    "aria-keyshortcuts": "Space",
                    onclick: move |_| {
                        open.set(true);
                        on_answer.call(true);
                    },
                    {tx::REVEAL}
                }
            }
        }
    }
}

#[component]
fn Mnemonic(word: String, parts: Vec<MnemonicPart>, hook: Option<String>) -> Element {
    let n = parts.len();
    let mut shown = use_signal(|| vec![false; n]);
    let all = shown.read().iter().all(|s| *s);
    rsx! {
        div { class: "ls-mnemonic",
            p { class: "ls-kicker", {tx::MNEMONIC} }
            div { class: "ls-mn-letters", dir: "ltr", lang: "en",
                for (i , p) in parts.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: "ls-mn-key",
                        "aria-pressed": if shown.read()[i] { "true" } else { "false" },
                        onclick: move |_| shown.write()[i] = true,
                        "{p.key}"
                    }
                }
            }
            p { class: "ls-mn-word", dir: "ltr", lang: "en", "{word}" }
            ol { class: "ls-mn-lines", dir: "ltr", lang: "en",
                for (i , p) in parts.iter().enumerate() {
                    li { key: "{i}", "data-open": if shown.read()[i] { "yes" } else { "no" },
                        span { class: "ls-mn-k", "{p.key}" }
                        if shown.read()[i] {
                            bdi { class: "ls-pop", "{p.text}" }
                        } else {
                            span { class: "muted", "…" }
                        }
                    }
                }
            }
            if !all {
                button { class: "btn-link", onclick: move |_| shown.set(vec![true; n]), {tx::SHOW_ALL} }
            }
            if let Some(h) = hook {
                div { class: "ls-mn-hook", Rich { text: h } }
            }
        }
    }
}

#[component]
fn Aside(tone: Tone, text: String) -> Element {
    let (name, label, glyph) = match tone {
        Tone::Tip => ("tip", tx::TIP, "💡"),
        Tone::Exam => ("exam", tx::EXAM, "🎯"),
        Tone::Trap => ("trap", tx::TRAP, "⚠️"),
        Tone::Analogy => ("analogy", tx::ANALOGY, "🔁"),
        Tone::Joke => ("joke", tx::JOKE, "😄"),
    };
    rsx! {
        div { class: "ls-aside", "data-tone": name,
            p { class: "ls-aside-label", span { aria_hidden: "true", "{glyph} " } {label} }
            Rich { text }
        }
    }
}

#[component]
fn Check(
    question: String,
    options: Vec<String>,
    answer: usize,
    why: Vec<String>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let order = use_hook(|| shuffled(options.len(), &seed));
    let mut picked = use_signal(|| None::<usize>);
    let ltr = options.iter().all(|o| is_latin(o));
    rsx! {
        div { class: "ls-check",
            p { class: "ls-kicker", {tx::CHECK} }
            div { class: "ls-q", dir: if is_latin(&question) { "ltr" } else { "rtl" }, Rich { text: question.clone() } }
            div { class: "ls-options", dir: if ltr { "ltr" } else { "rtl" },
                for (pos , &i) in order.iter().enumerate() {
                    {
                        let state = match picked() {
                            None => "open",
                            Some(_) if i == answer => "right",
                            Some(p) if p == i => "wrong",
                            Some(_) => "other",
                        };
                        rsx! {
                            div { key: "{i}", class: "ls-option-wrap",
                                button {
                                    class: "ls-option",
                                    "data-state": state,
                                    "aria-keyshortcuts": LETTERS.get(pos).copied().unwrap_or(""),
                                    disabled: picked().is_some(),
                                    onclick: move |_| {
                                        if picked().is_none() {
                                            picked.set(Some(i));
                                            on_answer.call(i == answer);
                                            crate::sounds::play(if i == answer { crate::sounds::Cue::Right } else { crate::sounds::Cue::Wrong });
                                        }
                                    },
                                    span { class: "ls-letter", "{LETTERS.get(pos).copied().unwrap_or(\"\")}" }
                                    span { Line { text: options[i].clone() } }
                                }
                                if picked().is_some() && (i == answer || picked() == Some(i)) {
                                    p { class: "ls-why small ls-pop", Line { text: why.get(i).cloned().unwrap_or_default() } }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(p) = picked() {
                p { class: if p == answer { "ls-verdict good" } else { "ls-verdict bad" },
                    {if p == answer { tx::RIGHT } else { tx::NOT_QUITE }}
                }
            }
        }
    }
}

#[component]
fn Order(
    prompt: String,
    steps: Vec<String>,
    why: Option<String>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let n = steps.len();
    let pool = use_hook(|| shuffled(n, &seed));
    let mut placed = use_signal(Vec::<usize>::new);
    let mut checked = use_signal(|| false);
    let wrong = if checked() { misplaced(&placed.read()) } else { Vec::new() };
    let full = placed.read().len() == n;
    use_check_bar(
        move || placed.read().len() == n && !checked(),
        move |_| {
            checked.set(true);
            let right = misplaced(&placed.peek()).is_empty();
            on_answer.call(right);
            crate::sounds::play(if right { crate::sounds::Cue::Right } else { crate::sounds::Cue::Wrong });
        },
    );
    rsx! {
        div { class: "ls-order",
            p { class: "ls-kicker", {tx::ORDER} }
            div { class: "ls-q", Rich { text: prompt } }
            ol { class: "ls-placed", dir: "ltr", lang: "en",
                for (pos , &s) in placed.read().iter().enumerate() {
                    li {
                        key: "{s}",
                        "data-state": if !checked() { "open" } else if wrong.contains(&pos) { "wrong" } else { "right" },
                        button {
                            class: "ls-tile",
                            disabled: checked(),
                            onclick: move |_| {
                                placed.write().remove(pos);
                            },
                            span { class: "ls-num", "{pos + 1}" }
                            bdi { "{steps[s]}" }
                        }
                        if checked() && wrong.contains(&pos) {
                            span { class: "ls-should small", dir: "rtl", lang: "ar", {tx::should_be(s + 1)} }
                        }
                    }
                }
            }
            if !full {
                p { class: "muted small", {tx::ORDER_HINT} }
                div { class: "ls-pool", dir: "ltr", lang: "en",
                    for &s in pool.iter().filter(|s| !placed.read().contains(s)) {
                        button { key: "{s}", class: "ls-tile", onclick: move |_| placed.write().push(s), bdi { "{steps[s]}" } }
                    }
                }
            } else if !checked() {
                button { class: "btn-link", onclick: move |_| placed.write().clear(), {tx::CLEAR} }
            }
            if checked() {
                p { class: if wrong.is_empty() { "ls-verdict good" } else { "ls-verdict bad" },
                    {if wrong.is_empty() { tx::RIGHT.to_string() } else { tx::in_place(n - wrong.len(), n) }}
                }
                if !wrong.is_empty() {
                    ol { class: "ls-key small", dir: "ltr", lang: "en",
                        for (i , s) in steps.iter().enumerate() { li { key: "{i}", "{s}" } }
                    }
                }
                if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
            }
        }
    }
}

#[component]
fn Match(
    prompt: String,
    pairs: Vec<[String; 2]>,
    why: Option<String>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let n = pairs.len();
    let rights = use_hook(|| shuffled(n, &format!("{seed}/r")));
    let mut done = use_signal(Vec::<usize>::new);
    let mut left = use_signal(|| None::<usize>);
    let mut flash = use_signal(|| None::<usize>);
    let mut misses = use_signal(|| 0usize);
    let mut reported = use_signal(|| false);
    let finished = done.read().len() == n;
    let mut tap_right = move |r: usize| {
        let Some(l) = left() else { return };
        if l == r {
            done.write().push(l);
            left.set(None);
            flash.set(None);
            if done.read().len() == n && !reported() {
                reported.set(true);
                let right = misses() == 0;
                on_answer.call(right);
                crate::sounds::play(crate::sounds::Cue::Right);
            }
        } else {
            *misses.write() += 1;
            flash.set(Some(r));
        }
    };
    rsx! {
        div { class: "ls-match",
            p { class: "ls-kicker", {tx::MATCH} }
            div { class: "ls-q", Rich { text: prompt } }
            if !finished { p { class: "muted small", {tx::MATCH_HINT} } }
            div { class: "ls-match-cols", dir: "ltr", lang: "en",
                div { class: "ls-match-col",
                    for (i , p) in pairs.iter().enumerate() {
                        button {
                            key: "l{i}",
                            class: "ls-tile",
                            "data-state": if done.read().contains(&i) { "right" } else if left() == Some(i) { "picked" } else { "open" },
                            disabled: done.read().contains(&i),
                            onclick: move |_| {
                                left.set(Some(i));
                                flash.set(None);
                            },
                            bdi { "{p[0]}" }
                        }
                    }
                }
                div { class: "ls-match-col",
                    for &r in rights.iter() {
                        button {
                            key: "r{r}",
                            class: "ls-tile",
                            "data-state": if done.read().contains(&r) { "right" } else if flash() == Some(r) { "wrong" } else { "open" },
                            disabled: done.read().contains(&r) || left().is_none(),
                            onclick: move |_| tap_right(r),
                            bdi { "{pairs[r][1]}" }
                        }
                    }
                }
            }
            if finished {
                p { class: if misses() == 0 { "ls-verdict good" } else { "ls-verdict" },
                    {if misses() == 0 { tx::RIGHT.to_string() } else { tx::misses(misses()) }}
                }
                if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
            }
        }
    }
}

#[component]
fn Sort(
    prompt: String,
    bins: Vec<String>,
    cards: Vec<SortCard>,
    why: Option<String>,
    seed: String,
    on_answer: EventHandler<bool>,
) -> Element {
    let n = cards.len();
    let order = use_hook(|| shuffled(n, &seed));
    // Where each card was dropped.
    let mut put = use_signal(|| vec![None::<usize>; n]);
    let mut held = use_signal(|| None::<usize>);
    let mut checked = use_signal(|| false);
    let all_in = put.read().iter().all(Option::is_some);
    let wrong = put.read().iter().enumerate().filter(|(i, b)| **b != Some(cards[*i].bin)).count();
    let key: Vec<usize> = cards.iter().map(|c| c.bin).collect();
    use_check_bar(
        move || put.read().iter().all(Option::is_some) && !checked(),
        move |_| {
            checked.set(true);
            let right = put.peek().iter().zip(&key).all(|(p, b)| *p == Some(*b));
            on_answer.call(right);
            crate::sounds::play(if right { crate::sounds::Cue::Right } else { crate::sounds::Cue::Wrong });
        },
    );
    let mut drop_in = move |bin: usize| {
        if let Some(c) = held() {
            put.write()[c] = Some(bin);
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
                                    button {
                                        class: "ls-chiplet",
                                        disabled: checked(),
                                        onclick: move |e| {
                                            // Back to the pool, without dropping a held card here.
                                            e.stop_propagation();
                                            put.write()[c] = None;
                                        },
                                        Line { text: cards[c].text.clone() }
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
                p { class: if wrong == 0 { "ls-verdict good" } else { "ls-verdict bad" },
                    {if wrong == 0 { tx::RIGHT.to_string() } else { tx::in_place(n - wrong, n) }}
                }
                if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
            }
        }
    }
}

#[component]
fn MiniCase(stem: String, steps: Vec<CaseStep>, seed: String, on_answer: EventHandler<bool>) -> Element {
    let n = steps.len();
    // Each question's options in a stable shuffled order, like a check step.
    let orders = use_hook(|| {
        steps
            .iter()
            .enumerate()
            .map(|(si, s)| shuffled(s.options.len(), &format!("{seed}.{si}")))
            .collect::<Vec<_>>()
    });
    let mut at = use_signal(|| 0usize);
    // The option picked at each step (a step without options: Some(0) once shown).
    let mut picks = use_signal(|| vec![None::<usize>; n]);
    let mut all_right = use_signal(|| true);
    let mut reported = use_signal(|| false);
    rsx! {
        div { class: "ls-case",
            p { class: "ls-kicker", {tx::CASE} }
            div { class: "ls-case-stem", dir: "ltr", lang: "en", Rich { text: stem } }
            for (si , s) in steps.iter().take(at() + 1).enumerate() {
                {
                    let pick = picks.read()[si];
                    let last = si == at();
                    rsx! {
                        div { key: "{si}", class: "ls-case-step ls-pop",
                            if let Some(r) = &s.reveal { p { class: "ls-case-reveal", dir: "ltr", lang: "en", Line { text: r.clone() } } }
                            p { class: "ls-case-q", dir: "ltr", lang: "en", strong { Line { text: s.question.clone() } } }
                            if s.options.is_empty() {
                                if pick.is_none() {
                                    button { class: "btn btn-secondary", "aria-keyshortcuts": "Space", onclick: move |_| picks.write()[si] = Some(0), {tx::CASE_SHOW} }
                                }
                            } else {
                                div { class: "ls-options", dir: "ltr", lang: "en",
                                    for (pos , &oi) in orders[si].iter().enumerate() {
                                        button {
                                            key: "{oi}",
                                            class: "ls-option",
                                            "aria-keyshortcuts": if last { LETTERS.get(pos).copied().unwrap_or("") } else { "" },
                                            "data-state": match pick {
                                                None => "open",
                                                Some(_) if Some(oi) == s.answer => "right",
                                                Some(p) if p == oi => "wrong",
                                                Some(_) => "other",
                                            },
                                            disabled: pick.is_some(),
                                            onclick: {
                                                let right = Some(oi) == s.answer;
                                                move |_| {
                                                    picks.write()[si] = Some(oi);
                                                    if !right {
                                                        all_right.set(false);
                                                    }
                                                    crate::sounds::play(if right { crate::sounds::Cue::Right } else { crate::sounds::Cue::Wrong });
                                                }
                                            },
                                            span { class: "ls-letter", "{LETTERS.get(pos).copied().unwrap_or(\"\")}" }
                                            Line { text: s.options[oi].clone() }
                                        }
                                    }
                                }
                            }
                            if pick.is_some() {
                                div { class: "ls-why-box ls-pop", dir: "ltr", lang: "en", Rich { text: s.why.clone() } }
                                if last && !reported() {
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| {
                                            if si + 1 < n {
                                                at.set(si + 1);
                                            } else {
                                                reported.set(true);
                                                on_answer.call(all_right());
                                            }
                                        },
                                        {if si + 1 < n { tx::CASE_NEXT } else { tx::CASE_DONE }}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Table(title: String, columns: Vec<String>, rows: Vec<Vec<String>>, hide: Vec<[usize; 2]>) -> Element {
    let mut open = use_signal(Vec::<[usize; 2]>::new);
    let hidden_left = hide.iter().filter(|c| !open.read().contains(c)).count();
    rsx! {
        div { class: "ls-table",
            h2 { class: "ls-h", Line { text: title } }
            if hidden_left > 0 { p { class: "muted small", {tx::TABLE_HINT} } }
            div { class: "ls-table-scroll", dir: "ltr", lang: "en",
                table {
                    thead { tr { for (c , h) in columns.iter().enumerate() { th { key: "{c}", "{h}" } } } }
                    tbody {
                        for (r , row) in rows.iter().enumerate() {
                            tr { key: "{r}",
                                for (c , cell) in row.iter().enumerate() {
                                    if hide.contains(&[r, c]) && !open.read().contains(&[r, c]) {
                                        td { key: "{c}",
                                            button { class: "ls-cell-hidden", onclick: move |_| open.write().push([r, c]), aria_label: tx::REVEAL, "؟" }
                                        }
                                    } else if c == 0 {
                                        th { key: "{c}", scope: "row", "{cell}" }
                                    } else {
                                        td { key: "{c}", "data-opened": if hide.contains(&[r, c]) { "yes" } else { "no" }, "{cell}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if hidden_left > 0 {
                button { class: "btn-link", onclick: move |_| open.set(hide.clone()), {tx::SHOW_ALL} }
            }
        }
    }
}
