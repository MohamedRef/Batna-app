//! The web build (`web/`, wasm32): what the browser stands in for.
//!
//! - **The database.** SQLite runs in wasm, in memory. Its bytes are kept in
//!   IndexedDB (`batna` › `files` › `batna.db`): loaded once before the app
//!   starts, and written back a moment after anything changes, and when the
//!   tab is hidden or closed. See docs/WEB.md.
//! - **One tab writes.** The newest tab owns the database; an older one stops
//!   saving and says so, so two tabs never overwrite each other's work.
//! - **Device prefs** live in `localStorage` (`prefs/storage.rs`).
//! - **Exports** are browser downloads.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
const DB = "batna", STORE = "files";
function open() {
  return new Promise((ok, fail) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE);
    req.onsuccess = () => ok(req.result);
    req.onerror = () => fail(req.error);
  });
}
export async function idb_get(key) {
  const db = await open();
  return await new Promise((ok, fail) => {
    const req = db.transaction(STORE, "readonly").objectStore(STORE).get(key);
    req.onsuccess = () => ok(req.result instanceof Uint8Array ? req.result : null);
    req.onerror = () => fail(req.error);
  });
}
export async function idb_put(key, bytes) {
  const db = await open();
  await new Promise((ok, fail) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(bytes, key);
    tx.oncomplete = () => ok();
    tx.onerror = () => fail(tx.error);
    tx.onabort = () => fail(tx.error);
  });
}
export function persist() {
  if (navigator.storage && navigator.storage.persist) navigator.storage.persist();
}
export function download(name, mime, bytes) {
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  const a = document.createElement("a");
  a.href = url; a.download = name; a.style.display = "none";
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 60000);
}
export function ls_get(key) { try { return localStorage.getItem(key); } catch (_) { return null; } }
export function ls_set(key, value) {
  try { localStorage.setItem(key, value); return true; } catch (_) { return false; }
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn idb_get(key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn idb_put(key: &str, bytes: &[u8]) -> Result<JsValue, JsValue>;
    fn persist();
    #[wasm_bindgen(js_name = download)]
    fn js_download(name: &str, mime: &str, bytes: &[u8]);
    fn ls_get(key: &str) -> Option<String>;
    fn ls_set(key: &str, value: &str) -> bool;
}

const DB_KEY: &str = "batna.db";
/// Which tab owns the database (`localStorage`).
const OWNER_KEY: &str = "batna:tab";

thread_local! {
    static SNAPSHOT: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    static SAVING: Cell<bool> = const { Cell::new(true) };
    /// SQLite's change count at the last save.
    static SAVED_AT: Cell<Option<u64>> = const { Cell::new(None) };
    static TAB: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Before the app starts: reads the kept database and claims this tab.
/// A browser that can't read IndexedDB starts empty and doesn't save, so
/// it never overwrites what it couldn't read.
pub async fn boot() {
    persist();
    let tab = format!("{:x}", js_sys::Date::now() as u64 ^ (js_sys::Math::random() * 1e12) as u64);
    ls_set(OWNER_KEY, &tab);
    TAB.with(|t| *t.borrow_mut() = tab);
    match idb_get(DB_KEY).await {
        Ok(v) if v.is_null() || v.is_undefined() => {}
        Ok(v) => SNAPSHOT.with(|s| *s.borrow_mut() = Some(js_sys::Uint8Array::new(&v).to_vec())),
        Err(e) => {
            stop_saving();
            crate::store::write_failed("browser storage", &format!("{e:?}"));
        }
    }
}

/// The database bytes `boot` found (empty for a first visit). Once.
pub fn take_snapshot() -> Vec<u8> {
    SNAPSHOT.with(|s| s.borrow_mut().take()).unwrap_or_default()
}

pub fn stop_saving() {
    SAVING.with(|s| s.set(false));
}

fn owns_tab() -> bool {
    TAB.with(|t| ls_get(OWNER_KEY).is_none_or(|owner| owner == *t.borrow()))
}

/// Writes the database to IndexedDB when it changed since the last save.
pub async fn save_now() {
    if !SAVING.with(Cell::get) {
        return;
    }
    if !owns_tab() {
        stop_saving();
        crate::store::write_failed("browser storage", &OTHER_TAB);
        return;
    }
    let store = crate::store::shared();
    if SAVED_AT.with(Cell::get) == Some(store.changes()) {
        return;
    }
    match store.snapshot() {
        Ok((changes, bytes)) => match idb_put(DB_KEY, &bytes).await {
            Ok(_) => {
                SAVED_AT.with(|s| s.set(Some(changes)));
                crate::store::write_recovered("browser storage");
            }
            Err(e) => crate::store::write_failed("browser storage", &format!("{e:?}")),
        },
        Err(e) => crate::store::write_failed("browser storage", &e),
    }
}

/// Shown on Today when another tab took over.
const OTHER_TAB: &str = "Batna is open in another tab; reload this one to keep studying here";

/// Keeps the database saved: shortly after changes, and when the tab is
/// hidden or closed. Call once in `App`.
pub fn use_saver() {
    use_future(|| async {
        // The first visit saves at once, so the device id sticks.
        save_now().await;
        loop {
            crate::platform::time::sleep(Duration::from_millis(1500)).await;
            save_now().await;
        }
    });
    use_future(|| async {
        let mut eval = document::eval(
            r#"
            const send = () => dioxus.send(true);
            document.addEventListener("visibilitychange", () => { if (document.hidden) send(); });
            window.addEventListener("pagehide", send);
            await new Promise(() => {});
            "#,
        );
        while eval.recv::<bool>().await.is_ok() {
            save_now().await;
        }
    });
}

/// Hands `bytes` to the browser as a download called `name`.
pub fn download(name: &str, mime: &str, bytes: &[u8]) -> Result<String, String> {
    js_download(name, mime, bytes);
    Ok(name.to_string())
}

/// Device prefs (`prefs/storage.rs`).
pub fn read_pref(file: &str) -> Option<String> {
    ls_get(&format!("batna:{file}"))
}

pub fn write_pref(file: &str, text: &str) -> bool {
    ls_set(&format!("batna:{file}"), text)
}
