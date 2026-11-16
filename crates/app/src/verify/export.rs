//! Settings → "Export my checks": every ✓ / ✎ / ✗ as `batna.verdicts/v1`
//! YAML. `batna-tools` reads it back to print a fix list per lecture and to
//! add his approvals to `content/approved.txt`.

use batna_core::review::verify::Verdict;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::verify as tx;
use crate::store::{export_dir, now_ms, use_review};

/// The synced setting that remembers the last file (#1352).
pub const EXPORTED_KEY: &str = "verify.exported";

/// How long an unsent ✗ waits before Today mentions it.
pub const WAIT_MS: i64 = 3 * 86_400_000;

/// When the last file was made and how many verdicts it held. Synced, so a
/// file made on the Mac counts on the phone too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastExport {
    pub at_ms: i64,
    pub verdicts: usize,
}

pub fn last_export() -> Option<LastExport> {
    crate::store::shared().setting(EXPORTED_KEY)
}

/// Unsent ✗ items for Today's quiet line: 0 until the oldest has waited
/// [`WAIT_MS`].
pub fn waiting_wrong(review: &crate::store::Review, now_ms: i64) -> usize {
    review.backend.borrow().unsent_verdicts(last_export().map(|l| l.at_ms)).waiting_wrong(now_ms, WAIT_MS)
}

/// What the last tap did: the file's path and text, and for "send", whether
/// the share menu opened.
#[derive(Clone, PartialEq)]
struct Made {
    path: String,
    yaml: String,
    shared: Option<bool>,
}

#[component]
pub fn ExportVerdicts() -> Element {
    let review = use_review();
    let mut result = use_signal(|| None::<Result<Made, String>>);
    // Each export opens the Android "Save to" dialog afresh.
    let mut attempt = use_signal(|| 0u32);
    let mut last = use_signal(last_export);
    let counts = review.backend.borrow().verdict_counts();
    let count = |v| counts.get(&v).copied().unwrap_or(0);
    let total: usize = counts.values().sum();
    let unsent = review.backend.borrow().unsent_verdicts(last().map(|l| l.at_ms));
    let can_share = crate::platform::share::available();

    let export = move |share: bool| {
        let now = now_ms();
        let verdicts = review.backend.borrow().verdicts(now);
        let yaml = match serde_yaml_ng::to_string(&verdicts) {
            Ok(y) => y,
            Err(e) => return result.set(Some(Err(e.to_string()))),
        };
        // His local date, not UTC: an export at 1 am must not overwrite yesterday's (#36).
        let date = crate::store::zone().calendar_day(now).iso();
        #[cfg(target_arch = "wasm32")]
        let saved = crate::platform::web::download(
            &format!("batna-verdicts-{date}.yaml"),
            "application/yaml",
            yaml.as_bytes(),
        );
        #[cfg(not(target_arch = "wasm32"))]
        let saved = save_in(export_dir(), &date, &yaml);
        attempt += 1;
        let made = saved.map(|path| {
            let mark = LastExport { at_ms: now, verdicts: verdicts.verdicts.len() };
            if let Err(e) = crate::store::shared().put_setting(EXPORTED_KEY, &mark) {
                crate::diag::report!("verdict export mark: {e}");
            }
            last.set(Some(mark));
            let shared = share.then(|| {
                crate::platform::share::file(&path, "text/plain", tx::EXPORT_SUBJECT)
                    .map_err(|e| crate::diag::report!("verdict share: {e}"))
                    .is_ok()
            });
            Made { path, yaml, shared }
        });
        result.set(Some(made));
    };

    rsx! {
        crate::ui::Card { class: "verdict-export",
            h2 { {tx::EXPORT_TITLE} }
            p { class: "small",
                "✓ {count(Verdict::Approved)} · ✎ {count(Verdict::Edited)} · ✗ {count(Verdict::Wrong)}"
            }
            p { class: "small muted verdict-last",
                match last() {
                    Some(l) => {
                        let (_, month, day) = crate::store::zone().calendar_day(l.at_ms).ymd();
                        tx::last_export(day, crate::i18n::plan::MONTHS[month as usize - 1], l.verdicts)
                    }
                    None => tx::EXPORT_NEVER.to_string(),
                }
            }
            if unsent.total() > 0 && last().is_some() {
                p { class: "small verdict-unsent", {tx::unsent(unsent.approved, unsent.edited, unsent.wrong)} }
            } else if last().is_some() && total > 0 {
                p { class: "small muted verdict-unsent", {tx::EXPORT_ALL_IN} }
            }
            div { class: "verify-actions",
                if can_share {
                    button { class: "btn btn-primary verdict-share", disabled: total == 0, onclick: {
                            let mut export = export.clone();
                            move |_| export(true)
                        },
                        {tx::EXPORT_SHARE}
                    }
                }
                button {
                    class: if can_share { "btn btn-secondary verdict-save" } else { "btn btn-primary verdict-save" },
                    disabled: total == 0,
                    onclick: {
                        let mut export = export.clone();
                        move |_| export(false)
                    },
                    {if can_share { tx::EXPORT_SAVE_ONLY } else { tx::EXPORT_BUTTON }}
                }
            }
            match result() {
                Some(Ok(Made { path, yaml, shared })) => rsx! {
                    p { class: "small", role: "status",
                        match shared {
                            Some(true) => tx::EXPORT_SHARED,
                            Some(false) => tx::EXPORT_SHARE_FAILED,
                            None => tx::EXPORT_SAVED,
                        }
                        " "
                        bdi { dir: "ltr", "{path}" }
                    }
                    if cfg!(target_os = "android") && shared.is_none() {
                        crate::data_export::SaveTo { key: "{attempt}", path: path.clone() }
                    }
                    details {
                        summary { class: "small", {tx::EXPORT_SHOW} }
                        textarea { class: "export-text", dir: "ltr", readonly: true, rows: 10, value: "{yaml}" }
                    }
                },
                Some(Err(e)) => rsx! {
                    crate::persistence::WriteFailed {
                        message: tx::EXPORT_FAILED,
                        detail: e,
                        on_retry: {
                            let mut export = export.clone();
                            move |_| export(false)
                        },
                    }
                },
                None => rsx! {},
            }
        }
    }
}

/// Writes the export into `dir` as `batna-verdicts-<date>.yaml`; the path, or why not.
#[cfg(not(target_arch = "wasm32"))]
fn save_in(dir: Result<std::path::PathBuf, String>, date: &str, yaml: &str) -> Result<String, String> {
    let path = dir?.join(format!("batna-verdicts-{date}.yaml"));
    std::fs::write(&path, yaml).map(|_| path.display().to_string()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_export_lands_in_the_folder_under_todays_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(Ok(dir.path().to_path_buf()), "2026-11-04", "items: []\n").unwrap();
        assert!(path.ends_with("batna-verdicts-2026-11-04.yaml"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "items: []\n");
        assert_eq!(save_in(Err("no folder".into()), "d", "x"), Err("no folder".to_string()));
        let missing = dir.path().join("gone");
        assert!(save_in(Ok(missing), "d", "x").is_err(), "a folder that isn't there says so");
    }
}
