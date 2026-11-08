//! Keyboard shortcuts for tablet keyboards and the Mac (PARITY.md §6.5):
//! Ctrl/⌘ + 1–7 for the screens, Ctrl/⌘ + , for Settings, and on study
//! screens Space to reveal or rate Good, 1–4 to rate, Z to undo, A–E for MCQ options, Enter to check or rate Good,
//! ← / → between questions of a timed paper, F to flag, Esc to close, ? for
//! the list.
//!
//! Screens don't register handlers: a key presses the visible, enabled button
//! whose `aria-keyshortcuts` names it (`"Space"`, `"1"`, `"A"`, `"Enter"`,
//! `"Escape"`, `"ArrowLeft"`, `"F"`). That is also how screen readers learn the key.
// adapted from kabar-app main: crates/app/src/settings/menu.rs ("?" key) and
// crates/app/src/review.rs (keys on visible buttons).

mod palette;
pub use palette::{MobileSearchButton, Palette};

use dioxus::prelude::*;

use crate::i18n::{keys as tx, nav};
use crate::settings::SettingsNav;
use crate::{Screen, Ui};

static SHEET_OPEN: GlobalSignal<bool> = Signal::global(|| false);
static PALETTE_OPEN: GlobalSignal<bool> = Signal::global(|| false);

pub fn open_palette(open: bool) {
    *PALETTE_OPEN.write() = open;
}

pub fn open_sheet(open: bool) {
    *SHEET_OPEN.write() = open;
}

