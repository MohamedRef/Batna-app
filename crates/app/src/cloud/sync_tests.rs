//! The app's real sync engine, two devices at a time, against Batna Cloud's
//! own hub on 127.0.0.1 (`fake.rs`): an answer on the phone shows on the
//! tablet, the newest setting wins a conflict, an offline queue flushes,
//! remote sign-out, Google sign-in and the problems Mohamed would see.

use std::time::{Duration, Instant};

use batna_core::review::{Grade, Origin, ReviewEvent};
use batna_protocol::cloud::{DeviceInfo, Platform, Session};
use tokio::sync::mpsc::error::TryRecvError;

use super::fake::{self, FakeCloud};
use super::oauth::{Endpoints, GoogleClient};
use super::session::{MemoryVault, SessionVault};
use super::{CloudConfig, CloudHandle, CloudStatus, Command, Event, LoginStatus, Problem, SyncState};
use crate::store::Store;

const WAIT: Duration = Duration::from_secs(20);

/// A vault the test can still read after the engine took it.
#[derive(Clone, Default)]
struct SharedVault(std::sync::Arc<MemoryVault>);

impl SessionVault for SharedVault {
    fn load(&self) -> Option<Session> {
        self.0.load()
    }
    fn save(&self, session: &Session) -> bool {
        self.0.save(session)
    }
    fn clear(&self) {
        self.0.clear()
    }
}

pub(super) struct Device {
    pub(super) store: Store,
    handle: CloudHandle,
    vault: SharedVault,
    status: Option<CloudStatus>,
    /// Events that arrived while waiting for something else.
    seen: std::collections::VecDeque<Event>,
    _dir: tempfile::TempDir,
}

impl Device {
    /// A new install. `session` picks what its vault holds at launch.
    fn new(
        cloud: &FakeCloud,
        name: &str,
        google: Option<GoogleClient>,
        session: impl FnOnce(&FakeCloud, &DeviceInfo) -> Option<Session>,
    ) -> Self {
        Self::open(&cloud.url, name, google, |i| session(cloud, i))
    }

    /// A new install against the Batna Cloud at `url`.
    pub(super) fn open(
        url: &str,
        name: &str,
        google: Option<GoogleClient>,
        session: impl FnOnce(&DeviceInfo) -> Option<Session>,
    ) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("batna.db");
        let store = Store::open(&path).unwrap();
        let vault = SharedVault::default();
        if let Some(s) = session(&info(&store, name)) {
            vault.save(&s);
        }
        let config = CloudConfig { base_url: url.into(), google };
        let handle = super::start(config, path, info(&store, name), Box::new(vault.clone())).unwrap();
        Self { store, handle, vault, status: None, seen: Default::default(), _dir: dir }
    }

    /// A device that signed in on this server before.
    fn signed_in(cloud: &FakeCloud, name: &str) -> Self {
        Self::new(cloud, name, None, |c, i| Some(c.session_for(i)))
    }

    /// Signed in before, with its saved session changed by `edit`.
    fn with_session(cloud: &FakeCloud, name: &str, edit: impl FnOnce(&mut Session)) -> Self {
        Self::new(cloud, name, None, |c, i| {
            let mut s = c.session_for(i);
            edit(&mut s);
            Some(s)
        })
    }

    pub(super) fn id(&self) -> String {
        self.store.device()
    }

    pub(super) fn send(&self, c: Command) {
        self.handle.commands.send(c).unwrap();
    }

    /// Waits for the first event `f` accepts; statuses are remembered.
    pub(super) fn wait<T>(&mut self, what: &str, mut f: impl FnMut(&Event) -> Option<T>) -> T {
        if let Some(i) = self.seen.iter().position(|e| f(e).is_some()) {
            let event = self.seen.remove(i).unwrap();
            return f(&event).unwrap();
        }
        let deadline = Instant::now() + WAIT;
        loop {
            assert!(Instant::now() < deadline, "timed out waiting for {what}; last status {:?}", self.status);
            match self.handle.events.try_recv() {
                Ok(event) => {
                    if let Event::Status(s) = &event {
                        self.status = Some(s.clone());
                    }
                    if let Some(v) = f(&event) {
                        return v;
                    }
                    // A status is state, not news: only the latest counts.
                    if !matches!(event, Event::Status(_)) {
                        self.seen.push_back(event);
                    }
                }
                Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => panic!("engine stopped: {e}"),
            }
        }
    }

    /// Waits until the status (the last one, or a new one) passes `f`.
    pub(super) fn wait_status(&mut self, what: &str, f: impl Fn(&CloudStatus) -> bool) -> CloudStatus {
        if let Some(s) = self.status.clone().filter(|s| f(s)) {
            return s;
        }
        self.wait(what, |e| match e {
            Event::Status(s) if f(s) => Some(s.clone()),
            _ => None,
        })
    }

    pub(super) fn wait_live(&mut self) -> CloudStatus {
        self.wait_status("live", |s| s.sync == SyncState::Live)
    }

    pub(super) fn wait_changed(&mut self, table: &str) {
        self.wait(table, |e| matches!(e, Event::Changed(t) if t.contains(table)).then_some(()));
    }

    pub(super) fn devices(&mut self) -> Result<Vec<batna_protocol::cloud::DeviceSummary>, Problem> {
        self.send(Command::LoadDevices);
        self.wait("devices", |e| match e {
            Event::Devices(d) => Some(d.clone()),
            _ => None,
        })
    }
}

