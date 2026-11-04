//! Settings › Your data › Export everything: one click writes the whole
//! store to `batna-export-YYYY-MM-DD.json`. The document comes from
//! `Store::export`; this module names, writes and reports the file.
//! On Android the file lands in the app's own folder first, then the system
//! "Save to" dialog offers Downloads or Drive (`platform::documents`).
// adapted from kabar-app main: crates/app/src/data_export.rs

use std::io::Write;
use std::path::{Path, PathBuf};

use dioxus::prelude::*;

use crate::i18n::data as tx;
use crate::platform::documents::{self, DocState};
use crate::store::{self, Store};

/// `batna-export-2026-10-27.json`, or `…-2.json`, `…-3.json` when one exists.
pub fn file_path(dir: &Path, date: &str) -> PathBuf {
    let base = format!("batna-export-{date}");
    let first = dir.join(format!("{base}.json"));
    if !first.exists() {
        return first;
    }
    (2..).map(|n| dir.join(format!("{base}-{n}.json"))).find(|p| !p.exists()).unwrap_or(first)
}

/// Today's local date, `YYYY-MM-DD`.
pub fn local_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Writes the export into `dir` (created if missing); returns the path and
/// the number of rows. Written to a temporary file first, so a failure never
/// leaves half a file behind.
pub fn save(store: &Store, dir: &Path, date: &str, now_ms: i64) -> Result<(PathBuf, u64), String> {
    let export = store.export(env!("CARGO_PKG_VERSION"), now_ms).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = file_path(dir, date);
    let tmp = path.with_extension("json.part");
    let write = || -> std::io::Result<()> {
        let mut out = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        serde_json::to_writer_pretty(&mut out, &export.document)?;
        out.write_all(b"\n")?;
        out.into_inner().map_err(|e| e.into_error())?.sync_all()?;
        std::fs::rename(&tmp, &path)
    };
    write().map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })?;
    Ok((path, export.rows))
}

/// The web: the same export as a browser download; returns its name and rows.
#[cfg(target_arch = "wasm32")]
fn download(store: &Store, date: &str, now_ms: i64) -> Result<(String, u64), String> {
    let export = store.export(env!("CARGO_PKG_VERSION"), now_ms).map_err(|e| e.to_string())?;
    let mut text = serde_json::to_vec_pretty(&export.document).map_err(|e| e.to_string())?;
    text.push(b'\n');
    let name = format!("batna-export-{date}.json");
    crate::platform::web::download(&name, "application/json", &text).map(|n| (n, export.rows))
}

/// Android: hands a file the app just wrote to the system "Save to" dialog
/// (Downloads, Drive …), once on its own, then again on a tap. Key it by
/// attempt so each new export opens the dialog.
#[component]
pub fn SaveTo(path: String) -> Element {
    let mut moved = use_signal(|| None::<DocState>);
    let mut save_to = move |path: String| {
        let name = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        moved.set(Some(DocState::Busy));
        match documents::save(&path, &name) {
            Ok(()) => {
                spawn(async move { moved.set(Some(documents::finished().await)) });
            }
            Err(_) => moved.set(Some(DocState::Failed)),
        }
    };
    use_hook({
        let path = path.clone();
        move || save_to(path)
    });
    match moved() {
        Some(DocState::Busy) => rsx! { p { class: "muted small", {tx::EXPORT_SAVING} } },
        Some(DocState::Saved(Some(name))) => rsx! {
            p { class: "small", "{tx::EXPORT_SAVED_AS} " bdi { dir: "ltr", "{name}" } }
        },
        Some(DocState::Saved(None)) => rsx! { p { class: "small", {tx::EXPORT_SAVED_OUT} } },
        other => rsx! {
            p { class: "muted small",
                if other == Some(DocState::Cancelled) {
                    {tx::EXPORT_SAVE_CANCELLED}
                } else if other.is_some() {
                    {tx::EXPORT_SAVE_FAILED}
                }
                " {tx::EXPORT_ANDROID_HINT}"
            }
            button { class: "verify-btn plain", onclick: move |_| save_to(path.clone()), {tx::EXPORT_SAVE_TO} }
        },
    }
}

#[component]
pub fn ExportCard() -> Element {
    rsx! { ExportCardIn { dir: ExportDir(store::export_dir) } }
}

/// Where exports are written: Downloads in the app, a scratch folder in tests.
#[derive(Clone, Copy)]
struct ExportDir(#[cfg_attr(target_arch = "wasm32", allow(dead_code))] fn() -> Result<PathBuf, String>);

impl PartialEq for ExportDir {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.0, other.0)
    }
}

#[component]
fn ExportCardIn(dir: ExportDir) -> Element {
    let mut outcome = use_signal(|| None::<Result<(String, u64), String>>);
    // Each export opens the "Save to" dialog afresh.
    let mut attempt = use_signal(|| 0u32);
    let mut run = move || {
        #[cfg(target_arch = "wasm32")]
        let result = download(store::shared(), &local_date(), store::now_ms());
        #[cfg(not(target_arch = "wasm32"))]
        let result = (dir.0)()
            .and_then(|dir| save(store::shared(), &dir, &local_date(), store::now_ms()))
            .map(|(p, rows)| (p.display().to_string(), rows));
        attempt += 1;
        outcome.set(Some(result));
    };
    rsx! {
        crate::ui::Card {
            h2 { {tx::EXPORT_TITLE} }
            button { class: "btn btn-primary", onclick: move |_| run(), {tx::EXPORT_BUTTON} }
            if let Some(result) = outcome() {
                ExportResult { key: "{attempt}", result, on_retry: move |_| run() }
            }
        }
    }
}

