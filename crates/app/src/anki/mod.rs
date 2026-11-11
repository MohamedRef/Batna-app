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
//! His ✓/✎/✗ from verify-while-study apply: ✗ items are left out, ✎ items
//! carry his edited text, ✓ items are tagged `approved`.
//!
//! The app holds merged lectures only; the shared-folder file can also carry
//! drafts. Both use the same note ids, so either import updates the other.

use dioxus::prelude::*;

use std::collections::{BTreeMap, BTreeSet};

use batna_core::content::{Item, Lecture};
use batna_core::review::verify::{self, Check, Judgement, Verdict};

use crate::content::Library;
use crate::i18n::anki as tx;
use crate::store::use_review;

#[cfg(target_os = "android")]
mod android;

const FILE_NAME: &str = "batna.apkg";

const CSS: &str = "
.anki-export .btn:disabled { cursor: progress; }
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
    rsx! { AnkiCard { start: State::Idle, save: Save(save_and_open) } }
}

/// Where a built deck goes: saved and opened in the app, somewhere harmless in tests.
#[derive(Clone, Copy)]
struct Save(fn(&[u8]) -> Result<String, String>);

impl PartialEq for Save {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

/// The export card from `start` (idle in the app).
#[component]
fn AnkiCard(start: State, save: Save) -> Element {
    let mut state = use_signal(|| start.clone());
    let review = use_review();
    let lib = Library::get();
    let lectures = lib.lectures.len();
    let items: usize = lib.lectures.values().map(|l| l.items.len()).sum();
    let busy = *state.read() == State::Working;

    let export = move || {
        state.set(State::Working);
        // Let the button show "working" before the (sub-second) build runs.
        let review = review.clone();
        spawn(async move {
            futures_yield().await;
            let backend = review.backend.borrow();
            let mine = item_notes(backend.notes());
            state.set(match export_now(|id| backend.judgement(id), &mine, save) {
                Ok((notes, place)) => State::Done { notes, place },
                Err(e) => State::Failed(e),
            });
        });
    };

    rsx! {
        style { {CSS} }
        crate::ui::Card { class: "anki-export",
            h2 { {tx::TITLE} }
            p { class: "muted small", {tx::contents(lectures, items)} }
            button { class: "btn btn-primary", disabled: busy,
                onclick: {
                    let mut export = export.clone();
                    move |_| export()
                },
                if busy { {tx::WORKING} } else { {tx::BUTTON} }
            }
            match &*state.read() {
                State::Done { notes, place } => rsx! {
                    p { class: "note result", {tx::done(*notes)} " {tx::SAVED_IN} " bdi { dir: "ltr", "{place}" } }
                },
                State::Failed(e) => rsx! {
                    crate::persistence::WriteFailed {
                        message: tx::FAILED,
                        detail: e.clone(),
                        on_retry: {
                            let mut export = export.clone();
                            move |_| export()
                        },
                    }
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

/// What his verdicts mean for the deck: ✗ ids, ✎ items with the edit
/// applied, and ✓ ids. A verdict counts only while it matches the item's
/// current text (`Judgement::current`), the same rule review uses.
#[derive(Default)]
struct Verdicts {
    wrong: BTreeSet<String>,
    edits: BTreeMap<String, Item>,
    approved: BTreeSet<String>,
}

fn verdicts<'a>(lectures: &[Lecture], judgement: impl Fn(&str) -> Option<&'a Judgement>) -> Verdicts {
    let mut out = Verdicts::default();
    for item in lectures.iter().flat_map(|l| &l.items) {
        let j = judgement(&item.id);
        if verify::check(item, j) == Check::Hidden {
            out.wrong.insert(item.id.clone());
            continue;
        }
        match j.filter(|j| j.current(item)).map(|j| j.verdict) {
            Some(Verdict::Edited) => {
                out.edits.insert(item.id.clone(), verify::effective(item, j));
            }
            Some(Verdict::Approved) => {
                out.approved.insert(item.id.clone());
            }
            _ => {}
        }
    }
    out
}

/// His notes on items, by item id, for the deck's `Notes` field. Notes on
/// pages and lesson steps have no Anki card to sit on. Adapted from Steps-app
/// main: crates/app/src/anki/mod.rs.
fn item_notes(notes: &[batna_core::note::Note]) -> BTreeMap<String, String> {
    notes
        .iter()
        .filter_map(|n| match &n.anchor {
            batna_core::note::Anchor::Item(id) => Some((id.clone(), n.text.clone())),
            _ => None,
        })
        .collect()
}

/// Builds the deck from the embedded content, his verdicts and his notes;
/// returns the note count and where the file went.
fn export_now<'a>(
    judgement: impl Fn(&str) -> Option<&'a Judgement>,
    mine: &BTreeMap<String, String>,
    save: Save,
) -> Result<(usize, String), String> {
    let (notes, bytes) = build_deck(judgement, mine)?;
    let place = (save.0)(&bytes)?;
    Ok((notes, place))
}

/// The `.apkg` bytes and how many notes they hold.
fn build_deck<'a>(
    judgement: impl Fn(&str) -> Option<&'a Judgement>,
    mine: &BTreeMap<String, String>,
) -> Result<(usize, Vec<u8>), String> {
    let mut lectures: Vec<_> = Library::get().lectures.values().cloned().collect();
    // His own cards ride along as `Batna::My own cards`.
    lectures.extend(crate::own::lecture().cloned());
    let v = verdicts(&lectures, judgement);
    let opts = batna_anki::Options {
        approved: Some(&v.approved),
        wrong: Some(&v.wrong),
        edits: Some(&v.edits),
        drafts: None,
        program: Some(&crate::content::Library::get().program),
        notes: Some(mine),
    };
    let deck = batna_anki::build(&lectures, &opts);
    let now_ms = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let bytes = batna_anki::to_apkg(&deck, now_ms)?;
    Ok((deck.notes.len(), bytes))
}

#[cfg(target_os = "android")]
fn save_and_open(bytes: &[u8]) -> Result<String, String> {
    android::save_and_open(FILE_NAME, bytes)
}

/// The web: the deck downloads; the browser offers to open it in Anki.
#[cfg(target_arch = "wasm32")]
fn save_and_open(bytes: &[u8]) -> Result<String, String> {
    crate::platform::web::download(FILE_NAME, "application/octet-stream", bytes)
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn save_and_open(bytes: &[u8]) -> Result<String, String> {
    let home =
        std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).ok_or("no home folder")?;
    let path = save_in(std::path::Path::new(&home), bytes)?;
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

/// Writes the deck to `home`'s Downloads, or `home` itself without one.
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn save_in(home: &std::path::Path, bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    let downloads = home.join("Downloads");
    let dir = if downloads.is_dir() { downloads } else { home.to_path_buf() };
    let path = dir.join(FILE_NAME);
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn judged(item: &Item, verdict: Verdict) -> Judgement {
        Judgement {
            verdict,
            edit: Default::default(),
            note: None,
            fingerprint: verify::fingerprint(item),
            at_ms: 1,
        }
    }

    #[test]
    fn his_verdicts_shape_the_deck() {
        let lib = Library::get();
        // Unreviewed items: his verdict is what decides.
        let open = |l: &&Lecture| {
            l.items.len() >= 4
                && l.items[..4].iter().all(|i| i.status == batna_core::content::Status::Unverified)
        };
        let lecture = lib.lectures.values().find(open).expect("unreviewed content").clone();
        let items = &lecture.items;
        let mut stale = judged(&items[3], Verdict::Wrong);
        stale.fingerprint = "old text".into();
        let js: BTreeMap<String, Judgement> = [
            (items[0].id.clone(), judged(&items[0], Verdict::Approved)),
            (items[1].id.clone(), judged(&items[1], Verdict::Edited)),
            (items[2].id.clone(), judged(&items[2], Verdict::Wrong)),
            (items[3].id.clone(), stale),
        ]
        .into();
        let v = verdicts(std::slice::from_ref(&lecture), |id| js.get(id));
        assert_eq!(
            v.wrong,
            BTreeSet::from([items[2].id.clone()]),
            "✗ leaves the deck; a verdict on old text no longer counts"
        );
        assert_eq!(v.approved, BTreeSet::from([items[0].id.clone()]), "✓ is tagged approved");
        assert_eq!(v.edits.keys().collect::<Vec<_>>(), [&items[1].id], "✎ carries his text");
        assert!(!v.approved.contains(&items[1].id) && !v.wrong.contains(&items[0].id));
    }

    #[test]
    fn only_notes_on_items_go_to_anki() {
        use batna_core::note::{Anchor, Note};
        let n =
            |anchor, text: &str| Note { anchor, text: text.into(), page: None, created_ms: 1, edited_ms: 1 };
        let notes = [
            n(Anchor::Item("hb.L01.001".into()), "Child-Pugh"),
            n(Anchor::Lesson("hb.L01.lesson1#2".into()), "step"),
            n(Anchor::page("hb", 3), "page"),
            n(Anchor::Item("own.x".into()), "mine"),
        ];
        let mine = item_notes(&notes);
        assert_eq!(
            mine,
            BTreeMap::from([
                ("hb.L01.001".to_string(), "Child-Pugh".to_string()),
                ("own.x".to_string(), "mine".to_string())
            ])
        );
        // They reach the deck: the built file holds the note's text.
        let (_, bytes) = build_deck(|_| None, &mine).unwrap();
        let (_, plain) = build_deck(|_| None, &BTreeMap::new()).unwrap();
        assert_ne!(bytes, plain);
    }

    #[test]
    fn the_deck_builds_and_lands_in_downloads() {
        let (notes, bytes) = build_deck(|_| None, &BTreeMap::new()).unwrap();
        assert!(notes > 0 && bytes.starts_with(b"PK"));
        let home = tempfile::tempdir().unwrap();
        assert_eq!(save_in(home.path(), b"deck").unwrap(), home.path().join(FILE_NAME), "no Downloads: home");
        std::fs::create_dir(home.path().join("Downloads")).unwrap();
        let path = save_in(home.path(), b"deck").unwrap();
        assert_eq!(path, home.path().join("Downloads").join(FILE_NAME));
        assert_eq!(std::fs::read(path).unwrap(), b"deck");
        assert!(save_in(&home.path().join("gone"), b"x").is_err());
    }

    fn render(start: State) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct P {
            start: State,
        }
        fn app(p: P) -> Element {
            crate::store::use_provide_review();
            rsx! { AnkiCard { start: p.start.clone(), save: Save(nowhere) } }
        }
        crate::store::use_memory_store();
        let mut dom = VirtualDom::new_with_props(app, P { start });
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn the_card_says_what_it_holds_and_how_the_export_went() {
        let lib = Library::get();
        let items: usize = lib.lectures.values().map(|l| l.items.len()).sum();
        let page = render(State::Idle);
        assert!(page.contains(tx::TITLE) && page.contains(tx::BUTTON), "{page}");
        assert!(page.contains(&tx::contents(lib.lectures.len(), items)), "{page}");
        let page = render(State::Working);
        assert!(page.contains(tx::WORKING) && page.contains("disabled"), "{page}");
        let page = render(State::Done { notes: 340, place: "/home/m/Downloads/batna.apkg".into() });
        assert!(page.contains(&tx::done(340)) && page.contains(tx::SAVED_IN), "{page}");
        assert!(page.contains("/home/m/Downloads/batna.apkg"));
        let page = render(State::Failed("disk full".into()));
        assert!(page.contains(tx::FAILED) && page.contains("disk full"), "{page}");
    }

    #[test]
    fn the_embedded_content_exports() {
        let lectures: Vec<_> = super::Library::get().lectures.values().cloned().collect();
        let deck = batna_anki::build(&lectures, &batna_anki::Options::default());
        let bytes = batna_anki::to_apkg(&deck, 0).unwrap();
        assert!(bytes.starts_with(b"PK"), "an .apkg is a zip");
        assert_eq!(deck.per_lecture.len(), lectures.len());
    }

    /// Keeps the deck in memory: tests never write to the real home folder.
    fn nowhere(bytes: &[u8]) -> Result<String, String> {
        if bytes.starts_with(b"PK") { Ok("memory".into()) } else { Err("not an .apkg".into()) }
    }

    fn broken(_: &[u8]) -> Result<String, String> {
        Err("disk full".into())
    }

    #[test]
    fn a_tap_builds_the_deck_and_says_where_it_went() {
        use crate::tap_dom::TestDom;
        #[derive(Props, Clone, PartialEq)]
        struct P {
            save: Save,
        }
        fn app(p: P) -> Element {
            crate::store::use_provide_review();
            rsx! { AnkiCard { start: State::Idle, save: p.save } }
        }
        crate::store::use_memory_store();
        let mut t = TestDom::new(VirtualDom::new_with_props(app, P { save: Save(nowhere) }));
        t.click(tx::BUTTON);
        assert!(t.html().contains(tx::SAVED_IN) && t.html().contains("memory"), "{}", t.html());
        let mut t = TestDom::new(VirtualDom::new_with_props(app, P { save: Save(broken) }));
        t.click(tx::BUTTON);
        assert!(t.html().contains(tx::FAILED) && t.html().contains("disk full"), "{}", t.html());
    }

    #[test]
    fn retry_after_a_failure_exports_again() {
        use crate::tap_dom::TestDom;
        #[derive(Props, Clone, PartialEq)]
        struct P {
            start: State,
        }
        fn app(p: P) -> Element {
            crate::store::use_provide_review();
            rsx! { AnkiCard { start: p.start.clone(), save: Save(nowhere) } }
        }
        crate::store::use_memory_store();
        let mut t = TestDom::new(VirtualDom::new_with_props(app, P { start: State::Idle }));
        t.click(tx::BUTTON);
        assert!(t.html().contains("memory"), "{}", t.html());

        let mut t =
            TestDom::new(VirtualDom::new_with_props(app, P { start: State::Failed("disk full".into()) }));
        t.click("class=\"btn btn-secondary save-retry\"");
        assert!(t.html().contains("memory") && !t.html().contains("disk full"), "{}", t.html());
    }

    #[test]
    fn yielding_once_lets_the_label_paint() {
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        let mut f = std::pin::pin!(futures_yield());
        assert!(f.as_mut().poll(&mut cx).is_pending());
        assert!(f.as_mut().poll(&mut cx).is_ready());
    }
}
