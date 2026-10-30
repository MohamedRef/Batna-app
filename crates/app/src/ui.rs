// adapted from kabar-app: crates/app/src/ui.rs
//! Shared UI primitives. Features compose these and own their state;
//! surface, spacing and shape come from the shared CSS tokens.

use dioxus::prelude::*;

/// A content surface. Keep actions on real buttons inside it; a Card is
/// never itself clickable. `class` adds feature layout, not another skin.
#[component]
pub fn Card(#[props(default)] class: &'static str, children: Element) -> Element {
    rsx! { section { class: "card {class}", {children} } }
}

/// A content surface that retains its native `div` semantics. Use this for
/// existing div cards; `Card` retains `section` semantics instead.
#[component]
pub fn DivCard(#[props(default)] class: &'static str, children: Element) -> Element {
    rsx! { div { class: "card {class}", {children} } }
}

/// A navigation row: one button owns the whole hit area and accessible
/// name. Settings and other grouped lists share its markup and CSS.
#[component]
pub fn ActionRow(
    title: String,
    detail: String,
    icon: Element,
    #[props(default)] selected: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "activity-row",
            aria_current: if selected { "page" } else { "false" },
            onclick: move |event| onclick.call(event),
            span { class: "row-icon", aria_hidden: "true", {icon} }
            span { class: "row-copy",
                strong { bdi { class: "row-title", "{title}" } }
                if !detail.is_empty() { small { "{detail}" } }
            }
            span { class: "row-end", span { class: "chev", aria_hidden: "true" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise the real UI components, including native child controls. These
    // are VirtualDom smoke tests, not a substitute for WebView visual QA.
    fn production_surfaces() -> Element {
        rsx! {
            Card { class: "grouped",
                h2 { "Section heading" }
                button { r#type: "button", aria_label: "Start", "Start" }
            }
            DivCard { class: "flat",
                label { r#for: "choice", "Your choice" }
                input { id: "choice", r#type: "text", aria_label: "Your choice" }
            }
            ActionRow {
                title: "Open".to_string(),
                detail: "An accessible navigation row".to_string(),
                icon: rsx! { span { "›" } },
                onclick: move |_| {},
            }
        }
    }

    #[test]
    fn shared_surfaces_render_with_native_controls() {
        let mut dom = VirtualDom::new(production_surfaces);
        dom.rebuild_in_place();
        let _ = dom.render_immediate_to_vec();
    }

    #[test]
    fn shared_surface_styles_cover_widths_themes_text_and_reduced_motion() {
        let base = include_str!("../assets/main.css");
        let shape = include_str!("../assets/shape.css");
        let appearance = include_str!("../assets/appearance.css");
        let parts = include_str!("../assets/parts.css");
        assert!(shape.contains("@media (max-width: 599.98px)"));
        assert!(shape.contains("@media (max-width: 839.98px)"));
        assert!(base.contains("--card-pad"));
        assert!(parts.contains("--radius-card"));
        assert!(base.contains("bdi[dir=\\\"ltr\\\"]"));
        assert!(base.contains("prefers-reduced-motion"));
        assert!(appearance.contains("data-appearance=\\\"glossy\\\""));
        assert!(appearance.contains("prefers-reduced-transparency"));
    }
}
