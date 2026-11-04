//! The signed-in session. The refresh token lives in the macOS Keychain or
//! the Android Keystore (through `keyring-core`), and in the web app in the
//! site's own `localStorage` (`platform::web`); never in the database, so it
//! is never in an export either.
// adapted from kabar-app main: crates/app/src/cloud/session.rs

use batna_protocol::cloud::Session;

use super::api::{Api, CloudError};

/// Where the session is kept between launches.
pub trait SessionVault: Send {
    fn load(&self) -> Option<Session>;
    /// False when the session couldn't be kept for the next launch.
    fn save(&self, session: &Session) -> bool;
    fn clear(&self);
    /// False for a session kept only until the app or tab closes.
    fn remembers(&self) -> bool {
        true
    }
}

/// The browser: the site's own storage, which only this origin can read.
/// A sign-in on someone else's computer (`tab_only`) goes in the tab's
/// `sessionStorage` instead, which the browser drops with the tab.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct BrowserVault {
    tab_only: std::sync::atomic::AtomicBool,
}

#[cfg(target_arch = "wasm32")]
impl BrowserVault {
    pub fn tab_only() -> Self {
        Self { tab_only: true.into() }
    }

    fn is_tab_only(&self) -> bool {
        self.tab_only.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(target_arch = "wasm32")]
impl SessionVault for BrowserVault {
    fn load(&self) -> Option<Session> {
        use crate::platform::web::{read_pref, read_tab_pref};
        if let Some(s) = read_pref(BROWSER_KEY).and_then(|j| serde_json::from_str(&j).ok()) {
            return Some(s);
        }
        let s = serde_json::from_str(&read_tab_pref(BROWSER_KEY)?).ok()?;
        // A reload of a tab signed in for this tab only stays that way.
        self.tab_only.store(true, std::sync::atomic::Ordering::Relaxed);
        Some(s)
    }

    fn save(&self, session: &Session) -> bool {
        use crate::platform::web::{write_pref, write_tab_pref};
        let json = serde_json::to_string(session).unwrap_or_default();
        if self.is_tab_only() {
            write_pref(BROWSER_KEY, "");
            write_tab_pref(BROWSER_KEY, &json)
        } else {
            write_pref(BROWSER_KEY, &json)
        }
    }

