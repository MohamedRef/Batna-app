//! Sync ops that can't go up, kept instead of dropped (audit F12, F13).
//!
//! Batna Cloud turns some ops away for good (a table it doesn't know, a
//! payload too large, …), and an outbox row this build can't read can never
//! be sent. Both used to vanish from the outbox with only a diagnostics
//! line. Now they move to `sync_held` with their reason, Settings › Sync
//! says how many there are, and Mohamed can send them again (after an
//! update, say) or let them go.

use batna_protocol::sync::{Op, Rejected};
use rusqlite::{Connection, params};

use super::{Result, Store, now_ms};

/// The reason stored for an outbox row this build couldn't read.
const UNREADABLE: &str = "unreadable";
/// The newest this many stay; older ones go (and say so in diagnostics).
const KEEP: i64 = 1000;

/// Moves one outbox op to `sync_held`.
fn hold(conn: &Connection, op_id: &str, reason: &str, at_ms: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_held (op_id, op, reason, at_ms)
         SELECT op_id, op, ?2, ?3 FROM sync_outbox WHERE op_id = ?1",
        params![op_id, reason, at_ms],
    )?;
    conn.execute("DELETE FROM sync_outbox WHERE op_id = ?1", [op_id])?;
    Ok(())
}

fn trim(conn: &Connection) -> rusqlite::Result<()> {
    let gone = conn.execute(
        "DELETE FROM sync_held WHERE op_id NOT IN
           (SELECT op_id FROM sync_held ORDER BY at_ms DESC, op_id LIMIT ?1)",
        [KEEP],
    )?;
    if gone > 0 {
        crate::diag::report!("sync: {gone} old held ops dropped (over {KEEP})");
    }
    Ok(())
}

/// The reason as stored: the protocol's snake_case name.
fn reason_name(r: &Rejected) -> String {
    serde_json::to_value(r.reason).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default()
}

