//! The local SQLite store: review events, Mohamed's verdicts and review
//! settings. One file on each device, synced through Batna Cloud when
//! Mohamed signs in (`sync.rs`).
//!
//! Every synced table carries `account_id`, `device_id`, an HLC and a
//! tombstone, so sync can be added without a schema change. Review events
//! are append-only; verdicts and settings are last-writer-wins by HLC.
// adapted from kabar-app main: crates/app/src/store.rs

mod backend;
pub mod export;
pub mod import;
pub mod own;
mod paths;
pub mod sync;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use batna_core::review::verify::Judgement;
use batna_core::review::{Grade, Hlc, HlcClock, LoggedReview, Origin, ReviewEvent};
use batna_core::srs::ReviewSettings;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub use backend::{LectureProgress, Review, use_provide_review, use_review};

#[derive(Debug)]
pub enum StoreError {
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    TooNew(u32),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(e) => write!(f, "database: {e}"),
            Self::Json(e) => write!(f, "json: {e}"),
            Self::TooNew(v) => write!(f, "the database is from a newer Batna (schema {v})"),
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Numbered migrations; the index + 1 is the schema version.
const MIGRATIONS: &[&str] = &[
    "
    CREATE TABLE meta (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    ) STRICT;
    -- Append-only: one row per answer, never updated (a tombstone only for sync).
    CREATE TABLE review_events (
        id          TEXT PRIMARY KEY,
        item_id     TEXT NOT NULL,
        grade       TEXT NOT NULL,
        origin      TEXT NOT NULL,
        at_ms       INTEGER NOT NULL,
        took_ms     INTEGER NOT NULL,
        account_id  TEXT,
        device_id   TEXT NOT NULL,
        hlc_wall    INTEGER NOT NULL,
        hlc_counter INTEGER NOT NULL,
        deleted     INTEGER NOT NULL DEFAULT 0
    ) STRICT;
    CREATE INDEX review_events_item ON review_events (item_id);
    -- Mohamed's verdict per item: unverified (no row, or a tombstone),
    -- approved, edited (with his override) or wrong. Last writer wins.
    CREATE TABLE item_status (
        item_id     TEXT PRIMARY KEY,
        status      TEXT NOT NULL,
        value       TEXT NOT NULL,
        account_id  TEXT,
        device_id   TEXT NOT NULL,
        hlc_wall    INTEGER NOT NULL,
        hlc_counter INTEGER NOT NULL,
        deleted     INTEGER NOT NULL DEFAULT 0
    ) STRICT;
    CREATE TABLE settings (
        key         TEXT PRIMARY KEY,
        value       TEXT NOT NULL,
        account_id  TEXT,
        device_id   TEXT NOT NULL,
        hlc_wall    INTEGER NOT NULL,
        hlc_counter INTEGER NOT NULL,
        deleted     INTEGER NOT NULL DEFAULT 0
    ) STRICT;
",
    "
    -- Local writes waiting for Batna Cloud (store/sync.rs), oldest first.
    CREATE TABLE sync_outbox (
        seq   INTEGER PRIMARY KEY AUTOINCREMENT,
        op_id TEXT NOT NULL UNIQUE,
        op    TEXT NOT NULL
    ) STRICT;
",
    "
    -- Mohamed's own cards (batna_core::own::OwnCard as JSON). Last writer wins.
    CREATE TABLE own_cards (
        id          TEXT PRIMARY KEY,
        value       TEXT NOT NULL,
        account_id  TEXT,
        device_id   TEXT NOT NULL,
        hlc_wall    INTEGER NOT NULL,
        hlc_counter INTEGER NOT NULL,
        deleted     INTEGER NOT NULL DEFAULT 0
    ) STRICT;
",
];

const REVIEW_SETTINGS_KEY: &str = "review";

pub struct Store {
    conn: Mutex<Connection>,
    clock: Mutex<HlcClock>,
    /// Where the database lives (`None` in memory).
    pub path: Option<PathBuf>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Self::init(conn, Some(path.to_path_buf()))
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, None)
    }

    fn init(conn: Connection, path: Option<PathBuf>) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let version: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version as usize > MIGRATIONS.len() {
            return Err(StoreError::TooNew(version));
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", i as u32 + 1)?;
            tx.commit()?;
        }
        let device = match conn
            .query_row("SELECT value FROM meta WHERE key = 'device_id'", [], |r| r.get::<_, String>(0))
            .optional()?
        {
            Some(d) => d,
            None => {
                let d = new_device_id();
                conn.execute("INSERT INTO meta (key, value) VALUES ('device_id', ?1)", [&d])?;
                d
            }
        };
        sync::seed(&conn, &device)?;
        sync::check_tables(&conn)?;
        let mut clock = HlcClock::new(device);
        // Start past every stamp already stored, whichever device wrote it.
        if let Some(last) = newest_stamp(&conn)? {
            clock.observe(&last);
        }
        Ok(Self { conn: Mutex::new(conn), clock: Mutex::new(clock), path })
    }

    /// Moves the clock past rows another connection wrote (the sync
    /// thread), so this device's next write sorts after them.
    pub fn observe_stored(&self) -> Result<()> {
        let last = newest_stamp(&self.conn.lock().unwrap())?;
        if let Some(last) = last {
            self.clock.lock().unwrap().observe(&last);
        }
        Ok(())
    }

    pub fn device(&self) -> String {
        self.clock.lock().unwrap().device().to_string()
    }

    fn tick(&self, now_ms: i64) -> Hlc {
        self.clock.lock().unwrap().tick(now_ms)
    }

    /// Appends one answer and returns it as logged.
    pub fn append_review(&self, event: ReviewEvent) -> Result<LoggedReview> {
        let hlc = self.tick(now_ms().max(event.at_ms));
        let logged =
            LoggedReview { id: format!("{}-{}-{}", hlc.device, hlc.wall_ms, hlc.counter), hlc, event };
        self.insert_review(&logged)?;
        sync::enqueue(&self.conn.lock().unwrap(), "review_events", "id", &logged.id)?;
        Ok(logged)
    }

    /// Inserts a logged event (local or, later, from sync). Duplicates are ignored.
    pub fn insert_review(&self, l: &LoggedReview) -> Result<()> {
        let e = &l.event;
        self.conn.lock().unwrap().execute(
            "INSERT OR IGNORE INTO review_events
             (id, item_id, grade, origin, at_ms, took_ms, device_id, hlc_wall, hlc_counter)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                l.id,
                e.item_id,
                e.grade.as_str(),
                e.origin.as_str(),
                e.at_ms,
                e.took_ms,
                l.hlc.device,
                l.hlc.wall_ms,
                l.hlc.counter
            ],
        )?;
        Ok(())
    }

    /// The whole review log, in HLC order. Rows this build can't read are skipped.
    pub fn reviews(&self) -> Result<Vec<LoggedReview>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, grade, origin, at_ms, took_ms, device_id, hlc_wall, hlc_counter
             FROM review_events WHERE deleted = 0
             ORDER BY hlc_wall, hlc_counter, device_id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, u32>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, u32>(8)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, item_id, grade, origin, at_ms, took_ms, device, wall_ms, counter) = row?;
            let (Some(grade), Some(origin)) = (Grade::parse(&grade), Origin::parse(&origin)) else {
                continue;
            };
            out.push(LoggedReview {
                id,
                hlc: Hlc { wall_ms, counter, device },
                event: ReviewEvent { item_id, grade, origin, at_ms, took_ms },
            });
        }
        Ok(out)
    }

    /// Every current verdict, by item id.
    pub fn judgements(&self) -> Result<BTreeMap<String, Judgement>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT item_id, value FROM item_status WHERE deleted = 0")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = BTreeMap::new();
        for row in rows {
            let (id, value) = row?;
            if let Ok(j) = serde_json::from_str::<Judgement>(&value) {
                out.insert(id, j);
            }
        }
        Ok(out)
    }

    /// Records a verdict, or clears it (`None`, a tombstone: back to unverified).
    pub fn set_judgement(&self, item_id: &str, judgement: Option<&Judgement>) -> Result<()> {
        let hlc = self.tick(now_ms());
        let (status, value, deleted) = match judgement {
            Some(j) => (j.verdict.as_str(), serde_json::to_string(j)?, false),
            None => ("unverified", "{}".to_string(), true),
        };
        self.put_lww("item_status", "item_id", item_id, Some(status), &value, deleted, &hlc)
    }

    /// A synced setting (last writer wins across devices), by key.
    pub fn setting<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM settings WHERE key = ?1 AND deleted = 0", [key], |r| {
            r.get::<_, String>(0)
        })
        .optional()
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str(&v).ok())
    }

    /// Every synced setting whose key starts with `prefix`, in key order.
    /// For records that are one key each, so two devices never overwrite
    /// each other (the bank's finished papers).
    pub fn settings_with_prefix<T: DeserializeOwned>(&self, prefix: &str) -> Vec<(String, T)> {
        let conn = self.conn.lock().unwrap();
        let Ok(mut stmt) = conn.prepare(
            "SELECT key, value FROM settings WHERE substr(key, 1, length(?1)) = ?1 AND deleted = 0 ORDER BY key",
        ) else {
            return Vec::new();
        };
        let rows = stmt.query_map([prefix], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
        rows.map(|rows| {
            rows.flatten().filter_map(|(k, v)| Some((k, serde_json::from_str(&v).ok()?))).collect()
        })
        .unwrap_or_default()
    }

    pub fn put_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let hlc = self.tick(now_ms());
        self.put_lww("settings", "key", key, None, &serde_json::to_string(value)?, false, &hlc)
    }

    /// A setting for this device only (never synced, never imported), kept
    /// in `meta` under `local.<key>`.
    pub fn local_setting<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM meta WHERE key = ?1", [format!("local.{key}")], |r| {
            r.get::<_, String>(0)
        })
        .optional()
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn put_local_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![format!("local.{key}"), serde_json::to_string(value)?],
        )?;
        Ok(())
    }

    pub fn review_settings(&self) -> ReviewSettings {
        self.setting(REVIEW_SETTINGS_KEY).unwrap_or_default()
    }

    pub fn put_review_settings(&self, settings: &ReviewSettings) -> Result<()> {
        self.put_setting(REVIEW_SETTINGS_KEY, settings)
    }

    /// Last-writer-wins upsert: the row changes only when `hlc` is newer
    /// than what is stored.
    #[allow(clippy::too_many_arguments)]
    fn put_lww(
        &self,
        table: &str,
        key_col: &str,
        key: &str,
        status: Option<&str>,
        value: &str,
        deleted: bool,
        hlc: &Hlc,
    ) -> Result<()> {
        let (status_col, status_val, status_set) = match status {
            Some(_) => (", status", ", ?7", ", status = excluded.status"),
            None => ("", "", ""),
        };
        let sql = format!(
            "INSERT INTO {table} ({key_col}, value, device_id, hlc_wall, hlc_counter, deleted{status_col})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6{status_val})
             ON CONFLICT ({key_col}) DO UPDATE SET
                value = excluded.value, device_id = excluded.device_id, hlc_wall = excluded.hlc_wall,
                hlc_counter = excluded.hlc_counter, deleted = excluded.deleted{status_set}
             WHERE (excluded.hlc_wall, excluded.hlc_counter, excluded.device_id)
                 > ({table}.hlc_wall, {table}.hlc_counter, {table}.device_id)"
        );
        let conn = self.conn.lock().unwrap();
        let base = params![key, value, hlc.device, hlc.wall_ms, hlc.counter, deleted];
        let changed = match status {
            Some(s) => {
                let mut all: Vec<&dyn rusqlite::ToSql> = base.to_vec();
                all.push(&s);
                conn.execute(&sql, all.as_slice())?
            }
            None => conn.execute(&sql, base)?,
        };
        // This device's writes go to Batna Cloud too.
        if changed > 0 && hlc.device == self.clock.lock().unwrap().device() {
            sync::enqueue(&conn, table, key_col, key)?;
        }
        Ok(())
    }
}