impl Device {
    /// Waits for what Settings would show about the username and password.
    fn login(&mut self, what: &str, f: impl Fn(&LoginStatus) -> bool) -> LoginStatus {
        self.wait(what, |e| match e {
            Event::Login(l) if f(l) => Some(l.clone()),
            _ => None,
        })
    }

    /// Sets the account's username and password and waits for Batna Cloud's yes.
    fn set_login(&mut self, username: &str, password: &str) -> LoginStatus {
        self.send(Command::SetLogin { username: username.into(), password: password.into() });
        self.login("saved", |l| l.saved)
    }

    /// Signs in with a username and password and waits until it's answered.
    fn sign_in_password(&mut self, username: &str, password: &str, remember: bool) {
        let command =
            Command::SignInPassword { username: username.into(), password: password.into(), remember };
        self.send(command);
        self.wait("password checked", |e| matches!(e, Event::SigningIn(false)).then_some(()));
    }

    fn logged_out(cloud: &FakeCloud, name: &str) -> Self {
        Self::new(cloud, name, None, |_, _| None)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.handle.commands.send(Command::Shutdown).ok();
    }
}

fn info(store: &Store, name: &str) -> DeviceInfo {
    DeviceInfo { device_id: store.device(), name: name.into(), model: None, platform: Platform::Android }
}

pub(super) fn answer(item: &str) -> ReviewEvent {
    ReviewEvent {
        item_id: item.into(),
        grade: Grade::Good,
        origin: Origin::Review,
        at_ms: crate::store::now_ms(),
        took_ms: 4000,
    }
}

pub(super) fn eventually(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + WAIT;
    while !f() {
        assert!(Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn an_answer_on_the_phone_shows_on_the_tablet() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "Galaxy S24 Ultra");
    let mut tablet = Device::signed_in(&cloud, "SM-X730");
    let live = phone.wait_live();
    assert_eq!(live.email.as_deref(), Some(fake::EMAIL));
    assert!(live.configured && live.problem.is_none());
    tablet.wait_live();

    phone.store.append_review(answer("hb.L01.003")).unwrap();
    phone.send(Command::Kick);
    tablet.wait_changed("review_events");
    let reviews = tablet.store.reviews().unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].event.item_id, "hb.L01.003");
    assert_eq!(reviews[0].hlc.device, phone.id());
    let s = phone.wait_status("outbox empty", |s| s.pending == 0 && s.last_sync_ms.is_some());
    assert_eq!(s.sync, SyncState::Live);

    // The Devices card: both, this one marked, the tablet last seen syncing.
    let list = phone.devices().unwrap();
    assert_eq!(list.len(), 2);
    let me = list.iter().find(|d| d.current).unwrap();
    assert_eq!(me.name, "Galaxy S24 Ultra");
    assert!(list.iter().any(|d| d.device_id == tablet.id() && !d.current && !d.revoked));
}

