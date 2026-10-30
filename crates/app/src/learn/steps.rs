//! One component per lesson step kind. Each takes the step and calls
//! `on_answer(right)` once when a question step is answered (the player
//! counts first tries of question steps only), or `on_answer(true)` when a
//! reveal opens (that only unlocks Continue).

use batna_core::content::CaseStep;
use batna_core::lesson::{MnemonicPart, Picture, SortCard, StepBody, Tone};
use batna_core::response::{Attempt, Draft};
use dioxus::prelude::*;

use super::text::{Line, Rich, is_latin};
use crate::i18n::learn as tx;
use crate::order::grade::{misplaced, shuffled};

mod decision;
use decision::{answer_state, is_value};

const LETTERS: [&str; 5] = ["A", "B", "C", "D", "E"];

/// The player's footer gate, one slot per step (visited steps stay
/// mounted). A step whose response is complete but not checked yet puts its
/// check here; the footer's primary then reads "صحّح" and runs it: one
/// primary, always in the same place.
#[derive(Clone, Copy)]
pub struct CheckBar(pub Signal<Vec<Gate>>);

#[derive(Clone, Default, PartialEq)]
pub struct Gate {
    pub check: Option<Callback<()>>,
}

/// The index of the step a component belongs to.
#[derive(Clone, Copy)]
pub struct StepAt(pub usize);

/// The open attempt's saved responses (`batna_core::response`) and a way to
/// keep a step's response. Every interactive step starts from its saved
/// response and writes each change back, so leaving and coming back (Back,
/// the page view, a rotation, a restart) shows exactly what he did.
#[derive(Clone, Copy)]
pub struct Responses {
    pub attempt: Signal<Attempt>,
    pub put: Callback<(usize, Draft)>,
}

/// The step's saved response (taken once, at mount) and a setter.
fn use_draft() -> (Draft, Callback<Draft>) {
    let r = try_use_context::<Responses>();
    let k = try_use_context::<StepAt>().map_or(0, |s| s.0);
    let saved = use_hook(|| {
        r.and_then(|r| r.attempt.peek().drafts.get(k).map(|s| s.draft.clone())).unwrap_or_default()
    });
    let put = use_callback(move |d: Draft| {
        if let Some(r) = r {
            r.put.call((k, d));
        }
    });
    (saved, put)
}

/// Wires a step to the footer: while `can_check` holds, the footer's
/// primary runs `check`.
pub(super) fn use_check_bar(can_check: impl Fn() -> bool + 'static, check: impl FnMut(()) + 'static) {
    let Some(CheckBar(mut slots)) = try_use_context::<CheckBar>() else { return };
    let k = try_use_context::<StepAt>().map_or(0, |s| s.0);
    let check = use_callback(check);
    use_effect(move || {
        let v = Gate { check: can_check().then_some(check) };
        if slots.peek().get(k).is_some_and(|g| *g != v) {
            slots.write()[k] = v;
        }
    });
    use_drop(move || {
        if let Some(slot) = slots.write().get_mut(k) {
            *slot = Gate::default();
        }
    });
}

