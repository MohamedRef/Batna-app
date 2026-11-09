//! The info sheet: what a tap opens to tell more (a book page's sheet, from
//! every "ص N" chip). A bottom sheet in its own raised colour (`--sheet`,
//! tested per theme in `batna_core::theme`) that Mohamed resizes by its
//! handle: free dragging, with a pull towards a peek, half and near-full
//! stop, a fling to the next stop, and a drag (or fling) down to close. It
//! reopens at the size it was left. There is no scrim: the page under it
//! keeps scrolling, and the sheet floats over it without moving it
//! (Mohamed, 2026-10-29). A tap outside it closes it (Mohamed, 2026-11-03).
//! On a tablet or the Mac (600dp wide and 480dp tall) it is a floating card
//! next to the chip that opened it instead, moved by its heading row and
//! sized from any edge or corner (remembered), never a tall strip at the
//! window's foot (Mohamed, 2026-11-09).
//!
//! Adapted from kabar-app main: crates/app/src/sheet.rs (PR #453). The
//! gestures live in `assets/sheet.js`, installed once by [`use_gestures`]:
//! a drag must follow the finger frame by frame, which a round trip to Rust
//! per pointer move can't do on a phone. It closes the sheet by clicking its
//! close button, so Rust stays the one owner of what is open.

use dioxus::prelude::*;

use crate::i18n::sheet as tx;
use crate::look::{Css, css};

pub const CSS: Css = css!("assets/sheet.css");
/// Where the floating card can be sized from: block start/end (top,
/// bottom), inline start/end, and the four corners.
const RESIZE_EDGES: [&str; 8] = ["bs", "be", "is", "ie", "bs-is", "bs-ie", "be-is", "be-ie"];
const SHEET_JS: &str = include_str!("../assets/sheet.js");

/// Installs the sheet gestures for the whole app (listeners are delegated
/// from `document`, so sheets opened later need nothing).
pub fn use_gestures() {
    use_hook(|| {
        document::eval(SHEET_JS);
    });
}

/// An info sheet around `children`. `head` is the sheet's heading row (the
/// close button sits at its end); Android Back, Esc, a drag down and a tap
/// outside close it.
#[component]
pub fn Sheet(label_id: String, on_close: EventHandler, head: Element, children: Element) -> Element {
    rsx! {
        div {
            class: "bsheet",
            role: "dialog",
            aria_modal: "false",
            aria_labelledby: "{label_id}",
            div {
                class: "bsheet-grip",
                role: "button",
                tabindex: "0",
                aria_label: tx::RESIZE,
                title: tx::RESIZE,
                span { class: "bsheet-bar" }
            }
            div { class: "bsheet-head",
                div { class: "bsheet-title", {head} }
                button {
                    class: "icon-button bsheet-close",
                    "data-back": "true",
                    "data-sheet-close": "true",
                    aria_label: tx::CLOSE,
                    title: tx::CLOSE,
                    onclick: move |_| on_close.call(()),
                    span { aria_hidden: "true", "✕" }
                }
            }
            div { class: "bsheet-body", {children} }
            // Edges and corners that size the floating card (tablet and
            // Mac only; sheet.js does the dragging).
            for edge in RESIZE_EDGES {
                div { key: "{edge}", class: "bsheet-rz", "data-rz": edge, aria_hidden: "true" }
            }
        }
    }
}