#[test]
fn the_newest_setting_wins_on_both_devices() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    let mut tablet = Device::signed_in(&cloud, "tablet");
    phone.wait_live();
    tablet.wait_live();
    // Both change the daily limit while the line is down: the tablet last.
    cloud.set_offline(true);
    phone.wait_status("offline", |s| s.sync != SyncState::Live);
    tablet.wait_status("offline", |s| s.sync != SyncState::Live);
    phone.store.put_setting("daily_new", &15).unwrap();
    std::thread::sleep(Duration::from_millis(5));
    tablet.store.put_setting("daily_new", &30).unwrap();
    phone.send(Command::Kick);
    let waiting = phone.wait_status("queued", |s| s.pending == 1);
    assert_ne!(waiting.sync, SyncState::Live);

    cloud.set_offline(false);
    eventually("both agree", || {
        phone.store.setting::<u32>("daily_new") == Some(30)
            && tablet.store.setting::<u32>("daily_new") == Some(30)
    });
    let on_hub = cloud.payload("settings", "daily_new").unwrap().to_string();
    assert!(on_hub.contains("30"), "{on_hub}");
    phone.wait_status("phone pushed", |s| s.pending == 0 && s.sync == SyncState::Live);
    tablet.wait_status("tablet pushed", |s| s.pending == 0 && s.sync == SyncState::Live);
}

#[test]
fn answers_made_offline_go_up_when_the_line_is_back() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    cloud.set_offline(true);
    phone.wait_status("offline", |s| s.sync == SyncState::Offline && s.problem == Some(Problem::Network));
    for item in ["hb.L02.001", "hb.L02.002", "hb.L02.003"] {
        phone.store.append_review(answer(item)).unwrap();
    }
    phone.send(Command::Kick);
    phone.wait_status("three waiting", |s| s.pending == 3);

    cloud.set_offline(false);
    let s = phone.wait_status("flushed", |s| s.sync == SyncState::Live && s.pending == 0);
    assert_eq!(s.problem, None);

    // A device signing in later gets them in its catch-up.
    let mut tablet = Device::signed_in(&cloud, "tablet");
    tablet.wait_live();
    let mut items: Vec<String> =
        tablet.store.reviews().unwrap().into_iter().map(|r| r.event.item_id).collect();
    items.sort();
    assert_eq!(items, ["hb.L02.001", "hb.L02.002", "hb.L02.003"]);
}

#[test]
fn signing_out_another_device_stops_it() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    let mut tablet = Device::signed_in(&cloud, "tablet");
    phone.wait_live();
    tablet.wait_live();
    phone.send(Command::RevokeDevice(tablet.id()));
    let s = tablet.wait_status("tablet out", |s| s.sync == SyncState::SignedOut);
    assert_eq!(s.problem, Some(Problem::SignedOutRemotely));
    assert_eq!(s.email, None);
    assert!(tablet.vault.load().is_none(), "the session is forgotten");
    let list = phone.wait("devices after revoke", |e| match e {
        Event::Devices(Ok(d)) => Some(d.clone()),
        _ => None,
    });
    assert!(list.iter().any(|d| d.device_id == tablet.id() && d.revoked));

    // Signing out "this device" from the list is a plain sign-out.
    phone.send(Command::RevokeDevice(phone.id()));
    let s = phone.wait_status("phone out", |s| s.sync == SyncState::SignedOut);
    assert_eq!(s.problem, None);
    phone.wait("devices cleared", |e| matches!(e, Event::Devices(Ok(d)) if d.is_empty()).then_some(()));
    assert!(phone.vault.load().is_none());
    assert_eq!(phone.store.last_seq(), 0, "the next account reads its log from the start");
}

#[test]
fn a_device_signed_out_while_away_finds_out_on_its_next_refresh() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    let gone = std::sync::Arc::new(std::sync::Mutex::new(None));
    let seen = gone.clone();
    // The tablet was signed out from the phone while it was off; it comes
    // back with an expired access token, and the refresh says why.
    let mut tablet = Device::new(&cloud, "tablet", None, |c, i| {
        let mut s = c.session_for(i);
        phone.send(Command::RevokeDevice(i.device_id.clone()));
        *seen.lock().unwrap() = Some(i.device_id.clone());
        // Give the phone's revoke time to land before the tablet's engine starts.
        std::thread::sleep(Duration::from_millis(300));
        s.access_expires_at_ms = 1;
        Some(s)
    });
    let list = phone.wait("revoked", |e| match e {
        Event::Devices(Ok(d)) => Some(d.clone()),
        _ => None,
    });
    let id = gone.lock().unwrap().clone().unwrap();
    assert!(list.iter().any(|d| d.device_id == id && d.revoked));
    let s = tablet.wait_status("signed out", |s| s.sync == SyncState::SignedOut && s.problem.is_some());
    assert_eq!(s.problem, Some(Problem::SignedOutRemotely));
}