#[component]
pub fn StepView(body: StepBody, seed: String, on_answer: EventHandler<bool>) -> Element {
    match body {
        StepBody::Explain { title, text } => rsx! {
            if let Some(t) = title { h2 { class: "ls-h", Line { text: t } } }
            Rich { text }
        },
        StepBody::Figure { title, svg, image, caption, alt } => rsx! {
            Figure { title, svg, image, caption, alt }
        },
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

/// Pinch, zoom buttons and double-tap in the full-screen figure viewer.
const VIEWER_JS: &str = include_str!("viewer.js");

#[component]
fn Figure(
    title: Option<String>,
    svg: String,
    image: Option<Picture>,
    caption: String,
    alt: String,
) -> Element {
    // A tap opens the figure full screen at every width (#337); in the
    // lesson it stays a plain picture, so a swipe on it turns the page (#338).
    let mut open = use_signal(|| false);
    use_hook(|| {
        document::eval(VIEWER_JS);
    });
    // Open-licence images show their author and licence; a book figure's
    // page chip is the step's own.
    let credit = image
        .as_ref()
        .filter(|p| !p.book)
        .and_then(|p| Some(format!("{} · {}", p.credit.as_deref()?, p.license.as_deref()?)));
    let picture = match &image {
        // Content-made SVG, checked by the validator (no scripts, links or fixed colours).
        None => rsx! { div { class: "ls-fig-svg", dir: "ltr", dangerous_inner_html: "{svg}" } },
        Some(p) => rsx! { img { src: "{p.data}", alt: "{alt}", decoding: "async" } },
    };
    rsx! {
        figure { class: "ls-figure",
            if let Some(t) = title.clone() { h2 { class: "ls-h", Line { text: t } } }
            button {
                class: if image.is_some() { "ls-svg ls-photo" } else { "ls-svg" },
                // English figures read from the top left.
                dir: "ltr",
                aria_label: "{alt}",
                title: tx::ZOOM_IN,
                onclick: move |_| open.set(true),
                {picture.clone()}
                span { class: "ls-zoom-hint", aria_hidden: "true", "⤢" }
            }
            figcaption {
                Rich { text: caption.clone() }
                if let Some(c) = credit {
                    p { class: "ls-credit muted small", bdi { dir: "ltr", lang: "en", "📷 {c}" } }
                }
            }
        }
        if open() {
            div {
                class: "ls-viewer",
                role: "dialog",
                aria_modal: "true",
                aria_label: "{alt}",
                tabindex: "-1",
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Escape {
                        open.set(false);
                    }
                },
                onmounted: move |e: MountedEvent| async move {
                    let _ = e.set_focus(true).await;
                },
                header { class: "ls-viewer-bar",
                    button {
                        class: "ls-close",
                        "data-back": "true",
                        aria_label: tx::ZOOM_CLOSE,
                        title: tx::ZOOM_CLOSE,
                        onclick: move |_| open.set(false),
                        "×"
                    }
                    if let Some(t) = title { strong { class: "ls-viewer-title", Line { text: t } } }
                    div { class: "ls-viewer-zoom", dir: "ltr",
                        button { class: "ls-arrow", "data-zoom": "out", aria_label: tx::ZOOM_OUT, title: tx::ZOOM_OUT, "−" }
                        button { class: "ls-arrow", "data-zoom": "fit", aria_label: tx::ZOOM_FIT, title: tx::ZOOM_FIT, "1:1" }
                        button { class: "ls-arrow", "data-zoom": "in", aria_label: tx::ZOOM_IN, title: tx::ZOOM_IN, "+" }
                    }
                }
                div { class: "ls-viewer-scroll", dir: "ltr",
                    div { class: "ls-viewer-inner", {picture} }
                }
                p { class: "ls-viewer-hint muted small", {tx::ZOOM_HINT} }
            }
        }
    }
}

#[component]
fn Reveal(prompt: String, answer: String, on_answer: EventHandler<bool>) -> Element {
    let (saved, put) = use_draft();
    let mut open = use_signal(|| matches!(saved, Draft::Reveal { open: true }));
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
                        put.call(Draft::Reveal { open: true });
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
    let (saved, put) = use_draft();
    let mut shown = use_signal(|| match saved {
        Draft::Mnemonic { shown } if shown.len() == n => shown,
        _ => vec![false; n],
    });
    let all = shown.read().iter().all(|s| *s);
    let mut show = move |v: Vec<bool>| {
        put.call(Draft::Mnemonic { shown: v.clone() });
        shown.set(v);
    };
    rsx! {
        div { class: "ls-mnemonic",
            p { class: "ls-kicker", {tx::MNEMONIC} }
            div { class: "ls-mn-letters", dir: "ltr", lang: "en",
                for (i , p) in parts.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: "ls-mn-key",
                        "aria-pressed": if shown.read()[i] { "true" } else { "false" },
                        onclick: move |_| {
                            let mut v = shown.peek().clone();
                            v[i] = true;
                            show(v);
                        },
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
                button { class: "btn-link", onclick: move |_| show(vec![true; n]), {tx::SHOW_ALL} }
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

fn cue(right: bool) {
    crate::sounds::play(if right { crate::sounds::Cue::Right } else { crate::sounds::Cue::Wrong });
}

/// Single best answer: a tap picks (and can change), the footer's "صحّح"
/// checks, then every option explains itself.
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
    let (saved, put) = use_draft();
    let (pick0, checked0) = match saved {
        Draft::Check { pick, checked } => (pick, checked && pick.is_some()),
        _ => (None, false),
    };
    let mut picked = use_signal(|| pick0);
    let mut checked = use_signal(|| checked0);
    use_check_bar(
        move || picked().is_some() && !checked(),
        move |_| {
            let Some(p) = *picked.peek() else { return };
            checked.set(true);
            put.call(Draft::Check { pick: Some(p), checked: true });
            on_answer.call(p == answer);
            cue(p == answer);
        },
    );
    let ltr = options.iter().all(|o| is_latin(o));
    rsx! {
        div { class: "ls-check",
            p { class: "ls-kicker", {tx::CHECK} }
            div { class: "ls-q", dir: if is_latin(&question) { "ltr" } else { "rtl" }, Rich { text: question.clone() } }
            div { class: "ls-options", dir: if ltr { "ltr" } else { "rtl" }, role: if checked() { "list" } else { "radiogroup" },
                for (pos , &i) in order.iter().enumerate() {
                    {
                        let (state, mark) = answer_state(checked(), picked(), i, answer);
                        rsx! {
                            div { key: "{i}", class: "ls-option-wrap",
                                button {
                                    class: "answer",
                                    role: if !checked() { "radio" } else { "listitem" },
                                    "aria-checked": if !checked() { if picked() == Some(i) { "true" } else { "false" } } else { "" },
                                    "data-state": state,
                                    "aria-keyshortcuts": LETTERS.get(pos).copied().unwrap_or(""),
                                    disabled: checked(),
                                    onclick: move |_| {
                                        if !checked() {
                                            picked.set(Some(i));
                                            put.call(Draft::Check { pick: Some(i), checked: false });
                                        }
                                    },
                                    span { class: "answer-key", "{LETTERS.get(pos).copied().unwrap_or(\"\")}" }
                                    span { class: "answer-text", Line { text: options[i].clone() } }
                                    if let Some(m) = mark {
                                        span { class: "answer-mark", dir: "rtl", lang: "ar", {m} }
                                    }
                                }
                                if checked() && (i == answer || picked() == Some(i)) {
                                    p { class: "ls-why small ls-pop", Line { text: why.get(i).cloned().unwrap_or_default() } }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(p) = picked().filter(|_| checked()) {
                p { class: if p == answer { "ls-verdict good" } else { "ls-verdict bad" }, role: "status",
                    {if p == answer { tx::RIGHT } else { tx::NOT_QUITE }}
                }
            }
        }
    }
}

/// Put the steps in order: numbered slots fill as he taps the pool. Before
/// Check any placed step can come out (tap it, or Undo the last); after
/// Check the order is read-only and every slot shows right or wrong.
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
    let (saved, put) = use_draft();
    let (placed0, checked0) = match saved {
        Draft::Order { placed, checked } => {
            let full = placed.len() == n;
            (placed, checked && full)
        }
        _ => (Vec::new(), false),
    };
    let mut placed = use_signal(|| placed0);
    let mut checked = use_signal(|| checked0);
    let mut set = move |v: Vec<usize>| {
        put.call(Draft::Order { placed: v.clone(), checked: false });
        placed.set(v);
    };
    use_check_bar(
        move || placed.read().len() == n && !checked(),
        move |_| {
            let v = placed.peek().clone();
            checked.set(true);
            let right = misplaced(&v).is_empty();
            put.call(Draft::Order { placed: v, checked: true });
            on_answer.call(right);
            cue(right);
        },
    );
    let wrong = if checked() { misplaced(&placed.read()) } else { Vec::new() };
    let k = placed.read().len();
    rsx! {
        div { class: "ls-order",
            p { class: "ls-kicker", {tx::ORDER} }
            div { class: "ls-q", Rich { text: prompt } }
            if !checked() {
                div { class: "ls-order-bar",
                    span { class: "ls-order-count small", role: "status", {tx::placed_of(k, n)} }
                    button {
                        class: "btn-link ls-undo",
                        disabled: k == 0,
                        onclick: move |_| {
                            let mut v = placed.peek().clone();
                            v.pop();
                            set(v);
                        },
                        span { aria_hidden: "true", "↶ " }
                        {tx::UNDO}
                    }
                }
            }
            ol { class: "ls-slots", dir: "ltr", lang: "en",
                for pos in 0..n {
                    match placed.read().get(pos).copied() {
                        Some(s) if !checked() => rsx! {
                            li { key: "{pos}", "data-state": "filled",
                                // The whole slot takes the step back out.
                                button {
                                    class: "ls-slot",
                                    aria_label: "{tx::REMOVE}: {steps[s]}",
                                    title: tx::REMOVE,
                                    onclick: move |_| {
                                        let mut v = placed.peek().clone();
                                        if pos < v.len() {
                                            v.remove(pos);
                                        }
                                        set(v);
                                    },
                                    span { class: "ls-num", "{pos + 1}" }
                                    bdi { class: "ls-slot-text", "{steps[s]}" }
                                    span { class: "ls-x", aria_hidden: "true", "×" }
                                }
                            }
                        },
                        Some(s) => rsx! {
                            li { key: "{pos}", "data-state": if wrong.contains(&pos) { "wrong" } else { "right" },
                                div { class: "ls-slot",
                                    span { class: "ls-num", "{pos + 1}" }
                                    bdi { class: "ls-slot-text", "{steps[s]}" }
                                    span { class: "ls-mark", aria_hidden: "true", if wrong.contains(&pos) { "✗" } else { "✓" } }
                                }
                                if wrong.contains(&pos) {
                                    span { class: "ls-should small", dir: "rtl", lang: "ar", {tx::should_be(s + 1)} }
                                }
                            }
                        },
                        None => rsx! {
                            li { key: "{pos}", "data-state": "empty",
                                div { class: "ls-slot",
                                    span { class: "ls-num", "{pos + 1}" }
                                    span { class: "ls-slot-text muted small", dir: "rtl", lang: "ar", {tx::slot(pos + 1)} }
                                }
                            }
                        },
                    }
                }
            }
            if k < n && !checked() {
                div { class: "ls-pool", dir: "ltr", lang: "en",
                    for &s in pool.iter().filter(|s| !placed.read().contains(s)) {
                        button {
                            key: "{s}",
                            class: "ls-tile",
                            onclick: move |_| {
                                let mut v = placed.peek().clone();
                                if !v.contains(&s) && v.len() < n {
                                    v.push(s);
                                }
                                set(v);
                            },
                            bdi { "{steps[s]}" }
                        }
                    }
                }
            }
            if checked() {
                p { class: if wrong.is_empty() { "ls-verdict good" } else { "ls-verdict bad" }, role: "status",
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

/// Match pairs: each pair is checked as he makes it, and wrong taps count.
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
    let (saved, put) = use_draft();
    let (done0, misses0) = match saved {
        Draft::Match { done, misses } => (done, misses as usize),
        _ => (Vec::new(), 0),
    };
    let mut done = use_signal(|| done0);
    let mut left = use_signal(|| None::<usize>);
    let mut flash = use_signal(|| None::<usize>);
    let mut misses = use_signal(|| misses0);
    let finished = done.read().len() == n;
    let mut tap_right = move |r: usize| {
        let Some(l) = left() else { return };
        if l == r {
            done.write().push(l);
            left.set(None);
            flash.set(None);
        } else {
            *misses.write() += 1;
            flash.set(Some(r));
        }
        put.call(Draft::Match { done: done.peek().clone(), misses: *misses.peek() as u32 });
        if l == r && done.peek().len() == n {
            on_answer.call(*misses.peek() == 0);
            crate::sounds::play(crate::sounds::Cue::Right);
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
                            "aria-pressed": if left() == Some(i) { "true" } else { "false" },
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
                p { class: if misses() == 0 { "ls-verdict good" } else { "ls-verdict" }, role: "status",
                    {if misses() == 0 { tx::RIGHT.to_string() } else { tx::misses(misses()) }}
                }
                if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
            }
        }
    }
}

/// Sort cards into bins: tap a card, then its bin. Before Check a placed
/// card goes back to the pool on a tap; after Check it is read-only.
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

/// A mini case, part by part: pick, check with the footer, read why, go on.
/// The last part's check finishes the case.
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
    let (saved, put) = use_draft();
    let (picks0, at0, pick0) = match saved {
        Draft::Case { picks, at, pick } if picks.len() == n && at < n => (picks, at, pick),
        _ => (vec![None; n], 0, None),
    };
    let mut at = use_signal(|| at0);
    // The checked pick per part (a part without options: Some(0) once shown).
    let mut picks = use_signal(|| picks0);
    // The current part's pick, before Check.
    let mut pick = use_signal(|| pick0);
    let save = use_callback(move |()| {
        put.call(Draft::Case { picks: picks.peek().clone(), at: *at.peek(), pick: *pick.peek() })
    });
    let answers: Vec<Option<usize>> = steps.iter().map(|s| s.answer).collect();
    let has_options: Vec<bool> = steps.iter().map(|s| !s.options.is_empty()).collect();
    let part_answers = answers.clone();
    // A part is settled: the whole case is done once the last one is.
    let settle = use_callback(move |(si, p): (usize, usize)| {
        picks.write()[si] = Some(p);
        pick.set(None);
        save.call(());
        if si + 1 == n {
            let v = picks.peek().clone();
            let right = v.iter().zip(&answers).zip(&has_options).all(|((p, a), o)| !*o || *p == *a);
            on_answer.call(right);
        }
    });
    use_check_bar(
        move || pick().is_some(),
        move |_| {
            let si = *at.peek();
            let Some(p) = *pick.peek() else { return };
            if picks.peek()[si].is_none() {
                settle.call((si, p));
                cue(part_answers[si] == Some(p));
            }
        },
    );
    rsx! {
        div { class: "ls-case",
            p { class: "ls-kicker", {tx::CASE} }
            div { class: "ls-case-stem", dir: "ltr", lang: "en", Rich { text: stem } }
            for (si , s) in steps.iter().take(at() + 1).enumerate() {
                {
                    let done = picks.read()[si];
                    let last = si == at();
                    rsx! {
                        div { key: "{si}", class: "ls-case-step ls-pop",
                            if let Some(r) = &s.reveal { p { class: "ls-case-reveal", dir: "ltr", lang: "en", Line { text: r.clone() } } }
                            p { class: "ls-case-q", dir: "ltr", lang: "en", strong { Line { text: s.question.clone() } } }
                            if s.options.is_empty() {
                                if done.is_none() {
                                    button { class: "btn btn-secondary", "aria-keyshortcuts": "Space", onclick: move |_| settle.call((si, 0)), {tx::CASE_SHOW} }
                                }
                            } else {
                                div { class: "ls-options", dir: "ltr", lang: "en",
                                    for (pos , &oi) in orders[si].iter().enumerate() {
                                        {
                                            let (state, mark) = match done {
                                                None => (if last && pick() == Some(oi) { Some("selected") } else { None }, None),
                                                Some(p) => answer_state(true, Some(p), oi, s.answer.unwrap_or(usize::MAX)),
                                            };
                                            rsx! {
                                                button {
                                                    key: "{oi}",
                                                    class: "answer",
                                                    "aria-keyshortcuts": if last && done.is_none() { LETTERS.get(pos).copied().unwrap_or("") } else { "" },
                                                    "data-state": state,
                                                    disabled: done.is_some(),
                                                    onclick: move |_| {
                                                        if picks.peek()[si].is_none() {
                                                            pick.set(Some(oi));
                                                            save.call(());
                                                        }
                                                    },
                                                    span { class: "answer-key", "{LETTERS.get(pos).copied().unwrap_or(\"\")}" }
                                                    span { class: "answer-text", Line { text: s.options[oi].clone() } }
                                                    if let Some(m) = mark {
                                                        span { class: "answer-mark", dir: "rtl", lang: "ar", {m} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if done.is_some() {
                                div { class: "ls-why-box ls-pop", dir: "ltr", lang: "en", Rich { text: s.why.clone() } }
                                if last && si + 1 < n {
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| {
                                            at.set(si + 1);
                                            save.call(());
                                        },
                                        {tx::CASE_NEXT}
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
    let (saved, put) = use_draft();
    let mut open = use_signal(|| match saved {
        Draft::Table { open } => open,
        _ => Vec::new(),
    });
    let mut show = move |v: Vec<[usize; 2]>| {
        put.call(Draft::Table { open: v.clone() });
        open.set(v);
    };
    let hidden_left = hide.iter().filter(|c| !open.read().contains(c)).count();
    rsx! {
        div { class: "ls-table",
            h2 { class: "ls-h", Line { text: title } }
            if hidden_left > 0 { p { class: "muted small", {tx::TABLE_HINT} } }
            div { class: "ls-table-scroll", dir: "ltr", lang: "en",
                table {
                    thead {
                        tr {
                            for (c , h) in columns.iter().enumerate() {
                                th { key: "{c}", "data-short": if is_value(h) { "yes" } else { "no" }, "{h}" }
                            }
                        }
                    }
                    tbody {
                        for (r , row) in rows.iter().enumerate() {
                            tr { key: "{r}",
                                for (c , cell) in row.iter().enumerate() {
                                    if hide.contains(&[r, c]) && !open.read().contains(&[r, c]) {
                                        td { key: "{c}",
                                            button { class: "ls-cell-hidden", onclick: move |_| {
                                                let mut v = open.peek().clone();
                                                v.push([r, c]);
                                                show(v);
                                            }, aria_label: tx::REVEAL, "؟" }
                                        }
                                    } else if c == 0 {
                                        th { key: "{c}", scope: "row", "{cell}" }
                                    } else {
                                        td {
                                            key: "{c}",
                                            "data-opened": if hide.contains(&[r, c]) { "yes" } else { "no" },
                                            "data-short": if is_value(cell) { "yes" } else { "no" },
                                            "{cell}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if hidden_left > 0 {
                button { class: "btn-link", onclick: move |_| show(hide.clone()), {tx::SHOW_ALL} }
            }
        }
    }
}

/// A question answered before its response was saved (an older version of
/// the app), or whose content changed since: the answer key, read-only,
/// with "حاول تاني" under it (player.rs). Never an empty task with Continue
/// already on, and no explanatory notes (Mohamed, 2026-10-29).
#[component]
pub fn AnsweredBefore(body: StepBody) -> Element {
    let (kicker, prompt) = match &body {
        StepBody::Check { question, .. } => (tx::CHECK, question.clone()),
        StepBody::Order { prompt, .. } => (tx::ORDER, prompt.clone()),
        StepBody::Match { prompt, .. } => (tx::MATCH, prompt.clone()),
        StepBody::Sort { prompt, .. } => (tx::SORT, prompt.clone()),
        StepBody::Case { stem, .. } => (tx::CASE, stem.clone()),
        _ => (tx::CHECK, String::new()),
    };
    let key = match body {
        StepBody::Check { options, answer, why, .. } => rsx! {
            p { class: "ls-key-line", Line { text: options.get(answer).cloned().unwrap_or_default() } }
            p { class: "ls-why small", Line { text: why.get(answer).cloned().unwrap_or_default() } }
        },
        StepBody::Order { steps, why, .. } => rsx! {
            ol { class: "ls-key small", dir: "ltr", lang: "en",
                for (i , s) in steps.iter().enumerate() { li { key: "{i}", "{s}" } }
            }
            if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
        },
        StepBody::Match { pairs, why, .. } => rsx! {
            ul { class: "ls-key small", dir: "ltr", lang: "en",
                for (i , p) in pairs.iter().enumerate() { li { key: "{i}", bdi { "{p[0]}" } " → " bdi { "{p[1]}" } } }
            }
            if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
        },
        StepBody::Sort { bins, cards, why, .. } => rsx! {
            ul { class: "ls-key small",
                for (b , name) in bins.iter().enumerate() {
                    li { key: "{b}",
                        strong { Line { text: name.clone() } }
                        ": "
                        for (i , c) in cards.iter().enumerate().filter(|(_, c)| c.bin == b) {
                            span { key: "{i}", class: "ls-key-item", Line { text: c.text.clone() } }
                        }
                    }
                }
            }
            if let Some(w) = why { div { class: "ls-why-box", Rich { text: w } } }
        },
        StepBody::Case { steps, .. } => rsx! {
            ol { class: "ls-key small", dir: "ltr", lang: "en",
                for (si , s) in steps.iter().enumerate() {
                    li { key: "{si}",
                        strong { Line { text: s.question.clone() } }
                        if let Some(a) = s.answer.and_then(|a| s.options.get(a)) {
                            " → "
                            Line { text: a.clone() }
                        }
                    }
                }
            }
        },
        _ => rsx! {},
    };
    rsx! {
        div { class: "ls-before",
            p { class: "ls-kicker", {kicker} }
            div { class: "ls-q", Rich { text: prompt } }
            p { class: "ls-kicker", {tx::KEY} }
            {key}
        }
    }
}
