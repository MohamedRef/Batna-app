//! Batna Cloud from the app's side: Google sign-in, live sync and the
//! Devices list. A background thread runs the [`engine::Engine`] on its own
//! tokio runtime and database connection; the UI talks to it through
//! channels and shows it in Settings › Sync (`ui.rs`).
//!
//! Batna Cloud is its own free Cloudflare Worker (`crates/server`): nothing
//! is shared with Kabar Cloud. See docs/CLOUD.md.
// adapted from kabar-app main: crates/app/src/cloud/mod.rs

pub mod api;
pub mod engine;
// The Mac and Android sign in through a loopback listener; the web app
// leaves the page for Google and comes back (docs/WEB.md).
mod held_ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod oauth;
#[cfg(target_arch = "wasm32")]
#[path = "oauth_web.rs"]
pub mod oauth;
pub mod password;
pub mod session;
mod socket;
mod ui;

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod fake;
#[cfg(test)]
mod sync_tests;

use std::collections::BTreeSet;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use batna_protocol::cloud::{DeviceInfo, DeviceSummary, Platform};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[cfg(not(target_arch = "wasm32"))]
pub use self::oauth::GoogleClient;

/// The web app's Google client: a "Web application" one, no secret.
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct GoogleClient {
    pub client_id: String,
}

/// Batna Cloud, for builds that don't say otherwise (the web app).
#[cfg(target_arch = "wasm32")]
const DEFAULT_CLOUD_URL: &str = "https://batna-cloud.hamorefaat85.workers.dev";
#[cfg(test)]
pub use ui::use_unconfigured_cloud;
pub use ui::{SyncSection, ago, use_provide_cloud};

/// Where Batna Cloud is and which Google client signs in. Set at build time
/// from `.env` (`BATNA_CLOUD_URL`, `BATNA_GOOGLE_CLIENT_ID`,
/// `BATNA_GOOGLE_CLIENT_SECRET`); a run-time `BATNA_CLOUD_URL` overrides the
/// URL for local development.
#[derive(Clone, Debug)]
pub struct CloudConfig {
    pub base_url: String,
    pub google: Option<GoogleClient>,
}

impl CloudConfig {
    /// The web app: Batna Cloud's URL and the Web Google client come from the
    /// build (`BATNA_CLOUD_URL`, `BATNA_WEB_GOOGLE_CLIENT_ID`).
    #[cfg(target_arch = "wasm32")]
    pub fn from_env() -> Option<Self> {
        let base_url =
            option_env!("BATNA_CLOUD_URL").filter(|v| !v.trim().is_empty()).unwrap_or(DEFAULT_CLOUD_URL);
        let google = option_env!("BATNA_WEB_GOOGLE_CLIENT_ID")
            .filter(|v| !v.trim().is_empty())
            .or(WEB_GOOGLE_CLIENT_ID)
            .map(|id| GoogleClient { client_id: id.to_string() });
        Some(Self { base_url: base_url.to_string(), google })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_env() -> Option<Self> {
        let built = |name: &str| match name {
            "BATNA_CLOUD_URL" => option_env!("BATNA_CLOUD_URL"),
            "BATNA_GOOGLE_CLIENT_ID" => option_env!("BATNA_GOOGLE_CLIENT_ID"),
            "BATNA_GOOGLE_CLIENT_SECRET" => option_env!("BATNA_GOOGLE_CLIENT_SECRET"),
            _ => None,
        };
        Self::from_vars(|name| std::env::var(name).ok(), built)
    }

    /// A run-time variable wins over the build's; blank means unset.
    #[cfg(not(target_arch = "wasm32"))]
    fn from_vars(
        run: impl Fn(&str) -> Option<String>,
        built: impl Fn(&str) -> Option<&'static str>,
    ) -> Option<Self> {
        let var = |name: &str| run(name).or(built(name).map(String::from)).filter(|v| !v.trim().is_empty());
        let base_url = var("BATNA_CLOUD_URL")?;
        let google = var("BATNA_GOOGLE_CLIENT_ID").map(|client_id| GoogleClient {
            client_id,
            client_secret: var("BATNA_GOOGLE_CLIENT_SECRET"),
            endpoints: Default::default(),
        });
        Some(Self { base_url, google })
    }
}

pub enum Command {
    SignInGoogle,
    /// Username + password, on a device without Google. `remember: false`
    /// (someone else's device) keeps the session only until the app or the
    /// browser tab closes.
    SignInPassword {
        username: String,
        password: String,
        remember: bool,
    },
    /// Read the account's username (`Event::Login`).
    LoadLogin,
    /// Set or change the username and password, from a signed-in device.
    SetLogin {
        username: String,
        password: String,
        /// The current password, when [`LoginStatus::needs_current`].
        current: Option<String>,
    },
    /// Turn the username and password off (`current` as for `SetLogin`).
    RemoveLogin {
        current: Option<String>,
    },
    SignOut,
    LoadDevices,
    RevokeDevice(String),
    /// Offer the held changes to Batna Cloud again.
    SendHeldAgain,
    /// Let the held changes go: they stay here but won't reach the others.
    ForgetHeld,
    /// A local write landed in the outbox: push it now rather than on the
    /// next two-second tick.
    Kick,
    #[allow(dead_code)] // the app runs until the process ends
    Shutdown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Status(CloudStatus),
    Devices(Result<Vec<DeviceSummary>, Problem>),
    /// Ops from another device changed these tables.
    Changed(BTreeSet<String>),
    /// The browser is open for Google sign-in, or a password is being checked.
    SigningIn(bool),
    Login(LoginStatus),
}

/// The account's username and password, as Settings › Sync shows them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoginStatus {
    /// `None` until Batna Cloud has answered.
    pub username: Option<Option<String>>,
    /// A change is on its way.
    pub saving: bool,
    /// The last change went through.
    pub saved: bool,
    pub problem: Option<Problem>,
    /// This device signed in with the password: a change or removal needs
    /// the current one (Batna Cloud checks it).
    pub needs_current: bool,
}

