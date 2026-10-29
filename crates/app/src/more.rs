//! Phones: the bottom bar keeps four screens (Today, Lessons, Review,
//! Questions) and a "More" tab. More opens a bounded sheet above the bar,
//! grouped as in the 2026-10 redesign: practice (Cases, Oral) and your
//! references and tools (Progress, Settings). From 600dp the rail and
//! sidebar show every screen, so the tab and the sheet hide (shape.css).

use dioxus::prelude::*;

use crate::{Screen, Ui, i18n, icons, settings};

/// Screens that sit under More on phones, in the order they appear there.
pub const SCREENS: [Screen; 3] = [Screen::Cases, Screen::Oral, Screen::Stats];

/// One row in More.
#[derive(Clone, Copy, PartialEq)]
pub enum Row {
    /// A screen of the bar (its label and icon).
    Screen(Screen),
    /// The Settings sheet.
    Settings,
}

/// More's groups: a heading and its rows, in order.
const GROUPS: [(&str, &[Row]); 2] = [
    (i18n::nav::MORE_PRACTICE, &[Row::Screen(Screen::Cases), Row::Screen(Screen::Oral)]),
    (i18n::nav::MORE_TOOLS, &[Row::Screen(Screen::Stats), Row::Settings]),
];

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
                for (title, rows) in GROUPS {
                    div { key: "{title}", class: "more-group", role: "group", aria_label: title,
                        h2 { class: "more-group-title", {title} }
                        for (k, &row) in rows.iter().enumerate() {
                            button {
                                key: "{k}",
                                class: "more-row",
                                "aria-current": if row == Row::Screen(current) { "page" } else { "false" },
                                onclick: move |_| {
                                    open.set(false);
                                    match row {
                                        Row::Screen(screen) => ui.screen.set(screen),
                                        Row::Settings => settings.show(None),
                                    }
                                },
                                span { class: "row-icon", aria_hidden: "true",
                                    match row {
                                        Row::Screen(screen) => icons::nav(screen),
                                        Row::Settings => icons::nav(Screen::Settings),
                                    }
                                }
                                span { class: "label",
                                    match row {
                                        Row::Screen(screen) => screen.label(),
                                        Row::Settings => Screen::Settings.label(),
                                    }
                                }
                                span { class: "chev", aria_hidden: "true" }
                            }
                        }
                    }
                }
                // Back (Android) or a tap here closes the sheet.
                button {
                    class: "btn btn-ghost more-close",
                    "data-back": "true",
                    onclick: move |_| open.set(false),
                    {i18n::settings::CLOSE}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_bar_is_today_lessons_review_questions_more() {
        let bar: Vec<Screen> = Screen::NAV.into_iter().filter(|s| !holds(*s)).collect();
        assert_eq!(bar, [Screen::Today, Screen::Learn, Screen::Study, Screen::Bank]);
    }

    #[test]
    fn every_screen_is_reachable_on_a_phone() {
        // Nothing is dropped: a screen off the bar is in one of More's groups.
        for screen in Screen::NAV.into_iter().filter(|s| holds(*s)) {
            assert!(
                GROUPS.iter().any(|(_, rows)| rows.contains(&Row::Screen(screen))),
                "{screen:?} has no row in More"
            );
        }
        let grouped: Vec<Screen> = GROUPS
            .iter()
            .flat_map(|(_, rows)| rows.iter())
            .filter_map(|r| if let Row::Screen(s) = r { Some(*s) } else { None })
            .collect();
        assert_eq!(grouped, SCREENS);
        assert!(GROUPS.iter().any(|(_, rows)| rows.contains(&Row::Settings)), "Settings is in More");
    }
}