/// The newest HLC in any synced table.
fn newest_stamp(conn: &Connection) -> Result<Option<Hlc>> {
    let mut newest: Option<Hlc> = None;
    for table in ["review_events", "item_status", "settings"] {
        let last: Option<(i64, u32, String)> = conn
            .query_row(
                &format!(
                    "SELECT hlc_wall, hlc_counter, device_id FROM {table}
                     ORDER BY hlc_wall DESC, hlc_counter DESC LIMIT 1"
                ),
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((wall_ms, counter, device)) = last {
            let h = Hlc { wall_ms, counter, device };
            if newest.as_ref().is_none_or(|n| h > *n) {
                newest = Some(h);
            }
        }
    }
    Ok(newest)
}

/// The app's store, opened once in the data directory. If that fails the
/// app still runs on an in-memory store and says so on Today.
pub fn shared() -> &'static Store {
    STORE.get_or_init(|| {
        let opened = paths::data_dir().and_then(|dir| {
            Store::open(&dir.join("batna.db")).map_err(|e| format!("{}: {e}", dir.display()))
        });
        match opened {
            Ok(s) => s,
            Err(e) => {
                let _ = OPEN_ERROR.set(e);
                Store::open_in_memory().expect("an in-memory database always opens")
            }
        }
    })
}

