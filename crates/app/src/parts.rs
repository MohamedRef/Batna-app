//! Shared pieces of the 2026-10 redesign that need code, not only a class
//! (the classes are in assets/parts.css, listed in docs/DESIGN.md).

use dioxus::prelude::*;

use crate::Ui;

/// "3 / 12" that reads left to right inside Arabic text; without the
/// isolation the bidi algorithm shows it as "12 / 3".
#[component]
pub fn Count(done: usize, total: usize) -> Element {
    rsx! {
        span { class: "num-ltr", "{done} / {total}" }
    }
}

/// Focused study: while the calling component is mounted (a running
/// session, a lesson, a question set), the phone's bottom bar hides so the
/// task gets the screen. The component must show its own way out (a close
/// or finish button with `"data-back": "true"`, so Android Back takes it).
/// From 600dp the rail stays: there is room for both.
pub fn use_focus_mode(on: bool) {
    let mut focus = use_context::<Ui>().focus;
    use_effect(use_reactive!(|on| {
        if *focus.peek() != on {
            focus.set(on);
        }
    }));
    use_drop(move || focus.set(false));
}
