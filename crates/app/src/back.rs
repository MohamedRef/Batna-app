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

use std::cell::{Cell, RefCell};
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
        {
            if self.popping {
                // Back already consumed one entry; do not record the return trip.
                self.popping = false;
            } else if remembered(last) {
                // Preserve every visit, even when a screen was visited before.
                self.stack.push(last);
                if self.stack.len() > DEPTH {
                    self.stack.remove(0);
                }
            } else if self.stack.last() == Some(&now) {
                // A transient screen (player/Settings) was closed normally.
                // Its origin is already visible and must not become a ghost Back.
                self.stack.pop();
            }
        }
        self.last = Some(now);
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
    // What the script was last told: the Shell re-renders often, and only a
    // change needs a round trip to the WebView.
    let told = use_hook(|| Rc::new(Cell::new(None::<bool>)));
    let can_back = trail.borrow_mut().visit(current);
    if told.replace(Some(can_back)) != Some(can_back) {
        tell(can_back);
    }

    use_future({
        let trail = trail.clone();
        let told = told.clone();
        move || {
            let trail = trail.clone();
            let told = told.clone();
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
                        // Always answer: the script waits for this after a Back.
                        let can_back = trail.borrow_mut().visit(*ui.screen.peek());
                        told.set(Some(can_back));
                        tell(can_back);
                    }
                }
            }
        }
    });
}

/// Tells the script whether the screen trail has somewhere to go back to.
fn tell(can_back: bool) {
    document::eval(if can_back {
        "window.__batnaSetScreenBack && window.__batnaSetScreenBack(true);"
    } else {
        "window.__batnaSetScreenBack && window.__batnaSetScreenBack(false);"
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
    fn revisits_follow_the_exact_navigation_order() {
        let mut t = Trail::default();
        for screen in [
            Screen::Today,
            Screen::Study,
            Screen::Bank,
            Screen::Study,
            Screen::Bank,
        ] {
            t.visit(screen);
        }
        for expected in [
            Screen::Study,
            Screen::Bank,
            Screen::Study,
            Screen::Today,
        ] {
            let from = t.last.expect("visited screen");
            assert_eq!(t.back(from), Some(expected));
            t.visit(expected);
        }
        assert_eq!(t.back(Screen::Today), None);
    }

    #[test]
    fn closing_a_transient_screen_does_not_create_a_ghost_step() {
        let mut t = Trail::default();
        t.visit(Screen::Today);
        t.visit(Screen::Study);
        t.visit(Screen::Bank);
        t.visit(Screen::Settings);
        t.visit(Screen::Bank);
        assert_eq!(t.back(Screen::Bank), Some(Screen::Study));
        t.visit(Screen::Study);
        assert_eq!(t.back(Screen::Study), Some(Screen::Today));
        t.visit(Screen::Today);
        assert_eq!(t.back(Screen::Today), None);
    }
}
