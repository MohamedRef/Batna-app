//! Writes that must land together (engineering audit F09): finishing a
//! lesson stores its mark and clears its saved place in one transaction, so
//! a crash or a full disk can't leave a done lesson that reopens mid-way, or
//! a cleared place with no mark.

use serde::Serialize;

use super::{Result, Store, now_ms};

impl Store {
    /// Stores the synced setting `mark_key` = `mark` and resets the place
    /// `place_key` to `empty`, all or nothing.
    pub fn finish_place<M: Serialize, P: Serialize>(
        &self,
        mark_key: &str,
        mark: &M,
        place_key: &str,
        empty: &P,
    ) -> Result<()> {
        let mark = serde_json::to_string(mark)?;
        let empty = serde_json::to_string(empty)?;
        let mark_hlc = self.tick(now_ms());
        let place_hlc = self.tick(now_ms());
        let mut conn = self.conn.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let tx = conn.transaction()?;
        self.put_lww_in(&tx, "settings", "key", mark_key, None, &mark, false, &mark_hlc)?;
        // A place too big to sync lived in meta; it goes too.
        tx.execute("DELETE FROM meta WHERE key = ?1", [format!("local.{place_key}")])?;
        self.put_lww_in(&tx, "settings", "key", place_key, None, &empty, false, &place_hlc)?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finish_lands_whole_or_not_at_all() {
        let s = Store::open_in_memory().unwrap();
        s.put_place("lessonpos.hb.L01.1", &serde_json::json!({ "at": 3 })).unwrap();
        // The outbox refuses: neither the mark nor the cleared place lands.
        s.conn.lock().unwrap().execute("ALTER TABLE sync_outbox RENAME TO away", []).unwrap();
        let empty = serde_json::json!({});
        assert!(s.finish_place("lessondone.hb.L01.1", &1, "lessonpos.hb.L01.1", &empty).is_err());
        assert_eq!(s.setting::<i32>("lessondone.hb.L01.1"), None);
        assert_eq!(s.place::<serde_json::Value>("lessonpos.hb.L01.1"), Some(serde_json::json!({ "at": 3 })));
        s.conn.lock().unwrap().execute("ALTER TABLE away RENAME TO sync_outbox", []).unwrap();
        s.finish_place("lessondone.hb.L01.1", &1, "lessonpos.hb.L01.1", &empty).unwrap();
        assert_eq!(s.setting::<i32>("lessondone.hb.L01.1"), Some(1));
        assert_eq!(s.place::<serde_json::Value>("lessonpos.hb.L01.1"), Some(empty));
    }
}