static STORE: OnceLock<Store> = OnceLock::new();
static OPEN_ERROR: OnceLock<String> = OnceLock::new();

/// Why the store fell back to memory, if it did.
pub fn open_error() -> Option<&'static str> {
    OPEN_ERROR.get().map(String::as_str)
}

pub use paths::export_dir;

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Minutes east of UTC right now (Cairo: 120, or 180 in summer time).
pub fn utc_offset_minutes() -> i32 {
    use chrono::Offset;
    chrono::Local::now().offset().fix().local_minus_utc() / 60
}

/// A random-enough id for this installation, without a crypto dependency:
/// it only has to differ between Mohamed's three devices.
fn new_device_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_i64(now_ms());
    h.write_u32(std::process::id());
    format!("d{:012x}", h.finish() & 0xffff_ffff_ffff)
}

#[cfg(test)]
mod tests {
    use batna_core::review::verify::{Edit, FieldValue, Verdict};
    use batna_core::srs::Retention;

    use super::*;

    fn ev(item: &str, grade: Grade, at_ms: i64) -> ReviewEvent {
        ReviewEvent { item_id: item.into(), grade, origin: Origin::Study, at_ms, took_ms: 3000 }
    }

    fn judgement(verdict: Verdict) -> Judgement {
        Judgement { verdict, edit: Edit::new(), note: None, fingerprint: "f".into(), at_ms: 1 }
    }

