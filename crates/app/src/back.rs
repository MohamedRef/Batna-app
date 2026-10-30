//! System Back (the Android gesture or button) moves back inside Batna.
//!
//! wry's Android activity only handles Back while the WebView can go back in
//! its history, and otherwise sends the app to the background. So while
//! there is somewhere to go back to, one "trap" history entry is kept; Back
//! pops it, and Batna steps back in this order:
//! 1. the topmost on-screen control marked `data-back="true"` (the Settings
//!    sheet's back or close, a player's ×, "Stay" in a leave-session
//!    confirm), clicked as if tapped;
//! 2. the previous screen, then Today.
//!
//! On Today with nothing open no entry is kept (one left over from a sheet
//! closed by tap is popped), so Back leaves Batna as Android users expect.
// adapted from kabar-app main: crates/app/src/back.rs

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::{Screen, Ui};

const BACK_JS: &str = include_str!("../assets/system-back.js");

/// How many screens Back remembers.
const DEPTH: usize = 30;

/// Screens Back never returns to: Settings is a sheet, not a screen.
fn remembered(screen: Screen) -> bool {
    screen != Screen::Settings
}

/// The screens visited, newest last.
#[derive(Default)]
struct Trail {
    stack: Vec<Screen>,
    last: Option<Screen>,
    /// The next screen change comes from Back: don't remember it.
    popping: bool,
}

impl Trail {
    /// Records a move to `now`; returns whether Back has somewhere to go.
    fn visit(&mut self, now: Screen) -> bool {
        if let Some(last) = self.last
            && last != now
            && !std::mem::take(&mut self.popping)
            && remembered(last)
            && self.stack.last() != Some(&last)
        {
            self.stack.push(last);
            if self.stack.len() > DEPTH {
                self.stack.remove(0);
            }
        }
        self.last = Some(now);
        self.stack.retain(|s| *s != now);
        !self.stack.is_empty() || now != Screen::Today
    }

    /// Where Back goes from `now`, if anywhere.
    fn back(&mut self, now: Screen) -> Option<Screen> {
        let to = self.stack.pop().or((now != Screen::Today).then_some(Screen::Today))?;
        self.popping = true;
        Some(to)
    }
}

/// One navigation trail for top-level screens. Every visible control with
/// data-back="true" participates automatically, without feature-specific flags.
pub fn use_back(ui: Ui, current: Screen) {
    let trail = use_hook(|| Rc::new(RefCell::new(Trail::default())));
    let can_back = trail.borrow_mut().visit(current);
    document::eval(if can_back {
        "window.__batnaSetScreenBack && window.__batnaSetScreenBack(true);"
    } else {
        "window.__batnaSetScreenBack && window.__batnaSetScreenBack(false);"
    });

    use_future({
        let trail = trail.clone();
        move || {
            let trail = trail.clone();
            async move {
                let mut eval = document::eval(BACK_JS);
                while let Ok(message) = eval.recv::<String>().await {
                    if message == "back" {
                        let now = *ui.screen.peek();
                        if let Some(to) = trail.borrow_mut().back(now) {
                            let mut screen = ui.screen;
                            screen.set(to);
                        }
                    }
                    if message == "ready" || message == "back" {
                        let can_back = trail.borrow_mut().visit(*ui.screen.peek());
                        document::eval(if can_back {
                            "window.__batnaSetScreenBack && window.__batnaSetScreenBack(true);"
                        } else {
                            "window.__batnaSetScreenBack && window.__batnaSetScreenBack(false);"
                        });
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_walks_the_visited_screens_then_today() {
        let mut t = Trail::default();
        assert!(!t.visit(Screen::Today), "nothing to go back to on Today");
        assert!(t.visit(Screen::Study));
        assert!(t.visit(Screen::Bank));
        assert_eq!(t.back(Screen::Bank), Some(Screen::Study));
        assert!(t.visit(Screen::Study));
        assert_eq!(t.back(Screen::Study), Some(Screen::Today));
        assert!(!t.visit(Screen::Today));
        assert_eq!(t.back(Screen::Today), None, "Back on Today leaves the app");
    }

    #[test]
    fn settings_is_not_returned_to() {
        let mut t = Trail::default();
        t.visit(Screen::Bank);
        t.visit(Screen::Settings);
        t.visit(Screen::Bank);
        assert_eq!(t.back(Screen::Bank), Some(Screen::Today));
    }

    #[test]
    fn a_screen_is_remembered_once() {
        let mut t = Trail::default();
        for s in [Screen::Today, Screen::Stats, Screen::Today, Screen::Stats] {
            t.visit(s);
        }
        assert_eq!(t.back(Screen::Stats), Some(Screen::Today));
        t.visit(Screen::Today);
        assert_eq!(t.back(Screen::Today), None);
    }
}