const KEYS_JS: &str = r#"
if (window.__batnaKeys) document.removeEventListener("keydown", window.__batnaKeys, true);
window.__batnaKeys = (e) => {
  // Review keys are single actions, never auto-repeat through multiple cards.
  if (e.defaultPrevented || e.isComposing || e.repeat) return;
  const t = e.target;
  // Only unmodified separator arrows/Home/End belong to the split divider;
  // Cmd/Ctrl screen navigation and ? still work while the handle has focus.
  if (t?.closest?.(".desk-resize") && !e.metaKey && !e.ctrlKey && !e.altKey &&
      ["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) return;
  const typing = t && (t.closest?.('input, textarea, select, [contenteditable=""], [contenteditable="true"], [role="textbox"]') ||
    t.isContentEditable);
  const mod = e.metaKey || e.ctrlKey;
  if (mod && !e.altKey && !e.shiftKey) {
    if (/^Digit[1-7]$/.test(e.code)) { e.preventDefault(); dioxus.send("go:" + e.code.slice(5)); return; }
    if (e.code === "Comma") { e.preventDefault(); dioxus.send("go:settings"); return; }
    if (e.code === "KeyK") { e.preventDefault(); dioxus.send("palette"); return; }
    return;
  }
  if (e.altKey || mod || e.shiftKey && e.key !== "?" && e.key !== "\u061F") return;
  // With the command palette open, arrow keys move focus between actual
  // result buttons, and Enter on the search box opens its first match.
  const palette = document.querySelector('.command-palette[role="dialog"]');
  if (palette && !mod && !e.altKey && e.key === "Tab") {
    const focusable = [...palette.querySelectorAll('button:not(:disabled), input:not(:disabled)')];
    const first = focusable[0], last = focusable[focusable.length - 1];
    if (focusable.length && (e.shiftKey ? document.activeElement === first :
      document.activeElement === last)) {
      e.preventDefault();
      (e.shiftKey ? last : first).focus();
      return;
    }
  }
  if (palette && !mod && !e.altKey && !e.shiftKey) {
    const rows = [...palette.querySelectorAll('.palette-result:not(:disabled)')];
    const index = rows.indexOf(document.activeElement);
    if ((e.key === "ArrowDown" || e.key === "ArrowUp") && rows.length) {
      e.preventDefault();
      const step = e.key === "ArrowDown" ? 1 : -1;
      rows[index < 0 ? (step > 0 ? 0 : rows.length - 1) : (index + step + rows.length) % rows.length].focus();
      return;
    }
    if (e.key === "Enter" && t?.matches?.(".palette-search") && rows.length) {
      e.preventDefault();
      rows[0].click();
      return;
    }
  }
  if (e.key === "Escape" && window.__batnaHover?.visible()) {
    window.__batnaHover.hide(); e.preventDefault(); return;
  }
  // Typing a clinical answer (or editing a search/filter) must never trigger
  // grading, revealing, moving, the help sheet, or character shortcuts.
  // Escape may still close a modal via its own button.
  if (typing && e.key !== "Escape") return;
  if (e.key === "?" || e.key === "\u061F") {
    if (!typing && !palette) { e.preventDefault(); dioxus.send("help"); }
    return;
  }
  let key = null;
  if (e.code === "Space") key = "space";
  else if (e.key === "Enter") key = "enter";
  else if (e.key === "Escape") key = "escape";
  else if (/^(Digit|Numpad)[1-9]$/.test(e.code)) key = e.code.slice(-1);
  else if (e.key === "ArrowLeft" || e.key === "ArrowRight") key = e.key.toLowerCase();
  else if (/^Key[A-FZ]$/.test(e.code)) key = e.code.slice(3).toLowerCase();
  if (!key) return;
  // Modals take precedence. A shortcut must not click behind a dialog.
  const modal = document.querySelector('[role="alertdialog"][aria-modal="true"], [role="dialog"][aria-modal="true"]');
  const scope = modal || document.querySelector("main.content") || document;
  const visible = el => !el.disabled && !el.closest('[hidden], [inert], [aria-hidden="true"]') &&
    el.getClientRects().length > 0 && getComputedStyle(el).visibility !== "hidden";
  const find = name => [...scope.querySelectorAll("[aria-keyshortcuts]")].find(el =>
    visible(el) && el.getAttribute("aria-keyshortcuts").toLowerCase().split(/\s+/).includes(name));
  let hit = find(key);
  // Anki-style: once the answer is shown, Space/Enter chooses Good (3).
  // This is only for the active Study card, never lessons, dialogs or bank exams.
  const inStudy = !modal && document.querySelector('main.content[data-screen="Study"] .st-item');
  if (!hit && inStudy && (key === "space" || key === "enter")) hit = find("3");
  if (!hit) return;
  // Let focused buttons process their native Space/Enter once, not twice.
  if ((key === "enter" || key === "space") && t?.closest?.("button, a[href], [role=button]")) return;
  e.preventDefault();
  hit.click();
};
document.addEventListener("keydown", window.__batnaKeys, true);
await new Promise(() => {});
"#;

/// What a message from the key script asks for.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Go(Screen),
    Settings,
    Help,
    Palette,
}

fn parse(msg: &str) -> Option<Command> {
    match msg {
        "help" => Some(Command::Help),
        "palette" => Some(Command::Palette),
        "go:settings" => Some(Command::Settings),
        _ => {
            let n: usize = msg.strip_prefix("go:")?.parse().ok()?;
            Screen::NAV.get(n.checked_sub(1)?).copied().map(Command::Go)
        }
    }
}

pub fn use_shortcuts() {
    let ui = use_context::<Ui>();
    let settings = use_context::<SettingsNav>();
    use_future(move || async move {
        let mut eval = document::eval(KEYS_JS);
        while let Ok(msg) = eval.recv::<String>().await {
            if let Some(cmd) = parse(&msg) {
                apply(cmd, ui, settings);
            }
        }
    });
    // Keep focus on the new task after a keyboard/screen-reader action.
    // The script never modifies grades or navigation; pointer input is ignored.
    use_future(|| async move {
        let mut eval = document::eval(include_str!("focus.js"));
        while eval.recv::<String>().await.is_ok() {}
    });
}

/// Does what a key asked: a screen closes the sheets first.
fn apply(cmd: Command, ui: Ui, settings: SettingsNav) {
    match cmd {
        Command::Go(screen) => {
            open_sheet(false);
            open_palette(false);
            settings.close();
            let mut s = ui.screen;
            s.set(screen);
        }
        Command::Settings => {
            open_sheet(false);
            open_palette(false);
            settings.show(None);
        }
        Command::Help => {
            open_palette(false);
            let open = !*SHEET_OPEN.peek();
            open_sheet(open);
        }
        Command::Palette => {
            open_sheet(false);
            let open = !*PALETTE_OPEN.peek();
            open_palette(open);
        }
    }
}

/// The shortcut list: `?`, Help › Keyboard Shortcuts on the Mac, or the
/// button in Settings › Tablet and Mac.
#[component]
pub fn KeysSheet() -> Element {
    if !*SHEET_OPEN.read() {
        return rsx! {};
    }
    let screens = format!("{} … {}", nav::TODAY, nav::STATS);
    let rows: [(&[&str], String); 14] = [
        (&["Ctrl", "1–7"], format!("{} ({screens})", tx::SCREENS)),
        (&["Ctrl", ","], tx::SETTINGS.into()),
        (&["Ctrl/⌘", "K"], tx::PALETTE.into()),
        (&["Space", "Enter"], tx::REVEAL.into()),
        (&["Space", "Enter"], tx::GOOD.into()),
        (&["1", "2", "3", "4"], tx::RATE.into()),
        (&["Z"], tx::UNDO.into()),
        (&["A", "B", "C", "D", "E"], tx::OPTIONS.into()),
        (&["Enter"], tx::NEXT.into()),
        (&["←", "→"], tx::MOVE.into()),
        (&["F"], tx::FLAG.into()),
        (&["Esc"], tx::ESC.into()),
        (&["?"], tx::HELP.into()),
        (&["⌘"], tx::NOTE.into()),
    ];
    rsx! {
        // The shared bounded sheet (parts.css): tap outside, ✕, Esc or
        // Android Back closes it.
        div { class: "sheet-backdrop keys-backdrop", onclick: move |_| open_sheet(false) }
        div {
            class: "sheet keys-sheet",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "keys-title",
            div { class: "sheet-head",
                h2 { id: "keys-title", {tx::TITLE} }
                button {
                    class: "icon-button",
                    "aria-keyshortcuts": "Escape",
                    "aria-label": tx::CLOSE,
                    "data-back": "true",
                    onclick: move |_| open_sheet(false),
                    "✕"
                }
            }
            table { class: "keys-table",
                tbody {
                    for (keys, what) in rows {
                        tr { key: "{what}",
                            td { {what} }
                            td { class: "keys-cell",
                                span { dir: "ltr",
                                    for k in keys.iter() {
                                        kbd { key: "{k}", "{k}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewer_keys_remain_safe_and_discoverable() {
        // The dispatcher is an external JS script in a WKWebView. Check
        // contract invariants alongside the Rust DOM regression tests.
        for s in [
            "e.isComposing",
            "e.repeat",
            "t.isContentEditable",
            "[role=\"textbox\"]",
            "if (typing && e.key !== \"Escape\") return",
            "const modal =",
            "if (!hit && inStudy",
            "find(\"3\")",
            "Key[A-FZ]",
            "e.defaultPrevented",
        ] {
            assert!(KEYS_JS.contains(s), "missing keyboard safety contract: {s}");
        }
    }

    #[test]
    fn messages_map_to_screens() {
        assert_eq!(parse("go:1"), Some(Command::Go(Screen::Today)));
        assert_eq!(parse("go:7"), Some(Command::Go(Screen::Stats)));
        assert_eq!(parse("go:settings"), Some(Command::Settings));
        assert_eq!(parse("help"), Some(Command::Help));
        assert_eq!(parse("palette"), Some(Command::Palette));
        assert_eq!(parse("go:0"), None);
        assert_eq!(parse("go:8"), None);
        assert_eq!(parse("other"), None);
    }

    #[test]
    fn keys_move_between_screens_settings_and_the_list() {
        crate::store::use_memory_store();
        fn app() -> Element {
            let settings = use_context_provider(SettingsNav::new);
            let ui = use_context_provider(|| Ui {
                screen: Signal::new(Screen::Today),
                prefs: Signal::new(Default::default()),
                focus: Signal::new(false),
            });
            use_hook(|| {
                apply(Command::Help, ui, settings);
                assert!(*SHEET_OPEN.peek(), "? opens the list");
                apply(Command::Settings, ui, settings);
                assert!(!*SHEET_OPEN.peek() && *settings.open.peek(), "Ctrl+, opens Settings over it");
                apply(Command::Help, ui, settings);
                apply(Command::Go(Screen::Stats), ui, settings);
                assert_eq!(*ui.screen.peek(), Screen::Stats);
                assert!(!*SHEET_OPEN.peek() && !*settings.open.peek(), "a screen closes both");
                apply(Command::Help, ui, settings);
                apply(Command::Help, ui, settings);
                assert!(!*SHEET_OPEN.peek(), "? again closes the list");
            });
            rsx! { "ok" }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        assert_eq!(dioxus::ssr::render(&dom), "ok");
    }

    #[test]
    fn the_key_listener_mounts_and_the_list_closes_on_tap() {
        crate::store::use_memory_store();
        fn app() -> Element {
            use_context_provider(SettingsNav::new);
            use_context_provider(|| Ui {
                screen: Signal::new(Screen::Today),
                prefs: Signal::new(Default::default()),
                focus: Signal::new(false),
            });
            use_shortcuts();
            use_hook(|| open_sheet(true));
            rsx! { KeysSheet {} }
        }
        let mut t = crate::tap_dom::TestDom::of(app);
        t.click(&format!("aria-label=\"{}\"", tx::CLOSE));
        assert_eq!(t.html(), "", "✕ closes the list");
        t.with(|| open_sheet(true));
        t.settle();
        t.click("class=\"sheet-backdrop keys-backdrop\"");
        assert_eq!(t.html(), "", "so does a tap outside");
    }

    #[test]
    fn the_list_names_every_key_and_closes_with_escape() {
        fn app() -> Element {
            use_hook(|| open_sheet(true));
            rsx! { KeysSheet {} }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        assert!(html.contains(tx::TITLE) && html.contains("aria-modal=\"true\""), "{html}");
        assert_eq!(html.matches("<tr").count(), 14, "{html}");
        for text in [tx::SCREENS, tx::REVEAL, tx::RATE, tx::FLAG, tx::HELP, nav::TODAY, nav::STATS] {
            assert!(html.contains(text), "missing {text}: {html}");
        }
        assert!(html.contains("<kbd>Space</kbd>") && html.contains("aria-keyshortcuts=\"Escape\""), "{html}");

        fn closed() -> Element {
            rsx! { KeysSheet {} }
        }
        let mut dom = VirtualDom::new(closed);
        dom.rebuild_in_place();
        assert_eq!(dioxus::ssr::render(&dom), "", "closed by default");
    }
}
