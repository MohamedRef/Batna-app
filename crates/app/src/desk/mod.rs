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

use dioxus::prelude::*;

use crate::Ui;
use crate::i18n::{desk as tx, hover as htx, keys as ktx};
use crate::look::{Css, css};
use crate::prefs::NavStyle;
use crate::settings::{DeviceOnly, Section};

pub(crate) const DESK_CSS: Css = css!("assets/desk.css");
pub(crate) const NAV_CSS: Css = css!("assets/nav.css");

/// Rendered once by the app root: the desk's styles, the stored device
/// switches, the hover script, the keyboard and (on the Mac) the menu.
#[component]
pub fn Chrome() -> Element {
    let ui = use_context::<Ui>();
    // The device switches reach nav.css and hover.js as data-* on <html>.
    use_effect(move || {
        let p = *ui.prefs.read();
        let on_off = |on: bool| if on { "on" } else { "off" };
        document::eval(&format!(
            "const d = document.documentElement.dataset; d.navStyle = {:?}; d.penHover = {:?}; d.mouseHover = {:?};",
            p.nav_style.as_str(),
            on_off(p.pen_hover),
            on_off(p.mouse_hover),
        ));
        // CSS follows the saved width; an unsaved drag remains instant in JS.
        // Zero restores the original 60/40 or 40/60 preset per study item.
        document::eval(&format!(
            "if ({} === 0) document.documentElement.style.removeProperty('--desk-split-width'); else document.documentElement.style.setProperty('--desk-split-width', '{}%');",
            p.desk_split,
            p.desk_split,
        ));
    });
    // A content pack switched in while open restyles the nav and desk too.
    let _ = crate::content::REV.read();
    crate::hover::use_hover();
    crate::shortcuts::use_shortcuts();
    use_future(move || async move {
        let mut script = document::eval(include_str!("resize.js"));
        while let Ok(message) = script.recv::<String>().await {
            if let Some(width) = message.strip_prefix("split:").and_then(|s| s.parse::<u8>().ok())
                && (width == 0 || (35..=70).contains(&width))
            {
                ui.update_prefs(|prefs| prefs.desk_split = width);
            }
        }
    });
    #[cfg(all(feature = "desktop", not(target_os = "android")))]
    crate::settings::menu::use_menu_events();
    rsx! {
        style { {NAV_CSS.text()} }
        style { {DESK_CSS.text()} }
    }
}

/// Two panes from 840dp: `primary` (the question) and `secondary` (the
/// explanation, `why` lines, guideline update and book page). Narrower, the
/// secondary pane follows the primary one. With no `secondary` yet (before
/// the answer), wide windows keep the place with a quiet hint.
///
/// `wide_side` gives the secondary pane the wide column, for long answers.
/// `footer` (the grade row) sits under both panes at full width.
#[component]
pub fn Panes(
    primary: Element,
    secondary: Option<Element>,
    #[props(default)] wide_side: bool,
    footer: Option<Element>,
) -> Element {
    let original = if wide_side { 40 } else { 60 };
    let saved = try_use_context::<Ui>().map(|ui| ui.prefs.read().desk_split).unwrap_or(0);
    let width = if saved == 0 { original } else { saved };
    rsx! {
        div { class: "desk-split", "data-wide": if wide_side { "side" } else { "main" },
            div { class: "desk-main", {primary} }
            button {
                r#type: "button",
                class: "desk-resize",
                role: "separator",
                tabindex: "0",
                aria_orientation: "vertical",
                aria_valuemin: "35",
                aria_valuemax: "70",
                aria_valuenow: "{width}",
                aria_valuetext: "{width}%",
                aria_label: tx::SPLIT_HANDLE,
                title: tx::SPLIT_HELP,
                "data-default": "{original}",
                span { aria_hidden: "true", class: "desk-resize-grip" }
            }
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
        if let Some(f) = footer {
            div { class: "desk-footer", {f} }
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

/// Settings › Tablet (Android: pen hover) and Settings › Mac (mouse hover):
/// the sidebar style, the hover peek and the keyboard shortcut list.
#[component]
pub fn DeviceSettings(section: Section) -> Element {
    let ui = use_context::<Ui>();
    let now = *ui.prefs.read();
    let terms = crate::hover::term_count();
    let set = move |f: &dyn Fn(&mut crate::prefs::DevicePrefs)| ui.update_prefs(f);
    let tablet = section == Section::Tablet;
    rsx! {
        crate::ui::Card { class: "desk-settings",
            h2 { {tx::NAV_STYLE} DeviceOnly {} }
            div { class: "desk-segment", role: "radiogroup", aria_label: tx::NAV_STYLE,
                for (style, label) in [(NavStyle::Labels, tx::NAV_LABELS), (NavStyle::Slim, tx::NAV_SLIM)] {
                    button {
                        key: "{style.as_str()}",
                        role: "radio",
                        aria_checked: if now.nav_style == style { "true" } else { "false" },
                        onclick: move |_| set(&|p| p.nav_style = style),
                        {label}
                    }
                }
            }
        }
        crate::ui::Card { class: "desk-settings",
            h2 { {htx::TITLE} DeviceOnly {} }
            if tablet {
                Toggle {
                    label: htx::PEN,
                    note: htx::PEN_NOTE,
                    on: now.pen_hover,
                    onchange: move |v: bool| set(&|p| p.pen_hover = v),
                }
            } else {
                Toggle {
                    label: htx::MOUSE,
                    note: htx::MOUSE_NOTE,
                    on: now.mouse_hover,
                    onchange: move |v: bool| set(&|p| p.mouse_hover = v),
                }
            }
            p { class: "muted small", bdi { "{terms}" } " " {htx::TERMS} }
        }
        // The whole card opens the shortcut sheet (the button's click bubbles up).
        crate::ui::Card { class: "card-action desk-settings",
            onclick: move |_| crate::shortcuts::open_sheet(true),
            h2 { {ktx::TITLE} }
            button {
                class: "desk-keys-button",
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

/// A deliberate one-click focus for Mac reading. Unlike phone focus, this
/// keeps its choice on this device and only hides the sidebar while a study
/// item or lesson player is actually present.
#[component]
pub fn FocusToggle() -> Element {
    let ui = use_context::<Ui>();
    let focused = ui.prefs.read().desk_focus;
    let label = if focused { tx::FOCUS_LEAVE } else { tx::FOCUS_ENTER };
    rsx! {
        button {
            r#type: "button",
            class: "desk-focus-toggle",
            aria_pressed: if focused { "true" } else { "false" },
            aria_label: label,
            title: label,
            onclick: move |_| ui.update_prefs(|p| p.desk_focus = !p.desk_focus),
            "{label}"
        }
    }
}

#[cfg(test)]
mod resize_tests {
    use super::*;

    #[test]
    fn divider_has_keyboard_and_pointer_contract() {
        let script = include_str!("resize.js");
        for required in [
            "setPointerCapture", "ArrowLeft", "ArrowRight", "pointercancel",
            "dblclick", "desk-resizing", "split:0", "dioxus.send",
            "getBoundingClientRect", "rtl",
        ] {
            assert!(script.contains(required), "desktop divider needs {required}");
        }
        let css = DESK_CSS.text();
        assert!(css.contains(".desk-resize") && css.contains("data-desk-focus"));
    }
}
