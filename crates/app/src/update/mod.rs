//! Updates over the air (`docs/RELEASE.md` §5): Batna checks its feed on
//! GitHub Releases, verifies the feed's Ed25519 signature and each
//! download's SHA-256, then installs it: PackageInstaller on Android, a
//! swapped Batna.app on the Mac. The content pack rides the same feed, so
//! new lectures arrive even while app updates wait or are off.
//!
//! Device-local, never synced:
//!
//! ```text
//! <data>/updates/prefs.json      automatic / tell me first / off, channel, last check,
//!                                the last "new version" notification sent
//! <data>/updates/staged.json     the verified app download waiting to install
//! <data>/updates/<file>          that download (one at a time)
//! <data>/updates/content.pack    the newest verified content pack
//! <data>/updates/content.json    which feed it came from (version, build, SHA-256)
//! <data>/updates/history.json    what each build changed (changelog.rs)
//! ```
//!
//! The work runs on its own thread; the UI follows [`Status`] through a
//! `tokio::sync::watch` channel.
//!
//! A new version is also announced with a system notification ([`Notice`]):
//! by the running app's own checks, and on Android by a periodic background
//! job (`app.batna.UpdateCheckJob`) that runs [`background_check`] even while
//! Batna is closed.
//!
//! Adapted from kabar-app main: crates/app/src/update/mod.rs. Changes for
//! Batna: `BATNA_*` build variables, the content pack (in place of Kabar's
//! offline packs), and one language.

pub mod bundle;
pub mod changelog;
pub mod sheet;
pub mod view;

use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use batna_core::pack::Pack;
use batna_core::update::{
    BuildChanges, Channel, Decision, Installed, MANIFEST_FILE, PackOrigin, SIGNATURE_FILE, Stage,
    UpdateAsset, UpdateError, UpdateManifest, UpdateNotes, UpdatePlatform, VerifyingKey, Version,
    check_download, content_update, decide, parse_public_key, published_secs, this_platform, verify_manifest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::watch;

use crate::diag::report;

/// The release public key (`just update-keygen` fills it once).
const RELEASE_KEY: &str = include_str!("../../update-key.pub");
/// How often a running Batna looks for an update by itself.
const CHECK_EVERY: Duration = Duration::from_secs(6 * 3600);
/// First automatic check after launch: late enough not to slow the start.
const FIRST_CHECK_AFTER: Duration = Duration::from_secs(20);
/// Android's background job runs about every four hours
/// (`UpdateNotice.PERIOD_MS`); it looks at the feed when the last look, its
/// own or the app's, is at least this old.
const BACKGROUND_EVERY: Duration = Duration::from_secs(3 * 3600);
/// `update.json` and its signature are small; anything bigger is refused.
const MAX_FEED_BYTES: u64 = 256 * 1024;
/// Largest build accepted (the APK is a few tens of MB, Batna.app less).
const MAX_BUILD_BYTES: u64 = 400 * 1024 * 1024;
/// Largest content pack accepted (the whole chapter packs to a few MB).
const MAX_PACK_BYTES: u64 = 64 * 1024 * 1024;

const PACK_FILE: &str = "content.pack";
const PACK_INFO: &str = "content.json";

/// What Mohamed chose in Settings › Updates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Check and download by itself; install when it's calm (Mac: next
    /// launch; Android: while Batna is in the background and may update
    /// quietly, otherwise one tap).
    #[default]
    Auto,
    /// Check by itself and say so; download and install on a tap.
    Notify,
    /// Only when "Check now" is pressed.
    Off,
}

impl Mode {
    pub const ALL: [Self; 3] = [Self::Auto, Self::Notify, Self::Off];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default)]
    pub mode: Mode,
    /// Release builds only (a preview build always follows `preview`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_check_ms: Option<u64>,
    /// The banner for this offer (`version+build`) was closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<String>,
    /// An automatic install of this download failed once; wait for a tap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_failed: Option<String>,
    /// Announce a new version with a system notification (on by default).
    #[serde(default = "yes")]
    pub notify: bool,
    /// The last notification sent (`version+build:stage`), so each goes out once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notified: Option<String>,
    /// The build (`version+build`) that ran last, to notice an update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<String>,
    /// Set at the first launch after an update to the build that ran
    /// before: "What's new" opens once for what came since, then clears it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whats_new_from: Option<String>,
}

fn yes() -> bool {
    true
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            channel: None,
            last_check_ms: None,
            dismissed: None,
            auto_failed: None,
            notify: true,
            notified: None,
            last_run: None,
            whats_new_from: None,
        }
    }
}

/// The "new version" notification, as `app.batna.UpdateNotice.show` reads
/// it (JSON).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Notice {
    /// `version+build:stage`: remembered once shown. Android's job hands it
    /// back through `UpdateCheckJob.shown` after the notification is up.
    pub key: String,
    pub title: String,
    pub body: String,
    /// What the notification's button does in the app: `download` or
    /// `install`; empty for no button.
    pub action: &'static str,
    pub button: &'static str,
    /// The Android notification channel's name.
    pub channel: &'static str,
}

/// What's new, Egyptian Arabic first.
pub(crate) fn notes_of(o: &Offer) -> String {
    [&o.notes.ar_eg, &o.notes.en]
        .into_iter()
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
        .unwrap_or_default()
        .to_owned()
}

/// The notification due for `s`, if any: one per offer and step (out, then
/// downloaded and ready), never for an offer whose banner was closed, and
/// nothing with the switch off or in "only when I check" mode.
pub fn notice(s: &Status) -> Option<Notice> {
    use crate::i18n::update as tx;
    if s.off.is_some() || s.prefs.mode == Mode::Off || !s.prefs.notify {
        return None;
    }
    let (offer, stage, title, hint, action, button) = match &s.phase {
        Phase::Available(o) => {
            (o, "available", tx::NOTICE_AVAILABLE, tx::NOTICE_TAP_AVAILABLE, "download", tx::DOWNLOAD)
        }
        Phase::Ready(o) | Phase::NeedsPermission(o) => {
            (o, "ready", tx::NOTICE_READY, tx::NOTICE_TAP_READY, "install", tx::INSTALL_NOW)
        }
        Phase::Installed(o) => (o, "installed", tx::NOTICE_INSTALLED, tx::NEXT_LAUNCH, "", ""),
        _ => return None,
    };
    if s.prefs.dismissed.as_deref() == Some(offer.key().as_str()) {
        return None;
    }
    let key = format!("{}:{stage}", offer.key());
    if s.prefs.notified.as_deref() == Some(key.as_str()) {
        return None;
    }
    let mut whats_new = if offer.changes.is_empty() {
        notes_of(offer)
    } else {
        changelog::lines_upto(&offer.changes, tx::NOTICE_MORE, 3)
    };
    if whats_new.chars().count() > 300 {
        whats_new = whats_new.chars().take(299).collect::<String>() + "…";
    }
    let body = if whats_new.is_empty() {
        hint.to_owned()
    } else if offer.changes.is_empty() {
        format!("{}: {whats_new}\n{hint}", tx::WHATS_NEW)
    } else {
        format!("{whats_new}\n{hint}")
    };
    Some(Notice {
        key,
        title: format!("{title} · {}", offer.label()),
        body,
        action,
        button,
        channel: tx::NOTICE_CHANNEL,
    })
}

