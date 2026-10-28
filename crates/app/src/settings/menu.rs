//! The Mac menu bar: Batna › Settings… (⌘,), the standard Edit and Window
//! menus (so ⌘C, ⌘V and ⌘W work as everywhere else), Go with ⌘1–⌘6, and
//! Help › Keyboard Shortcuts (also the "?" key).
//! Built on every desktop so it is type-checked here; only macOS installs it.
// adapted from kabar-app main: crates/app/src/settings/menu.rs

use dioxus::desktop::muda::accelerator::{Accelerator, Code, Modifiers};
use dioxus::desktop::muda::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use dioxus::prelude::*;

use super::SettingsNav;
use crate::{Screen, Ui};

const SETTINGS: &str = "batna.settings";
const SHORTCUTS: &str = "batna.shortcuts";

/// Go menu: (id, title, key, screen). Titles are English like the rest of
/// the macOS menu bar.
const GO: [(&str, &str, Code, Screen); 6] = [
    ("batna.go.today", "Today", Code::Digit1, Screen::Today),
    ("batna.go.study", "Study", Code::Digit2, Screen::Study),
    ("batna.go.bank", "Question Bank", Code::Digit3, Screen::Bank),
    ("batna.go.cases", "Cases", Code::Digit4, Screen::Cases),
    ("batna.go.oral", "Oral", Code::Digit5, Screen::Oral),
    ("batna.go.stats", "Progress", Code::Digit6, Screen::Stats),
];

fn cmd(key: Code) -> Option<Accelerator> {
    Some(Accelerator::new(Some(Modifiers::SUPER), key))
}

/// The menu bar the window starts with: Batna's own on macOS, none elsewhere
/// (Linux and Windows keep a clean window; the keys still work there).
pub fn initial() -> Option<Menu> {
    if cfg!(target_os = "macos") { Some(menu_bar()) } else { None }
}

pub fn menu_bar() -> Menu {
    let about = AboutMetadata {
        name: Some("Batna".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        comments: Some("Internal medicine study, one short session at a time.".into()),
        ..Default::default()
    };
    // On macOS the first submenu is the app menu, titled with the app's name.
    let app = Submenu::with_items(
        "Batna",
        true,
        &[
            &PredefinedMenuItem::about(Some("About Batna"), Some(about)),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(SETTINGS, "Settings…", true, cmd(Code::Comma)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(Some("Hide Batna")),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("Quit Batna")),
        ],
    )
    .expect("app menu");
    let edit = Submenu::with_items(
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ],
    )
    .expect("edit menu");
    let go = Submenu::new("Go", true);
    for (id, title, key, _) in GO {
        go.append(&MenuItem::with_id(id, title, true, cmd(key))).expect("go item");
    }
    let window = Submenu::with_items(
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(Some("Zoom")),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(None),
        ],
    )
    .expect("window menu");
    let help =
        Submenu::with_items("Help", true, &[&MenuItem::with_id(SHORTCUTS, "Keyboard Shortcuts", true, None)])
            .expect("help menu");
    let menu = Menu::new();
    menu.append_items(&[&app, &edit, &go, &window, &help]).expect("menu bar");
    #[cfg(target_os = "macos")]
    {
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();
    }
    menu
}

/// Answers the menu's own items.
pub fn use_menu_events() {
    let ui = use_context::<Ui>();
    let settings = use_context::<SettingsNav>();
    dioxus::desktop::use_muda_event_handler(move |event| {
        let id = event.id().0.as_str();
        let mut screen = ui.screen;
        match id {
            SETTINGS => {
                crate::shortcuts::open_sheet(false);
                settings.show(None);
            }
            SHORTCUTS => crate::shortcuts::open_sheet(true),
            _ => {
                if let Some((_, _, _, s)) = GO.iter().find(|(go, ..)| *go == id) {
                    crate::shortcuts::open_sheet(false);
                    settings.close();
                    screen.set(*s);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_menu_covers_every_nav_screen_in_order() {
        let screens: Vec<Screen> = GO.iter().map(|g| g.3).collect();
        assert_eq!(screens, Screen::NAV.to_vec());
    }
}