/// Passwords stay out of logs.
impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SignInGoogle => write!(f, "SignInGoogle"),
            Self::SignInPassword { username, remember, .. } => {
                write!(f, "SignInPassword {{ username: {username:?}, remember: {remember} }}")
            }
            Self::LoadLogin => write!(f, "LoadLogin"),
            Self::SetLogin { username, .. } => write!(f, "SetLogin {{ username: {username:?} }}"),
            Self::RemoveLogin { .. } => write!(f, "RemoveLogin"),
            Self::SignOut => write!(f, "SignOut"),
            Self::LoadDevices => write!(f, "LoadDevices"),
            Self::RevokeDevice(d) => write!(f, "RevokeDevice({d:?})"),
            Self::SendHeldAgain => write!(f, "SendHeldAgain"),
            Self::ForgetHeld => write!(f, "ForgetHeld"),
            Self::Kick => write!(f, "Kick"),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncState {
    SignedOut,
    Connecting,
    CatchingUp,
    Live,
    /// Waiting to reconnect.
    Offline,
    /// Needs an app update or a new sign-in.
    Stopped,
}

/// Something Mohamed should hear about, in Arabic (`ui.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    NotConfigured,
    NotAllowed,
    SignInCancelled,
    SignInFailed,
    SignedOutRemotely,
    Network,
    /// Google approved the sign-in; the network half is waiting to retry.
    SignInWaiting,
    Server,
    Outdated,
    /// The hub turned changes away because this device's clock is ahead.
    ClockOff,
    /// Username and password don't match.
    WrongPassword,
    /// The current password given for a change is wrong.
    CurrentWrong,
    /// Too many wrong passwords: the next try counts in this many minutes.
    Locked(u64),
    UsernameTaken,
    /// The username isn't 3–32 of a-z 0-9 . _ - (checked before sending).
    BadUsername,
    /// The new password is shorter than `MIN_PASSWORD_CHARS`.
    ShortPassword,
    /// The two new passwords differ (Settings only).
    Mismatch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CloudStatus {
    /// This build knows a Batna Cloud URL.
    pub configured: bool,
    pub email: Option<String>,
    pub device_id: String,
    pub sync: SyncState,
    /// Local changes not yet acked.
    pub pending: u64,
    /// Changes Batna Cloud refused for good, or this build couldn't read,
    /// kept on the device (`store/held.rs`).
    pub held: u64,
    pub last_sync_ms: Option<u64>,
    pub problem: Option<Problem>,
    /// Google approved a sign-in that is still reaching the server.
    pub finishing_sign_in: bool,
    /// Signed in on someone else's device: the session ends with the app or tab.
    pub temporary: bool,
}

