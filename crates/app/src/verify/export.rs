//! Settings → "Export my checks": every ✓ / ✎ / ✗ as `batna.verdicts/v1`
//! YAML. `batna-tools` reads it back to print a fix list per lecture and to
//! add his approvals to `content/approved.txt`.

use batna_core::review::verify::{Verdict, civil_from_days};
use dioxus::prelude::*;

use crate::i18n::verify as tx;
use crate::store::{export_dir, now_ms, use_review};

#[component]
pub fn ExportVerdicts() -> Element {
    let review = use_review();
    let mut result = use_signal(|| None::<Result<(String, String), String>>);
    let counts = review.backend.borrow().verdict_counts();
    let count = |v| counts.get(&v).copied().unwrap_or(0);
    let total: usize = counts.values().sum();

    let export = move |_| {
        let now = now_ms();
        let verdicts = review.backend.borrow().verdicts(now);
        let yaml = match serde_yaml_ng::to_string(&verdicts) {
            Ok(y) => y,
            Err(e) => return result.set(Some(Err(e.to_string()))),
        };
        let (y, m, d) = civil_from_days(now.div_euclid(86_400_000));
        let saved = export_dir().and_then(|dir| {
            let path = dir.join(format!("batna-verdicts-{y:04}-{m:02}-{d:02}.yaml"));
            std::fs::write(&path, &yaml).map(|_| path.display().to_string()).map_err(|e| e.to_string())
        });
        result.set(Some(saved.map(|p| (p, yaml))));
    };

    rsx! {
        section { class: "card",
            h2 { {tx::EXPORT_TITLE} }
            p { class: "muted small", {tx::EXPORT_BODY} }
            p { class: "small",
                "✓ {count(Verdict::Approved)} · ✎ {count(Verdict::Edited)} · ✗ {count(Verdict::Wrong)}"
            }
            button { class: "btn btn-primary", disabled: total == 0, onclick: export, {tx::EXPORT_BUTTON} }
            match result() {
                Some(Ok((path, yaml))) => rsx! {
                    p { class: "small", role: "status", "{tx::EXPORT_SAVED} " bdi { dir: "ltr", "{path}" } }
                    details {
                        summary { class: "small", {tx::EXPORT_SHOW} }
                        textarea { class: "export-text", dir: "ltr", readonly: true, rows: 10, value: "{yaml}" }
                    }
                },
                Some(Err(e)) => rsx! { p { class: "verify-error", role: "alert", "{tx::EXPORT_FAILED} " bdi { dir: "ltr", "{e}" } } },
                None => rsx! {},
            }
        }
    }
}