    #[test]
    fn settings_by_prefix() {
        let s = Store::open_in_memory().unwrap();
        s.put_setting("bank.paper.a.1", &1).unwrap();
        s.put_setting("bank.paper.b.2", &2).unwrap();
        s.put_setting("bank.papers", &vec![9]).unwrap();
        s.put_setting("other", &3).unwrap();
        let got: Vec<(String, i32)> = s.settings_with_prefix("bank.paper.");
        assert_eq!(got, vec![("bank.paper.a.1".into(), 1), ("bank.paper.b.2".into(), 2)]);
    }

    #[test]
    fn reviews_are_appended_in_order_and_survive_reopening() {
        let dir = std::env::temp_dir().join(format!("batna-store-{}", new_device_id()));
        let path = dir.join("batna.db");
        let device;
        {
            let s = Store::open(&path).unwrap();
            device = s.device();
            s.append_review(ev("hb.L01.001", Grade::Good, 10)).unwrap();
            s.append_review(ev("hb.L01.002", Grade::Again, 10)).unwrap();
        }
        let s = Store::open(&path).unwrap();
        assert_eq!(s.device(), device);
        let log = s.reviews().unwrap();
        assert_eq!(log.len(), 2);
        assert!(log[0].hlc < log[1].hlc);
        assert_eq!(log[1].event.grade, Grade::Again);
        // A re-inserted event (sync echo) is not duplicated.
        s.insert_review(&log[0]).unwrap();
        assert_eq!(s.reviews().unwrap().len(), 2);
        // New stamps sort after the stored ones.
        let next = s.append_review(ev("hb.L01.003", Grade::Good, 0)).unwrap();
        assert!(next.hlc > log[1].hlc);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn verdicts_are_last_writer_wins_and_can_be_undone() {
        let s = Store::open_in_memory().unwrap();
        s.set_judgement("hb.L01.001", Some(&judgement(Verdict::Wrong))).unwrap();
        let mut edited = judgement(Verdict::Edited);
        edited.edit.insert("back".into(), FieldValue::Text("mine".into()));
        s.set_judgement("hb.L01.001", Some(&edited)).unwrap();
        assert_eq!(s.judgements().unwrap()["hb.L01.001"], edited);
        // An older write (a late sync row) loses.
        let old = Hlc { wall_ms: 0, counter: 0, device: "tab".into() };
        s.put_lww("item_status", "item_id", "hb.L01.001", Some("approved"), "{}", false, &old).unwrap();
        assert_eq!(s.judgements().unwrap()["hb.L01.001"].verdict, Verdict::Edited);
        // Undo leaves a tombstone: unverified again.
        s.set_judgement("hb.L01.001", None).unwrap();
        assert!(s.judgements().unwrap().is_empty());
    }

    #[test]
    fn generic_and_device_only_settings() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(s.setting::<u32>("sound"), None);
        s.put_setting("sound", &2u32).unwrap();
        s.put_setting("sound", &3u32).unwrap();
        assert_eq!(s.setting::<u32>("sound"), Some(3));
        s.put_local_setting("rings", &true).unwrap();
        assert_eq!(s.local_setting::<bool>("rings"), Some(true));
        assert_eq!(s.setting::<bool>("rings"), None);
    }

    #[test]
    fn settings_default_then_persist() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(s.review_settings(), ReviewSettings::default());
        let mine = ReviewSettings { retention: Retention::R95, new_per_day: 45, fsrs: None };
        s.put_review_settings(&mine).unwrap();
        assert_eq!(s.review_settings(), mine);
    }
}