/// Why this build doesn't update itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Off {
    /// A development build (`dx serve`, `cargo run`).
    DevBuild,
    /// No update key yet: `just update-keygen` hasn't run.
    NoKey,
    /// No builds are published for this system (Linux, x86 Android).
    NoPlatform,
    /// The Mac app runs from a build folder, not from a Batna.app.
    NotInBundle,
}

/// A newer build, from a verified manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub version: String,
    pub build: u64,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub notes: UpdateNotes,
    pub asset: UpdateAsset,
    /// What it brings: the feed's builds after the installed one, newest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<BuildChanges>,
}

impl Offer {
    fn from_manifest(m: &UpdateManifest, asset: UpdateAsset, me: &Installed) -> Self {
        let changes = match changelog::order(&m.version, m.build) {
            Some(to) => changelog::between(&m.history, Some((me.version, me.build)), to)
                .into_iter()
                .cloned()
                .collect(),
            None => Vec::new(),
        };
        Self {
            version: m.version.clone(),
            build: m.build,
            commit: m.commit.clone(),
            notes: m.notes.clone(),
            asset,
            changes,
        }
    }

    /// `1.0.0-alpha.2`, or `0.1.0 (141)` for previews.
    pub fn label(&self) -> String {
        if self.build > 0 { format!("{} ({})", self.version, self.build) } else { self.version.clone() }
    }

    fn key(&self) -> String {
        format!("{}+{}", self.version, self.build)
    }
}

/// Where the content pack in `<data>/updates/content.pack` came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PackInfo {
    version: String,
    build: u64,
    sha256: String,
    /// The feed's `published_at` in seconds, so a stale feed of the same
    /// build can't roll the pack back (absent in packs taken before).
    #[serde(default)]
    published_at: Option<i64>,
}

impl PackInfo {
    fn origin(&self) -> Option<PackOrigin> {
        Some(PackOrigin {
            version: Version::parse(&self.version)?,
            build: self.build,
            published_at: self.published_at,
            sha256: self.sha256.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No connection, or GitHub said no; try later.
    Network,
    /// The feed or the download didn't verify. Never installed.
    Verify(String),
    /// The system refused the install.
    Install(String),
    /// The feed needs a newer Batna: install by hand once.
    TooNew,
    /// Preview only: CI's signing key was renewed; install the newest
    /// preview by hand once.
    KeyChanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Checking,
    UpToDate,
    /// Newer, but not published for this device yet.
    NotForThisDevice,
    Available(Offer),
    Downloading {
        offer: Offer,
        done: u64,
    },
    /// Downloaded and verified.
    Ready(Offer),
    Installing(Offer),
    /// Android: "Install unknown apps" is off for Batna.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    NeedsPermission(Offer),
    /// Mac: the new Batna.app is in place; it starts at the next launch.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Installed(Offer),
    Failed {
        offer: Option<Offer>,
        why: Failure,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub off: Option<Off>,
    pub phase: Phase,
    pub prefs: Prefs,
    /// The channel being followed right now.
    pub channel: Channel,
    /// A preview build: the channel is fixed.
    pub preview: bool,
    /// A newer content pack is downloaded and verified; it opens at the
    /// next start.
    pub content_waiting: bool,
    /// This build: version and build number.
    pub version: String,
    pub build: u64,
    /// Every build's changes seen so far, newest first (changelog.rs).
    pub history: Vec<BuildChanges>,
}

impl Status {
    /// This build's place among the history's builds.
    pub fn current(&self) -> Option<(Version, u64)> {
        changelog::order(&self.version, self.build)
    }

    /// "What's new" to open by itself after an update: the builds after the
    /// one that ran before, up to this one. Empty once seen.
    pub fn whats_new(&self) -> Vec<BuildChanges> {
        let (Some(from), Some(to)) = (self.prefs.whats_new_from.as_deref(), self.current()) else {
            return Vec::new();
        };
        let from = from.split_once('+').and_then(|(v, b)| changelog::order(v, b.parse().ok()?));
        if from.is_none() {
            return Vec::new();
        }
        changelog::between(&self.history, from, to).into_iter().cloned().collect()
    }

    /// The offer the banner should mention, unless closed for it.
    pub fn banner(&self) -> Option<&Offer> {
        let offer = match &self.phase {
            Phase::Available(o) if self.prefs.mode == Mode::Notify => o,
            Phase::Ready(o) | Phase::NeedsPermission(o) | Phase::Installed(o) => o,
            _ => return None,
        };
        (self.prefs.dismissed.as_deref() != Some(offer.key().as_str())).then_some(offer)
    }
}

/// What this binary is, from the build (CI sets the env vars for previews).
#[derive(Clone, Debug)]
struct BuildInfo {
    version: Version,
    build: u64,
    preview: bool,
    key: Option<VerifyingKey>,
}

impl BuildInfo {
    fn from_env() -> Self {
        let preview = option_env!("BATNA_UPDATE_CHANNEL") == Some("preview");
        let key_text = dev_override("BATNA_UPDATE_KEY").or_else(|| {
            if preview {
                option_env!("BATNA_UPDATE_KEY").map(str::to_owned)
            } else {
                Some(RELEASE_KEY.to_owned())
            }
        });
        let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version {
            major: 0,
            minor: 0,
            patch: 0,
            stage: Stage::Alpha(0),
        });
        let build = dev_override("BATNA_BUILD")
            .as_deref()
            .or(option_env!("BATNA_BUILD"))
            .and_then(|b| b.trim().parse().ok())
            .unwrap_or(0);
        Self { version, build, preview, key: key_text.and_then(|k| parse_public_key(&k).ok()) }
    }

    /// A preview build follows `preview`; a release build its chosen
    /// release channel (alpha by default), never `preview`.
    fn channel(&self, chosen: Option<Channel>) -> Channel {
        if self.preview {
            Channel::Preview
        } else {
            chosen.filter(|c| *c != Channel::Preview).unwrap_or(Channel::Alpha)
        }
    }

    fn installed(&self, chosen: Option<Channel>) -> Installed {
        Installed { version: self.version, build: self.build, channel: self.channel(chosen) }
    }
}

/// The build number (CI's run number for previews, 0 otherwise).
pub fn build_number() -> u64 {
    get().map_or_else(|| BuildInfo::from_env().build, |u| u.info.build)
}

/// The commit this build was made from (CI and `just apk` set it).
pub fn commit() -> Option<&'static str> {
    option_env!("BATNA_COMMIT").filter(|c| !c.is_empty())
}

/// Run-time overrides for trying updates from a development build against a
/// local feed (`docs/RELEASE.md` §5). Release builds ignore them.
fn dev_override(name: &str) -> Option<String> {
    if cfg!(debug_assertions) { std::env::var(name).ok().filter(|v| !v.trim().is_empty()) } else { None }
}

pub struct Updater {
    dir: PathBuf,
    info: BuildInfo,
    platform: Option<UpdatePlatform>,
    /// A local feed folder instead of GitHub (development only).
    feed_override: Option<String>,
    /// Pause before re-reading a feed whose signature didn't match.
    retry_after: Duration,
    tx: watch::Sender<Status>,
    /// One check, download or install at a time.
    job: Mutex<()>,
}

static UPDATER: OnceLock<Updater> = OnceLock::new();

/// Opens the updater under the app's data folder and starts the background
/// checks. Called once at start.
pub fn init() {
    let Some(data) = crate::prefs::data_dir() else { return };
    let feed_override = dev_override("BATNA_UPDATE_URL");
    let platform =
        dev_override("BATNA_UPDATE_PLATFORM").and_then(|p| UpdatePlatform::parse(&p)).or_else(this_platform);
    let off = if cfg!(debug_assertions) && feed_override.is_none() {
        Some(Off::DevBuild)
    } else if cfg!(target_os = "macos") && bundle::running_bundle().is_none() && feed_override.is_none() {
        Some(Off::NotInBundle)
    } else {
        None
    };
    let updater = Updater::new(data.join("updates"), BuildInfo::from_env(), platform, feed_override, off);
    let start = updater.status().off.is_none();
    let background = updater.background_checks();
    if UPDATER.set(updater).is_err() {
        return;
    }
    crate::platform::update_notice::schedule(background);
    if !start {
        return;
    }
    let _ = std::thread::Builder::new().name("batna-update".into()).spawn(|| {
        std::thread::sleep(FIRST_CHECK_AFTER);
        loop {
            if let Some(u) = UPDATER.get() {
                u.tick();
            }
            std::thread::sleep(Duration::from_secs(30 * 60));
        }
    });
}

pub fn get() -> Option<&'static Updater> {
    UPDATER.get()
}

/// The updater Android's background job (`UpdateCheckJob`) works with, in a
/// process that may have no UI: the app's own if it's running here, else
/// one opened for the job under `data_dir` (Context.getFilesDir).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn job_updater(data_dir: &Path) -> &'static Updater {
    static HEADLESS: OnceLock<Updater> = OnceLock::new();
    match UPDATER.get() {
        Some(u) => u,
        None => HEADLESS.get_or_init(|| {
            let off = cfg!(debug_assertions).then_some(Off::DevBuild);
            Updater::new(data_dir.join("updates"), BuildInfo::from_env(), this_platform(), None, off)
        }),
    }
}

