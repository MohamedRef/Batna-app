//! «Export to Anki»: builds `batna.apkg` from the content inside the app (the
//! same deck `batna-tools anki` writes, via `batna-anki`) and hands it to
//! Anki, so Mohamed never moves files by hand.
//!
//! - Android: saved to `Download/Batna/batna.apkg` through MediaStore (no
//!   permission and no FileProvider needed), then opened with an
//!   `ACTION_VIEW` intent, which AnkiDroid answers with its import screen.
//! - Mac and other desktops: saved to `~/Downloads/batna.apkg` and opened with
//!   the default app (Anki registers `.apkg`).
//!
//! The app holds merged lectures only; the shared-folder file can also carry
//! drafts. Both use the same note ids, so either import updates the other.

use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::anki as tx;

#[cfg(target_os = "android")]
mod android;

const FILE_NAME: &str = "batna.apkg";

const CSS: &str = "
.anki-export .actions { display: flex; flex-wrap: wrap; gap: var(--sp-3); align-items: center; margin-block-start: var(--sp-3); }
.anki-export .primary {
  border: 0; border-radius: var(--radius); padding-block: var(--sp-3); padding-inline: var(--sp-4);
  background: var(--accent); color: var(--bg, #fff); font-weight: 700; cursor: pointer;
}
.anki-export .primary[disabled] { opacity: .6; cursor: progress; }
.anki-export .result { margin-block-start: var(--sp-3); }
";

#[derive(Clone, PartialEq)]
enum State {
    Idle,
    Working,
    Done { notes: usize, place: String },
    Failed(String),
}

#[component]
pub fn AnkiExport() -> Element {
    let mut state = use_signal(|| State::Idle);
    let lib = Library::get();
    let lectures = lib.lectures.len();
    let items: usize = lib.lectures.values().map(|l| l.items.len()).sum();
    let busy = *state.read() == State::Working;

    let export = move |_| {
        state.set(State::Working);
        // Let the button show "working" before the (sub-second) build runs.
        spawn(async move {
            futures_yield().await;
            state.set(match export_now() {
                Ok((notes, place)) => State::Done { notes, place },
                Err(e) => State::Failed(e),
            });
        });
    };

    rsx! {
        style { {CSS} }
        section { class: "card anki-export",
            h1 { {tx::TITLE} }
            p { {tx::BODY} }
            p { class: "muted small", "{lectures} {tx::LECTURES} · {items} {tx::ITEMS}" }
            div { class: "actions",
                button { class: "primary", disabled: busy, onclick: export,
                    if busy { {tx::WORKING} } else { {tx::BUTTON} }
                }
            }
            match &*state.read() {
                State::Done { notes, place } => rsx! {
                    p { class: "note result", "{tx::DONE} ({notes} {tx::NOTES}). {tx::SAVED_IN} " bdi { dir: "ltr", "{place}" } }
                    p { class: "muted small", {tx::REIMPORT} }
                },
                State::Failed(e) => rsx! {
                    p { class: "note result", {tx::FAILED} " " bdi { dir: "ltr", "{e}" } }
                },
                _ => rsx! {},
            }
        }
    }
}

/// Yields once to the UI so the "working" label paints.
async fn futures_yield() {
    let mut yielded = false;
    std::future::poll_fn(|cx| {
        if yielded {
            std::task::Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    })
    .await
}

/// Builds the deck from the embedded content; returns the note count and
/// where the file went.
fn export_now() -> Result<(usize, String), String> {
    let lectures: Vec<_> = Library::get().lectures.values().cloned().collect();
    // TODO(chunk 3): pass the in-app ✓/✎/✗ verdicts once the review store lands.
    let deck = batna_anki::build(&lectures, &batna_anki::Options::default());
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let bytes = batna_anki::to_apkg(&deck, now_ms)?;
    let place = save_and_open(&bytes)?;
    Ok((deck.notes.len(), place))
}

#[cfg(target_os = "android")]
fn save_and_open(bytes: &[u8]) -> Result<String, String> {
    android::save_and_open(FILE_NAME, bytes)
}

#[cfg(not(target_os = "android"))]
fn save_and_open(bytes: &[u8]) -> Result<String, String> {
    let home =
        std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).ok_or("no home folder")?;
    let downloads = std::path::Path::new(&home).join("Downloads");
    let dir = if downloads.is_dir() { downloads } else { std::path::PathBuf::from(home) };
    let path = dir.join(FILE_NAME);
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    // Opening is a convenience: the file is saved either way.
    let _ = std::process::Command::new(opener).arg(&path).spawn();
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_embedded_content_exports() {
        let lectures: Vec<_> = super::Library::get().lectures.values().cloned().collect();
        let deck = batna_anki::build(&lectures, &batna_anki::Options::default());
        let bytes = batna_anki::to_apkg(&deck, 0).unwrap();
        assert!(bytes.starts_with(b"PK"), "an .apkg is a zip");
        assert_eq!(deck.per_lecture.len(), lectures.len());
    }
}
