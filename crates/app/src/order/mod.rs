//! `order` items: put an algorithm or management sequence in order.
//!
//! Tap a step in the pool to place it next; tap a placed step to send it
//! back, nudge it with ↑ ↓, or drag it with a mouse or pen. "Check" shows the
//! correct order beside the learner's: a step is green only at its own
//! number, every other step says where it belongs (see [`grade::misplaced`]),
//! and the fewest moves to fix it ([`grade::slips`]) sets the suggested grade. The tile pattern is adapted from
//! kabar-app main: crates/app/src/lesson.rs (`Tiles`).

pub mod grade;

use batna_core::content::{Body, Item};
use batna_core::review::Grade;
use dioxus::prelude::*;

use crate::i18n::order as tx;
use crate::look::{Css, css};
use crate::study::grade::GradeRow;

const CSS: Css = css!("src/order/order.css");

#[derive(Clone, Default, PartialEq)]
struct Draft {
    /// Placed steps, by their index in the correct order.
    placed: Vec<usize>,
    /// Position in `placed` being dragged.
    dragging: Option<usize>,
    checked: bool,
}

impl Draft {
    /// Moves the placed step at `from` to `to`.
    fn shift(&mut self, from: usize, to: usize) {
        if from < self.placed.len() && to < self.placed.len() {
            let s = self.placed.remove(from);
            self.placed.insert(to, s);
        }
    }

    /// A dragged step dropped on position `on`.
    fn drop_on(&mut self, on: usize) {
        if let Some(from) = self.dragging.take() {
            self.shift(from, on);
        }
    }
}

#[component]
pub fn OrderItem(item: Item, on_grade: EventHandler<Grade>) -> Element {
    rsx! { OrderBoard { item, on_grade, start: Draft::default() } }
}