/// Android's background job: checks when due and returns the notification
/// to post, if any. It isn't marked as sent yet: the job calls
/// [`background_shown`] once Android has shown it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn background_check(data_dir: &Path, metered: bool, can_notify: bool) -> Option<Notice> {
    job_updater(data_dir).background(metered, can_notify)
}

/// The job's notification `key` is up: it doesn't go out again.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn background_shown(data_dir: &Path, key: &str) {
    job_updater(data_dir).shown(key);
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

fn is_newer(me: &Installed, offer: &Offer) -> bool {
    Version::parse(&offer.version).is_some_and(|v| (v, offer.build) > (me.version, me.build))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

impl Updater {
    fn new(
        dir: PathBuf,
        info: BuildInfo,
        platform: Option<UpdatePlatform>,
        feed_override: Option<String>,
        off: Option<Off>,
    ) -> Self {
        let mut prefs = load_prefs(&dir);
        let off = off.or(if platform.is_none() {
            Some(Off::NoPlatform)
        } else if info.key.is_none() {
            Some(Off::NoKey)
        } else {
            None
        });
        let this = format!("{}+{}", info.version, info.build);
        if off.is_none() && prefs.last_run.as_deref() != Some(this.as_str()) {
            // Updated (not a first install): say what's new once.
            if let Some(prev) = prefs.last_run.take() {
                prefs.whats_new_from = Some(prev);
            }
            prefs.last_run = Some(this);
            if let Err(e) = save_prefs(&dir, &prefs) {
                report!("update prefs: {e}");
            }
        }
        let history = changelog::load(&dir);
        let installed = info.installed(prefs.channel);
        let mut phase = Phase::Idle;
        if off.is_none()
            && let Some(offer) = load_staged(&dir)
            && is_newer(&installed, &offer)
            && dir.join(&offer.asset.file).metadata().is_ok_and(|m| m.len() == offer.asset.size)
        {
            phase = Phase::Ready(offer);
        }
        let status = Status {
            off,
            phase,
            channel: installed.channel,
            preview: info.preview,
            prefs,
            content_waiting: false,
            version: info.version.to_string(),
            build: info.build,
            history,
        };
        let (tx, _) = watch::channel(status);
        Self {
            dir,
            info,
            platform,
            feed_override,
            retry_after: Duration::from_secs(20),
            tx,
            job: Mutex::new(()),
        }
    }

    /// What's installed, on the channel chosen now.
    fn installed(&self) -> Installed {
        self.info.installed(self.tx.borrow().prefs.channel)
    }

    /// The feed folder: the local override in development, else the
    /// channel's GitHub release.
    fn feed_url(&self) -> String {
        match &self.feed_override {
            Some(url) => url.trim_end_matches('/').to_owned(),
            None => {
                let url = self.installed().channel.feed_url();
                url.trim_end_matches(MANIFEST_FILE).trim_end_matches('/').to_owned()
            }
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<Status> {
        self.tx.subscribe()
    }

    pub fn status(&self) -> Status {
        self.tx.borrow().clone()
    }

    fn set_phase(&self, phase: Phase) {
        self.tx.send_modify(|s| s.phase = phase);
    }

    fn update_prefs(&self, f: impl FnOnce(&mut Prefs)) {
        self.tx.send_modify(|s| {
            f(&mut s.prefs);
            s.channel = self.info.channel(s.prefs.channel);
            if let Err(e) = save_prefs(&self.dir, &s.prefs) {
                report!("update prefs: {e}");
            }
        });
    }

    /// The background loop's step: check when due, download in Auto mode,
    /// then announce what it found.
    fn tick(&'static self) {
        self.step(CHECK_EVERY, crate::platform::network::metered());
        self.send_notice(crate::platform::update_notice::post);
    }

    /// Checks when the last check is `every` old, or starts the download an
    /// earlier check left waiting for Wi-Fi.
    fn step(&'static self, every: Duration, metered: bool) {
        let s = self.status();
        if s.off.is_some() || s.prefs.mode == Mode::Off {
            return;
        }
        let due =
            s.prefs.last_check_ms.is_none_or(|t| now_ms().saturating_sub(t) >= every.as_millis() as u64);
        // Automatic downloads wait for Wi-Fi (an APK on mobile data, up to
        // every push on the preview channel); the offer still shows.
        let wifi = !metered;
        let auto = s.prefs.mode == Mode::Auto && wifi;
        if due {
            self.check_with(auto, wifi);
        } else if auto && let Phase::Available(_) = s.phase {
            self.download();
        }
    }

    /// Sends the notification due now through `post` (false: not shown, try
    /// again at the next check) and remembers it once shown.
    fn send_notice(&self, post: impl FnOnce(&Notice) -> bool) -> Option<Notice> {
        let n = notice(&self.status())?;
        if !post(&n) {
            return None;
        }
        self.update_prefs(|p| p.notified = Some(n.key.clone()));
        Some(n)
    }

    /// Android's background job: the app loop's step, then the notification
    /// to post if `can_notify` (notifications allowed, Batna not in front).
    /// Not marked as sent: a notification Android doesn't show is tried
    /// again at the next run ([`Self::shown`] marks it).
    fn background(&'static self, metered: bool, can_notify: bool) -> Option<Notice> {
        self.step(BACKGROUND_EVERY, metered);
        if !can_notify {
            return None;
        }
        notice(&self.status())
    }

    /// The notification `key` was shown.
    fn shown(&self, key: &str) {
        if self.status().prefs.notified.as_deref() != Some(key) {
            self.update_prefs(|p| p.notified = Some(key.to_owned()));
        }
    }

    /// Whether Android's background job should run: updates work here and
    /// aren't switched off.
    pub fn background_checks(&self) -> bool {
        let s = self.status();
        s.off.is_none() && s.prefs.mode != Mode::Off
    }

    pub fn set_notify(&self, on: bool) {
        self.update_prefs(|p| p.notify = on);
        if !on {
            crate::platform::update_notice::clear();
        }
    }

    /// "Check now": looks at the feed, and fetches a newer content pack.
    /// With `download`, fetches a newer build right away too.
    pub fn check(&'static self, download: bool) {
        self.check_with(download, true);
    }

    fn check_with(&self, download: bool, content: bool) {
        let Ok(_job) = self.job.try_lock() else { return };
        if self.status().off.is_some() {
            return;
        }
        let keep = match self.status().phase {
            // A finished download for the same offer stays; it's re-checked anyway.
            Phase::Ready(o) | Phase::Installed(o) | Phase::NeedsPermission(o) => Some(o),
            _ => None,
        };
        self.set_phase(Phase::Checking);
        let manifest = self.fetch_manifest();
        self.update_prefs(|p| p.last_check_ms = Some(now_ms()));
        let m = match manifest {
            Ok(m) => m,
            Err(why) => return self.set_phase(Phase::Failed { offer: None, why }),
        };
        self.remember_history(&m.history);
        if content {
            self.refresh_content(&m);
        }
        match self.offer(&m) {
            Ok(Some(offer)) => {
                if let Some(k) = keep.filter(|k| k.asset == offer.asset) {
                    self.set_phase(match self.status().phase {
                        Phase::Checking => Phase::Ready(k),
                        other => other,
                    });
                    return;
                }
                self.set_phase(Phase::Available(offer.clone()));
                if download {
                    self.download_locked(offer);
                }
            }
            Ok(None) => {}
            Err(why) => self.set_phase(Phase::Failed { offer: None, why }),
        }
    }

    /// The channel's `update.json`, signature checked.
    fn fetch_manifest(&self) -> Result<UpdateManifest, Failure> {
        let key = self.info.key.ok_or(Failure::Verify("no key".into()))?;
        let base = self.feed_url();
        let fetch = || -> Result<_, Failure> {
            let json = get_small(&format!("{base}/{MANIFEST_FILE}"))?;
            let sig = get_small(&format!("{base}/{SIGNATURE_FILE}"))?;
            let sig = String::from_utf8(sig).map_err(|_| Failure::Verify("signature".into()))?;
            Ok(verify_manifest(&json, &sig, &key))
        };
        // A feed caught mid-upload pairs a new signature with the old
        // manifest: look once more before calling it a failure.
        let m = match fetch()? {
            Err(UpdateError::NotSigned) => {
                std::thread::sleep(self.retry_after);
                fetch()?
            }
            first => first,
        };
        m.map_err(|e| match e {
            // A preview whose CI key was renewed can't verify any newer feed.
            UpdateError::NotSigned if self.info.preview => Failure::KeyChanged,
            other => update_failure(other),
        })
    }

    /// `Ok(None)` after setting UpToDate / NotForThisDevice.
    fn offer(&self, m: &UpdateManifest) -> Result<Option<Offer>, Failure> {
        let platform = self.platform.ok_or(Failure::Verify("no platform".into()))?;
        match decide(&self.installed(), m, platform).map_err(update_failure)? {
            Decision::UpToDate => {
                self.set_phase(Phase::UpToDate);
                self.clear_downloads(None);
                Ok(None)
            }
            Decision::NotForThisPlatform => {
                self.set_phase(Phase::NotForThisDevice);
                Ok(None)
            }
            Decision::Available(asset) => {
                if asset.size > MAX_BUILD_BYTES {
                    return Err(Failure::Verify("build too large".into()));
                }
                Ok(Some(Offer::from_manifest(m, asset, &self.installed())))
            }
        }
    }

    /// Downloads the feed's content pack when it differs from the one in
    /// use, checks it decodes, and keeps it for the next start. Failures
    /// are only reported: the pack built into the app keeps working.
    fn refresh_content(&self, m: &UpdateManifest) {
        let current = load_pack_info(&self.dir).and_then(|i| i.origin());
        let asset = match content_update(&self.installed(), m, current.as_ref()) {
            Ok(Some(a)) => a.clone(),
            Ok(None) => return,
            Err(e) => return report!("content pack: {e}"),
        };
        if asset.size > MAX_PACK_BYTES {
            return report!("content pack: {} bytes is too large", asset.size);
        }
        let result = (|| -> Result<(), String> {
            let url = format!("{}/{}", self.feed_url(), asset.file);
            let bytes =
                get_limited(&url, asset.size, Duration::from_secs(10 * 60)).map_err(|f| format!("{f:?}"))?;
            check_download(&asset, bytes.len() as u64, &sha256_hex(&bytes)).map_err(|e| e.to_string())?;
            Pack::decode(&bytes).map_err(|e| e.to_string())?;
            fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
            let tmp = self.dir.join(format!("{PACK_FILE}.part"));
            fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
            fs::rename(&tmp, self.dir.join(PACK_FILE)).map_err(|e| e.to_string())?;
            let info = PackInfo {
                version: m.version.clone(),
                build: m.build,
                sha256: asset.sha256.clone(),
                published_at: published_secs(&m.published_at),
            };
            write_json(&self.dir, PACK_INFO, &info)
        })();
        match result {
            Ok(()) => self.tx.send_modify(|s| s.content_waiting = true),
            Err(e) => report!("content pack {} not taken: {e}", asset.file),
        }
    }

    /// Keeps a verified feed's history, and lets a pending "What's new" go
    /// when the feed has nothing to say about this build.
    fn remember_history(&self, fresh: &[BuildChanges]) {
        let known = self.status().history;
        if let Some(all) = changelog::merge(&known, fresh) {
            if let Err(e) = write_json(&self.dir, changelog::FILE, &all) {
                report!("update history: {e}");
            }
            self.tx.send_modify(|s| s.history = all);
        }
        let s = self.status();
        if s.prefs.whats_new_from.is_some() && s.whats_new().is_empty() {
            self.update_prefs(|p| p.whats_new_from = None);
        }
    }

    /// "What's new" was seen (closed).
    pub fn close_whats_new(&self) {
        if self.status().prefs.whats_new_from.is_some() {
            self.update_prefs(|p| p.whats_new_from = None);
        }
    }

    /// Downloads the offered build (a tap in "tell me first" mode).
    pub fn download(&'static self) {
        let Ok(_job) = self.job.try_lock() else { return };
        if let Phase::Available(offer) | Phase::Failed { offer: Some(offer), .. } = self.status().phase {
            self.download_locked(offer);
        }
    }

    fn download_locked(&self, offer: Offer) {
        let base = self.feed_url();
        let _ = fs::create_dir_all(&self.dir);
        self.clear_downloads(Some(&offer.asset.file));
        let path = self.dir.join(&offer.asset.file);
        self.set_phase(Phase::Downloading { offer: offer.clone(), done: 0 });
        match self.fetch_build(&format!("{base}/{}", offer.asset.file), &offer, &path) {
            Ok(()) => {
                if let Err(e) = save_staged(&self.dir, &offer) {
                    report!("update staged.json: {e}");
                }
                self.set_phase(Phase::Ready(offer));
            }
            Err(why) => {
                let _ = fs::remove_file(&path);
                self.set_phase(Phase::Failed { offer: Some(offer), why });
            }
        }
    }

    /// Streams the build to `<file>.part` while hashing it, checks size and
    /// SHA-256 against the signed manifest, then renames it into place.
    fn fetch_build(&self, url: &str, offer: &Offer, path: &Path) -> Result<(), Failure> {
        let mut res =
            agent(url, Duration::from_secs(30 * 60)).get(url).call().map_err(|_| Failure::Network)?;
        if res.status().as_u16() != 200 {
            return Err(Failure::Network);
        }
        let part = path.with_extension("part");
        let mut file = fs::File::create(&part).map_err(|e| Failure::Install(e.to_string()))?;
        let mut reader = res.body_mut().as_reader().take(offer.asset.size + 1);
        let mut hash = Sha256::new();
        let (mut done, mut shown) = (0u64, 0u64);
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = reader.read(&mut buf).map_err(|_| Failure::Network)?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(|e| Failure::Install(e.to_string()))?;
            done += n as u64;
            if done - shown > offer.asset.size / 100 {
                shown = done;
                self.set_phase(Phase::Downloading { offer: offer.clone(), done });
            }
        }
        file.sync_all().map_err(|e| Failure::Install(e.to_string()))?;
        drop(file);
        if let Err(e) = check_download(&offer.asset, done, &hex(&hash.finalize())) {
            let _ = fs::remove_file(&part);
            return Err(update_failure(e));
        }
        fs::rename(&part, path).map_err(|e| Failure::Install(e.to_string()))
    }

    /// Re-hashes the staged file: whatever sat on disk since the download
    /// is checked again before it's installed.
    fn verified_staged(&self, offer: &Offer) -> Result<PathBuf, Failure> {
        let path = self.dir.join(&offer.asset.file);
        let mut file = fs::File::open(&path).map_err(|_| Failure::Verify("download missing".into()))?;
        let mut hash = Sha256::new();
        let size = std::io::copy(&mut file, &mut hash).map_err(|e| Failure::Verify(e.to_string()))?;
        check_download(&offer.asset, size, &hex(&hash.finalize())).map_err(update_failure)?;
        Ok(path)
    }

    /// Installs the ready download (a tap, or a calm moment in Auto mode).
    pub fn install(&'static self) {
        let Ok(_job) = self.job.try_lock() else { return };
        let offer = match self.status().phase {
            Phase::Ready(o)
            | Phase::NeedsPermission(o)
            | Phase::Failed { offer: Some(o), why: Failure::Install(_) } => o,
            _ => return,
        };
        let path = match self.verified_staged(&offer) {
            Ok(p) => p,
            Err(why) => {
                self.clear_downloads(None);
                self.set_phase(Phase::Failed { offer: Some(offer), why });
                return;
            }
        };
        self.set_phase(Phase::Installing(offer.clone()));
        let next = install_build(&path, &offer);
        if let Phase::Installed(_) = next {
            self.clear_downloads(None);
        }
        self.set_phase(next);
    }

    pub fn set_mode(&self, mode: Mode) {
        self.update_prefs(|p| p.mode = mode);
        crate::platform::update_notice::schedule(self.background_checks());
    }

    /// Switches release channel and looks again.
    pub fn set_channel(&'static self, channel: Channel) {
        if self.status().preview || channel == Channel::Preview {
            return;
        }
        self.update_prefs(|p| p.channel = Some(channel));
        self.clear_downloads(None);
        self.set_phase(Phase::Idle);
        let auto = self.status().prefs.mode == Mode::Auto;
        std::thread::spawn(move || self.check(auto));
    }

    pub fn dismiss(&self) {
        let key = self.status().banner().map(Offer::key);
        self.update_prefs(|p| p.dismissed = key);
        crate::platform::update_notice::clear();
    }

    /// Android: polls PackageInstaller after an install started.
    #[cfg(target_os = "android")]
    pub fn poll_install(&self) {
        use crate::platform::install::{InstallState, install_state};
        let Phase::Installing(offer) = self.status().phase else { return };
        match install_state() {
            // A success ends this process: the new Batna starts in its place.
            Ok(InstallState::Working | InstallState::WaitingUser) => {}
            Ok(InstallState::Idle | InstallState::Cancelled) => self.set_phase(Phase::Ready(offer)),
            Ok(InstallState::Failed(why)) | Err(why) => {
                self.update_prefs(|p| p.auto_failed = Some(offer.key()));
                self.set_phase(Phase::Failed { offer: Some(offer), why: Failure::Install(why) });
            }
        }
    }

    /// Removes every app download except `keep`, and the staged record
    /// unless kept. Prefs, the changelog and the content pack stay.
    fn clear_downloads(&self, keep: Option<&str>) {
        let Ok(entries) = fs::read_dir(&self.dir) else { return };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if matches!(name.as_str(), "prefs.json" | changelog::FILE | PACK_FILE | PACK_INFO)
                || Some(name.as_str()) == keep
            {
                continue;
            }
            if name == "staged.json" && keep.is_some() {
                continue;
            }
            let _ = fs::remove_file(e.path());
        }
    }
}

#[cfg(target_os = "android")]
fn install_build(path: &Path, offer: &Offer) -> Phase {
    use crate::i18n::update as tx;
    use crate::platform::install;
    if !install::can_request_installs().unwrap_or(false) {
        if let Err(e) = install::open_install_permission() {
            report!("{e}");
        }
        return Phase::NeedsPermission(offer.clone());
    }
    let path = path.to_string_lossy();
    match install::install_apk(
        &path,
        offer.asset.version_code.unwrap_or(0),
        tx::DONE_TITLE,
        // "Batna was updated" says what's new when the feed says it.
        &Some(notes_of(offer)).filter(|n| !n.is_empty()).unwrap_or_else(|| tx::DONE_BODY.to_owned()),
        tx::READY_TITLE,
    ) {
        Ok(()) => Phase::Installing(offer.clone()),
        Err(e) => Phase::Failed { offer: Some(offer.clone()), why: Failure::Install(e) },
    }
}

#[cfg(target_os = "macos")]
fn install_build(path: &Path, offer: &Offer) -> Phase {
    let Some(app) = bundle::running_bundle() else {
        return Phase::Failed {
            offer: Some(offer.clone()),
            why: Failure::Install("not running from Batna.app".into()),
        };
    };
    match bundle::install_zip(path, &app, bundle::ditto_extract, |new| bundle::check_signature(new, &app)) {
        Ok(()) => Phase::Installed(offer.clone()),
        Err(e) => Phase::Failed { offer: Some(offer.clone()), why: Failure::Install(e) },
    }
}

#[cfg(not(any(target_os = "android", target_os = "macos")))]
fn install_build(_path: &Path, offer: &Offer) -> Phase {
    Phase::Failed { offer: Some(offer.clone()), why: Failure::Install("no installer on this system".into()) }
}

/// Mac, automatic mode: a verified download staged by the last session
/// goes in before the window opens, and Batna reopens as the new version.
/// Called first thing in `main`.
#[cfg(target_os = "macos")]
pub fn apply_staged_on_launch() {
    let Some(data) = crate::prefs::data_dir() else { return };
    let dir = data.join("updates");
    let prefs = load_prefs(&dir);
    if prefs.mode != Mode::Auto {
        return;
    }
    let Some(offer) = load_staged(&dir) else { return };
    let info = BuildInfo::from_env();
    if info.key.is_none()
        || !is_newer(&info.installed(prefs.channel), &offer)
        || prefs.auto_failed.as_deref() == Some(&offer.key())
    {
        return;
    }
    let Some(app) = bundle::running_bundle() else { return };
    let path = dir.join(&offer.asset.file);
    let verified = fs::read(&path)
        .is_ok_and(|bytes| check_download(&offer.asset, bytes.len() as u64, &sha256_hex(&bytes)).is_ok());
    let result = if verified {
        bundle::install_zip(&path, &app, bundle::ditto_extract, |new| bundle::check_signature(new, &app))
    } else {
        Err("changed on disk".into())
    };
    match result {
        Ok(()) => {
            let _ = fs::remove_file(&path);
            let _ = fs::remove_file(dir.join("staged.json"));
            bundle::relaunch(&app);
        }
        Err(e) => {
            report!("update {} not applied: {e}", offer.label());
            let mut prefs = prefs;
            prefs.auto_failed = Some(offer.key());
            let _ = save_prefs(&dir, &prefs);
        }
    }
}

/// Mac: quits and reopens the swapped-in Batna.app.
#[cfg(target_os = "macos")]
pub fn restart() {
    if let Some(app) = bundle::running_bundle() {
        bundle::relaunch(&app);
    }
}

/// The content pack a feed delivered, when it is at least as new as this
/// build and still decodes; `content::Library` uses it instead of the
/// embedded one. A pack from an older build (the app was updated since) is
/// removed.
pub fn installed_pack() -> Option<Pack> {
    installed_pack_in(&crate::prefs::data_dir()?.join("updates"), &BuildInfo::from_env())
}

fn installed_pack_in(dir: &Path, info: &BuildInfo) -> Option<Pack> {
    let meta = load_pack_info(dir)?;
    let from = Version::parse(&meta.version).map(|v| (v, meta.build));
    if from.is_none_or(|from| from < (info.version, info.build)) {
        let _ = fs::remove_file(dir.join(PACK_FILE));
        let _ = fs::remove_file(dir.join(PACK_INFO));
        return None;
    }
    let bytes = fs::read(dir.join(PACK_FILE)).ok()?;
    if !sha256_hex(&bytes).eq_ignore_ascii_case(&meta.sha256) {
        report!("content pack changed on disk; using the built-in one");
        return None;
    }
    match Pack::decode(&bytes) {
        Ok(p) => Some(p),
        Err(e) => {
            report!("content pack doesn't open ({e}); using the built-in one");
            None
        }
    }
}

fn update_failure(e: UpdateError) -> Failure {
    match e {
        UpdateError::TooNew => Failure::TooNew,
        other => Failure::Verify(other.to_string()),
    }
}

/// An HTTP agent; local development feeds skip any proxy in the environment.
fn agent(url: &str, timeout: Duration) -> ureq::Agent {
    let local = url.starts_with("http://127.0.0.1") || url.starts_with("http://localhost");
    let mut config = ureq::Agent::config_builder().http_status_as_error(false).timeout_global(Some(timeout));
    if local {
        config = config.proxy(None);
    }
    config.build().into()
}

fn get_small(url: &str) -> Result<Vec<u8>, Failure> {
    let bytes = get_limited(url, MAX_FEED_BYTES, Duration::from_secs(30))?;
    if bytes.len() as u64 > MAX_FEED_BYTES {
        return Err(Failure::Verify("feed too large".into()));
    }
    Ok(bytes)
}

/// The body, cut one byte past `limit` (so a caller can tell it was longer).
fn get_limited(url: &str, limit: u64, timeout: Duration) -> Result<Vec<u8>, Failure> {
    let mut res = agent(url, timeout).get(url).call().map_err(|_| Failure::Network)?;
    if res.status().as_u16() != 200 {
        return Err(Failure::Network);
    }
    let mut bytes = Vec::new();
    res.body_mut().as_reader().take(limit + 1).read_to_end(&mut bytes).map_err(|_| Failure::Network)?;
    Ok(bytes)
}

fn load_prefs(dir: &Path) -> Prefs {
    fs::read(dir.join("prefs.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn save_prefs(dir: &Path, prefs: &Prefs) -> Result<(), String> {
    write_json(dir, "prefs.json", prefs)
}

fn load_staged(dir: &Path) -> Option<Offer> {
    serde_json::from_slice(&fs::read(dir.join("staged.json")).ok()?).ok()
}

fn save_staged(dir: &Path, offer: &Offer) -> Result<(), String> {
    write_json(dir, "staged.json", offer)
}

fn load_pack_info(dir: &Path) -> Option<PackInfo> {
    serde_json::from_slice(&fs::read(dir.join(PACK_INFO)).ok()?).ok()
}

fn write_json(dir: &Path, name: &str, value: &impl Serialize) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{name}.part"));
    fs::write(&tmp, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    fs::rename(&tmp, dir.join(name)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes behind the signed fixture's assets (no .apk is tracked).
    const FIXTURE_APK: &[u8] = b"fixture apk bytes";
    const FIXTURE_PACK: &[u8] = b"fixture pack bytes";

    fn offer(version: &str, build: u64) -> Offer {
        Offer {
            version: version.into(),
            build,
            commit: None,
            notes: UpdateNotes::default(),
            asset: UpdateAsset {
                platform: UpdatePlatform::AndroidArm64,
                file: "batna.apk".into(),
                size: 1,
                sha256: "00".repeat(32),
                version_code: None,
            },
            changes: Vec::new(),
        }
    }

    fn status(phase: Phase, mode: Mode) -> Status {
        Status {
            off: None,
            phase,
            prefs: Prefs { mode, ..Prefs::default() },
            channel: Channel::Alpha,
            preview: false,
            content_waiting: false,
            version: "0.1.0".into(),
            build: 41,
            history: Vec::new(),
        }
    }

    fn build(version: &str, build: u64, preview: bool) -> BuildInfo {
        let key = include_str!("../../../../fixtures/update/test-key.pub");
        BuildInfo {
            version: Version::parse(version).unwrap(),
            build,
            preview,
            key: Some(parse_public_key(key).unwrap()),
        }
    }

    #[test]
    fn prefs_round_trip_and_default_to_automatic_alpha() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_prefs(dir.path()), Prefs::default());
        assert_eq!(Prefs::default().mode, Mode::Auto);
        let p = Prefs {
            mode: Mode::Notify,
            channel: Some(Channel::Beta),
            last_check_ms: Some(5),
            ..Prefs::default()
        };
        save_prefs(dir.path(), &p).unwrap();
        assert_eq!(load_prefs(dir.path()), p);
        let release = BuildInfo { key: None, ..build("1.0.0", 0, false) };
        assert_eq!(release.channel(None), Channel::Alpha);
        assert_eq!(release.channel(Some(Channel::Beta)), Channel::Beta);
        // A release build never follows the preview feed, whatever the file says.
        assert_eq!(release.channel(Some(Channel::Preview)), Channel::Alpha);
        let preview = BuildInfo { preview: true, ..release };
        assert_eq!(preview.channel(Some(Channel::Stable)), Channel::Preview);
    }

    #[test]
    fn the_banner_waits_for_something_to_do_and_can_be_closed() {
        let o = offer("1.0.0-alpha.2", 0);
        assert!(
            status(Phase::Available(o.clone()), Mode::Auto).banner().is_none(),
            "auto mode is still downloading"
        );
        assert!(status(Phase::Available(o.clone()), Mode::Notify).banner().is_some());
        assert!(status(Phase::Ready(o.clone()), Mode::Auto).banner().is_some());
        let mut s = status(Phase::Ready(o.clone()), Mode::Auto);
        s.prefs.dismissed = Some(o.key());
        assert!(s.banner().is_none());
        // A newer offer shows again.
        s.phase = Phase::Ready(offer("1.0.0-alpha.3", 0));
        assert!(s.banner().is_some());
    }

    #[test]
    fn offers_label_previews_by_build() {
        assert_eq!(offer("1.0.0-alpha.2", 0).label(), "1.0.0-alpha.2");
        assert_eq!(offer("0.1.0", 141).label(), "0.1.0 (141)");
        let me =
            Installed { version: Version::parse("0.1.0").unwrap(), build: 140, channel: Channel::Preview };
        assert!(is_newer(&me, &offer("0.1.0", 141)));
        assert!(!is_newer(&me, &offer("0.1.0", 140)));
    }

    #[test]
    fn staged_offer_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_staged(dir.path()).is_none());
        let o = offer("1.0.0-alpha.2", 0);
        save_staged(dir.path(), &o).unwrap();
        assert_eq!(load_staged(dir.path()), Some(o));
    }

    /// Serves `dir` over plain HTTP on localhost, one request per connection.
    fn serve(dir: PathBuf) -> String {
        use std::io::{BufRead as _, BufReader};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/").trim_start_matches('/').to_owned();
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).is_err() || h.trim().is_empty() {
                        break;
                    }
                }
                let mut stream = reader.into_inner();
                let _ = match fs::read(dir.join(&path)) {
                    Ok(body) => {
                        let head = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        stream.write_all(head.as_bytes()).and_then(|()| stream.write_all(&body))
                    }
                    Err(_) => stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    ),
                };
            }
        });
        url
    }

    fn fixture_feed() -> tempfile::TempDir {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/update");
        let feed = tempfile::tempdir().unwrap();
        for name in [MANIFEST_FILE, SIGNATURE_FILE] {
            fs::copy(src.join(name), feed.path().join(name)).unwrap();
        }
        fs::write(feed.path().join("batna-0.1.0-b42-android-arm64.apk"), FIXTURE_APK).unwrap();
        fs::write(feed.path().join("batna-content-0.1.0-b42-content.pack"), FIXTURE_PACK).unwrap();
        feed
    }

    fn updater(feed: &str, info: BuildInfo) -> (&'static Updater, tempfile::TempDir) {
        let data = tempfile::tempdir().unwrap();
        (updater_in(data.path(), feed, info), data)
    }

    fn updater_in(data: &Path, feed: &str, info: BuildInfo) -> &'static Updater {
        let mut u = Updater::new(
            data.join("updates"),
            info,
            Some(UpdatePlatform::AndroidArm64),
            Some(feed.to_owned()),
            None,
        );
        u.retry_after = Duration::from_millis(10);
        Box::leak(Box::new(u))
    }

    #[test]
    fn whats_new_before_and_after_an_update() {
        use crate::i18n::update as tx;
        let feed = fixture_feed();
        let url = serve(feed.path().to_path_buf());
        let data = tempfile::tempdir().unwrap();

        // A first install says nothing about what's new.
        let u = updater_in(data.path(), &url, build("0.1.0", 39, true));
        assert!(u.status().whats_new().is_empty());
        assert_eq!(u.status().prefs.last_run.as_deref(), Some("0.1.0+39"));

        // Before installing: the offer brings the builds after this one.
        u.check(false);
        let Phase::Available(o) = u.status().phase else { panic!("{:?}", u.status().phase) };
        assert_eq!(o.changes.iter().map(BuildChanges::key).collect::<Vec<_>>(), ["0.1.0+42", "0.1.0+40"]);
        assert_eq!(u.status().history.len(), 2, "the feed's history is kept");
        let mut s = u.status();
        s.prefs.mode = Mode::Notify;
        let n = notice(&s).unwrap();
        assert!(n.body.starts_with("• "), "{}", n.body);
        assert!(!n.body.starts_with(tx::WHATS_NEW), "{}", n.body);

        // The first launch of the new build opens "What's new" once.
        let u = updater_in(data.path(), &url, build("0.1.0", 42, true));
        let w = u.status().whats_new();
        assert_eq!(w.iter().map(BuildChanges::key).collect::<Vec<_>>(), ["0.1.0+42", "0.1.0+40"]);
        u.close_whats_new();
        assert!(u.status().whats_new().is_empty());
        let u = updater_in(data.path(), &url, build("0.1.0", 42, true));
        assert!(u.status().whats_new().is_empty(), "seen stays seen");
    }

    #[test]
    fn checks_downloads_and_verifies_from_a_signed_feed() {
        let feed = fixture_feed();
        let url = serve(feed.path().to_path_buf());

        let (u, _data) = updater(&url, build("0.1.0", 41, true));
        u.check(true);
        let Phase::Ready(offer) = u.status().phase else { panic!("{:?}", u.status().phase) };
        assert_eq!(offer.label(), "0.1.0 (42)");
        assert_eq!(offer.asset.version_code, Some(100_999));
        assert!(u.verified_staged(&offer).is_ok());
        assert!(u.status().prefs.last_check_ms.is_some());
        // A fresh start finds the staged download again.
        let again = Updater::new(u.dir.clone(), u.info.clone(), u.platform, u.feed_override.clone(), None);
        assert_eq!(again.status().phase, Phase::Ready(offer.clone()));
        // Changed on disk after the download: refused at install time.
        fs::write(u.dir.join(&offer.asset.file), b"fixture apk bytez").unwrap();
        assert!(matches!(u.verified_staged(&offer), Err(Failure::Verify(_))));

        // Already on that build: nothing to do, and old downloads go.
        let (u, _data) = updater(&url, build("0.1.0", 42, true));
        u.check(true);
        assert_eq!(u.status().phase, Phase::UpToDate);
    }

    #[test]
    fn a_notice_goes_out_once_per_offer_and_step() {
        use crate::i18n::update as tx;
        let mut o = offer("0.1.0", 42);
        o.notes.ar_eg = "محاضرة الاستسقاء اتضافت".into();
        let mut s = status(Phase::Available(o.clone()), Mode::Notify);
        let n = notice(&s).expect("a new version is announced");
        assert_eq!(n.title, format!("{} · 0.1.0 (42)", tx::NOTICE_AVAILABLE));
        assert!(n.body.starts_with(&format!("{}: محاضرة الاستسقاء اتضافت\n", tx::WHATS_NEW)), "{}", n.body);
        assert_eq!((n.action, n.button), ("download", tx::DOWNLOAD));
        s.prefs.notified = Some(n.key.clone());
        assert!(notice(&s).is_none(), "sent once");
        s.phase = Phase::Ready(o.clone());
        let ready = notice(&s).unwrap();
        assert_eq!(ready.action, "install");
        let mut closed = s.clone();
        closed.prefs.dismissed = Some(o.key());
        assert!(notice(&closed).is_none());
        let mut switch_off = s.clone();
        switch_off.prefs.notify = false;
        assert!(notice(&switch_off).is_none());
        let mut off = s.clone();
        off.prefs.mode = Mode::Off;
        assert!(notice(&off).is_none());
        assert!(notice(&status(Phase::UpToDate, Mode::Auto)).is_none());
        // Old prefs files keep the switch on.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("prefs.json"), r#"{"mode":"notify"}"#).unwrap();
        assert!(load_prefs(dir.path()).notify);
    }

    #[test]
    fn the_background_check_downloads_on_wifi_and_announces_once() {
        let feed = fixture_feed();
        let url = serve(feed.path().to_path_buf());

        let (u, _data) = updater(&url, build("0.1.0", 41, true));
        let n = u.background(false, true).expect("announced");
        assert_eq!(n.action, "install");
        assert!(matches!(u.status().phase, Phase::Ready(_)));
        // Not shown (Android said no): offered again at the next run.
        assert_eq!(u.background(false, true).as_ref(), Some(&n));
        u.shown(&n.key);
        assert!(u.background(false, true).is_none(), "once shown, never again");
        assert!(serde_json::to_string(&n).unwrap().contains(&format!("\"key\":\"{}\"", n.key)));

        // Mobile data: announced, not downloaded.
        let (u, _data) = updater(&url, build("0.1.0", 41, true));
        assert_eq!(u.background(true, true).unwrap().action, "download");
        assert!(matches!(u.status().phase, Phase::Available(_)));

        // Batna in front: kept for the next run.
        let (u, _data) = updater(&url, build("0.1.0", 41, true));
        assert!(u.background(true, false).is_none());
        assert!(u.background(true, true).is_some());
        assert_eq!(u.status().prefs.notified, None, "marked only once shown");

        let (u, _data) = updater(&url, build("0.1.0", 42, true));
        assert!(u.background(false, true).is_none());
    }

    #[test]
    fn a_swapped_build_or_manifest_is_never_offered() {
        // The build on the server isn't the one the manifest signed.
        let feed = fixture_feed();
        fs::write(feed.path().join("batna-0.1.0-b42-android-arm64.apk"), b"evil apk bytes!!!").unwrap();
        let url = serve(feed.path().to_path_buf());
        let (u, data) = updater(&url, build("0.1.0", 41, true));
        u.check(true);
        assert!(
            matches!(u.status().phase, Phase::Failed { why: Failure::Verify(_), .. }),
            "{:?}",
            u.status().phase
        );
        let left: Vec<_> =
            fs::read_dir(data.path().join("updates")).unwrap().flatten().map(|e| e.file_name()).collect();
        assert!(
            !left.iter().any(|n| n.to_string_lossy().ends_with(".apk")),
            "nothing unverified stays: {left:?}"
        );

        // The manifest changed after signing: a preview calls it a renewed key.
        let feed = fixture_feed();
        let json = fs::read_to_string(feed.path().join(MANIFEST_FILE)).unwrap();
        fs::write(feed.path().join(MANIFEST_FILE), json.replace("\"build\": 42", "\"build\": 99")).unwrap();
        let url = serve(feed.path().to_path_buf());
        let (u, _data) = updater(&url, build("0.1.0", 41, true));
        u.check(true);
        assert!(matches!(u.status().phase, Phase::Failed { why: Failure::KeyChanged, .. }));

        // No connection.
        let (u, _data) = updater("http://127.0.0.1:9", build("0.1.0", 41, true));
        u.check(false);
        assert!(matches!(u.status().phase, Phase::Failed { why: Failure::Network, .. }));
    }

    #[test]
    fn a_content_pack_must_verify_and_decode_before_it_is_kept() {
        // The fixture's pack bytes match the manifest but aren't a real pack.
        let feed = fixture_feed();
        let url = serve(feed.path().to_path_buf());
        let (u, data) = updater(&url, build("0.1.0", 42, true));
        u.check(false);
        assert!(!u.status().content_waiting);
        assert!(!data.path().join("updates").join(PACK_FILE).exists());

        // A real pack that matches the signed manifest is kept for the next start.
        let pack = Pack::new(crate::content::Library::get().index.clone(), Vec::new()).encode();
        let signer = ed25519_dalek::SigningKey::from_bytes(&[5; 32]);
        let feed = tempfile::tempdir().unwrap();
        let manifest = UpdateManifest {
            format: batna_core::update::UPDATE_FORMAT,
            channel: Channel::Preview,
            version: "0.1.0".into(),
            build: 42,
            commit: None,
            published_at: "2026-10-27T08:00:00Z".into(),
            notes: UpdateNotes::default(),
            history: Vec::new(),
            assets: vec![UpdateAsset {
                platform: UpdatePlatform::Content,
                file: "batna-content.pack".into(),
                size: pack.len() as u64,
                sha256: sha256_hex(&pack),
                version_code: None,
            }],
        };
        let json = serde_json::to_vec(&manifest).unwrap();
        fs::write(feed.path().join(MANIFEST_FILE), &json).unwrap();
        use base64::Engine as _;
        use ed25519_dalek::Signer as _;
        let sig = base64::engine::general_purpose::STANDARD.encode(signer.sign(&json).to_bytes());
        fs::write(feed.path().join(SIGNATURE_FILE), sig).unwrap();
        fs::write(feed.path().join("batna-content.pack"), &pack).unwrap();
        let url = serve(feed.path().to_path_buf());
        let info = BuildInfo { key: Some(signer.verifying_key()), ..build("0.1.0", 42, true) };
        let (u, data) = updater(&url, info.clone());
        u.check(false);
        assert_eq!(u.status().phase, Phase::UpToDate, "the same build: only the pack is new");
        assert!(u.status().content_waiting);
        let dir = data.path().join("updates");
        assert_eq!(installed_pack_in(&dir, &info).unwrap().lectures.len(), 0);
        // A later build drops a pack from an older one.
        assert!(installed_pack_in(&dir, &BuildInfo { build: 43, ..info }).is_none());
        assert!(!dir.join(PACK_FILE).exists());
    }
}
