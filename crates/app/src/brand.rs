//! The Batna mark and its lockup with the name (rebrand B0, 2026-11-08).
//!
//! The mark is the liver drawn as one clean shape with its fissure, and a bile
//! drop. It uses the same paths as the launcher icon (assets/src/icon/*.svg),
//! so the icon on the home screen and the head of the sidebar match. Colours
//! come from the theme: the liver is `--accent`, the drop `--good` (main.css).

use dioxus::prelude::*;

use crate::i18n;

/// Liver outline on a 100×100 grid.
pub const LIVER: &str = "M12 42C12 27 28 21 49 22C66 23 80 25 87 31C91 35 89 41 83 44C71 51 59 59 47 65C36 71 22 70 16 60C13 55 12 49 12 42Z";
/// The fissure between the lobes.
pub const FISSURE: &str = "M50 24C53 36 53 48 49 62";
/// The bile drop under the liver.
pub const DROP: &str = "M38 64C38 64 29 76 29 82A9 9 0 0 0 47 82C47 76 38 64 38 64Z";

/// The mark alone, decorative (the name beside it is the accessible text).
#[component]
pub fn Mark() -> Element {
    rsx! {
        svg {
            class: "brand-mark",
            view_box: "8 18 84 76",
            "aria-hidden": "true",
            path { class: "liver", d: LIVER }
            path { class: "fissure", d: FISSURE }
            path { class: "drop", d: DROP }
        }
    }
}

/// The phone top bar: mark, name and tagline on one line.
#[component]
pub fn TopBrand() -> Element {
    rsx! {
        div { class: "brand",
            Mark {}
            span { class: "brand-name", {i18n::app::NAME} }
            span { class: "brand-tag", {i18n::app::TAGLINE} }
        }
    }
}

/// The head of the rail (mark over the name) and the sidebar (mark beside
/// the name and tagline), from 600px (nav.css).
#[component]
pub fn NavBrand() -> Element {
    rsx! {
        div { class: "nav-brand", aria_hidden: "true",
            Mark {}
            span { class: "brand-words",
                span { class: "brand-name", {i18n::app::NAME} }
                span { class: "brand-tag", {i18n::app::TAGLINE} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The in-app mark and the launcher icon must not drift apart.
    #[test]
    fn mark_matches_the_launcher_icon() {
        let icon = include_str!("../../../assets/src/icon/full.svg");
        for d in [LIVER, FISSURE, DROP] {
            assert!(icon.contains(d), "assets/src/icon/full.svg lost path {d}");
        }
    }
}
