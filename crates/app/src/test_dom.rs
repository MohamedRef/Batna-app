//! Test-only: drive a rendered screen the way a tap would. SSR renders the
//! HTML but never runs event handlers; this walks the mounted tree, finds an
//! element by what the learner sees (class, attribute, text), calls its real
//! `onclick`/`oninput` handler inside the dom's runtime, and re-renders. So a
//! test can press Continue, pick an option or retry a step and assert on the
//! HTML and the saved state that follow.

use dioxus::core::{
    AttributeValue, DynamicNode, Event, ListenerCallback, NoOpMutations, TemplateAttribute, TemplateNode,
    VNode,
};
use dioxus::html::{
    PlatformEventData, SerializedFormData, SerializedHtmlEventConverter, SerializedMouseData,
};
use dioxus::prelude::*;
use std::any::Any;
use std::rc::Rc;

/// One element of the mounted tree, in document order.
#[derive(Clone)]
pub struct El {
    pub tag: &'static str,
    pub attrs: Vec<(String, String)>,
    /// Every text inside it, child components included.
    pub text: String,
    /// Inside an element with `hidden` (a step the learner has left).
    pub hidden: bool,
    /// Indexes of the elements around it, outermost first.
    pub ancestors: Vec<usize>,
    listeners: Vec<(String, ListenerCallback)>,
}

impl El {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }
    pub fn has_class(&self, class: &str) -> bool {
        self.attr("class").is_some_and(|c| c.split_whitespace().any(|x| x == class))
    }
    pub fn disabled(&self) -> bool {
        self.attr("disabled").is_some_and(|v| v == "true")
    }
}

/// A screen mounted for a test.
pub struct Screen {
    pub dom: VirtualDom,
}

impl Screen {
    pub fn new(app: fn() -> Element) -> Self {
        Self::mount(VirtualDom::new(app))
    }