impl Store {
    /// The oldest `limit` ops waiting for the hub. A row this build can't
    /// read moves to `sync_held` rather than blocking or vanishing.
    pub fn outbox(&self, limit: usize) -> Result<Vec<Op>> {
        let mut conn = self.conn.lock().unwrap();
        let rows: Vec<(String, String)> = {
            let mut stmt = conn.prepare("SELECT op_id, op FROM sync_outbox ORDER BY seq LIMIT ?1")?;
            stmt.query_map([limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?
        };
        let mut ops = Vec::with_capacity(rows.len());
        let mut bad = Vec::new();
        for (id, json) in rows {
            match serde_json::from_str::<Op>(&json) {
                Ok(op) => ops.push(op),
                Err(e) => {
                    crate::diag::report!("sync: outbox op {id} unreadable, held: {e}");
                    bad.push(id);
                }
            }
        }
        if !bad.is_empty() {
            let tx = conn.transaction()?;
            let at = now_ms();
            for id in &bad {
                hold(&tx, id, UNREADABLE, at)?;
            }
            trim(&tx)?;
            tx.commit()?;
        }
        Ok(ops)
    }

    /// Settles a push in one transaction: acked ops leave the outbox, ops
    /// the hub refused for good move to `sync_held` with their reason.
    pub fn settle_outbox(&self, done: &[String], rejected: &[Rejected]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for id in done {
            tx.execute("DELETE FROM sync_outbox WHERE op_id = ?1", [id])?;
        }
        let at = now_ms();
        for r in rejected {
            hold(&tx, &r.op_id, &reason_name(r), at)?;
        }
        if !rejected.is_empty() {
            trim(&tx)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// How many ops are held back.
    pub fn held_len(&self) -> u64 {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM sync_held", [], |r| r.get::<_, i64>(0)).unwrap_or(0) as u64
    }

    /// Puts the held ops back at the end of the outbox, to be offered again
    /// (a newer build may read them, or the hub may take them now). Rows
    /// still unreadable stay held. Returns how many went back.
    pub fn send_held_again(&self) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let rows: Vec<(String, String)> = {
            let mut stmt = tx.prepare("SELECT op_id, op FROM sync_held ORDER BY at_ms, op_id")?;
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<_>>()?
        };
        let mut moved = 0;
        for (id, json) in rows {
            if serde_json::from_str::<Op>(&json).is_err() {
                continue;
            }
            tx.execute("INSERT OR IGNORE INTO sync_outbox (op_id, op) VALUES (?1, ?2)", params![id, json])?;
            tx.execute("DELETE FROM sync_held WHERE op_id = ?1", [&id])?;
            moved += 1;
        }
        tx.commit()?;
        Ok(moved)
    }

    /// Lets every held op go: the changes stay on this device, they just
    /// won't reach the others.
    pub fn forget_held(&self) -> Result<()> {
        self.conn.lock().unwrap().execute("DELETE FROM sync_held", [])?;
        Ok(())
    }

    /// The reason one op was held, if it was (tests and diagnostics).
    #[cfg(test)]
    pub fn held_reason(&self, op_id: &str) -> Option<String> {
        use rusqlite::OptionalExtension;
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT reason FROM sync_held WHERE op_id = ?1", [op_id], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use batna_core::review::{Grade, Origin, ReviewEvent};
    use batna_protocol::sync::RejectReason;
    use rusqlite::OptionalExtension;

    use super::*;

    fn store_with_two() -> (Store, Vec<Op>) {
        let s = Store::open_in_memory().unwrap();
        for at in [1, 2] {
            s.append_review(ReviewEvent::new("hb.L01.003", Grade::Good, Origin::Study, at, 3000)).unwrap();
        }
        let ops = s.outbox(10).unwrap();
        assert_eq!(ops.len(), 2);
        (s, ops)
    }

    #[test]
    fn a_refused_op_is_held_with_its_reason_not_dropped() {
        let (s, ops) = store_with_two();
        let refused = Rejected { op_id: ops[1].id.clone(), reason: RejectReason::PayloadTooLarge };
        s.settle_outbox(&[ops[0].id.clone()], &[refused]).unwrap();
        assert_eq!(s.outbox_len(), 0);
        assert_eq!(s.held_len(), 1);
        assert_eq!(s.held_reason(&ops[1].id).as_deref(), Some("payload_too_large"));

        // Sent again: back in the outbox, as it was.
        assert_eq!(s.send_held_again().unwrap(), 1);
        assert_eq!(s.held_len(), 0);
        assert_eq!(s.outbox(10).unwrap(), vec![ops[1].clone()]);
    }

    #[test]
    fn an_unreadable_outbox_row_is_held_and_the_rest_still_go() {
        let (s, ops) = store_with_two();
        s.conn
            .lock()
            .unwrap()
            .execute("UPDATE sync_outbox SET op = '{not json' WHERE op_id = ?1", [&ops[0].id])
            .unwrap();
        assert_eq!(s.outbox(10).unwrap(), vec![ops[1].clone()]);
        assert_eq!(s.held_reason(&ops[0].id).as_deref(), Some(UNREADABLE));
        assert_eq!(s.outbox_len(), 1);
        // Still unreadable: sending again leaves it held.
        assert_eq!(s.send_held_again().unwrap(), 0);
        assert_eq!(s.held_len(), 1);
        s.forget_held().unwrap();
        assert_eq!(s.held_len(), 0);
    }

    #[test]
    fn only_the_newest_held_ops_are_kept() {
        let s = Store::open_in_memory().unwrap();
        let conn = s.conn.lock().unwrap();
        for i in 0..KEEP + 5 {
            conn.execute(
                "INSERT INTO sync_held (op_id, op, reason, at_ms) VALUES (?1, '{}', 'x', ?2)",
                params![format!("op{i}"), i],
            )
            .unwrap();
        }
        trim(&conn).unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM sync_held", [], |r| r.get(0)).unwrap();
        assert_eq!(n, KEEP);
        let oldest: Option<String> = conn
            .query_row("SELECT op_id FROM sync_held WHERE op_id = 'op0'", [], |r| r.get(0))
            .optional()
            .unwrap();
        assert!(oldest.is_none());
    }
}