/// The board, from a given draft (tests start it part-way).
#[component]
fn OrderBoard(item: Item, on_grade: EventHandler<Grade>, start: Draft) -> Element {
    let Body::Order { prompt, steps } = &item.body else {
        return rsx! {};
    };
    let n = steps.len();
    let pool_order = use_hook(|| grade::shuffled(n, &item.id));
    let mut draft = use_signal(|| start);
    let d = draft.read().clone();
    let wrong = if d.checked { grade::misplaced(&d.placed) } else { Vec::new() };
    let full = d.placed.len() == n;

    rsx! {
        style { {CSS.text()} }
        section { class: "or-item",
            h2 { class: "or-prompt", dir: "ltr", lang: "en", "{prompt}" }
            if !d.checked {
                p { class: "muted small", {tx::HINT} }
            }
            div { class: "or-board", "data-checked": if d.checked { "yes" } else { "no" },
                div { class: "or-col",
                    h3 { class: "or-head", if d.checked { {tx::YOUR_ORDER} } else { {tx::PLACED} } }
                    ol { class: "or-list", dir: "ltr", lang: "en",
                        for (pos, &step) in d.placed.iter().enumerate() {
                            PlacedStep {
                                key: "p{step}",
                                pos,
                                step,
                                text: steps[step].clone(),
                                last: pos + 1 == d.placed.len(),
                                wrong: wrong.contains(&pos),
                                draft,
                            }
                        }
                        if !full {
                            li { class: "or-slot", aria_hidden: "true",
                                span { class: "or-num", "{d.placed.len() + 1}" }
                                span { class: "muted small", dir: "rtl", lang: "ar", {tx::NEXT_SLOT} }
                            }
                        }
                    }
                }
                if d.checked {
                    div { class: "or-col",
                        h3 { class: "or-head", {tx::CORRECT_ORDER} }
                        ol { class: "or-list", dir: "ltr", lang: "en",
                            for (i, text) in steps.iter().enumerate() {
                                li { key: "c{i}", class: "or-step or-key",
                                    span { class: "or-num", "{i + 1}" }
                                    bdi { class: "or-text", dir: "ltr", "{text}" }
                                }
                            }
                        }
                    }
                } else if !full {
                    div { class: "or-col",
                        h3 { class: "or-head", {tx::POOL} }
                        div { class: "or-pool", dir: "ltr", lang: "en",
                            for &step in pool_order.iter().filter(|s| !d.placed.contains(s)) {
                                button {
                                    key: "b{step}",
                                    class: "or-tile",
                                    onclick: move |_| draft.write().placed.push(step),
                                    bdi { dir: "ltr", "{steps[step]}" }
                                }
                            }
                        }
                    }
                }
            }
            if d.checked {
                p { class: if wrong.is_empty() { "or-verdict good" } else { "or-verdict" },
                    if wrong.is_empty() {
                        {tx::ALL_IN_PLACE}
                    } else {
                        {tx::in_place(n - wrong.len(), n)}
                    }
                }
                if grade::slips(&d.placed) < wrong.len() {
                    p { class: "muted small", {tx::moves_to_fix(grade::slips(&d.placed))} }
                }
                GradeRow { item_id: item.id.clone(), suggested: Some(grade::suggest(grade::slips(&d.placed))), on_grade }
            } else {
                div { class: "or-actions",
                    button {
                        class: "btn btn-primary",
                        disabled: !full,
                        onclick: move |_| draft.write().checked = true,
                        {tx::CHECK}
                    }
                    if !d.placed.is_empty() {
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| draft.write().placed.clear(),
                            {tx::CLEAR}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlacedStep(
    pos: usize,
    step: usize,
    text: String,
    last: bool,
    wrong: bool,
    draft: Signal<Draft>,
) -> Element {
    let checked = draft.read().checked;
    let dragging = draft.read().dragging == Some(pos);
    let class = match (checked, wrong) {
        (false, _) => "or-step",
        (true, false) => "or-step ok",
        (true, true) => "or-step wrong",
    };
    let mut nudge = move |to: usize| draft.write().shift(pos, to);
    rsx! {
        li {
            class,
            "data-dragging": if dragging { "yes" } else { "no" },
            draggable: if checked { "false" } else { "true" },
            ondragstart: move |_| draft.write().dragging = Some(pos),
            ondragend: move |_| draft.write().dragging = None,
            ondragover: move |e| e.prevent_default(),
            ondrop: move |e| {
                e.prevent_default();
                draft.write().drop_on(pos);
            },
            span { class: "or-num", "{pos + 1}" }
            if checked {
                bdi { class: "or-text", dir: "ltr", "{text}" }
                if wrong {
                    span { class: "or-should", dir: "rtl", lang: "ar", {tx::should_be(step + 1)} }
                } else {
                    span { class: "or-tick", aria_label: tx::IN_PLACE, "✓" }
                }
            } else {
                button {
                    class: "or-text or-back",
                    aria_label: tx::SEND_BACK,
                    title: tx::SEND_BACK,
                    onclick: move |_| {
                        draft.write().placed.remove(pos);
                    },
                    bdi { dir: "ltr", "{text}" }
                }
                span { class: "or-nudge",
                    button {
                        class: "or-mini",
                        aria_label: tx::MOVE_UP,
                        disabled: pos == 0,
                        onclick: move |_| nudge(pos - 1),
                        "↑"
                    }
                    button {
                        class: "or-mini",
                        aria_label: tx::MOVE_DOWN,
                        disabled: last,
                        onclick: move |_| nudge(pos + 1),
                        "↓"
                    }
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
            .find(|i| matches!(&i.body, Body::Order { steps, .. } if steps.len() >= 4))
            .cloned()
            .expect("the library has order items")
    }

    fn steps(item: &Item) -> Vec<String> {
        let Body::Order { steps, .. } = &item.body else { unreachable!() };
        steps.clone()
    }

    fn render(placed: Vec<usize>, checked: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct Case {
            start: Draft,
        }
        fn root(c: Case) -> Element {
            rsx! { OrderBoard { item: item(), on_grade: |_| {}, start: c.start } }
        }
        let mut dom =
            VirtualDom::new_with_props(root, Case { start: Draft { placed, dragging: None, checked } });
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        html.split_once("</style>").map_or(html.clone(), |(_, r)| r.to_string())
    }

    #[test]
    fn a_fresh_board_offers_every_step_and_waits_to_check() {
        let n = steps(&item()).len();
        let html = render(Vec::new(), false);
        assert!(html.contains(tx::HINT) && html.contains(tx::POOL), "{html}");
        assert_eq!(html.matches("class=\"or-tile\"").count(), n, "{html}");
        assert!(html.contains("disabled=true"), "nothing to check yet: {html}");
        assert!(!html.contains(tx::CLEAR), "{html}");
        // The same item always shuffles the same way, and never to the answer.
        let order = grade::shuffled(n, &item().id);
        assert_ne!(order, (0..n).collect::<Vec<_>>());
    }

    #[test]
    fn placed_steps_can_go_back_or_move() {
        let s = steps(&item());
        let html = render(vec![0, 1], false);
        assert_eq!(html.matches("class=\"or-tile\"").count(), s.len() - 2, "{html}");
        assert_eq!(html.matches("class=\"or-step\"").count(), 2, "{html}");
        assert!(html.contains(tx::NEXT_SLOT) && html.contains(tx::CLEAR), "{html}");
        assert!(html.contains(&format!("aria-label=\"{}\"", tx::SEND_BACK)), "{html}");
        // The first can't go up, the last can't go down.
        assert_eq!(html.matches("disabled=true").count(), 3, "↑ on the first, ↓ on the last, Check: {html}");
    }

    #[test]
    fn a_full_board_can_be_checked() {
        let n = steps(&item()).len();
        let html = render((0..n).collect(), false);
        assert!(!html.contains("class=\"or-tile\"") && !html.contains(tx::NEXT_SLOT), "{html}");
        assert_eq!(html.matches("disabled=true").count(), 2, "only the ends' arrows: {html}");
    }

    #[test]
    fn the_right_order_is_all_green() {
        let n = steps(&item()).len();
        let html = render((0..n).collect(), true);
        assert!(html.contains(tx::ALL_IN_PLACE) && html.contains(tx::CORRECT_ORDER), "{html}");
        assert_eq!(html.matches("class=\"or-step ok\"").count(), n, "{html}");
        assert_eq!(html.matches("class=\"or-step or-key\"").count(), n);
        assert!(!html.contains(tx::HINT));
    }

    #[test]
    fn one_early_slip_marks_the_steps_and_counts_the_moves() {
        let n = steps(&item()).len();
        // The last step put first pushes every other step one place down.
        let mut placed: Vec<usize> = (0..n - 1).collect();
        placed.insert(0, n - 1);
        let html = render(placed.clone(), true);
        let wrong = grade::misplaced(&placed).len();
        assert_eq!(wrong, n);
        assert!(html.contains(&tx::in_place(0, n)), "{html}");
        assert_eq!(html.matches("class=\"or-step wrong\"").count(), n, "{html}");
        assert!(html.contains(&tx::should_be(n)), "{html}");
        assert!(html.contains(&tx::moves_to_fix(1)), "one move fixes it: {html}");
    }

    #[test]
    fn moves_and_drops_keep_every_step() {
        let mut d = Draft { placed: vec![0, 1, 2, 3], ..Draft::default() };
        d.shift(0, 2);
        assert_eq!(d.placed, [1, 2, 0, 3]);
        d.shift(3, 9);
        assert_eq!(d.placed, [1, 2, 0, 3], "out of range: nothing moves");
        d.dragging = Some(3);
        d.drop_on(0);
        assert_eq!(d.placed, [3, 1, 2, 0]);
        assert_eq!(d.dragging, None);
        d.drop_on(1);
        assert_eq!(d.placed, [3, 1, 2, 0], "a drop with no drag does nothing");
    }
}

#[cfg(test)]
mod tap_tests {
    use super::*;

    #[test]
    fn tapping_places_moves_sends_back_and_checks() {
        fn app() -> Element {
            let lib = crate::content::Library::get();
            let item = lib
                .lectures
                .values()
                .flat_map(|l| &l.items)
                .find(|i| matches!(&i.body, Body::Order { steps, .. } if steps.len() >= 4))
                .cloned()
                .unwrap();
            rsx! { OrderItem { item, on_grade: |_| {} } }
        }
        let mut t = crate::tap_dom::TestDom::of(app);
        let tiles = |t: &crate::tap_dom::TestDom| t.html().matches("class=\"or-tile\"").count();
        let n = tiles(&t);
        t.click("class=\"or-tile\"");
        assert_eq!(tiles(&t), n - 1);
        t.click("class=\"or-tile\"");
        t.click(&format!("aria-label=\"{}\"", tx::MOVE_DOWN));
        t.click_nth(&format!("aria-label=\"{}\"", tx::MOVE_UP), 1);
        t.click(&format!("aria-label=\"{}\"", tx::SEND_BACK));
        assert_eq!(tiles(&t), n - 1, "one sent back");
        t.click(tx::CLEAR);
        assert_eq!(tiles(&t), n, "all back in the pool");
        while tiles(&t) > 0 {
            t.click("class=\"or-tile\"");
        }
        t.click(tx::CHECK);
        assert!(t.html().contains(tx::CORRECT_ORDER), "{}", t.html());
    }
}