/// What the last export did: the file and its row count, or why it failed.
#[component]
fn ExportResult(result: Result<(String, u64), String>, on_retry: EventHandler) -> Element {
    match result {
        Ok((path, rows)) => rsx! {
            div { role: "status",
                p { class: "small", {tx::export_saved(rows)} }
                p { class: "small", bdi { dir: "ltr", "{path}" } }
                if cfg!(target_os = "android") {
                    SaveTo { path }
                }
            }
        },
        // Nothing was written; the same export runs again.
        Err(e) => rsx! {
            crate::persistence::WriteFailed { message: tx::EXPORT_FAILED, detail: e, on_retry: move |_| on_retry(()) }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_never_overwrite_and_files_are_whole() {
        let dir = std::env::temp_dir().join(format!("batna-export-test-{}", store::now_ms()));
        let s = Store::open_in_memory().unwrap();
        let (a, rows) = save(&s, &dir, "2026-10-27", 1).unwrap();
        let (b, _) = save(&s, &dir, "2026-10-27", 2).unwrap();
        assert!(a.ends_with("batna-export-2026-10-27.json"));
        assert!(b.ends_with("batna-export-2026-10-27-2.json"));
        let doc = crate::store::import::parse(&std::fs::read_to_string(&a).unwrap()).unwrap();
        assert_eq!(doc["row_count"], rows);
        assert!(
            std::fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().path().to_string_lossy().ends_with(".part"))
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_folder_that_cannot_be_made_fails_without_a_file() {
        let base = std::env::temp_dir().join(format!("batna-export-block-{}", store::now_ms()));
        std::fs::write(&base, b"a file, not a folder").unwrap();
        let s = Store::open_in_memory().unwrap();
        let err = save(&s, &base.join("inside"), "2026-10-27", 1).unwrap_err();
        assert!(err.contains("inside"), "{err}");
        let _ = std::fs::remove_file(base);
    }

    #[test]
    fn the_local_date_is_iso() {
        let d = local_date();
        assert_eq!(d.len(), 10, "{d}");
        assert!(chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").is_ok(), "{d}");
    }

    #[test]
    fn the_card_offers_one_button_before_any_export() {
        crate::store::use_memory_store();
        let html = dioxus::ssr::render_element(rsx! { ExportCard {} });
        assert!(html.contains(tx::EXPORT_TITLE) && html.contains(tx::EXPORT_BUTTON), "{html}");
        assert!(!html.contains("role=\"status\"") && !html.contains(tx::EXPORT_FAILED), "{html}");
    }

    fn shown(result: Result<(String, u64), String>) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct Case {
            result: Result<(String, u64), String>,
        }
        fn root(c: Case) -> Element {
            rsx! { ExportResult { result: c.result, on_retry: |_| {} } }
        }
        let mut dom = VirtualDom::new_with_props(root, Case { result });
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn a_result_names_the_file_or_the_failure() {
        let ok: Result<(String, u64), String> = Ok(("/data/batna-export-2026-10-27.json".into(), 1200));
        let html = shown(ok);
        assert!(html.contains(&tx::export_saved(1200)), "{html}");
        assert!(html.contains("batna-export-2026-10-27.json") && html.contains("role=\"status\""), "{html}");
        assert!(!html.contains(tx::EXPORT_SAVE_TO), "only Android hands it on: {html}");

        let bad: Result<(String, u64), String> = Err("disk full".into());
        let html = shown(bad);
        assert!(html.contains(tx::EXPORT_FAILED) && html.contains("disk full"), "{html}");
        assert!(html.contains("role=\"alert\""), "{html}");
    }

    fn scratch() -> Result<PathBuf, String> {
        static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
        Ok(DIR.get_or_init(|| tempfile::tempdir().unwrap()).path().to_path_buf())
    }

    fn nowhere() -> Result<PathBuf, String> {
        Err("no Downloads folder".into())
    }

    #[test]
    fn a_tap_writes_the_backup_and_says_where() {
        use crate::test_dom::TestDom;
        crate::store::use_memory_store();
        let mut t = TestDom::of(|| rsx! { ExportCardIn { dir: ExportDir(scratch) } });
        t.click(tx::EXPORT_BUTTON);
        let html = t.html();
        assert!(html.contains("role=\"status\"") && html.contains("batna-export-"), "{html}");
        let written: Vec<_> = std::fs::read_dir(scratch().unwrap()).unwrap().collect();
        assert!(!written.is_empty());

        let mut t = TestDom::of(|| rsx! { ExportCardIn { dir: ExportDir(nowhere) } });
        t.click(tx::EXPORT_BUTTON);
        assert!(
            t.html().contains(tx::EXPORT_FAILED) && t.html().contains("no Downloads folder"),
            "{}",
            t.html()
        );
        t.click("class=\"btn btn-secondary save-retry\"");
        assert!(t.html().contains("no Downloads folder"), "the same export ran again");
    }

    #[test]
    fn away_from_android_save_to_fails_and_offers_a_retry() {
        let html = dioxus::ssr::render_element(rsx! { SaveTo { path: "/tmp/batna-export.json" } });
        assert!(html.contains(tx::EXPORT_SAVE_FAILED) && html.contains(tx::EXPORT_ANDROID_HINT), "{html}");
        assert!(html.contains(tx::EXPORT_SAVE_TO), "{html}");
    }
}
