//! `batna-tools verdicts --cloud`: Mohamed's current ✓ ✎ ✗ marks straight
//! from Batna Cloud (#1352), so the fix list never waits for a file.
//!
//! It reads with the verdicts read key he makes in Settings › Sync
//! (`BATNA_VERDICTS_KEY`, kept as an environment secret, never in the repo).
//! That key reads his marks and nothing else; `BATNA_CLOUD_URL` points at
//! another Batna Cloud (a local one in tests). The answer becomes the same
//! `batna.verdicts/v1` an app export is, so everything downstream is shared.

use batna_core::review::verify::{Judgement, Verdicts};
use batna_protocol::cloud::VerdictRows;

const DEFAULT_URL: &str = "https://batna-cloud.hamorefaat85.workers.dev";

pub fn fetch() -> Result<Verdicts, String> {
    let key = std::env::var("BATNA_VERDICTS_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or("set BATNA_VERDICTS_KEY to the read key from Settings › Sync (Batna Cloud)")?;
    let base = std::env::var("BATNA_CLOUD_URL").unwrap_or_else(|_| DEFAULT_URL.into());
    let answer = ureq::get(&format!("{}/v1/verdicts", base.trim_end_matches('/')))
        .header("authorization", &format!("Bearer {}", key.trim()))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("Batna Cloud: {e}"))?;
    let status = answer.status().as_u16();
    let body = answer.into_body().read_to_string().map_err(|e| format!("Batna Cloud: {e}"))?;
    match status {
        200 => from_json(&body, now_ms()),
        401 => Err("Batna Cloud refused the read key: it was replaced or removed. Make a new one in Settings › Sync.".into()),
        _ => Err(format!("Batna Cloud answered {status}: {}", body.chars().take(300).collect::<String>())),
    }
}

/// `GET /v1/verdicts`'s answer as an export made at `now_ms`. A row this
/// build can't read is an error, never silently left out.
pub fn from_json(body: &str, now_ms: i64) -> Result<Verdicts, String> {
    let rows: VerdictRows = serde_json::from_str(body).map_err(|e| format!("Batna Cloud's answer: {e}"))?;
    let judged = rows
        .rows
        .iter()
        .map(|r| {
            serde_json::from_str::<Judgement>(&r.value)
                .map(|j| (r.item_id.as_str(), j))
                .map_err(|e| format!("{}: can't read the mark: {e}", r.item_id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Verdicts::from_judgements(judged.iter().map(|(id, j)| (*id, j)), now_ms))
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use batna_core::review::verify::Verdict;

    use super::*;

    const FIXTURE: &str = include_str!("../../../fixtures/cloud/verdicts.json");

    #[test]
    fn the_cloud_answer_becomes_an_export() {
        let v = from_json(FIXTURE, 1_794_000_000_000).unwrap();
        assert_eq!(v.schema, batna_core::review::verify::VERDICTS_SCHEMA);
        assert_eq!(v.exported, "2026-11-06T21:20:00Z");
        let ids: Vec<(&str, Verdict)> = v.verdicts.iter().map(|e| (e.id.as_str(), e.verdict)).collect();
        assert_eq!(
            ids,
            [
                ("hb.L01.001", Verdict::Approved),
                ("hb.L01.004", Verdict::Wrong),
                ("hb.L02.010", Verdict::Edited)
            ]
        );
        let wrong = &v.verdicts[1];
        assert_eq!(wrong.note.as_deref(), Some("page 14 says 7-9"));
        assert_eq!(wrong.fingerprint.as_deref(), Some("0123456789abcdef"));
        assert_eq!(wrong.at, "2026-11-15T09:00:00Z");
    }

    #[test]
    fn an_unreadable_mark_is_reported_not_dropped() {
        let bad = r#"{"rows":[{"item_id":"hb.L01.001","value":"{\"verdict\":\"maybe\"}"}]}"#;
        assert!(from_json(bad, 0).unwrap_err().starts_with("hb.L01.001: can't read the mark"));
        assert!(from_json("<html>", 0).unwrap_err().starts_with("Batna Cloud's answer"));
    }
}
