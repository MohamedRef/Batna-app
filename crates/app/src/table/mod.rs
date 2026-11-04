//! `table` items: a comparison grid from the book with some cells blanked.
//!
//! Each blank is tap-to-reveal, then self-marked ✓ / ✗; or, with typing on,
//! the learner types every blank and "Check" marks them (and can be overruled
//! per cell). When every blank is marked, the share recalled suggests a grade.
//! On phones the grid scrolls sideways in its own box with the row labels
//! (column 0) pinned.

pub mod grade;

use batna_core::content::{Body, Item};
use batna_core::review::Grade;
use dioxus::prelude::*;

use crate::i18n::table as tx;
use crate::look::{Css, css};
use crate::study::grade::GradeRow;

const CSS: Css = css!("src/table/table.css");

/// Cells longer than this are tap-to-reveal even with typing on.
const TYPABLE_CHARS: usize = 40;

#[derive(Clone, Debug, Default, PartialEq)]
struct Blank {
    typed: String,
    shown: bool,
    /// Recalled or not: set by "Check" for typed cells, by the learner otherwise.
    right: Option<bool>,
}

/// "Check" / "Reveal all": opens every hidden cell and marks the typed ones.
fn check(blanks: &mut [Blank], answers: &[String]) {
    for (b, answer) in blanks.iter_mut().zip(answers) {
        if !b.shown {
            b.shown = true;
            if !b.typed.trim().is_empty() {
                b.right = Some(grade::matches(&b.typed, answer));
            }
        }
    }
}

#[component]
pub fn TableItem(item: Item, on_grade: EventHandler<Grade>) -> Element {
    rsx! { TableBoard { item, on_grade, start: None, typing_on: false } }
}