    fn clear(&self) {
        crate::platform::web::write_pref(BROWSER_KEY, "");
        crate::platform::web::write_tab_pref(BROWSER_KEY, "");
        // The next sign-in decides afresh.
        self.tab_only.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn remembers(&self) -> bool {
        !self.is_tab_only()
    }
}

#[cfg(target_arch = "wasm32")]
const BROWSER_KEY: &str = "cloud-session";

/// Where a sign-in on someone else's device keeps its session: this tab
/// in the browser, memory on the Mac and Android, so it ends when the app
/// or the tab closes.
pub fn not_remembered() -> Box<dyn SessionVault> {
    #[cfg(target_arch = "wasm32")]
    return Box::new(BrowserVault::tab_only());
    #[cfg(not(target_arch = "wasm32"))]
    Box::new(MemoryVault::forgetful())
}

/// The platform credential store.
#[cfg(not(target_arch = "wasm32"))]
pub struct KeyringVault {
    entry: Option<keyring_core::Entry>,
}

#[cfg(not(target_arch = "wasm32"))]
const SERVICE: &str = "app.batna.cloud";
#[cfg(not(target_arch = "wasm32"))]
const USER: &str = "session";

#[cfg(not(target_arch = "wasm32"))]
impl KeyringVault {
    pub fn new() -> Self {
        init_default_store();
        let entry = keyring_core::Entry::new(SERVICE, USER)
            .map_err(|e| crate::diag::report!("no credential store: {e}"))
            .ok();
        Self { entry }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SessionVault for KeyringVault {
    fn load(&self) -> Option<Session> {
        let json = self.entry.as_ref()?.get_password().ok()?;
        serde_json::from_str(&json).ok()
    }

    fn save(&self, session: &Session) -> bool {
        let Some(entry) = &self.entry else { return false };
        let json = serde_json::to_string(session).unwrap_or_default();
        entry.set_password(&json).map_err(|e| crate::diag::report!("saving the session failed: {e}")).is_ok()
    }

    fn clear(&self) {
        if let Some(entry) = &self.entry {
            let _ = entry.delete_credential();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn init_default_store() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    type Store = keyring_core::Result<std::sync::Arc<keyring_core::CredentialStore>>;
    ONCE.call_once(|| {
        #[cfg(target_os = "macos")]
        let store: Store = apple_native_keyring_store::keychain::Store::new().map(|s| s as _);
        #[cfg(target_os = "android")]
        let store: Store = android_native_keyring_store::Store::new().map(|s| s as _);
        // Linux is only for development and CI: an in-memory store, so a
        // session never lands in a plain file.
        #[cfg(not(any(target_os = "macos", target_os = "android")))]
        let store: Store = keyring_core::mock::Store::new().map(|s| s as _);
        match store {
            Ok(store) => keyring_core::set_default_store(store),
            Err(e) => crate::diag::report!("credential store unavailable: {e}"),
        }
    });
}

/// In memory only: tests, and a sign-in on someone else's Mac or phone
/// (`forgetful`), which ends when the app quits.
#[derive(Default)]
pub struct MemoryVault(std::sync::Mutex<Option<Session>>, bool);

impl MemoryVault {
    pub fn forgetful() -> Self {
        Self(Default::default(), true)
    }
}

impl SessionVault for MemoryVault {
    fn remembers(&self) -> bool {
        !self.1
    }

    fn load(&self) -> Option<Session> {
        self.0.lock().ok()?.clone()
    }

    fn save(&self, session: &Session) -> bool {
        self.0.lock().map(|mut s| *s = Some(session.clone())).is_ok()
    }

    fn clear(&self) {
        if let Ok(mut s) = self.0.lock() {
            *s = None;
        }
    }
}

/// Refresh this long before the access token expires.
const REFRESH_MARGIN_MS: u64 = 60_000;

pub struct Auth {
    session: Session,
    vault: Box<dyn SessionVault>,
}

impl Auth {
    pub fn new(session: Session, vault: Box<dyn SessionVault>) -> Result<Self, Box<dyn SessionVault>> {
        if !vault.save(&session) {
            return Err(vault);
        }
        Ok(Self { session, vault })
    }

    pub fn restore(vault: Box<dyn SessionVault>, now_ms: u64) -> Result<Self, Box<dyn SessionVault>> {
        match vault.load() {
            Some(session) if session.refresh_expires_at_ms > now_ms => Ok(Self { session, vault }),
            _ => Err(vault),
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// A valid access token, refreshing (and rotating the refresh token) when
    /// it is about to expire.
    pub async fn access_token(&mut self, api: &Api, now_ms: u64, force: bool) -> Result<String, CloudError> {
        if force || now_ms + REFRESH_MARGIN_MS >= self.session.access_expires_at_ms {
            let fresh = api.refresh(&self.session.refresh_token).await?;
            self.session = if self.vault.save(&fresh) {
                fresh
            } else {
                // The vault still holds the old refresh token, so keep using
                // it: the hub accepts it until the new one is presented (#258),
                // and the next launch, loading the old one, stays signed in.
                Session {
                    refresh_token: std::mem::take(&mut self.session.refresh_token),
                    refresh_expires_at_ms: self.session.refresh_expires_at_ms,
                    ..fresh
                }
            };
        }
        Ok(self.session.access_token.clone())
    }

    /// False when this session ends with the app or the browser tab.
    pub fn remembered(&self) -> bool {
        self.vault.remembers()
    }

    /// Forgets the session on this device. Returns the vault for the next sign-in.
    pub fn forget(self) -> Box<dyn SessionVault> {
        self.vault.clear();
        self.vault
    }
}
