//! Settings › Your data › Export everything: one click writes the whole
//! store to `batna-export-YYYY-MM-DD.json`. The document comes from
//! `Store::export`; this module names, writes and reports the file.
// adapted from kabar-app main: crates/app/src/data_export.rs (the Android
// "Save to" dialog follows once the android/ module exists)

use std::io::Write;
use std::path::{Path, PathBuf};

use dioxus::prelude::*;

use crate::i18n::data as tx;
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

#[component]
pub fn ExportCard() -> Element {
    let mut outcome = use_signal(|| None::<Result<(String, u64), String>>);
    let run = move |_| {
        let result = store::export_dir()
            .and_then(|dir| save(store::shared(), &dir, &local_date(), store::now_ms()))
            .map(|(p, rows)| (p.display().to_string(), rows));
        outcome.set(Some(result));
    };
    rsx! {
        section { class: "card",
            h2 { {tx::EXPORT_TITLE} }
            p { class: "muted small", {tx::EXPORT_BODY} }
            button { class: "primary", onclick: run, {tx::EXPORT_BUTTON} }
            match outcome() {
                Some(Ok((path, rows))) => rsx! {
                    div { role: "status",
                        p { class: "small", "{tx::EXPORT_SAVED_1} {rows} {tx::EXPORT_SAVED_2}" }
                        p { class: "small", bdi { dir: "ltr", "{path}" } }
                        if cfg!(target_os = "android") {
                            p { class: "muted small", {tx::EXPORT_ANDROID_HINT} }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { class: "verify-error", role: "alert", "{tx::EXPORT_FAILED} " bdi { dir: "ltr", "{e}" } }
                },
                None => rsx! {},
            }
        }
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
}
