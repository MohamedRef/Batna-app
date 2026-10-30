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

const BACK_JS: &str = r#"
window.__batnaBackSend = (m) => dioxus.send(m);
if (!window.__batnaBackInit) {
  window.__batnaBackInit = true;
  // Trust the real WebView history entry rather than a stale local flag.
  // Each overlay and screen shares one synthetic entry, never an entry per layer.
  let untrapping = false;
  window.__batnaTrap = () => {
    if (history.state?.batna !== true) history.pushState({ batna: true }, "");
  };
  window.__batnaUntrap = () => {
    if (history.state?.batna === true) {
      untrapping = true;
      history.back();
    }
  };
  window.addEventListener("popstate", () => {
    if (untrapping) {
      untrapping = false;
      return;
    }
    // The last *visible* Back control is topmost. Hidden Settings controls
    // must not swallow Back while a full-screen update sheet is open.
    const targets = [...document.querySelectorAll('[data-back="true"]')]
      .filter((e) => !e.disabled && e.getClientRects().length > 0 && !e.closest('[hidden], [inert]'));
    const top = targets[targets.length - 1];
    if (top) {
      top.click();
      window.__batnaTrap();
    } else {
      window.__batnaBackSend(true);
    }
  });
}
await new Promise(() => {});
"#;

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

/// Wires Back for the Shell. `overlay_open`: a sheet that Back should close
/// is showing (it carries its own `data-back` control).
pub fn use_back(ui: Ui, current: Screen, back_layers: u8) {
    let trail = use_hook(|| Rc::new(RefCell::new(Trail::default())));
    let armed = use_hook(|| Rc::new(Cell::new(None::<(Screen, u8)>)));
    let wants_trap = use_hook(|| Rc::new(Cell::new(false)));

    // Re-arm the trap whenever the screen or an overlay changes, or drop it
    // when there is nowhere left to go back to.
    let can_back = trail.borrow_mut().visit(current) || back_layers != 0;
    wants_trap.set(can_back);
    if armed.replace(Some((current, back_layers))) != Some((current, back_layers)) {
        document::eval(if can_back {
            "window.__batnaTrap && window.__batnaTrap();"
        } else {
            "window.__batnaUntrap && window.__batnaUntrap();"
        });
    }

    use_future({
        let trail = trail.clone();
        move || {
            let trail = trail.clone();
            let wants_trap = wants_trap.clone();
            async move {
                let mut eval = document::eval(BACK_JS);
                // The first screen may already have somewhere to go (the
                // evals above ran before the script was there).
                if wants_trap.get() {
                    document::eval("window.__batnaTrap && window.__batnaTrap();");
                }
                while eval.recv::<bool>().await.is_ok() {
                    let now = *ui.screen.peek();
                    let to = trail.borrow_mut().back(now);
                    if let Some(to) = to {
                        let mut screen = ui.screen;
                        screen.set(to);
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
