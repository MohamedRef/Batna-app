//! The verdicts read key (#1352). Mohamed makes it in Settings › Sync; the
//! content tools send it as a bearer token to `GET /v1/verdicts` and get his
//! current ✓ ✎ ✗ marks (`item_status`), nothing else. It can't write, can't
//! sync and can't reach any other table. One key per account; a new one
//! replaces the old, and removing it stops it at once.
//!
//! The key is `bvk1.<account id>.<secret>`: the account id routes it to the
//! account's Durable Object, which keeps only the secret's SHA-256.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use batna_protocol::cloud::VerdictRow;

use crate::sql::{Sql, SqlError, col};
use crate::tokens::{sha256_hex, valid_account_id};

pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS read_key (
    one INTEGER PRIMARY KEY CHECK (one = 1),
    hash TEXT NOT NULL,
    created_ms INTEGER NOT NULL
)";

const PREFIX: &str = "bvk1";

/// A new key for `account_id` from 32 random bytes, and the hash to keep.
pub fn new_key(account_id: &str, secret: [u8; 32]) -> (String, String) {
    let encoded = URL_SAFE_NO_PAD.encode(secret);
    (format!("{PREFIX}.{account_id}.{encoded}"), sha256_hex(encoded.as_bytes()))
}

/// The account a presented key names, and its secret's hash; `None` for
/// anything that isn't a read key.
pub fn parse(token: &str) -> Option<(String, String)> {
    let mut parts = token.trim().split('.');
    let (Some(PREFIX), Some(account), Some(secret), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    (valid_account_id(account) && secret.len() == 43)
        .then(|| (account.to_owned(), sha256_hex(secret.as_bytes())))
}

/// The account object's read key.
pub struct ReadKey<'a, S: Sql> {
    db: &'a S,
}

impl<'a, S: Sql> ReadKey<'a, S> {
    pub fn open(db: &'a S) -> Self {
        Self { db }
    }

    /// Keeps `hash` as the account's only key.
    pub fn set(&self, hash: &str, now_ms: u64) -> Result<(), SqlError> {
        self.db.exec(
            "INSERT INTO read_key (one, hash, created_ms) VALUES (1, ?, ?)
             ON CONFLICT(one) DO UPDATE SET hash = excluded.hash, created_ms = excluded.created_ms",
            &[hash.into(), now_ms.into()],
        )?;
        Ok(())
    }

    /// Removes the key; true when there was one.
    pub fn clear(&self) -> Result<bool, SqlError> {
        let had = self.created()?.is_some();
        self.db.exec("DELETE FROM read_key", &[])?;
        Ok(had)
    }

    /// When the current key was made.
    pub fn created(&self) -> Result<Option<u64>, SqlError> {
        let rows = self.db.exec("SELECT created_ms FROM read_key WHERE one = 1", &[])?;
        Ok(rows.first().map(|r| col(r, 0, "created_ms")).transpose()?.and_then(|v| v.as_u64()))
    }

    /// Whether `hash` is the current key's.
    pub fn matches(&self, hash: &str) -> Result<bool, SqlError> {
        let rows = self.db.exec("SELECT hash FROM read_key WHERE one = 1", &[])?;
        let Some(row) = rows.first() else { return Ok(false) };
        let kept = col(row, 0, "hash")?.as_str().unwrap_or_default();
        // Both are hex SHA-256: compare every byte, not up to the first difference.
        Ok(kept.len() == hash.len() && kept.bytes().zip(hash.bytes()).fold(0u8, |d, (a, b)| d | (a ^ b)) == 0)
    }
}