#[test]
fn sign_out_forgets_the_session_and_tells_the_hub() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    let mut tablet = Device::signed_in(&cloud, "tablet");
    phone.wait_live();
    tablet.wait_live();
    let phone_id = phone.id();
    phone.send(Command::SignOut);
    phone.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    assert!(phone.vault.load().is_none());
    let list = tablet.devices().unwrap();
    assert!(list.iter().any(|d| d.device_id == phone_id && d.revoked), "{list:?}");
}

#[test]
fn an_expired_access_token_is_refreshed_before_connecting() {
    let cloud = FakeCloud::start();
    let old = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let keep = old.clone();
    let mut phone = Device::with_session(&cloud, "phone", |s| {
        s.access_expires_at_ms = 1;
        *keep.lock().unwrap() = s.refresh_token.clone();
    });
    phone.wait_live();
    assert_eq!(cloud.refreshes(), 1);
    let kept = phone.vault.load().unwrap();
    assert_ne!(kept.refresh_token, *old.lock().unwrap(), "the rotated token is kept for the next launch");
    assert!(kept.access_expires_at_ms > crate::store::now_ms() as u64);
}

#[test]
fn a_refused_upgrade_refreshes_and_tries_again() {
    let cloud = FakeCloud::start();
    cloud.refuse_upgrades(1);
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    assert_eq!(cloud.refreshes(), 1, "a 401 on the socket means: refresh first");
}

#[test]
fn a_clock_running_ahead_holds_answers_back_and_says_why() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    // The hub's clock is 10 minutes behind this phone's.
    cloud.set_skew_ms(-10 * 60 * 1000);
    phone.store.append_review(answer("hb.L03.001")).unwrap();
    phone.send(Command::Kick);
    let s = phone.wait_status("clock off", |s| s.problem == Some(Problem::ClockOff));
    assert_eq!(s.pending, 1, "the answer stays in the outbox");
    assert_eq!(s.sync, SyncState::Live, "pulls go on");
}

#[test]
fn an_older_app_stops_and_asks_for_an_update() {
    let cloud = FakeCloud::start();
    cloud.set_outdated(true);
    let mut phone = Device::signed_in(&cloud, "phone");
    let s = phone.wait_status("stopped", |s| s.sync == SyncState::Stopped);
    assert_eq!(s.problem, Some(Problem::Outdated));
    assert_eq!(s.email.as_deref(), Some(fake::EMAIL), "still signed in");
}

#[test]
fn hub_hiccups_reconnect_and_junk_is_ignored() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    let id = phone.id();
    cloud.send_raw(&id, "{not json");
    cloud.send_raw(&id, r#"{"type":"error","code":"internal","message":"db busy"}"#);
    phone.wait_status("reconnecting", |s| s.sync != SyncState::Live);
    phone.wait_live();
    // The edge drops the socket: it comes back by itself.
    cloud.close(&id, 1001);
    phone.wait_status("dropped", |s| s.sync != SyncState::Live);
    let s = phone.wait_live();
    assert_eq!(s.problem, None);
    // Binary frames are ignored; a line cut without goodbye reconnects.
    cloud.send_binary(&id);
    cloud.cut(&id);
    phone.wait_status("cut", |s| s.sync != SyncState::Live);
    phone.wait_live();
    // 4001 means this device was signed out elsewhere.
    cloud.close(&id, 4001);
    let s = phone.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    assert_eq!(s.problem, Some(Problem::SignedOutRemotely));
}

#[test]
fn a_write_without_a_kick_still_goes_up_within_seconds() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    let mut tablet = Device::signed_in(&cloud, "tablet");
    phone.wait_live();
    tablet.wait_live();
    // A screen that doesn't send Kick: the two-second outbox tick sends it.
    phone.store.put_setting("theme_hint", &"night").unwrap();
    tablet.wait_changed("settings");
    assert_eq!(tablet.store.setting::<String>("theme_hint").as_deref(), Some("night"));
}

#[test]
fn a_token_the_hub_finds_stale_is_refreshed_once_and_the_call_retried() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    // The phone's clock runs behind the hub's: its "fresh" token is old there.
    cloud.unauthorized_next(1);
    assert_eq!(phone.devices().unwrap().len(), 1);
    assert_eq!(cloud.refreshes(), 1);
    assert_eq!(phone.status.as_ref().unwrap().sync, SyncState::Live, "not signed out");
}