impl CloudStatus {
    pub fn not_configured(device_id: String) -> Self {
        Self {
            configured: false,
            email: None,
            device_id,
            sync: SyncState::SignedOut,
            pending: 0,
            held: 0,
            last_sync_ms: None,
            problem: None,
            finishing_sign_in: false,
            temporary: false,
        }
    }
}

/// The web app's own Google client id ("Batna web", public, not a secret;
/// docs/WEB.md). `BATNA_WEB_GOOGLE_CLIENT_ID` overrides it.
#[cfg(target_arch = "wasm32")]
const WEB_GOOGLE_CLIENT_ID: Option<&str> =
    Some("1001052243528-f6b44kj27r95v7qc49vkslfv7v4oau9l.apps.googleusercontent.com");

pub struct CloudHandle {
    pub commands: UnboundedSender<Command>,
    pub events: UnboundedReceiver<Event>,
}

/// Starts the engine on its own thread. `db_path` is opened a second time
/// (SQLite WAL), so the UI's connection never waits on the network.
#[cfg(not(target_arch = "wasm32"))]
pub fn start(
    config: CloudConfig,
    db_path: PathBuf,
    device: DeviceInfo,
    vault: Box<dyn session::SessionVault>,
) -> Result<CloudHandle, String> {
    // Lives as long as the app (the sync thread never ends).
    let store: &'static crate::store::Store =
        Box::leak(Box::new(crate::store::Store::open(&db_path).map_err(|e| e.to_string())?));
    // Off the UI thread, sync can wait longer for the UI's lock.
    store.set_busy_timeout(crate::store::SYNC_BUSY).map_err(|e| e.to_string())?;
    let (cmd_tx, cmd_rx) = unbounded_channel();
    let (ev_tx, ev_rx) = unbounded_channel();
    std::thread::Builder::new()
        .name("batna-cloud".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(r) => r,
                Err(e) => {
                    crate::diag::report!("no runtime for sync: {e}");
                    return;
                }
            };
            let api = api::Api::new(&config.base_url);
            let engine = engine::Engine::new(store, api, config.google, device, vault, ev_tx);
            runtime.block_on(engine.run(cmd_rx));
        })
        .map_err(|e| e.to_string())?;
    Ok(CloudHandle { commands: cmd_tx, events: ev_rx })
}

/// The web app: the engine runs on the page's own event loop, on the store
/// the screens use (the browser has one thread and one database).
#[cfg(target_arch = "wasm32")]
pub fn start(config: CloudConfig, device: DeviceInfo) -> CloudHandle {
    let (cmd_tx, cmd_rx) = unbounded_channel();
    let (ev_tx, ev_rx) = unbounded_channel();
    let api = api::Api::new(&config.base_url);
    let vault = Box::new(session::BrowserVault::default());
    let engine = engine::Engine::new(crate::store::shared(), api, config.google, device, vault, ev_tx);
    wasm_bindgen_futures::spawn_local(engine.run(cmd_rx));
    CloudHandle { commands: cmd_tx, events: ev_rx }
}

/// What this device calls itself on the Devices list.
pub fn device_info(device_id: String) -> DeviceInfo {
    let model = device_model();
    let (name, platform) = if cfg!(target_os = "android") {
        (model.clone().unwrap_or_else(|| "Android".into()), Platform::Android)
    } else if cfg!(target_os = "macos") {
        (mac_name().unwrap_or_else(|| "Mac".into()), Platform::Macos)
    } else if cfg!(target_arch = "wasm32") {
        (browser_name(), Platform::Web)
    } else {
        ("Batna (dev)".into(), Platform::Other)
    };
    DeviceInfo { device_id, name, model, platform }
}

#[cfg(target_arch = "wasm32")]
fn browser_name() -> String {
    crate::platform::web::browser_name()
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_name() -> String {
    "Browser".into()
}

fn mac_name() -> Option<String> {
    let out = std::process::Command::new("scutil").args(["--get", "ComputerName"]).output().ok()?;
    let name = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!name.is_empty()).then_some(name)
}

