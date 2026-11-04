//! A tiny in-memory DOM for tests: it applies the VirtualDom's mutations,
//! so a test can find a button by its text or attributes, fire a real
//! click / input / change / keydown on it, and read the re-rendered HTML.
//!
//! SSR alone shows one frame; this runs the component's own handlers, so a
//! test checks what happens after a tap (a grade saved, a step shown, a
//! choice marked), not only how the first frame looks.

use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use dioxus::core::{
    AttributeValue, ElementId, Event, Template, TemplateAttribute, TemplateNode, WriteMutations,
};
use dioxus::html::{
    Code, Key, Location, Modifiers, PlatformEventData, SerializedFormData, SerializedHtmlEventConverter,
    SerializedKeyboardData, SerializedMouseData,
};
use dioxus::prelude::*;

#[derive(Debug, Clone)]
enum Kind {
    Element { tag: String, attrs: Vec<(String, String)>, listeners: Vec<String> },
    Text(String),
    Placeholder,
}

#[derive(Debug, Clone)]
struct Node {
    kind: Kind,
    parent: Option<usize>,
    children: Vec<usize>,
}

/// The mirror of the real DOM the renderers would build.
#[derive(Default)]
struct Mirror {
    nodes: Vec<Node>,
    ids: HashMap<ElementId, usize>,
    by_node: HashMap<usize, ElementId>,
    stack: Vec<usize>,
    root: usize,
}

impl Mirror {
    fn new() -> Self {
        let mut m = Mirror::default();
        m.root = m.push(Kind::Element { tag: "#root".into(), attrs: Vec::new(), listeners: Vec::new() });
        m.ids.insert(ElementId(0), m.root);
        m.by_node.insert(m.root, ElementId(0));
        m.stack.push(m.root);
        m
    }

    fn push(&mut self, kind: Kind) -> usize {
        self.nodes.push(Node { kind, parent: None, children: Vec::new() });
        self.nodes.len() - 1
    }

    fn assign(&mut self, id: ElementId, node: usize) {
        if let Some(old) = self.ids.insert(id, node) {
            self.by_node.remove(&old);
        }
        self.by_node.insert(node, id);
    }

    fn build(&mut self, t: &TemplateNode) -> usize {
        match t {
            TemplateNode::Element { tag, attrs, children, .. } => {
                let attrs = attrs
                    .iter()
                    .filter_map(|a| match a {
                        TemplateAttribute::Static { name, value, .. } => {
                            Some((name.to_string(), value.to_string()))
                        }
                        TemplateAttribute::Dynamic { .. } => None,
                    })
                    .collect();
                let n = self.push(Kind::Element { tag: tag.to_string(), attrs, listeners: Vec::new() });
                for c in children.iter() {
                    let c = self.build(c);
                    self.nodes[c].parent = Some(n);
                    self.nodes[n].children.push(c);
                }
                n
            }
            TemplateNode::Text { text } => self.push(Kind::Text(text.to_string())),
            TemplateNode::Dynamic { .. } => self.push(Kind::Placeholder),
        }
    }

    fn at_path(&self, mut node: usize, path: &[u8]) -> usize {
        for i in path {
            node = self.nodes[node].children[*i as usize];
        }
        node
    }

    fn take(&mut self, m: usize) -> Vec<usize> {
        let at = self.stack.len() - m;
        self.stack.split_off(at)
    }

    fn detach(&mut self, n: usize) {
        if let Some(p) = self.nodes[n].parent.take() {
            self.nodes[p].children.retain(|c| *c != n);
        }
    }

    /// Puts `new` in `old`'s place (`after`: just after it, `before`: just before).
    fn splice(&mut self, old: usize, new: Vec<usize>, mode: i8) {
        for n in &new {
            self.detach(*n);
        }
        let Some(p) = self.nodes[old].parent else { return };
        let i = self.nodes[p].children.iter().position(|c| *c == old).unwrap_or(0);
        let at = match mode {
            0 => {
                self.nodes[p].children.remove(i);
                self.nodes[old].parent = None;
                i
            }
            1 => i + 1,
            _ => i,
        };
        for (k, n) in new.into_iter().enumerate() {
            self.nodes[n].parent = Some(p);
            self.nodes[p].children.insert(at + k, n);
        }
    }

    fn text(&self, n: usize) -> String {
        match &self.nodes[n].kind {
            Kind::Text(t) => t.clone(),
            Kind::Placeholder => String::new(),
            Kind::Element { .. } => self.nodes[n].children.iter().map(|c| self.text(*c)).collect(),
        }
    }

