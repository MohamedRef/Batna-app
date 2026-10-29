//! Preferences that belong to one device and never sync: the phone, the
//! tablet and the Mac each keep their own theme and text size. A small
//! `key=value` file in the app's data folder, written whole on every change.
// adapted from kabar-app main: crates/app/src/settings/device.rs

use std::collections::BTreeMap;
use std::path::PathBuf;

use batna_core::theme::Theme;

const FILE: &str = "device-prefs.txt";
const THEME: &str = "theme";
const TEXT_SIZE: &str = "text_size";
const NAV_STYLE: &str = "nav_style";
const PEN_HOVER: &str = "pen_hover";
const MOUSE_HOVER: &str = "mouse_hover";
const APPEARANCE: &str = "appearance";

/// The app's own data folder: the local database and device files live here.
pub fn data_dir() -> Option<PathBuf> {
    // Previews and tests point the whole app at a scratch folder.
    if let Some(dir) = std::env::var_os("BATNA_DATA_DIR") {
        return Some(PathBuf::from(dir));
    }
    if cfg!(target_os = "android") {
        // The app's private files directory (Context.getFilesDir for the
        // device owner). A JNI lookup can replace this once platform/ has one.
        return Some(PathBuf::from("/data/data/app.batna/files"));
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        return home.map(|h| h.join("Library/Application Support/app.batna"));
    }
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".local/share")))
        .map(|d| d.join("batna"))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextSize {
    Smaller,
    Small,
    #[default]
    Normal,
    Large,
    Larger,
}

impl TextSize {
    pub const ALL: [Self; 5] = [Self::Smaller, Self::Small, Self::Normal, Self::Large, Self::Larger];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smaller => "smaller",
            Self::Small => "small",
            Self::Normal => "normal",
            Self::Large => "large",
            Self::Larger => "larger",
        }
    }

    fn parse(s: &str) -> Self {
        Self::ALL.into_iter().find(|t| t.as_str() == s).unwrap_or_default()
    }

    /// The root size: type, spacing and control heights are all in rem, so
    /// this scales the whole interface, not only the text.
    pub fn percent(self) -> u32 {
        match self {
            Self::Smaller => 82,
            Self::Small => 90,
            Self::Normal => 100,
            Self::Large => 112,
            Self::Larger => 125,
        }
    }
}

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
    pub const ALL: [Self; 2] = [Self::Labels, Self::Slim];

    /// Stored value and `data-nav-style` on `<html>` (nav.css).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Labels => "labels",
            Self::Slim => "slim",
        }
    }

    fn parse(s: &str) -> Self {
        Self::ALL.into_iter().find(|t| t.as_str() == s).unwrap_or_default()
    }
}

/// The surface style (Settings › Look › "نمط الواجهة"), independent of the
/// colour theme: Calm is solid and quiet; Glossy adds tonal gradients, a
/// soft top highlight, pill buttons and blur on small floating surfaces.
/// `data-appearance` on `.app` (assets/appearance.css).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    #[default]
    Calm,
    Glossy,
}

impl Appearance {
    pub const ALL: [Self; 2] = [Self::Calm, Self::Glossy];

    /// Stored value and `data-appearance`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Calm => "calm",
            Self::Glossy => "glossy",
        }
    }

    fn parse(s: &str) -> Self {
        Self::ALL.into_iter().find(|t| t.as_str() == s).unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevicePrefs {
    pub theme: Theme,
    pub text_size: TextSize,
    pub nav_style: NavStyle,
    /// Peeks while the S Pen hovers (Settings › Tablet).
    pub pen_hover: bool,
    /// Peeks when the mouse rests on a word (Settings › Mac).
    pub mouse_hover: bool,
    /// Calm or Glossy. A file from before the redesign has no line for it
    /// and loads as Calm.
    pub appearance: Appearance,
}

impl Default for DevicePrefs {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            text_size: TextSize::default(),
            nav_style: NavStyle::default(),
            pen_hover: true,
            mouse_hover: true,
            appearance: Appearance::default(),
        }
    }
}

impl DevicePrefs {
    pub fn load() -> Self {
        let text = data_dir().and_then(|d| std::fs::read_to_string(d.join(FILE)).ok()).unwrap_or_default();
        Self::parse(&text)
    }