#[cfg(target_os = "android")]
fn device_model() -> Option<String> {
    use jni::{jni_sig, jni_str};
    crate::platform::tts_android::with_activity(|env, _| {
        let model = env
            .get_static_field(
                jni_str!("android/os/Build"),
                jni_str!("MODEL"),
                jni_sig!("Ljava/lang/String;"),
            )?
            .l()?;
        let model = env.cast_local::<jni::objects::JString>(model)?;
        model.try_to_string(env)
    })
    .ok()
}

#[cfg(not(target_os = "android"))]
fn device_model() -> Option<String> {
    None
}

/// Opens `url` in the default browser (Google sign-in).
pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        android_open_url(url)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or("no window")?;
        window.open_with_url_and_target(url, "_blank").map(drop).map_err(|e| format!("{e:?}"))
    }
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        std::process::Command::new(opener).arg(url).spawn().map(drop).map_err(|e| format!("{opener}: {e}"))
    }
}

// adapted from kabar-app main: crates/app/src/platform/android.rs (open_url)
#[cfg(target_os = "android")]
fn android_open_url(url: &str) -> Result<(), String> {
    use jni::{jni_sig, jni_str};
    crate::platform::tts_android::with_activity(|env, activity| {
        let action = env
            .get_static_field(
                jni_str!("android/content/Intent"),
                jni_str!("ACTION_VIEW"),
                jni_sig!("Ljava/lang/String;"),
            )?
            .l()?;
        let url = env.new_string(url)?;
        let uri = env
            .call_static_method(
                jni_str!("android/net/Uri"),
                jni_str!("parse"),
                jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
                &[(&url).into()],
            )?
            .l()?;
        let intent = env.new_object(
            jni_str!("android/content/Intent"),
            jni_sig!("(Ljava/lang/String;Landroid/net/Uri;)V"),
            &[(&action).into(), (&uri).into()],
        )?;
        // FLAG_ACTIVITY_NEW_TASK
        env.call_method(
            &intent,
            jni_str!("addFlags"),
            jni_sig!("(I)Landroid/content/Intent;"),
            &[0x1000_0000.into()],
        )?;
        env.call_method(
            activity,
            jni_str!("startActivity"),
            jni_sig!("(Landroid/content/Intent;)V"),
            &[(&intent).into()],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cloud_comes_from_the_build_unless_the_run_says_otherwise() {
        let built = |name: &str| match name {
            "BATNA_CLOUD_URL" => Some("https://batna-cloud.example.workers.dev"),
            "BATNA_GOOGLE_CLIENT_ID" => Some("desk.apps.googleusercontent.com"),
            _ => None,
        };
        let c = CloudConfig::from_vars(|_| None, built).unwrap();
        assert_eq!(c.base_url, "https://batna-cloud.example.workers.dev");
        let g = c.google.unwrap();
        assert_eq!(g.client_id, "desk.apps.googleusercontent.com");
        assert_eq!(g.client_secret, None);
        assert!(g.endpoints.auth.starts_with("https://accounts.google.com/"));

        // A local server for development, and a blank client id means no Google.
        let run = |name: &str| match name {
            "BATNA_CLOUD_URL" => Some("http://127.0.0.1:8787".to_string()),
            "BATNA_GOOGLE_CLIENT_ID" => Some("  ".to_string()),
            _ => None,
        };
        let c = CloudConfig::from_vars(run, built).unwrap();
        assert_eq!(c.base_url, "http://127.0.0.1:8787");
        assert!(c.google.is_none());

        // No URL anywhere: this build has no Batna Cloud.
        assert!(CloudConfig::from_vars(|_| None, |_| None).is_none());
    }

    #[test]
    fn only_a_mac_reads_its_computer_name() {
        // scutil exists only on macOS; elsewhere the name falls back.
        assert_eq!(mac_name().is_some(), cfg!(target_os = "macos"));
        assert_eq!(browser_name(), "Browser");
    }

    #[test]
    fn a_new_device_starts_signed_out_and_unconfigured() {
        let s = CloudStatus::not_configured("dev-9".into());
        assert!(!s.configured);
        assert_eq!((s.sync, s.pending, s.problem), (SyncState::SignedOut, 0, None));
        assert_eq!(s.device_id, "dev-9");
    }
}
