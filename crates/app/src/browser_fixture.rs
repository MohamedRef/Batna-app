//! Shared parts of the screen browser fixtures (`learn/browser.rs`,
//! `study/browser.rs`): production screens rendered by the test-only SSR
//! renderer inside the app's shell. The production stylesheets are written
//! once (`app.css`) and the real theme and text-size CSS once per preference
//! (`prefs-<theme>-<size>.css`), so the browser job switches them the way the
//! app does instead of keeping a file per combination. Rust callbacks are not
//! hydrated: these fixtures check layout, not taps or saving.

use crate::prefs::{DevicePrefs, TextSize};
use crate::{Screen, Ui};
use batna_core::theme::Theme;
use dioxus::prelude::*;
use std::path::PathBuf;

/// `$APP_BROWSER_FIXTURES/<screen>`, created, with the stylesheets in it.
pub fn out_dir(screen: &str) -> PathBuf {
    let root = std::env::var_os("APP_BROWSER_FIXTURES").expect("APP_BROWSER_FIXTURES");
    out_dir_in(std::path::Path::new(&root), screen)
}

/// [`out_dir`] under `root` (tests write the fixtures to a temporary folder).
pub fn out_dir_in(root: &std::path::Path, screen: &str) -> PathBuf {
    let dir = root.join(screen);
    std::fs::create_dir_all(&dir).unwrap();
    let app = [
        crate::fonts::css().to_string(),
        crate::MAIN_CSS.text().to_string(),
        crate::SHAPE_CSS.text().to_string(),
        crate::PARTS_CSS.text().to_string(),
        crate::APPEARANCE_CSS.text().to_string(),
        crate::sheet::CSS.text().to_string(),
        crate::LIQUID_GLASS_CSS.text().to_string(),
        crate::desk::NAV_CSS.text().to_string(),
        crate::desk::DESK_CSS.text().to_string(),
    ]
    .join("\n");
    std::fs::write(dir.join("app.css"), app).unwrap();
    // Browser regression loads the same drag logic as the Mac WebView.
    std::fs::write(dir.join("desk-resize.js"), include_str!("desk/resize.js")).unwrap();
    for theme in Theme::ALL {
        for size in [TextSize::Normal, TextSize::Larger] {
            let prefs = DevicePrefs { theme, text_size: size, ..DevicePrefs::default() };
            let file = format!("prefs-{}-{}.css", theme.as_str(), size.as_str());
            std::fs::write(dir.join(file), prefs.css()).unwrap();
        }
    }
    dir
}

/// The app's frame around one screen, as `App` draws it (main.rs).
/// Keep its navigation structure and production desktop styles: focus mode
/// only hides navigation on phones. Fixture navigation is deliberately inert:
/// click-through tests must target the screen, not a duplicate test nav.
#[component]
pub fn Shell(screen: Screen, focus: bool, children: Element) -> Element {
    use_context_provider(|| Ui {
        screen: Signal::new(screen),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(focus),
    });
    // The normal Rust click-through tests search screen text; duplicate nav
    // labels would intercept their taps. The browser export sets this path
    // explicitly, so only its screenshots need the desktop shell chrome.
    let browser_preview = std::env::var_os("APP_BROWSER_FIXTURES").is_some();
    rsx! {
        div { class: "app", lang: "ar", dir: "rtl", "data-appearance": "calm",
            "data-desk-focus": "false",
            "data-focus": if focus { "true" } else { "false" },
            if browser_preview {
                header { class: "topbar",
                div { class: "brand",
                    span { class: "brand-name", {crate::i18n::app::NAME} }
                    span { class: "brand-tag", {crate::i18n::app::TAGLINE} }
                }
                div { class: "topbar-actions",
                    button {
                        class: "icon-button topbar-search",
                        aria_label: crate::i18n::keys::SEARCH_OPEN,
                        onclick: move |_| crate::shortcuts::open_palette(true),
                        {crate::icons::search()}
                    }
                    button {
                        class: "icon-button",
                        aria_label: Screen::Settings.label(),
                        onclick: move |_| {},
                        {crate::icons::nav(Screen::Settings)}
                    }
                }
            }
            nav { class: "nav", aria_label: crate::i18n::nav::LABEL,
                div { class: "nav-brand", aria_hidden: "true",
                    span { class: "brand-name", {crate::i18n::app::NAME} }
                    span { class: "brand-tag", {crate::i18n::app::TAGLINE} }
                }
                for tab in Screen::NAV {
                    crate::NavButton {
                        key: "{tab:?}",
                        screen: tab,
                        class: crate::more::nav_class(tab),
                    }
                }
                button { class: "nav-item nav-foot",
                    aria_label: Screen::Settings.label(),
                    // Browser fixture only; Settings has its own production
                    // screen renderer and browser regression suite.
                    onclick: move |_| {},
                    span { class: "glyph", {crate::icons::nav(Screen::Settings)} }
                    span { class: "label", {Screen::Settings.label()} }
                }
            }
            }
            main { class: "content", "data-screen": "{screen:?}",
                div { class: "content-rev", {children} }
            }
            crate::abbrsheet::AbbrSheet {}
        }
    }
}

/// One rendered state as a page using the shared stylesheets.
pub fn write_page(dir: &std::path::Path, file: &str, dom: &VirtualDom) {
    let html = dioxus::ssr::render(dom);
    let page = format!(
        "<!doctype html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <link rel=\"stylesheet\" href=\"app.css\">\
         <link id=\"prefs\" rel=\"stylesheet\" href=\"prefs-system-normal.css\">\
         </head><body>{html}</body></html>"
    );
    std::fs::write(dir.join(file), page).unwrap();
}