#[test]
fn server_errors_on_the_devices_list_say_so() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "phone");
    phone.wait_live();
    cloud.fail_next(1);
    assert_eq!(phone.devices(), Err(Problem::Server));
    cloud.set_offline(true);
    assert_eq!(phone.devices(), Err(Problem::Network));
    cloud.set_offline(false);
    phone.wait_live();
    assert_eq!(phone.devices().unwrap().len(), 1);
    // Revoking a device the hub never heard of is a server-side "no".
    phone.send(Command::RevokeDevice("ghost".into()));
    let list = phone.wait("list after a failed revoke", |e| match e {
        Event::Devices(d) => Some(d.clone()),
        _ => None,
    });
    assert_eq!(list.map(|l| l.len()), Ok(1));
    phone.wait_status("problem", |s| s.problem == Some(Problem::Server));
}

// --- Google sign-in, with a stand-in browser and Google ---------------------

/// Plays the browser: reads where Google would send it back and goes there
/// with the code the client id asks for (`ok`, `stranger`, `cancel`).
fn browser(url: &str) -> Result<(), String> {
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
    let param = |name: &str| {
        query.split('&').find_map(|p| p.strip_prefix(&format!("{name}="))).map(super::oauth::decode)
    };
    let (Some(redirect), Some(state), Some(client)) =
        (param("redirect_uri"), param("state"), param("client_id"))
    else {
        return Err("no redirect".into());
    };
    if client == "no-browser" {
        return Err("xdg-open: not found".into());
    }
    let back = match client.as_str() {
        "cancel" => format!("{redirect}/?error=access_denied&state={state}"),
        "stranger" => format!("{redirect}/?state={state}&code={}", fake::CODE_STRANGER),
        "bad-code" => format!("{redirect}/?state={state}&code=expired"),
        _ => format!("{redirect}/?state={state}&code={}", fake::CODE_OK),
    };
    std::thread::spawn(move || {
        // A stray first, as browsers ask for the favicon.
        let _ = ureq::get(&format!("{redirect}/favicon.ico")).call();
        let page = ureq::get(&back).call().unwrap().body_mut().read_to_string().unwrap();
        assert!(page.contains("lang=\"ar\""));
    });
    Ok(())
}

fn google(cloud: &FakeCloud, client_id: &str) -> GoogleClient {
    GoogleClient {
        client_id: client_id.into(),
        client_secret: Some("not-a-secret".into()),
        endpoints: Endpoints {
            auth: format!("{}/google/auth", cloud.url),
            token: format!("{}/google/token", cloud.url),
            open: browser,
        },
    }
}

fn sign_in(cloud: &FakeCloud, client_id: &str) -> Device {
    let mut d = Device::new(cloud, "phone", Some(google(cloud, client_id)), |_, _| None);
    d.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    d.send(Command::SignInGoogle);
    d.wait("browser open", |e| matches!(e, Event::SigningIn(true)).then_some(()));
    d
}

#[test]
fn google_sign_in_syncs_the_account_down() {
    let cloud = FakeCloud::start();
    let mut tablet = Device::signed_in(&cloud, "tablet");
    tablet.wait_live();
    tablet.store.put_setting("onboard", &serde_json::json!({ "minutes": 15 })).unwrap();
    tablet.send(Command::Kick);
    tablet.wait_status("pushed", |s| s.pending == 0);

    let mut phone = sign_in(&cloud, "ok");
    phone.wait("browser closed", |e| matches!(e, Event::SigningIn(false)).then_some(()));
    let s = phone.wait_live();
    assert_eq!(s.email.as_deref(), Some(fake::EMAIL));
    assert!(!s.finishing_sign_in);
    assert_eq!(phone.vault.load().unwrap().device_id, phone.id());
    eventually("setting came down", || {
        phone.store.setting::<serde_json::Value>("onboard").is_some_and(|v| v["minutes"] == 15)
    });
}

#[test]
fn google_sign_in_problems_are_named() {
    let cloud = FakeCloud::start();
    for (client, problem) in [
        ("cancel", Problem::SignInCancelled),
        ("stranger", Problem::NotAllowed),
        ("bad-code", Problem::SignInFailed),
        ("no-browser", Problem::SignInFailed),
    ] {
        let mut phone = sign_in(&cloud, client);
        let s = phone.wait_status(client, |s| s.problem.is_some());
        assert_eq!(s.problem, Some(problem), "{client}");
        assert_eq!(s.sync, SyncState::SignedOut, "{client}");
        assert!(phone.vault.load().is_none());
    }
    // A build without a Google client says sign-in isn't set up here.
    let mut phone = Device::new(&cloud, "phone", None, |_, _| None);
    phone.send(Command::SignInGoogle);
    let s = phone.wait_status("not configured", |s| s.problem.is_some());
    assert_eq!(s.problem, Some(Problem::NotConfigured));
}