/// The grid, from given blanks (tests start it part-way; `None`: all hidden).
#[component]
fn TableBoard(
    item: Item,
    on_grade: EventHandler<Grade>,
    start: Option<Vec<Blank>>,
    typing_on: bool,
) -> Element {
    let Body::Table { title, columns, rows, blanks } = &item.body else {
        return rsx! {};
    };
    let cells: Vec<[usize; 2]> = blanks
        .iter()
        .copied()
        .filter(|&[r, c]| rows.get(r).is_some_and(|row| c < row.len() && c < columns.len()))
        .collect();
    let n = cells.len();
    let mut state =
        use_signal(|| start.filter(|s| s.len() == n).unwrap_or_else(|| vec![Blank::default(); n]));
    let mut typing = use_signal(|| typing_on);

    let s = state.read().clone();
    let hidden = s.iter().filter(|b| !b.shown).count();
    let marked: Vec<bool> = s.iter().filter_map(|b| b.right).collect();
    let done = marked.len() == n;
    let right = marked.iter().filter(|&&r| r).count();

    let answers: Vec<String> = cells.iter().map(|&[r, c]| rows[r][c].clone()).collect();
    let check_all = move |_| check(&mut state.write(), &answers);

    rsx! {
        style { {CSS.text()} }
        section { class: "tb-item",
            h2 { class: "tb-title", dir: "ltr", lang: "en", "{title}" }
            if hidden > 0 {
                p { class: "muted small", if *typing.read() { {tx::HINT_TYPE} } else { {tx::HINT_TAP} } }
            }
            div {
                class: "tb-scroll",
                dir: "ltr",
                lang: "en",
                tabindex: "0",
                role: "region",
                aria_label: "{title}",
                table { class: "tb-grid",
                    thead {
                        tr {
                            for (c, head) in columns.iter().enumerate() {
                                th { key: "h{c}", scope: "col", class: if c == 0 { "tb-pin" } else { "" }, "{head}" }
                            }
                        }
                    }
                    tbody {
                        for (r, row) in rows.iter().enumerate() {
                            tr { key: "r{r}",
                                for (c, text) in row.iter().enumerate().take(columns.len()) {
                                    match cells.iter().position(|&cell| cell == [r, c]) {
                                        Some(i) => rsx! {
                                            BlankCell {
                                                key: "c{r}-{c}",
                                                index: i,
                                                answer: text.clone(),
                                                pinned: c == 0,
                                                typing: *typing.read(),
                                                state,
                                            }
                                        },
                                        None if c == 0 => rsx! { th { key: "c{r}-{c}", scope: "row", class: "tb-pin", "{text}" } },
                                        None => rsx! { td { key: "c{r}-{c}", "{text}" } },
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if hidden > 0 {
                div { class: "tb-actions",
                    if *typing.read() {
                        button { class: "btn btn-primary", onclick: check_all, {tx::CHECK_TYPED} }
                    } else {
                        button { class: "btn btn-primary", onclick: check_all, {tx::REVEAL_ALL} }
                    }
                    button {
                        class: "btn btn-secondary",
                        aria_pressed: if *typing.read() { "true" } else { "false" },
                        onclick: move |_| typing.toggle(),
                        if *typing.read() { {tx::TYPING_OFF} } else { {tx::TYPING_ON} }
                    }
                }
            } else if !done {
                p { class: "muted small", {tx::mark_each(n - marked.len())} }
            }
            if done {
                p { class: if right == n { "tb-verdict good" } else { "tb-verdict" }, {tx::recalled(right, n)} }
                GradeRow { item_id: item.id.clone(), suggested: Some(grade::suggest(right, n)), on_grade }
            }
        }
    }
}

#[component]
fn BlankCell(index: usize, answer: String, pinned: bool, typing: bool, state: Signal<Vec<Blank>>) -> Element {
    let b = state.read()[index].clone();
    let class = match (pinned, b.right) {
        (_, Some(true)) => "tb-blank ok",
        (_, Some(false)) => "tb-blank wrong",
        _ => "tb-blank",
    };
    let class = if pinned { format!("{class} tb-pin") } else { class.to_string() };
    let mut set_right = move |value: bool| state.write()[index].right = Some(value);
    rsx! {
        td { class,
            if !b.shown {
                // Long cells are revealed and self-marked even in typing mode.
                if typing && answer.chars().count() <= TYPABLE_CHARS {
                    input {
                        class: "tb-input",
                        r#type: "text",
                        aria_label: tx::TYPE_CELL,
                        autocomplete: "off",
                        spellcheck: "false",
                        value: "{b.typed}",
                        oninput: move |e| state.write()[index].typed = e.value(),
                    }
                } else {
                    button {
                        class: "tb-reveal",
                        aria_label: tx::REVEAL_CELL,
                        onclick: move |_| state.write()[index].shown = true,
                        "?"
                    }
                }
            } else {
                span { class: "tb-answer", "{answer}" }
                if !b.typed.trim().is_empty() && b.right == Some(false) {
                    span { class: "tb-typed", s { "{b.typed}" } }
                }
                match b.right {
                    None => rsx! {
                        span { class: "tb-marks", dir: "rtl", lang: "ar",
                            button { class: "tb-mark yes", aria_label: tx::KNEW_IT, title: tx::KNEW_IT, onclick: move |_| set_right(true), "✓" }
                            button { class: "tb-mark no", aria_label: tx::MISSED_IT, title: tx::MISSED_IT, onclick: move |_| set_right(false), "✗" }
                        }
                    },
                    Some(r) => rsx! {
                        button {
                            class: "tb-flip",
                            dir: "rtl",
                            lang: "ar",
                            aria_label: if r { tx::MARK_WRONG } else { tx::MARK_RIGHT },
                            title: if r { tx::MARK_WRONG } else { tx::MARK_RIGHT },
                            onclick: move |_| set_right(!r),
                            if r { "✓" } else { "✗" }
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> Item {
        let lib = crate::content::Library::get();
        lib.lectures
            .values()
            .flat_map(|l| &l.items)
            .find(|i| match &i.body {
                Body::Table { blanks, rows, .. } => {
                    blanks.len() >= 2
                        && blanks.iter().any(|&[r, c]| c == 0 || rows[r][c].chars().count() <= TYPABLE_CHARS)
                }
                _ => false,
            })
            .cloned()
            .expect("the library has table items")
    }

    fn blanks(item: &Item) -> usize {
        let Body::Table { blanks, .. } = &item.body else { unreachable!() };
        blanks.len()
    }

    fn render(start: Option<Vec<Blank>>, typing_on: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct Case {
            start: Option<Vec<Blank>>,
            typing_on: bool,
        }
        fn root(c: Case) -> Element {
            rsx! { TableBoard { item: item(), on_grade: |_| {}, start: c.start, typing_on: c.typing_on } }
        }
        let mut dom = VirtualDom::new_with_props(root, Case { start, typing_on });
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        html.split_once("</style>").map_or(html.clone(), |(_, r)| r.to_string())
    }

    #[test]
    fn every_blank_starts_hidden_behind_a_question_mark() {
        let n = blanks(&item());
        let html = render(None, false);
        assert!(html.contains(tx::HINT_TAP) && html.contains(tx::REVEAL_ALL), "{html}");
        assert_eq!(html.matches("class=\"tb-reveal\"").count(), n, "{html}");
        assert!(html.contains(tx::TYPING_ON) && html.contains("aria-pressed=\"false\""), "{html}");
        assert!(!html.contains(tx::CHECK_TYPED));
    }

    #[test]
    fn typing_mode_asks_for_the_short_cells() {
        let html = render(None, true);
        assert!(html.contains(tx::HINT_TYPE) && html.contains(tx::CHECK_TYPED), "{html}");
        assert!(html.contains("class=\"tb-input\"") && html.contains(tx::TYPING_OFF), "{html}");
    }

    #[test]
    fn revealed_cells_wait_for_a_mark_then_flip() {
        let n = blanks(&item());
        let mut start = vec![Blank { shown: true, ..Blank::default() }; n];
        let html = render(Some(start.clone()), false);
        assert_eq!(html.matches("class=\"tb-mark yes\"").count(), n, "{html}");
        assert!(html.contains(&tx::mark_each(n)), "{html}");
        assert!(!html.contains(tx::REVEAL_ALL), "nothing left to reveal");

        start[0].right = Some(true);
        start[1] = Blank { typed: "a wrong guess".into(), shown: true, right: Some(false) };
        let html = render(Some(start), false);
        assert!(html.contains(&tx::mark_each(n - 2)), "{html}");
        assert!(html.contains(&format!("aria-label=\"{}\"", tx::MARK_WRONG)), "{html}");
        assert!(html.contains(&format!("aria-label=\"{}\"", tx::MARK_RIGHT)), "{html}");
        assert!(html.contains("<s>a wrong guess</s>"), "a wrong typed answer is shown struck: {html}");
    }

    #[test]
    fn all_marked_gives_the_share_recalled() {
        let n = blanks(&item());
        let all = vec![Blank { shown: true, right: Some(true), ..Blank::default() }; n];
        let html = render(Some(all), false);
        assert!(html.contains(&tx::recalled(n, n)) && html.contains("tb-verdict good"), "{html}");
        let mut some = vec![Blank { shown: true, right: Some(true), ..Blank::default() }; n];
        some[0].right = Some(false);
        let html = render(Some(some), false);
        assert!(html.contains(&tx::recalled(n - 1, n)) && !html.contains("tb-verdict good"), "{html}");
        assert!(html.contains("tb-blank wrong"), "{html}");
    }

    #[test]
    fn checking_marks_typed_cells_and_leaves_the_rest_to_the_learner() {
        let answers = vec!["Hepatitis B".to_string(), "Ascites".into(), "SBP".into()];
        let mut b = vec![
            Blank { typed: "hepatitis b".into(), ..Blank::default() },
            Blank { typed: "oedema".into(), ..Blank::default() },
            Blank::default(),
        ];
        check(&mut b, &answers);
        assert!(b.iter().all(|x| x.shown));
        assert_eq!(b.iter().map(|x| x.right).collect::<Vec<_>>(), [Some(true), Some(false), None]);
        // Already-open cells keep their marks.
        b[1].right = Some(true);
        check(&mut b, &answers);
        assert_eq!(b[1].right, Some(true));
    }
}

#[cfg(test)]
mod tap_tests {
    use super::*;

    #[test]
    fn revealing_marking_typing_and_checking_a_grid() {
        fn app() -> Element {
            let lib = crate::content::Library::get();
            let item = lib
                .lectures
                .values()
                .flat_map(|l| &l.items)
                .find(|i| matches!(&i.body, Body::Table { blanks, .. } if blanks.len() >= 2))
                .cloned()
                .unwrap();
            rsx! { TableItem { item, on_grade: |_| {} } }
        }
        let mut t = crate::test_dom::TestDom::of(app);
        t.click("class=\"tb-reveal\"");
        assert!(t.html().contains("class=\"tb-answer\""), "{}", t.html());
        t.click(&format!("aria-label=\"{}\"", tx::KNEW_IT));
        t.click(&format!("aria-label=\"{}\"", tx::MARK_WRONG));
        assert!(t.html().contains("tb-blank wrong"), "a flip: {}", t.html());
        // Typing mode for the rest, then Check marks them.
        t.click(&format!(">{}<", tx::TYPING_ON));
        if t.html().contains("class=\"tb-input\"") {
            t.input("class=\"tb-input\"", "a guess");
        }
        t.click(&format!(">{}<", tx::CHECK_TYPED));
        assert!(!t.html().contains("class=\"tb-reveal\"") && !t.html().contains("class=\"tb-input\""));
        while t.html().contains(&format!("aria-label=\"{}\"", tx::MISSED_IT)) {
            t.click(&format!("aria-label=\"{}\"", tx::MISSED_IT));
        }
        assert!(t.html().contains("class=\"tb-verdict\""), "every blank marked: {}", t.html());
    }
}
