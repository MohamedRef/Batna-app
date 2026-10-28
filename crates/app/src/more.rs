//! Phones: the bottom bar keeps four screens and a "More" tab; Cases, Oral
//! and Settings open from a small sheet above the bar. From 600dp the rail
//! and sidebar show every screen, so the tab and the sheet hide (shape.css).

use dioxus::prelude::*;

use crate::{Screen, Ui, i18n, icons, settings};

/// Screens that sit under More on phones.
pub const SCREENS: [Screen; 2] = [Screen::Cases, Screen::Oral];

pub fn holds(screen: Screen) -> bool {
    SCREENS.contains(&screen)
}

/// The bar class for a screen: More's screens hide from the phone bar.
pub fn nav_class(screen: Screen) -> &'static str {
    if holds(screen) { "nav-item nav-extra" } else { "nav-item" }
}

/// The More tab and its sheet.
#[component]
pub fn More() -> Element {
    let mut ui = use_context::<Ui>();
    let settings = use_context::<settings::SettingsNav>();
    let mut open = use_signal(|| false);
    let current = *ui.screen.read();
    rsx! {
        button {
            class: "nav-item nav-more",
            "aria-current": if holds(current) { "page" } else { "false" },
            "aria-expanded": if open() { "true" } else { "false" },
            "aria-haspopup": "dialog",
            "aria-label": i18n::nav::MORE,
            onclick: move |_| open.toggle(),
            span { class: "glyph", aria_hidden: "true", {icons::more()} }
            span { class: "label", {i18n::nav::MORE} }
        }
        if open() {
            div { class: "more-backdrop", onclick: move |_| open.set(false) }
            div {
                class: "more-sheet",
                role: "dialog",
                aria_label: i18n::nav::MORE,
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        open.set(false);
                    }
                },
                for screen in SCREENS {
                    button {
                        key: "{screen:?}",
                        class: "more-row",
                        "aria-current": if current == screen { "page" } else { "false" },
                        onclick: move |_| {
                            ui.screen.set(screen);
                            open.set(false);
                        },
                        span { class: "glyph", aria_hidden: "true", {icons::nav(screen)} }
                        span { class: "label", {screen.label()} }
                    }
                }
                button {
                    class: "more-row",
                    onclick: move |_| {
                        open.set(false);
                        settings.show(None);
                    },
                    span { class: "glyph", aria_hidden: "true", {icons::nav(Screen::Settings)} }
                    span { class: "label", {Screen::Settings.label()} }
                }
            }
        }
    }
}
