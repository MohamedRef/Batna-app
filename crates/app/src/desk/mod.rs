//! The tablet and Mac desk (PARITY.md §6): the labelled sidebar, two panes on
//! wide windows, and the per-device switches for it, hover peeks and the
//! keyboard. Phones get none of it in the way: everything here keys on window
//! width, never on the device.
//!
//! Feature screens opt in without depending on this module's internals:
//! - [`Panes`]: the question on one side, the explanation on the other (≥840).
//! - [`ListDetail`]: a list beside the open item (≥840).
//! - `"data-peek": text` on any element: its text shows on pen or mouse hover
//!   (a book page's `src.quote`, an MCQ option's `why` after answering).
//!   [`crate::hover::BookPage`] does it for the page chip.
//! - `"aria-keyshortcuts": "Space"` (or `"1"`, `"A"`, `"Enter"`, `"Escape"`)
//!   on a button: that key presses it (see [`crate::shortcuts`]).

mod prefs;

use dioxus::prelude::*;

pub use prefs::{MOUSE_HOVER, NAV_STYLE, NavStyle, PEN_HOVER};

use crate::i18n::{desk as tx, hover as htx, keys as ktx};

const DESK_CSS: &str = include_str!("../../assets/desk.css");
const NAV_CSS: &str = include_str!("../../assets/nav.css");

/// Rendered once by the app root: the desk's styles, the stored device
/// switches, the hover script, the keyboard and (on the Mac) the menu.
#[component]
pub fn Chrome() -> Element {
    prefs::use_prefs();
    crate::hover::use_hover();
    crate::shortcuts::use_shortcuts();
    #[cfg(all(feature = "desktop", not(target_os = "android")))]
    crate::settings::menu::use_menu_events();
    rsx! {
        style { {NAV_CSS} }
        style { {DESK_CSS} }
        crate::shortcuts::KeysSheet {}
    }
}

/// Two panes from 840dp: `primary` (the question) and `secondary` (the
/// explanation, `why` lines, guideline update and book page). Narrower, the
/// secondary pane follows the primary one. With no `secondary` yet (before
/// the answer), wide windows keep the place with a quiet hint.
#[component]
pub fn Panes(primary: Element, secondary: Option<Element>) -> Element {
    rsx! {
        div { class: "desk-split",
            div { class: "desk-main", {primary} }
            match secondary {
                Some(side) => rsx! {
                    aside { class: "desk-side", aria_label: tx::SIDE_LABEL, {side} }
                },
                None => rsx! {
                    aside { class: "desk-side desk-waiting", aria_label: tx::SIDE_LABEL,
                        p { class: "muted small", {tx::SIDE_WAITING} }
                    }
                },
            }
        }
    }
}

/// A list beside the open item from 840dp (the question bank). Narrower,
/// only one shows: the item when there is one, else the list.
#[component]
pub fn ListDetail(list: Element, detail: Option<Element>) -> Element {
    let open = detail.is_some();
    rsx! {
        div { class: "desk-list-detail", "data-open": if open { "yes" } else { "no" },
            nav { class: "desk-list", aria_label: tx::LIST_LABEL, {list} }
            if let Some(d) = detail {
                section { class: "desk-detail", {d} }
            }
        }
    }
}

/// Settings › Tablet and Mac: the sidebar style, hover peeks and the
/// keyboard shortcut list. Chunk 1's settings sheet shows it as one section.
#[component]
pub fn DeviceSettings() -> Element {
    let nav = *NAV_STYLE.read();
    let pen = *PEN_HOVER.read();
    let mouse = *MOUSE_HOVER.read();
    let terms = crate::hover::term_count();
    rsx! {
        section { class: "card desk-settings",
            h2 { {tx::SECTION} }
            p { class: "muted small", {tx::SECTION_NOTE} }

            fieldset { class: "desk-field",
                legend { {tx::NAV_STYLE} }
                div { class: "desk-segment", role: "radiogroup",
                    for (style, label) in [(NavStyle::Labels, tx::NAV_LABELS), (NavStyle::Slim, tx::NAV_SLIM)] {
                        button {
                            key: "{style:?}",
                            role: "radio",
                            "aria-checked": if nav == style { "true" } else { "false" },
                            onclick: move |_| *NAV_STYLE.write() = style,
                            {label}
                        }
                    }
                }
            }

            Toggle { label: htx::PEN, note: htx::PEN_NOTE, on: pen, onchange: move |v| *PEN_HOVER.write() = v }
            Toggle { label: htx::MOUSE, note: htx::MOUSE_NOTE, on: mouse, onchange: move |v| *MOUSE_HOVER.write() = v }
            p { class: "muted small", bdi { "{terms}" } " " {htx::TERMS} }

            button {
                class: "desk-keys-button",
                onclick: move |_| crate::shortcuts::open_sheet(true),
                "⌨ " {ktx::OPEN}
            }
        }
    }
}

#[component]
fn Toggle(label: &'static str, note: &'static str, on: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "desk-toggle",
            span { class: "desk-toggle-text",
                span { {label} }
                span { class: "muted small", {note} }
            }
            input {
                r#type: "checkbox",
                role: "switch",
                checked: on,
                onchange: move |e| onchange.call(e.checked()),
            }
        }
    }
}