    pub fn with_props<P: Clone + 'static>(app: fn(P) -> Element, props: P) -> Self {
        Self::mount(VirtualDom::new_with_props(app, props))
    }

    fn mount(mut dom: VirtualDom) -> Self {
        dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter));
        dom.rebuild_in_place();
        let mut s = Self { dom };
        s.settle();
        s
    }

    /// Runs queued signal writes and re-renders until nothing is dirty.
    pub fn settle(&mut self) {
        for _ in 0..8 {
            self.dom.render_immediate(&mut NoOpMutations);
        }
    }

    pub fn html(&self) -> String {
        dioxus::ssr::render(&self.dom)
    }

    /// Every element now mounted.
    pub fn elements(&self) -> Vec<El> {
        let mut out = Vec::new();
        walk(&self.dom, self.dom.base_scope().root_node(), false, &mut Vec::new(), &mut out);
        out
    }

    /// The visible elements that match.
    pub fn find(&self, pred: impl Fn(&El) -> bool) -> Vec<El> {
        self.elements().into_iter().filter(|e| !e.hidden && pred(e)).collect()
    }

    /// The visible elements matching `inner` inside one matching `outer`.
    pub fn find_in(&self, outer: impl Fn(&El) -> bool, inner: impl Fn(&El) -> bool) -> Vec<El> {
        let all = self.elements();
        all.iter()
            .filter(|e| !e.hidden && inner(e) && e.ancestors.iter().any(|&a| outer(&all[a])))
            .cloned()
            .collect()
    }

    /// Runs `f` inside the dom's runtime, in the app's root scope: reads a
    /// global signal or a context as the screen sees it.
    pub fn read<T>(&self, f: impl FnOnce() -> T) -> T {
        self.dom.in_scope(ScopeId::APP, f)
    }

    /// Taps the first visible, enabled element that matches and has an
    /// `onclick`; panics when there is none, so a missing button fails.
    pub fn click(&mut self, pred: impl Fn(&El) -> bool) {
        let el = self.find(|e| pred(e) && e.listeners.iter().any(|(n, _)| n == "onclick")).into_iter().next();
        self.tap(el);
    }

    /// Taps the first match of `inner` inside an element matching `outer`.
    pub fn click_in(&mut self, outer: impl Fn(&El) -> bool, inner: impl Fn(&El) -> bool) {
        let el = self
            .find_in(outer, |e| inner(e) && e.listeners.iter().any(|(n, _)| n == "onclick"))
            .into_iter()
            .next();
        self.tap(el);
    }

    fn tap(&mut self, el: Option<El>) {
        let el =
            el.into_iter().next().unwrap_or_else(|| panic!("nothing to tap matches in:\n{}", self.html()));
        assert!(!el.disabled(), "tapped a disabled <{}> {:?}", el.tag, el.attrs);
        self.fire(&el, "onclick", Box::new(SerializedMouseData::default()));
    }

    /// Taps the visible element with this class whose text contains `text`.
    pub fn click_text(&mut self, class: &str, text: &str) {
        self.click(|e| e.has_class(class) && e.text.contains(text));
    }

    /// Types `value` into the first visible matching field.
    pub fn input(&mut self, pred: impl Fn(&El) -> bool, value: &str) {
        let el = self
            .find(|e| pred(e) && e.listeners.iter().any(|(n, _)| n == "oninput"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no field matches in:\n{}", self.html()));
        self.fire(&el, "oninput", Box::new(SerializedFormData::new(value.to_string(), Vec::new())));
    }

    fn fire(&mut self, el: &El, name: &str, data: Box<dyn Any>) {
        let listener = el.listeners.iter().find(|(n, _)| n == name).map(|(_, l)| l.clone()).unwrap();
        let event: Event<dyn Any> = Event::new(Rc::new(PlatformEventData::new(data)) as Rc<dyn Any>, true);
        self.dom.in_runtime(|| listener.call(event));
        self.settle();
    }
}

/// Letters and digits only: compares what the learner reads, whatever
/// spacing marks the text renderer added (no-break and hair spaces).
pub fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Is `class` on any visible element?
pub fn shows(s: &Screen, class: &str) -> bool {
    !s.find(|e| e.has_class(class)).is_empty()
}

fn walk(dom: &VirtualDom, node: &VNode, hidden: bool, up: &mut Vec<usize>, out: &mut Vec<El>) -> String {
    node.template.roots.iter().map(|t| walk_t(dom, node, t, hidden, up, out)).collect()
}

fn walk_t(
    dom: &VirtualDom,
    node: &VNode,
    t: &TemplateNode,
    hidden: bool,
    up: &mut Vec<usize>,
    out: &mut Vec<El>,
) -> String {
    match t {
        TemplateNode::Element { tag, attrs, children, .. } => {
            let mut el = El {
                tag,
                attrs: Vec::new(),
                text: String::new(),
                hidden,
                ancestors: up.clone(),
                listeners: Vec::new(),
            };
            for a in attrs.iter() {
                match a {
                    TemplateAttribute::Static { name, value, .. } => {
                        el.attrs.push((name.to_string(), value.to_string()))
                    }
                    TemplateAttribute::Dynamic { id } => {
                        for attr in node.dynamic_attrs[*id].iter() {
                            let value = match &attr.value {
                                AttributeValue::Text(s) => s.clone(),
                                AttributeValue::Bool(b) => b.to_string(),
                                AttributeValue::Int(i) => i.to_string(),
                                AttributeValue::Float(f) => f.to_string(),
                                AttributeValue::Listener(l) => {
                                    el.listeners.push((attr.name.to_string(), l.clone()));
                                    continue;
                                }
                                _ => continue,
                            };
                            // A class split over static and dynamic parts.
                            match el.attrs.iter_mut().find(|(n, _)| n == attr.name) {
                                Some((_, v)) if attr.name == "class" => {
                                    v.push(' ');
                                    v.push_str(&value);
                                }
                                _ => el.attrs.push((attr.name.to_string(), value)),
                            }
                        }
                    }
                }
            }
            let hide = hidden || el.attr("hidden").is_some_and(|v| v != "false");
            el.hidden = hide;
            let at = out.len();
            out.push(el);
            up.push(at);
            let text: String = children.iter().map(|c| walk_t(dom, node, c, hide, up, out)).collect();
            up.pop();
            out[at].text = text.clone();
            text
        }
        TemplateNode::Text { text } => text.to_string(),
        TemplateNode::Dynamic { id } => match &node.dynamic_nodes[*id] {
            DynamicNode::Text(t) => t.value.clone(),
            DynamicNode::Fragment(nodes) => nodes.iter().map(|n| walk(dom, n, hidden, up, out)).collect(),
            DynamicNode::Component(c) => match c.mounted_scope(*id, node, dom) {
                Some(scope) => walk(dom, scope.root_node(), hidden, up, out),
                None => String::new(),
            },
            DynamicNode::Placeholder(_) => String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn Counter() -> Element {
        let mut n = use_signal(|| 0);
        let mut name = use_signal(String::new);
        rsx! {
            p { class: "count", "{n}" }
            button { class: "btn plus", onclick: move |_| n += 1, "Add" }
            button { class: "btn off", disabled: true, onclick: move |_| n += 10, "Off" }
            div { hidden: true, button { class: "btn plus", onclick: move |_| n += 100, "Hidden add" } }
            input { oninput: move |e| name.set(e.value()) }
            p { class: "name", "{name}" }
        }
    }

    #[test]
    fn a_tap_runs_the_real_handler_and_rerenders() {
        let mut s = Screen::new(Counter);
        assert!(s.html().contains("<p class=\"count\">0</p>"));
        s.click_text("plus", "Add");
        s.click(|e| e.has_class("plus"));
        // The hidden copy is never the one tapped.
        assert!(s.html().contains("<p class=\"count\">2</p>"), "{}", s.html());
        s.input(|e| e.tag == "input", "Cirrhosis");
        assert!(s.html().contains("<p class=\"name\">Cirrhosis</p>"));
    }

    #[test]
    #[should_panic(expected = "disabled")]
    fn a_disabled_button_cannot_be_tapped() {
        let mut s = Screen::new(Counter);
        s.click(|e| e.has_class("off"));
    }
}