    fn attr(&self, n: usize, name: &str) -> Option<&str> {
        match &self.nodes[n].kind {
            Kind::Element { attrs, .. } => attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Whether `n` sits somewhere inside `outer`.
    fn is_inside(&self, n: usize, outer: usize) -> bool {
        let mut at = self.nodes[n].parent;
        while let Some(p) = at {
            if p == outer {
                return true;
            }
            at = self.nodes[p].parent;
        }
        false
    }

    fn listens(&self, n: usize, event: &str) -> bool {
        matches!(&self.nodes[n].kind, Kind::Element { listeners, .. } if listeners.iter().any(|l| l == event))
    }

    /// Every element still in the tree, in document order.
    fn walk(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut todo = vec![self.root];
        while let Some(n) = todo.pop() {
            out.push(n);
            todo.extend(self.nodes[n].children.iter().rev());
        }
        out
    }

    /// A selector of compounds joined by spaces (descendant) or ` > `
    /// (child), each `tag.class[attr=value]`.
    fn matches(&self, n: usize, sel: &str) -> bool {
        let mut parts: Vec<&str> = sel.split_whitespace().collect();
        let Some(last) = parts.pop() else { return false };
        if !self.matches_one(n, last) {
            return false;
        }
        let mut at = self.nodes[n].parent;
        let mut child = false;
        while let Some(part) = parts.pop() {
            if part == ">" {
                child = true;
                continue;
            }
            loop {
                let Some(p) = at else { return false };
                at = self.nodes[p].parent;
                if self.matches_one(p, part) {
                    break;
                }
                if child {
                    return false;
                }
            }
            child = false;
        }
        true
    }

    /// `tag.class[attr=value]` (any part optional, several classes and
    /// attributes allowed, `[attr]` alone tests presence).
    fn matches_one(&self, n: usize, sel: &str) -> bool {
        let Kind::Element { tag, .. } = &self.nodes[n].kind else { return false };
        let (head, mut rest) = sel.split_once('[').map_or((sel, ""), |(h, r)| (h, r));
        let mut parts = head.split('.');
        let want_tag = parts.next().unwrap_or("");
        if !want_tag.is_empty() && want_tag != tag {
            return false;
        }
        let classes: Vec<&str> = self.attr(n, "class").unwrap_or("").split_whitespace().collect();
        if !parts.all(|c| classes.contains(&c)) {
            return false;
        }
        while !rest.is_empty() {
            let (inner, after) = rest.split_once(']').unwrap_or((rest, ""));
            let ok = match inner.split_once('=') {
                Some((k, v)) => self.attr(n, k) == Some(v.trim_matches('"')),
                None => self.attr(n, inner).is_some(),
            };
            if !ok {
                return false;
            }
            rest = after.trim_start_matches('[');
        }
        true
    }
}

impl WriteMutations for Mirror {
    fn append_children(&mut self, id: ElementId, m: usize) {
        let parent = self.ids[&id];
        for n in self.take(m) {
            self.detach(n);
            self.nodes[n].parent = Some(parent);
            self.nodes[parent].children.push(n);
        }
    }
    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        let top = *self.stack.last().expect("a node on the stack");
        let n = self.at_path(top, path);
        self.assign(id, n);
    }
    fn create_placeholder(&mut self, id: ElementId) {
        let n = self.push(Kind::Placeholder);
        self.assign(id, n);
        self.stack.push(n);
    }
    fn create_text_node(&mut self, value: &str, id: ElementId) {
        let n = self.push(Kind::Text(value.to_string()));
        self.assign(id, n);
        self.stack.push(n);
    }
    fn load_template(&mut self, template: Template, index: usize, id: ElementId) {
        let n = self.build(&template.roots[index]);
        self.assign(id, n);
        self.stack.push(n);
    }
    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        let old = self.ids[&id];
        let new = self.take(m);
        self.splice(old, new, 0);
    }
    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        let new = self.take(m);
        let top = *self.stack.last().expect("a node on the stack");
        let old = self.at_path(top, path);
        self.splice(old, new, 0);
    }
    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        let old = self.ids[&id];
        let new = self.take(m);
        self.splice(old, new, 1);
    }
    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
        let old = self.ids[&id];
        let new = self.take(m);
        self.splice(old, new, -1);
    }
    fn set_attribute(
        &mut self,
        name: &'static str,
        _ns: Option<&'static str>,
        value: &AttributeValue,
        id: ElementId,
    ) {
        let n = self.ids[&id];
        let Kind::Element { attrs, .. } = &mut self.nodes[n].kind else { return };
        attrs.retain(|(k, _)| k != name);
        let v = match value {
            AttributeValue::Text(s) => Some(s.clone()),
            AttributeValue::Float(f) => Some(f.to_string()),
            AttributeValue::Int(i) => Some(i.to_string()),
            AttributeValue::Bool(true) => Some("true".into()),
            AttributeValue::Bool(false) | AttributeValue::None => None,
            AttributeValue::Listener(_) | AttributeValue::Any(_) => Some(String::new()),
        };
        if let Some(v) = v {
            attrs.push((name.to_string(), v));
        }
    }
    fn set_node_text(&mut self, value: &str, id: ElementId) {
        let n = self.ids[&id];
        self.nodes[n].kind = Kind::Text(value.to_string());
    }
    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        let n = self.ids[&id];
        if let Kind::Element { listeners, .. } = &mut self.nodes[n].kind {
            listeners.push(name.to_string());
        }
    }
    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        let n = self.ids[&id];
        if let Kind::Element { listeners, .. } = &mut self.nodes[n].kind {
            listeners.retain(|l| l != name);
        }
    }
    fn remove_node(&mut self, id: ElementId) {
        if let Some(n) = self.ids.get(&id).copied() {
            self.detach(n);
        }
    }
    fn push_root(&mut self, id: ElementId) {
        let n = self.ids[&id];
        self.stack.push(n);
    }
}

