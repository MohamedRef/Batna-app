//! The app's stylesheets, with the update feed's newer copies on top.
//!
//! Every `.css` file is built in with [`css!`]. A content pack from the feed
//! may carry newer copies (`Pack::look`), which only the build they were made
//! with takes (`update::installed_pack`), so a change to the look reaches
//! devices without a new app build. Screens write a stylesheet as
//! `style { {CSS.text()} }`, which picks the newest copy at each render.

use std::collections::BTreeMap;
use std::sync::RwLock;

/// One stylesheet: its path under `crates/app` and the built-in text.
#[derive(Clone, Copy)]
pub struct Css {
    pub path: &'static str,
    pub text: &'static str,
}

/// `css!("src/lecture/lecture.css")`: a stylesheet under `crates/app`.
macro_rules! css {
    ($path:literal) => {
        $crate::look::Css { path: $path, text: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)) }
    };
}
pub(crate) use css;

/// The feed's stylesheets in use (leaked: replaced a few times a session).
static FROM_FEED: RwLock<Option<&'static BTreeMap<String, String>>> = RwLock::new(None);

/// Uses `look` (from a pack) over the built-in stylesheets; empty goes back
/// to the built-in ones.
pub fn set(look: BTreeMap<String, String>) {
    let look = (!look.is_empty()).then(|| &*Box::leak(Box::new(look)));
    *FROM_FEED.write().unwrap_or_else(|e| e.into_inner()) = look;
}

impl Css {
    /// The text in use: the feed's copy when it has one.
    pub fn text(self) -> &'static str {
        let feed = *FROM_FEED.read().unwrap_or_else(|e| e.into_inner());
        feed.and_then(|m| m.get(self.path)).map_or(self.text, String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_feed_copy_wins_and_goes_away() {
        const MAIN: Css = css!("assets/main.css");
        assert!(MAIN.text().contains("--"), "the built-in stylesheet is there");
        set(BTreeMap::from([("assets/main.css".to_owned(), ".x { color: red; }".to_owned())]));
        assert_eq!(MAIN.text(), ".x { color: red; }");
        set(BTreeMap::new());
        assert_eq!(MAIN.text(), MAIN.text, "back to the built-in one");
    }
}