#[test]
fn a_sign_in_approved_while_offline_finishes_once_the_network_is_back() {
    let cloud = FakeCloud::start();
    // Android keeps a backgrounded app off the network: the first exchanges fail.
    cloud.drop_next("/google/token", 2);
    let mut phone = sign_in(&cloud, "ok");
    let s = phone.wait_status("waiting", |s| s.problem == Some(Problem::SignInWaiting));
    assert!(s.finishing_sign_in);
    // Back in front: Kick retries now rather than on the timer.
    phone.send(Command::Kick);
    let s = phone.wait_live();
    assert_eq!(s.problem, None);
    assert!(!s.finishing_sign_in);
}

#[test]
fn batna_cloud_out_of_reach_during_sign_in_is_retried() {
    let cloud = FakeCloud::start();
    cloud.drop_next("/v1/auth/google", 1);
    let mut phone = sign_in(&cloud, "ok");
    let s = phone.wait_status("waiting", |s| s.problem == Some(Problem::SignInWaiting));
    assert!(s.finishing_sign_in);
    assert_eq!(phone.wait_live().email.as_deref(), Some(fake::EMAIL));
}

#[test]
fn a_session_the_device_cannot_keep_is_not_used() {
    let cloud = FakeCloud::start();
    let config = CloudConfig { base_url: cloud.url.clone(), google: Some(google(&cloud, "ok")) };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("batna.db");
    let store = Store::open(&path).unwrap();
    let mut handle = super::start(config, path, info(&store, "phone"), Box::new(Forgetful)).unwrap();
    handle.commands.send(Command::SignInGoogle).unwrap();
    let deadline = Instant::now() + WAIT;
    loop {
        assert!(Instant::now() < deadline);
        match handle.events.try_recv() {
            Ok(Event::Status(s)) if s.problem.is_some() => {
                assert_eq!(s.problem, Some(Problem::SignInFailed));
                assert_eq!(s.sync, SyncState::SignedOut);
                break;
            }
            Ok(_) => {}
            Err(_) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    handle.commands.send(Command::Shutdown).ok();
}

/// A credential store that keeps nothing (a locked Keystore).
struct Forgetful;

impl SessionVault for Forgetful {
    fn load(&self) -> Option<Session> {
        None
    }
    fn save(&self, _: &Session) -> bool {
        false
    }
    fn clear(&self) {}
}

#[test]
fn batna_cloud_failing_a_sign_in_is_not_retried() {
    let cloud = FakeCloud::start();
    let mut phone = Device::new(&cloud, "phone", Some(google(&cloud, "ok")), |_, _| None);
    phone.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    // Google answers; Batna Cloud's edge then answers 502. Armed before the
    // sign-in starts: the fake Google answers at once, so arming it after
    // `sign_in` returned raced the exchange on a busy runner.
    cloud.fail_next(1);
    phone.send(Command::SignInGoogle);
    let s = phone.wait_status("failed", |s| s.problem.is_some());
    assert_eq!(s.problem, Some(Problem::SignInFailed));
    assert!(!s.finishing_sign_in);
}

#[test]
fn an_old_saved_session_is_not_used() {
    let cloud = FakeCloud::start();
    let mut phone = Device::with_session(&cloud, "phone", |s| s.refresh_expires_at_ms = 1);
    let s = phone.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    assert_eq!(s.email, None);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(cloud.sockets(), 0);
}

#[test]
fn every_device_names_itself() {
    let d = super::device_info("dev-1".into());
    assert_eq!(d.device_id, "dev-1");
    assert_eq!((d.name.as_str(), d.platform), ("Batna (dev)", Platform::Other));
    assert_eq!(d.model, None);
}

/// Keeps the first session it is given and fails every later save (a
/// Keystore that locks after sign-in).
#[derive(Default)]
struct SavesOnce(std::sync::Mutex<Option<Session>>);

impl SessionVault for SavesOnce {
    fn load(&self) -> Option<Session> {
        self.0.lock().unwrap().clone()
    }
    fn save(&self, session: &Session) -> bool {
        let mut kept = self.0.lock().unwrap();
        if kept.is_some() {
            return false;
        }
        *kept = Some(session.clone());
        true
    }
    fn clear(&self) {}
}

#[test]
fn a_refresh_the_vault_cannot_keep_goes_on_with_the_saved_refresh_token() {
    let cloud = FakeCloud::start();
    let store = Store::open_in_memory().unwrap();
    let session = cloud.session_for(&info(&store, "phone"));
    let mut auth = super::session::Auth::new(session.clone(), Box::new(SavesOnce::default())).ok().unwrap();
    let api = super::api::Api::new(&cloud.url);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let now = crate::store::now_ms() as u64;
    runtime.block_on(auth.access_token(&api, now, true)).unwrap();
    assert_eq!(auth.session().refresh_token, session.refresh_token, "the one the next launch will load");
    // The hub still takes it, so this device never gets signed out by it.
    runtime.block_on(auth.access_token(&api, now, true)).unwrap();
    assert_eq!(cloud.refreshes(), 2);
}

const PASSWORD: &str = "correct horse battery";

#[test]
fn a_password_set_on_the_phone_signs_in_a_borrowed_computer() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "Galaxy S24 Ultra");
    phone.wait_live();
    phone.send(Command::LoadLogin);
    let none = phone.login("no username yet", |l| l.username.is_some());
    assert_eq!(none.username, Some(None));
    assert!(!none.saved, "reading is not saving");

    phone.send(Command::SetLogin { username: " Mo.Refaat ".into(), password: PASSWORD.into() });
    let busy = phone.login("saving", |l| l.saving);
    assert!(!busy.saved && busy.problem.is_none());
    let saved = phone.login("saved", |l| l.saved);
    assert_eq!(saved.username, Some(Some("mo.refaat".into())), "stored as typed, tidied");
    assert!(!saved.saving);
    phone.store.append_review(answer("hb.L01.003")).unwrap();
    phone.send(Command::Kick);
    phone.wait_status("sent", |s| s.pending == 0 && s.last_sync_ms.is_some());

    // Someone else's computer: no Google, and nothing kept after it closes.
    let mut pc = Device::logged_out(&cloud, "Library PC");
    pc.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    pc.sign_in_password("MO.REFAAT", PASSWORD, false);
    let live = pc.wait_live();
    assert!(live.temporary, "Settings says it ends with the app");
    assert_eq!(live.email.as_deref(), Some(fake::EMAIL));
    eventually("the phone's answer", || pc.store.reviews().unwrap().len() == 1);
    assert!(pc.vault.load().is_none(), "the session stays out of the device's own vault");

    // After sign-out the device's own vault is back: a remembered sign-in keeps its session.
    pc.send(Command::SignOut);
    let out = pc.wait_status("signed out again", |s| s.sync == SyncState::SignedOut);
    assert!(!out.temporary);
    pc.login("forgotten", |l| *l == LoginStatus::default());
    pc.sign_in_password("mo.refaat", PASSWORD, true);
    assert!(!pc.wait_live().temporary);
    assert!(pc.vault.load().is_some(), "remembered");
}