/// Every current (not deleted) `item_status` row, by item id.
pub fn verdicts(db: &impl Sql) -> Result<Vec<VerdictRow>, SqlError> {
    let rows = db.exec(
        "SELECT o.payload FROM row_state r JOIN ops o ON o.seq = r.seq
         WHERE r.tbl = 'item_status' AND r.deleted = 0 ORDER BY r.row_id",
        &[],
    )?;
    let mut out = Vec::new();
    for row in &rows {
        let Some(text) = col(row, 0, "payload")?.as_str() else { continue };
        let payload: serde_json::Value = serde_json::from_str(text).map_err(|e| SqlError(e.to_string()))?;
        let (Some(item_id), Some(value)) = (payload["item_id"].as_str(), payload["value"].as_str()) else {
            continue;
        };
        out.push(VerdictRow { item_id: item_id.to_owned(), value: value.to_owned() });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use batna_protocol::Hlc;
    use batna_protocol::sync::{ClientMsg, Op, OpKind};
    use serde_json::json;

    use super::*;
    use crate::hub::{Conn, Hub};
    use crate::sql::testing::MemSql;

    const ACCOUNT: &str = "g0123456789abcdef01234567";

    fn db() -> MemSql {
        let db = MemSql::new();
        crate::account::migrate(&db).unwrap();
        db
    }

    #[test]
    fn a_key_round_trips_and_nothing_else_parses() {
        let (key, hash) = new_key(ACCOUNT, [7; 32]);
        assert!(key.starts_with("bvk1.g0123456789abcdef01234567."), "{key}");
        assert_eq!(parse(&key), Some((ACCOUNT.to_owned(), hash)));
        assert_eq!(parse(&format!(" {key}\n")).map(|p| p.0).as_deref(), Some(ACCOUNT));
        let refresh = crate::tokens::new_refresh(ACCOUNT, "s24u", [7; 32]).token;
        for bad in [refresh.as_str(), "bvk1.nope.x", "bvk1.g0123456789abcdef01234567.short", "", "bvk1"] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn one_key_at_a_time_and_removing_it_stops_it() {
        let db = db();
        let keys = ReadKey::open(&db);
        assert_eq!(keys.created().unwrap(), None);
        let (_, first) = new_key(ACCOUNT, [1; 32]);
        let (_, second) = new_key(ACCOUNT, [2; 32]);
        assert!(!keys.matches(&first).unwrap(), "no key yet");
        keys.set(&first, 10).unwrap();
        assert!(keys.matches(&first).unwrap());
        assert_eq!(keys.created().unwrap(), Some(10));
        keys.set(&second, 20).unwrap();
        assert!(!keys.matches(&first).unwrap(), "a new key replaces the old");
        assert!(keys.matches(&second).unwrap());
        assert_eq!(keys.created().unwrap(), Some(20));
        assert!(keys.clear().unwrap());
        assert!(!keys.matches(&second).unwrap());
        assert!(!keys.clear().unwrap());
    }

    #[test]
    fn verdicts_are_the_current_item_status_rows_only() {
        let db = db();
        let hub = Hub::open(&db);
        let mut conn = Conn { device: "s24u".into(), conn_id: "c".into(), ready: false };
        let op = |id: &str, table: &str, row: &str, kind, wall_ms, payload| Op {
            id: id.into(),
            table: table.into(),
            row_id: row.into(),
            kind,
            hlc: Hlc { wall_ms, counter: 0, device: "s24u".into() },
            relay: false,
            payload,
        };
        let status =
            |item: &str, value: &str| Some(json!({ "item_id": item, "status": "wrong", "value": value }));
        let ops = vec![
            op("1", "item_status", "hb.L01.002", OpKind::Upsert, 1, status("hb.L01.002", "{\"v\":1}")),
            op("2", "item_status", "hb.L01.001", OpKind::Upsert, 2, status("hb.L01.001", "{\"v\":2}")),
            op("3", "item_status", "hb.L01.002", OpKind::Upsert, 3, status("hb.L01.002", "{\"v\":3}")),
            op("4", "item_status", "hb.L01.009", OpKind::Upsert, 4, status("hb.L01.009", "{}")),
            op("5", "item_status", "hb.L01.009", OpKind::Delete, 5, None),
            op(
                "6",
                "settings",
                "theme",
                OpKind::Upsert,
                6,
                Some(json!({ "key": "theme", "value": "\"night\"" })),
            ),
        ];
        for msg in [
            ClientMsg::Hello { protocol: batna_protocol::PROTOCOL_VERSION, last_seq: 0 },
            ClientMsg::Push { batch: 1, ops },
        ] {
            hub.handle(&mut conn, &serde_json::to_string(&msg).unwrap(), 10).unwrap();
        }
        let rows = verdicts(&db).unwrap();
        let got: Vec<(&str, &str)> = rows.iter().map(|r| (r.item_id.as_str(), r.value.as_str())).collect();
        assert_eq!(got, [("hb.L01.001", "{\"v\":2}"), ("hb.L01.002", "{\"v\":3}")]);
    }
}