    /// Best effort: a folder that can't be written keeps the choice for this
    /// run, and says why in the log.
    pub fn save(&self) {
        let Some(dir) = data_dir() else { return };
        let tmp = dir.join(format!("{FILE}.tmp"));
        let saved = std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::write(&tmp, self.render()))
            .and_then(|()| std::fs::rename(&tmp, dir.join(FILE)));
        if let Err(e) = saved {
            crate::diag::report!("saving device prefs in {}: {e}", dir.display());
        }
    }

    fn parse(text: &str) -> Self {
        let map: BTreeMap<&str, &str> =
            text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim(), v.trim())).collect();
        Self {
            theme: map.get(THEME).map_or_else(Theme::default, |v| Theme::parse(v)),
            text_size: map.get(TEXT_SIZE).map_or_else(TextSize::default, |v| TextSize::parse(v)),
            nav_style: map.get(NAV_STYLE).map_or_else(NavStyle::default, |v| NavStyle::parse(v)),
            pen_hover: map.get(PEN_HOVER) != Some(&"off"),
            mouse_hover: map.get(MOUSE_HOVER) != Some(&"off"),
            appearance: map.get(APPEARANCE).map_or_else(Appearance::default, |v| Appearance::parse(v)),
        }
    }

    fn render(&self) -> String {
        let on_off = |on: bool| if on { "on" } else { "off" };
        format!(
            "{THEME}={}\n{TEXT_SIZE}={}\n{NAV_STYLE}={}\n{PEN_HOVER}={}\n{MOUSE_HOVER}={}\n{APPEARANCE}={}\n",
            self.theme.as_str(),
            self.text_size.as_str(),
            self.nav_style.as_str(),
            on_off(self.pen_hover),
            on_off(self.mouse_hover),
            self.appearance.as_str(),
        )
    }

    /// The page-wide CSS for these choices: colour tokens and root text size.
    pub fn css(&self) -> String {
        self.css_for(None)
    }

    /// [`DevicePrefs::css`] with the device's dark mode when the platform
    /// knows it (Android), so System themes follow it.
    pub fn css_for(&self, device_dark: Option<bool>) -> String {
        format!("{}html{{font-size:{}%;}}", self.theme.css_for(device_dark), self.text_size.percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_ignores_junk() {
        let p = DevicePrefs {
            theme: Theme::NightShift,
            text_size: TextSize::Larger,
            nav_style: NavStyle::Slim,
            pen_hover: false,
            mouse_hover: true,
            appearance: Appearance::Glossy,
        };
        assert_eq!(DevicePrefs::parse(&p.render()), p);
        assert_eq!(DevicePrefs::parse("theme=neon\nnonsense\ntext_size=huge"), DevicePrefs::default());
    }

    #[test]
    fn css_carries_theme_and_size() {
        let p = DevicePrefs { theme: Theme::Paper, text_size: TextSize::Large, ..DevicePrefs::default() };
        let css = p.css();
        assert!(css.contains("--bg:#f3f6f9") && css.ends_with("html{font-size:112%;}"), "{css}");
    }

    #[test]
    fn a_file_from_before_the_redesign_loads_as_calm() {
        // The exact file an older build wrote: no appearance line.
        let old = "theme=night\ntext_size=small\nnav_style=labels\npen_hover=on\nmouse_hover=off\n";
        let p = DevicePrefs::parse(old);
        assert_eq!(p.appearance, Appearance::Calm);
        assert_eq!((p.theme, p.text_size, p.mouse_hover), (Theme::Night, TextSize::Small, false));
        assert_eq!(DevicePrefs::parse("appearance=sparkly").appearance, Appearance::Calm);
        let glossy = DevicePrefs { appearance: Appearance::Glossy, ..p };
        assert_eq!(DevicePrefs::parse(&glossy.render()), glossy);
    }

    #[test]
    fn smaller_interface_size_round_trips() {
        let p = DevicePrefs { text_size: TextSize::Smaller, ..DevicePrefs::default() };
        assert_eq!(DevicePrefs::parse(&p.render()), p);
        assert!(p.css().ends_with("html{font-size:82%;}"));
    }
}
