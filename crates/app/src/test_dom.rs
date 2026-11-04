//! Tests only: a rendered app whose buttons can be pressed and inputs typed
//! into, found by their visible text or label, so a test checks what a tap
//! does instead of only what is drawn.
//!
//! The SSR pre-render marks every element with a listener (`,click:1`) in
//! document order; walking the mounted templates in the same order gives each
//! listener's element id, so the n-th marker is the n-th id.

use dioxus::core::{AttributeValue, DynamicNode, ElementId, Event, VNode};
use dioxus::html::{EventData, SerializedFormData, SerializedMouseData};
use dioxus::prelude::*;

pub struct TestDom {
    pub dom: VirtualDom,
}

impl TestDom {
    /// Builds and renders `dom` once.
    pub fn new(mut dom: VirtualDom) -> Self {
        dioxus::html::set_event_converter(Box::new(dioxus::html::SerializedHtmlEventConverter));
        dom.rebuild_in_place();
        Self { dom }
    }

    pub fn of(app: fn() -> Element) -> Self {
        Self::new(VirtualDom::new(app))
    }

    /// Runs `f` inside the app's root scope (to read its signals).
    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        self.dom.in_scope(dioxus::core::ScopeId::APP, f)
    }

    /// The page as drawn, inline stylesheets left out (they name the classes too).
    pub fn html(&self) -> String {
        strip_styles(&dioxus::ssr::render(&self.dom))
    }

    /// Presses the element with an `onclick` whose text or attributes hold `text`.
    pub fn click(&mut self, text: &str) {
        let data = EventData::Mouse(SerializedMouseData::default());
        self.fire("click", text, data);
    }

    /// Presses the element holding the `n`th (from 0) `text` on the page.
    pub fn click_nth(&mut self, text: &str, n: usize) {
        let data = EventData::Mouse(SerializedMouseData::default());
        self.fire_nth("click", text, n, data);
    }

    /// Types `value` into the input with an `oninput` labelled `label`.
    pub fn input(&mut self, label: &str, value: &str) {
        let data = EventData::Form(SerializedFormData::new(value.to_string(), Vec::new()));
        self.fire("input", label, data);
    }

    /// Fires a form `event` (`change`, `input`) with `value` at `label`.
    pub fn input_event(&mut self, event: &str, label: &str, value: &str) {
        let data = EventData::Form(SerializedFormData::new(value.to_string(), Vec::new()));
        self.fire(event, label, data);
    }

    /// Fires `event` on the nearest element at or before `text` that listens for it.
    pub fn fire(&mut self, event: &str, text: &str, data: EventData) {
        self.fire_nth(event, text, 0, data);
    }

    fn fire_nth(&mut self, event: &str, text: &str, n: usize, data: EventData) {
        let marked = self.marked();
        let at = marked
            .match_indices(text)
            .nth(n)
            .unwrap_or_else(|| panic!("no {text:?} number {n} on the page: {marked}"))
            .0;
        let marker = format!(",{event}:");
        // A label in the tag itself: its marker comes later in the same tag.
        let end = marked[at..].find('>').map_or(marked.len(), |e| at + e);
        let nth = marked[..end]
            .matches(&marker)
            .count()
            .checked_sub(1)
            .unwrap_or_else(|| panic!("nothing listens for {event} before {text:?}: {marked}"));
        let ids = self.listeners(&format!("on{event}"));
        let id = *ids.get(nth).unwrap_or_else(|| panic!("listener {nth} of {}", ids.len()));
        self.dom.runtime().handle_event(event, Event::new(data.into_any(), true), id);
        self.settle();
    }

    /// Runs what the event queued: re-renders and ready tasks.
    pub fn settle(&mut self) {
        self.dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }

    fn marked(&self) -> String {
        let mut r = dioxus::ssr::Renderer::new();
        r.pre_render = true;
        // Text nodes' hydration comments would split interpolated text.
        let html = strip_styles_marked(&r.render(&self.dom));
        let mut out = String::with_capacity(html.len());
        let mut rest = html.as_str();
        while let Some(at) = rest.find("<!--") {
            out.push_str(&rest[..at]);
            rest = rest[at..].split_once("-->").map_or("", |(_, r)| r);
        }
        out + rest
    }

    /// The element ids of every `name` listener, in document order.
    fn listeners(&self, name: &str) -> Vec<ElementId> {
        let mut out = Vec::new();
        let root = self.dom.base_scope().root_node();
        walk(&self.dom, root, name, &mut out);
        out
    }
}

fn walk(dom: &VirtualDom, node: &VNode, name: &str, out: &mut Vec<ElementId>) {
    let template = node.template;
    // Attributes sit on their element; nodes are its children: by path,
    // both in document order.
    let mut steps: Vec<(&[u8], bool, usize)> = template
        .attr_paths
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, true, i))
        .chain(template.node_paths.iter().enumerate().map(|(i, p)| (*p, false, i)))
        .collect();
    steps.sort_by(|a, b| a.0.cmp(b.0).then(b.1.cmp(&a.1)));
    for (_, is_attr, i) in steps {
        if is_attr {
            for attr in node.dynamic_attrs[i].iter() {
                if attr.name == name
                    && matches!(attr.value, AttributeValue::Listener(_))
                    && let Some(id) = node.mounted_dynamic_attribute(i, dom)
                {
                    out.push(id);
                }
            }
        } else {
            match &node.dynamic_nodes[i] {
                DynamicNode::Component(c) => {
                    if let Some(scope) = c.mounted_scope(i, node, dom) {
                        walk(dom, scope.root_node(), name, out);
                    }
                }
                DynamicNode::Fragment(nodes) => {
                    for n in nodes {
                        walk(dom, n, name, out);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Pre-rendered stylesheets carry a hydration id: `<style data-node-hydration="0">`.
fn strip_styles_marked(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find("<style") {
        out.push_str(&rest[..at]);
        rest = rest[at..].split_once("</style>").map_or("", |(_, r)| r);
    }
    out + rest
}

pub fn strip_styles(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find("<style>") {
        out.push_str(&rest[..at]);
        rest = rest[at..].split_once("</style>").map_or("", |(_, r)| r);
    }
    out + rest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_reaches_the_button_named() {
        #[component]
        fn Counter(step: i32) -> Element {
            let mut n = use_signal(|| 0);
            rsx! {
                div { class: "c{step}",
                    span { "n{step}={n}" }
                    button { onclick: move |_| n += step, "add {step}" }
                }
            }
        }
        fn app() -> Element {
            let mut text = use_signal(String::new);
            let mut shown = use_signal(|| false);
            rsx! {
                Counter { step: 1 }
                if shown() {
                    p { "open" }
                }
                button { onclick: move |_| shown.toggle(), "toggle" }
                for step in [10, 100] {
                    Counter { key: "{step}", step }
                }
                input { aria_label: "name", oninput: move |e| text.set(e.value()) }
                p { "typed:{text}" }
            }
        }
        let mut t = TestDom::of(app);
        t.click("add 10");
        t.click("add 100");
        t.click("add 100");
        assert!(t.html().contains("n1=0") && t.html().contains("n10=10") && t.html().contains("n100=200"));
        t.click("toggle");
        assert!(t.html().contains("open"));
        // Listeners after a change still line up.
        t.click("add 1");
        t.click("add 10");
        assert!(t.html().contains("n1=1") && t.html().contains("n10=20"), "{}", t.html());
        t.input("aria-label=\"name\"", "Mohamed");
        assert!(t.html().contains("typed:Mohamed"));
    }
}
