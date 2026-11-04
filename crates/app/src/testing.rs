//! Test helper: press a button (or type into a field) of a rendered
//! `VirtualDom`, found by its text, so the screen's own handlers run as in
//! the app. SSR alone renders states; this drives the steps between them.

use std::any::Any;
use std::rc::Rc;

use dioxus::core::{AttributeValue, DynamicNode, ListenerCallback, NoOpMutations, TemplateNode, VNode};
use dioxus::html::{
    PlatformEventData, SerializedFormData, SerializedHtmlEventConverter, SerializedMouseData,
};
use dioxus::prelude::*;

/// One event listener and the text of the element that holds it.
struct Found {
    event: &'static str,
    label: String,
    callback: ListenerCallback,
}

/// Renders the DOM's current state as HTML.
pub fn html(dom: &VirtualDom) -> String {
    dioxus::ssr::render(dom)
}

/// Presses the `n`th (from 0) element whose text, title or aria-label
/// contains `label`, then lets the DOM re-render.
pub fn click_nth(dom: &mut VirtualDom, label: &str, n: usize) {
    let data = PlatformEventData::new(Box::new(SerializedMouseData::default()));
    fire(dom, "click", label, n, Rc::new(data));
}

pub fn click(dom: &mut VirtualDom, label: &str) {
    click_nth(dom, label, 0);
}

/// Types `value` into the field whose text, label attributes or input type
/// contain `label` (its `oninput` or `onchange`). A checkbox takes "true".
pub fn type_into(dom: &mut VirtualDom, label: &str, value: &str) {
    type_nth(dom, label, 0, value);
}

/// Like [`type_into`], into the `n`th (from 0) such field.
pub fn type_nth(dom: &mut VirtualDom, label: &str, n: usize, value: &str) {
    let data = PlatformEventData::new(Box::new(SerializedFormData::new(value.into(), Vec::new())));
    let found = listeners(dom);
    let event = ["input", "change"]
        .into_iter()
        .find(|e| found.iter().filter(|f| f.event == *e && f.label.contains(label)).nth(n).is_some())
        .unwrap_or_else(|| panic!("no field #{n} {label:?}"));
    fire(dom, event, label, n, Rc::new(data));
}

/// Submits the form whose text contains `label`, as Enter or its submit button would.
pub fn submit(dom: &mut VirtualDom, label: &str) {
    let data = PlatformEventData::new(Box::new(SerializedFormData::new(String::new(), Vec::new())));
    fire(dom, "submit", label, 0, Rc::new(data));
}

fn fire(dom: &mut VirtualDom, event: &'static str, label: &str, n: usize, data: Rc<dyn Any>) {
    static CONVERTER: std::sync::Once = std::sync::Once::new();
    CONVERTER.call_once(|| dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter)));
    let found = listeners(dom);
    let matches: Vec<&Found> = found.iter().filter(|f| f.event == event && f.label.contains(label)).collect();
    let Some(target) = matches.get(n) else {
        let all: Vec<&str> = found.iter().filter(|f| f.event == event).map(|f| f.label.as_str()).collect();
        panic!("no {event} #{n} for {label:?}; there are {all:?}");
    };
    let callback = target.callback.clone();
    dom.in_runtime(|| callback.call(dioxus::core::Event::new(data, true)));
    dom.render_immediate(&mut NoOpMutations);
}

fn listeners(dom: &VirtualDom) -> Vec<Found> {
    let mut out = Vec::new();
    walk(dom, dom.base_scope().root_node(), &mut out);
    out
}

fn walk(dom: &VirtualDom, node: &VNode, out: &mut Vec<Found>) {
    let template = node.template;
    for (idx, attrs) in node.dynamic_attrs.iter().enumerate() {
        for attr in attrs.iter() {
            if let AttributeValue::Listener(callback) = &attr.value {
                let path = template.attr_paths[idx];
                let element = at(template.roots, path);
                let mut label = String::new();
                text(node, element, &mut label);
                for a in attrs.iter() {
                    if let AttributeValue::Text(t) = &a.value
                        && matches!(a.name, "aria-label" | "title")
                    {
                        label.push(' ');
                        label.push_str(t);
                    }
                }
                let event = attr.name.strip_prefix("on").unwrap_or(attr.name);
                out.push(Found { event, label, callback: callback.clone() });
            }
        }
    }
    for (i, dynamic) in node.dynamic_nodes.iter().enumerate() {
        match dynamic {
            DynamicNode::Component(c) => {
                if let Some(scope) = c.mounted_scope(i, node, dom) {
                    walk(dom, scope.root_node(), out);
                }
            }
            DynamicNode::Fragment(children) => children.iter().for_each(|child| walk(dom, child, out)),
            DynamicNode::Text(_) | DynamicNode::Placeholder(_) => {}
        }
    }
}

fn at<'a>(roots: &'a [TemplateNode], path: &[u8]) -> &'a TemplateNode {
    let mut node = &roots[path[0] as usize];
    for &i in &path[1..] {
        let TemplateNode::Element { children, .. } = node else {
            unreachable!("paths lead through elements")
        };
        node = &children[i as usize];
    }
    node
}

/// The visible text under a template node, with its static title,
/// aria-label, placeholder and input type.
fn text(vnode: &VNode, node: &TemplateNode, out: &mut String) {
    match node {
        TemplateNode::Element { children, attrs, .. } => {
            for a in attrs.iter() {
                if let dioxus::core::TemplateAttribute::Static { name, value, .. } = a
                    && matches!(*name, "aria-label" | "title" | "type" | "placeholder")
                {
                    out.push_str(value);
                    out.push(' ');
                }
            }
            children.iter().for_each(|c| text(vnode, c, out));
        }
        TemplateNode::Text { text } => out.push_str(text),
        TemplateNode::Dynamic { id } => dynamic_text(&vnode.dynamic_nodes[*id], out),
    }
}

fn dynamic_text(node: &DynamicNode, out: &mut String) {
    match node {
        DynamicNode::Text(t) => out.push_str(&t.value),
        DynamicNode::Fragment(children) => {
            for child in children {
                child.template.roots.iter().for_each(|r| text(child, r, out));
            }
        }
        DynamicNode::Component(_) | DynamicNode::Placeholder(_) => {}
    }
}