#[test]
fn wrong_passwords_say_so_then_wait_a_minute() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "Galaxy S24 Ultra");
    phone.wait_live();
    phone.set_login("mo.refaat", PASSWORD);

    let mut pc = Device::logged_out(&cloud, "Library PC");
    pc.sign_in_password("mo.refaat", "correct horse batterY", true);
    pc.wait_status("wrong", |s| s.problem == Some(Problem::WrongPassword));
    // A name nobody has gets the same answer, so names can't be guessed.
    pc.sign_in_password("nobody.here", PASSWORD, true);
    pc.sign_in_password("x", PASSWORD, true);
    // Five wrong ones for this name lock it.
    for _ in 0..4 {
        pc.sign_in_password("mo.refaat", "wrong again, sorry", true);
    }
    let locked = pc.wait_status("locked", |s| matches!(s.problem, Some(Problem::Locked(_))));
    assert_eq!(locked.problem, Some(Problem::Locked(1)), "a minute, said in whole minutes");
    assert_eq!(locked.sync, SyncState::SignedOut);
    pc.sign_in_password("mo.refaat", PASSWORD, true);
    assert!(pc.vault.load().is_none(), "even the right password waits");

    // A new password from the phone lifts the lock.
    phone.set_login("mo.refaat", "a brand new password");
    pc.sign_in_password("mo.refaat", "a brand new password", true);
    pc.wait_live();
}

