//! Small stroke icons, drawn inline so they follow `currentColor`.
// adapted from kabar-app main: crates/app/src/icons.rs

use dioxus::prelude::*;

use crate::Screen;

/// The path for each screen's nav icon (24×24, stroked).
pub fn nav_path(screen: Screen) -> &'static str {
    match screen {
        // sun over a horizon
        Screen::Today => "M4 18h16M7 18a5 5 0 0 1 10 0M12 6V4M5.6 9.6 4.2 8.2M18.4 9.6l1.4-1.4",
        // a light bulb: understanding first
        Screen::Learn => {
            "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.5 1 1.2 1 2.1h5c0-.9.4-1.6 1-2.1A6 6 0 0 0 12 3z"
        }
        // open book
        Screen::Study => "M3 6c3-1 6-1 9 1 3-2 6-2 9-1v12c-3-1-6-1-9 1-3-2-6-2-9-1zM12 7v12",
        // a list of answer choices, the second one ticked
        Screen::Bank => "M4 6h.01M4 12l1.5 1.5L8 11M4 18h.01M10 6h10M11 12h9M10 18h10",
        // stethoscope
        Screen::Cases => "M6 3v5a4 4 0 0 0 8 0V3M10 12v3a4 4 0 0 0 8 0v-2M18 13a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
        // microphone
        Screen::Oral => "M9 4a3 3 0 0 1 6 0v7a3 3 0 0 1-6 0zM5 11a7 7 0 0 0 14 0M12 18v3M9 21h6",
        // three bars on a baseline
        Screen::Stats => "M4 20h16M7 16v-5M12 16V6M17 16v-8",
        Screen::Settings => GEAR,
    }
}

/// Settings: a cog with eight teeth.
const GEAR: &str = "M12 9a3 3 0 1 0 0 6 3 3 0 1 0 0-6zM10.6 3h2.8l.5 2.3 1.7.9 2.2-.8 1.4 2.4-1.7 1.6v1.9l1.7 1.6-1.4 2.4-2.2-.8-1.7.9-.5 2.3h-2.8l-.5-2.3-1.7-.9-2.2.8-1.4-2.4 1.7-1.6v-1.9L4.8 7.8l1.4-2.4 2.2.8 1.7-.9z";

pub fn nav(screen: Screen) -> Element {
    stroke(nav_path(screen))
}

fn stroke(d: &'static str) -> Element {
    rsx! {
        svg {
            class: "icon",
            width: "22",
            height: "22",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_screen_has_a_path_of_drawing_commands() {
        for s in [Screen::NAV.as_slice(), &[Screen::Settings]].concat() {
            let d = nav_path(s);
            assert!(d.starts_with('M'), "{s:?}");
            assert!(d.chars().all(|c| "MmLlHhVvCcSsQqTtAaZz0123456789.,- ".contains(c)), "{s:?}: {d}");
        }
    }
}
