//! The reset markers' storage (`batna_core::reset`, #1341): one synced
//! setting per scope, written with the marker's own stamp so the marker and
//! its row agree, and removed with a tombstone for undo.

use batna_core::reset::{Marker, PREFIX, Resets, Scope};

use super::{Result, Store, now_ms};

impl Store {
    /// Every reset in force (tombstoned markers are gone).
    pub fn resets(&self) -> Resets {
        Resets::from_settings(self.settings_with_prefix::<Marker>(PREFIX))
    }

    /// Writes a reset of `scope` stamped now; `fill` adds what the marker
    /// carries besides its stamp. Returns the marker as stored.
    pub fn put_reset(&self, scope: &Scope, fill: impl FnOnce(&mut Marker)) -> Result<Marker> {
        let hlc = self.tick(now_ms());
        let mut marker = Marker::new(hlc.clone());
        fill(&mut marker);
        let value = serde_json::to_string(&marker)?;
        self.put_lww("settings", "key", &scope.key(), None, &value, false, &hlc)?;
        Ok(marker)
    }

    /// Takes a reset back: its setting gets a tombstone on every device.
    pub fn remove_reset(&self, scope: &Scope) -> Result<()> {
        self.delete_setting(&scope.key())
    }

    /// A tombstone on a synced setting: it reads as never set, everywhere.
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let hlc = self.tick(now_ms());
        self.put_lww("settings", "key", key, None, "null", true, &hlc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reset_is_one_synced_setting_and_undo_tombstones_it() {
        let s = Store::open_in_memory().unwrap();
        let before = s.outbox_len();
        let m = s.put_reset(&Scope::Course("hb".into()), |m| m.papers = false).unwrap();
        assert_eq!(s.resets().courses.get("hb"), Some(&m));
        assert_eq!(s.outbox_len(), before + 1, "one op");
        s.remove_reset(&Scope::Course("hb".into())).unwrap();
        assert!(s.resets().is_empty());
        // Resetting again after an undo works: a newer stamp beats the tombstone.
        let again = s.put_reset(&Scope::Course("hb".into()), |_| {}).unwrap();
        assert!(again.hlc > m.hlc);
        assert_eq!(s.resets().courses.get("hb"), Some(&again));
    }
}