#[test]
fn a_new_login_is_checked_and_a_taken_name_refused() {
    let cloud = FakeCloud::start();
    let mut phone = Device::signed_in(&cloud, "Galaxy S24 Ultra");
    phone.wait_live();
    let problem = |phone: &mut Device, username: &str, password: &str| {
        phone.send(Command::SetLogin { username: username.into(), password: password.into() });
        phone.login("refused", |l| l.problem.is_some()).problem.unwrap()
    };
    assert_eq!(problem(&mut phone, "mo", PASSWORD), Problem::BadUsername);
    assert_eq!(problem(&mut phone, "mo.refaat", "short"), Problem::ShortPassword);
    cloud.take_username("dr.mo");
    assert_eq!(problem(&mut phone, "dr.mo", PASSWORD), Problem::UsernameTaken);

    phone.set_login("mo.refaat", PASSWORD);
    let renamed = phone.set_login("mo.refaat.md", PASSWORD);
    assert_eq!(renamed.username, Some(Some("mo.refaat.md".into())));
    let mut pc = Device::logged_out(&cloud, "Library PC");
    pc.sign_in_password("mo.refaat", PASSWORD, true);
    pc.wait_status("the old name is free", |s| s.problem == Some(Problem::WrongPassword));

    phone.send(Command::RemoveLogin);
    phone.login("removing", |l| l.saving);
    let removed = phone.login("removed", |l| l.saved);
    assert_eq!(removed.username, Some(None));
    pc.sign_in_password("mo.refaat.md", PASSWORD, true);
    pc.wait_status("no password any more", |s| s.problem == Some(Problem::WrongPassword));
}

#[test]
fn login_problems_reach_settings() {
    let cloud = FakeCloud::start();
    let mut pc = Device::logged_out(&cloud, "Library PC");
    pc.wait_status("signed out", |s| s.sync == SyncState::SignedOut);
    pc.send(Command::LoadLogin);
    assert_eq!(pc.login("no account", |l| l.problem.is_some()).problem, Some(Problem::Network));

    cloud.set_offline(true);
    pc.sign_in_password("mo.refaat", PASSWORD, true);
    pc.wait_status("no network", |s| s.problem == Some(Problem::Network));
    cloud.set_offline(false);

    let mut phone = Device::signed_in(&cloud, "Galaxy S24 Ultra");
    phone.wait_live();
    cloud.fail_next(1);
    phone.send(Command::LoadLogin);
    assert_eq!(phone.login("server", |l| l.problem.is_some()).problem, Some(Problem::Server));
    phone.send(Command::LoadLogin);
    let l = phone.login("answered", |l| l.username.is_some());
    assert_eq!(l.problem, None, "the next answer clears it");
}

/// Two web tabs share one session (`localStorage`) and refresh it in turn,
/// each from its own copy in memory. Neither may present a token the other
/// already rotated, or the hub signs the device out as a reused token (#646).
#[test]
fn two_tabs_sharing_a_session_take_turns_refreshing_and_stay_signed_in() {
    use super::api::Api;
    use super::session::Auth;
    let cloud = FakeCloud::start();
    let store = Store::open_in_memory().unwrap();
    let vault = SharedVault::default();
    assert!(vault.save(&cloud.session_for(&info(&store, "web"))));
    let api = Api::new(&cloud.url);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let now = crate::store::now_ms() as u64;
    let mut a = Auth::restore(Box::new(vault.clone()), now).ok().unwrap();
    let mut b = Auth::restore(Box::new(vault.clone()), now).ok().unwrap();
    for round in 0..4 {
        // Each forced refresh stands for an access token running out.
        let tab = if round % 2 == 0 { &mut a } else { &mut b };
        rt.block_on(tab.access_token(&api, now, true)).unwrap_or_else(|e| panic!("round {round}: {e}"));
        // The other tab, behind by one rotation, takes the kept session
        // instead of refreshing with its stale token.
        let other = if round % 2 == 0 { &mut b } else { &mut a };
        rt.block_on(other.access_token(&api, now, true)).unwrap_or_else(|e| panic!("round {round}: {e}"));
        assert_eq!(a.session().refresh_token, b.session().refresh_token, "round {round}");
    }
    assert_eq!(cloud.refreshes(), 4, "one refresh per round, not two");
    assert_eq!(vault.load().unwrap().refresh_token, a.session().refresh_token);
}
