//! Device switches that never sync: the phone, the tablet and the Mac each
//! keep their own sidebar style and hover peeks.
// adapted from kabar-app main: crates/app/src/settings/device.rs (NavStyle,
// pen/mouse hover). Kabar keeps them in its local `meta` table; Batna has no
// store yet, so they live in the webview's localStorage for now, and the
// store can take them over without changing these signals.

use dioxus::prelude::*;

/// The sidebar on wide windows (tablet landscape, the Mac).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavStyle {
    /// Icons with labels.
    #[default]
    Labels,
    /// The narrow icon rail, as on medium widths.
    Slim,
}

impl NavStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Labels => "labels",
            Self::Slim => "slim",
        }
    }

    fn parse(s: &str) -> Self {
        if s == "slim" { Self::Slim } else { Self::Labels }
    }
}

pub static NAV_STYLE: GlobalSignal<NavStyle> = Signal::global(NavStyle::default);
/// Peeks while the S Pen hovers.
pub static PEN_HOVER: GlobalSignal<bool> = Signal::global(|| true);
/// Peeks when the mouse rests on a word.
pub static MOUSE_HOVER: GlobalSignal<bool> = Signal::global(|| true);

const KEYS: [&str; 3] = ["batna.pref.nav_style", "batna.pref.pen_hover", "batna.pref.mouse_hover"];

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// Loads the stored switches once, then mirrors every change to storage and
/// to `data-*` attributes on `<html>` that nav.css and hover.js read.
pub fn use_prefs() {
    let mut loaded = use_signal(|| false);
    use_future(move || async move {
        let js = format!(
            r#"try {{ return [{}].map((k) => localStorage.getItem(k) || "").join("|"); }} catch (e) {{ return ""; }}"#,
            KEYS.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>().join(",")
        );
        if let Ok(stored) = document::eval(&js).join::<String>().await {
            let mut parts = stored.split('|');
            let (nav, pen, mouse) = (parts.next(), parts.next(), parts.next());
            if let Some(n) = nav.filter(|s| !s.is_empty()) {
                *NAV_STYLE.write() = NavStyle::parse(n);
            }
            if let Some(p) = pen.filter(|s| !s.is_empty()) {
                *PEN_HOVER.write() = p != "off";
            }
            if let Some(m) = mouse.filter(|s| !s.is_empty()) {
                *MOUSE_HOVER.write() = m != "off";
            }
        }
        loaded.set(true);
    });
    use_effect(move || {
        let nav = NAV_STYLE.read().as_str();
        let pen = on_off(*PEN_HOVER.read());
        let mouse = on_off(*MOUSE_HOVER.read());
        // Before the first load, only paint: never overwrite storage with defaults.
        let save = if *loaded.read() {
            format!(
                r#"try {{ localStorage.setItem({:?}, {nav:?}); localStorage.setItem({:?}, {pen:?}); localStorage.setItem({:?}, {mouse:?}); }} catch (e) {{}}"#,
                KEYS[0], KEYS[1], KEYS[2]
            )
        } else {
            String::new()
        };
        document::eval(&format!(
            r#"const d = document.documentElement.dataset; d.navStyle = {nav:?}; d.penHover = {pen:?}; d.mouseHover = {mouse:?}; {save}"#
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_style_round_trips_and_defaults_to_labels() {
        for s in [NavStyle::Labels, NavStyle::Slim] {
            assert_eq!(NavStyle::parse(s.as_str()), s);
        }
        assert_eq!(NavStyle::parse(""), NavStyle::Labels);
        assert_eq!(NavStyle::parse("wide"), NavStyle::Labels);
    }
}