/// A mounted app a test can tap through.
pub struct TestDom {
    pub dom: VirtualDom,
    mirror: Mirror,
}

impl TestDom {
    pub fn new(app: fn() -> Element) -> Self {
        Self::mount(VirtualDom::new(app))
    }

    /// Mounts an already built VirtualDom (with props or root contexts) and
    /// lets its first effects run.
    pub fn mount(mut dom: VirtualDom) -> Self {
        dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter));
        let mut mirror = Mirror::new();
        dom.rebuild(&mut mirror);
        let mut t = TestDom { dom, mirror };
        t.settle();
        t
    }

    /// Runs queued effects and tasks and applies the re-renders they cause.
    pub fn settle(&mut self) {
        for _ in 0..12 {
            self.dom.process_events();
            self.dom.render_immediate(&mut self.mirror);
        }
    }

    /// The current HTML (server renderer, the same markup the app shows).
    pub fn html(&self) -> String {
        dioxus::ssr::render(&self.dom)
    }

    /// The visible text of the whole page, whitespace collapsed.
    pub fn text(&self) -> String {
        collapse(&self.mirror.text(self.mirror.root))
    }

    /// Elements matching `sel` (see `Mirror::matches`).
    fn all(&self, sel: &str) -> Vec<usize> {
        self.mirror.walk().into_iter().filter(|n| self.mirror.matches(*n, sel)).collect()
    }

    /// How many elements match `sel`.
    pub fn count(&self, sel: &str) -> usize {
        self.all(sel).len()
    }

    /// The text of every element matching `sel`, in document order.
    pub fn texts(&self, sel: &str) -> Vec<String> {
        self.all(sel).into_iter().map(|n| collapse(&self.mirror.text(n))).collect()
    }

    /// The value of `attr` on the first element matching `sel`.
    pub fn attr(&self, sel: &str, attr: &str) -> Option<String> {
        self.all(sel).first().and_then(|n| self.mirror.attr(*n, attr).map(str::to_string))
    }

    /// The element (or its nearest ancestor) that listens for `event`.
    fn target(&self, n: usize, event: &str) -> ElementId {
        let mut at = Some(n);
        while let Some(x) = at {
            if self.mirror.listens(x, event) {
                return self.mirror.by_node[&x];
            }
            at = self.mirror.nodes[x].parent;
        }
        panic!("nothing listens for {event} at {:?}", self.mirror.nodes[n].kind)
    }

    fn pick(&self, sel: &str, text: Option<&str>, event: &str) -> ElementId {
        let hits: Vec<usize> = self
            .all(sel)
            .into_iter()
            .filter(|n| text.is_none_or(|t| collapse(&self.mirror.text(*n)).contains(t)))
            .collect();
        // The first hit with no other hit inside it, so `div` + text finds
        // the innermost element holding the text, not its parent.
        let inner = |n: &usize| !hits.iter().any(|h| h != n && self.mirror.is_inside(*h, *n));
        let Some(n) = hits.iter().find(|n| inner(n)) else {
            panic!("no `{sel}` with text {text:?} in:\n{}", self.text());
        };
        self.target(*n, event)
    }

    fn fire(&mut self, id: ElementId, name: &str, data: Box<dyn Any>, bubbles: bool) {
        let data: Rc<dyn Any> = Rc::new(PlatformEventData::new(data));
        self.dom.runtime().handle_event(name, Event::new(data, bubbles), id);
        self.settle();
    }

    /// Taps the first element matching `sel`.
    pub fn click(&mut self, sel: &str) {
        let id = self.pick(sel, None, "click");
        self.fire(id, "click", Box::new(SerializedMouseData::default()), true);
    }

    /// Taps the `i`th element matching `sel`.
    pub fn click_nth(&mut self, sel: &str, i: usize) {
        let n = *self.all(sel).get(i).unwrap_or_else(|| panic!("no element {i} for `{sel}`"));
        let id = self.target(n, "click");
        self.fire(id, "click", Box::new(SerializedMouseData::default()), true);
    }

    /// Taps the element matching `sel` whose text contains `text`.
    pub fn click_text(&mut self, sel: &str, text: &str) {
        let id = self.pick(sel, Some(text), "click");
        self.fire(id, "click", Box::new(SerializedMouseData::default()), true);
    }

    /// Types `value` into the field matching `sel` (an `input` event).
    pub fn input(&mut self, sel: &str, value: &str) {
        self.input_nth(sel, 0, value);
    }

    /// Types into the `i`th field matching `sel`.
    pub fn input_nth(&mut self, sel: &str, i: usize, value: &str) {
        let n = *self.all(sel).get(i).unwrap_or_else(|| panic!("no field {i} for `{sel}`"));
        let id = self.target(n, "input");
        let data = SerializedFormData::new(value.to_string(), Vec::new());
        self.fire(id, "input", Box::new(data), true);
    }

    /// Sends a `change` with `value` (a checkbox: "true" / "false"; a select: the option).
    pub fn change_nth(&mut self, sel: &str, i: usize, value: &str) {
        let n = *self.all(sel).get(i).unwrap_or_else(|| panic!("no field {i} for `{sel}`"));
        let id = self.target(n, "change");
        let data = SerializedFormData::new(value.to_string(), Vec::new());
        self.fire(id, "change", Box::new(data), true);
    }

    /// Presses `key` on the element matching `sel`.
    pub fn key(&mut self, sel: &str, key: Key) {
        let id = self.pick(sel, None, "keydown");
        let data = SerializedKeyboardData::new(
            key,
            Code::Unidentified,
            Location::Standard,
            false,
            Modifiers::empty(),
            false,
        );
        self.fire(id, "keydown", Box::new(data), true);
    }

    /// Fires `name` with `data` (a `Serialized*Data`) on the first element
    /// matching `sel` that listens for it.
    pub fn fire_on(&mut self, sel: &str, name: &str, data: Box<dyn Any>) {
        let id = self.pick(sel, None, name);
        self.fire(id, name, data, true);
    }

    /// The CSS animation on the element matching `sel` ran out.
    pub fn animation_end(&mut self, sel: &str) {
        let data: dioxus::html::SerializedAnimationData =
            serde_json::from_str(r#"{"animation_name":"drain","pseudo_element":"","elapsed_time":1.0}"#)
                .expect("animation data");
        self.fire_on(sel, "animationend", Box::new(data));
    }

    /// Runs `f` inside the app's root scope (to set a signal from a test).
    pub fn in_root<T>(&mut self, f: impl FnOnce() -> T) -> T {
        let out = self.dom.in_scope(ScopeId::APP, f);
        self.settle();
        out
    }
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicks_and_typing_rerender_the_mirror() {
        fn app() -> Element {
            let mut n = use_signal(|| 0);
            let mut name = use_signal(String::new);
            let mut on = use_signal(|| false);
            rsx! {
                p { class: "count", "taps {n}" }
                button { class: "btn add", onclick: move |_| n += 1, span { "add" } }
                if n() > 1 {
                    p { class: "many", "many" }
                }
                input { class: "name", value: "{name}", oninput: move |e| name.set(e.value()) }
                p { class: "hello", "hi {name}" }
                input { r#type: "checkbox", onchange: move |e| on.set(e.checked()) }
                p { "data-on": if on() { "yes" } else { "no" }, "flag" }
            }
        }
        let mut t = TestDom::new(app);
        assert_eq!(t.texts("p.count"), ["taps 0"]);
        t.click_text("button", "add");
        t.click(".btn.add");
        assert_eq!(t.texts("p.count"), ["taps 2"]);
        assert_eq!(t.count("p.many"), 1);
        t.input("input.name", "Mohamed");
        assert_eq!(t.texts(".hello"), ["hi Mohamed"]);
        assert!(t.html().contains("hi Mohamed"));
        t.change_nth("input[type=checkbox]", 0, "true");
        assert_eq!(t.attr("p[data-on]", "data-on").as_deref(), Some("yes"));
        assert_eq!(t.count("#root > p"), 4);
        assert_eq!(t.count("#root button span"), 1);
        assert_eq!(t.count("#root > span"), 0);
    }
}
