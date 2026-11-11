//! Settings › Sync: Google sign-in, the sync state, and the Devices list
//! with remote sign-out. `use_provide_cloud` starts the engine once (when
//! this build knows a Batna Cloud URL) and mirrors its events into signals.
// adapted from kabar-app main: crates/app/src/account.rs and main.rs (use_cloud)

use batna_protocol::cloud::{DeviceSummary, Platform};
use dioxus::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use super::{CloudStatus, Command, Event, LoginStatus, Problem, SyncState};
use crate::i18n::cloud as tx;

/// Batna Cloud state for the UI, shared through context.
#[derive(Clone, Copy)]
pub struct Cloud {
    pub status: Signal<CloudStatus>,
    pub devices: Signal<Option<Result<Vec<DeviceSummary>, Problem>>>,
    pub signing_in: Signal<bool>,
    pub login: Signal<LoginStatus>,
    commands: Signal<Option<UnboundedSender<Command>>>,
}

impl Cloud {
    pub fn send(&self, command: Command) {
        if let Some(tx) = &*self.commands.peek() {
            tx.send(command).ok();
        }
    }
}

/// Provides [`Cloud`] and, when this build knows a server, starts sync.
/// Call once in `App`, after `store::use_provide_review`.
pub fn use_provide_cloud() {
    let store = crate::store::shared();
    let device_id = store.device();
    let cloud = use_context_provider(|| Cloud {
        status: Signal::new(CloudStatus::not_configured(device_id.clone())),
        devices: Signal::new(None),
        signing_in: Signal::new(false),
        login: Signal::new(LoginStatus::default()),
        commands: Signal::new(None),
    });
    let review = use_context::<crate::store::Review>();
    use_hook(move || {
        let Some(config) = super::CloudConfig::from_env() else { return };
        let device = super::device_info(device_id);
        #[cfg(target_arch = "wasm32")]
        let mut handle = super::start(config, device);
        #[cfg(not(target_arch = "wasm32"))]
        let mut handle = {
            // An in-memory store (the data folder couldn't be opened) has nothing to sync.
            let Some(path) = store.path.clone() else { return };
            #[cfg(target_os = "macos")]
            let vault: Box<dyn super::session::SessionVault> = match path.parent() {
                Some(dir) => Box::new(super::session::FileVault::mac(dir)),
                None => Box::new(super::session::KeyringVault::new()),
            };
            #[cfg(not(target_os = "macos"))]
            let vault: Box<dyn super::session::SessionVault> = Box::new(super::session::KeyringVault::new());
            match super::start(config, path, device, vault) {
                Ok(h) => h,
                Err(e) => {
                    crate::diag::report!("sync didn't start: {e}");
                    return;
                }
            }
        };
        let mut commands = cloud.commands;
        commands.set(Some(handle.commands.clone()));
        let review = review.clone();
        spawn(async move {
            while let Some(event) = handle.events.recv().await {
                if mirror(event, cloud) {
                    // Another device answered, judged or changed a setting.
                    let _ = crate::store::shared().observe_stored();
                    review.backend.borrow_mut().reload_and_notify();
                }
            }
        });
    });
    // Back in front (Android hands a backgrounded app no network): a sign-in
    // Google approved meanwhile finishes now, and the outbox goes up.
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
            const send = () => { if (!document.hidden) dioxus.send(true); };
            document.addEventListener("visibilitychange", send);
            window.addEventListener("focus", send);
            window.addEventListener("pageshow", send);
            await new Promise(() => {});
            "#,
        );
        while eval.recv::<bool>().await.is_ok() {
            cloud.send(Command::Kick);
        }
    });
    // Every answer and verdict is in the outbox: push it now.
    let revision = use_context::<crate::store::Review>().revision;
    use_effect(move || {
        revision.read();
        cloud.send(Command::Kick);
    });
}

/// Tests of screens that hold Settings › Sync: the [`Cloud`] of a build
/// without Batna Cloud, and no engine.
#[cfg(test)]
pub fn use_unconfigured_cloud() {
    use_context_provider(|| Cloud {
        status: Signal::new(CloudStatus::not_configured("test-device".into())),
        devices: Signal::new(None),
        signing_in: Signal::new(false),
        login: Signal::new(LoginStatus::default()),
        commands: Signal::new(None),
    });
}

/// Shows one engine event. True when another device changed stored data,
/// so the screens should read it again.
fn mirror(event: Event, cloud: Cloud) -> bool {
    let (mut status, mut devices, mut signing_in, mut login) =
        (cloud.status, cloud.devices, cloud.signing_in, cloud.login);
    match event {
        Event::Status(s) => {
            if *status.peek() != s {
                status.set(s);
            }
        }
        Event::Devices(d) => devices.set(Some(d)),
        Event::SigningIn(on) => signing_in.set(on),
        Event::Login(l) => login.set(l),
        Event::Changed(_) => return true,
    }
    false
}

fn problem_text(p: Problem) -> &'static str {
    match p {
        Problem::NotConfigured => tx::P_NOT_CONFIGURED,
        Problem::NotAllowed => tx::P_NOT_ALLOWED,
        Problem::SignInCancelled => tx::P_CANCELLED,
        Problem::SignInFailed => tx::P_FAILED,
        Problem::SignedOutRemotely => tx::P_REMOTE,
        Problem::Network => tx::P_NETWORK,
        Problem::SignInWaiting => tx::P_SIGN_IN_WAITING,
        Problem::Server => tx::P_SERVER,
        Problem::Outdated => tx::P_OUTDATED,
        Problem::ClockOff => tx::P_CLOCK,
        Problem::WrongPassword => tx::P_WRONG_PASSWORD,
        Problem::CurrentWrong => tx::P_CURRENT_WRONG,
        Problem::Locked(_) => tx::P_LOCKED,
        Problem::UsernameTaken => tx::P_TAKEN,
        Problem::BadUsername => tx::P_BAD_USERNAME,
        Problem::ShortPassword => tx::P_SHORT_PASSWORD,
        Problem::Mismatch => tx::P_MISMATCH,
    }
}

/// The sentence for a problem, with the wait when the username is locked.
fn problem_line(p: Problem) -> String {
    match p {
        Problem::Locked(minutes) => tx::locked(minutes),
        p => problem_text(p).to_string(),
    }
}

fn state_text(s: SyncState) -> Option<&'static str> {
    match s {
        SyncState::SignedOut => None,
        SyncState::Connecting => Some(tx::CONNECTING),
        SyncState::CatchingUp => Some(tx::CATCHING_UP),
        SyncState::Live => Some(tx::LIVE),
        SyncState::Offline => Some(tx::OFFLINE),
        SyncState::Stopped => Some(tx::STOPPED),
    }
}

/// "من ٣ دقايق" with the Egyptian dual and plural forms.
pub fn ago(then_ms: u64, now_ms: u64) -> String {
    let secs = now_ms.saturating_sub(then_ms) / 1000;
    if secs < 60 {
        return tx::JUST_NOW.to_string();
    }
    use crate::i18n::count::{DAY, HOUR, MINUTE, count};
    let (n, noun) = match secs {
        s if s < 3600 => (s / 60, &MINUTE),
        s if s < 86_400 => (s / 3600, &HOUR),
        s => (s / 86_400, &DAY),
    };
    format!("{} {}", tx::AGO, count(n, noun))
}

fn now_ms() -> u64 {
    crate::store::now_ms() as u64
}

#[component]
pub fn SyncSection() -> Element {
    let cloud = use_context::<Cloud>();
    let status = cloud.status.read().clone();
    let signed_in = status.email.is_some() || status.sync != SyncState::SignedOut;

    // Load the device list when this opens, and again once connected.
    let state = use_memo(move || cloud.status.read().sync);
    use_effect(move || {
        if matches!(state(), SyncState::Live | SyncState::CatchingUp) {
            cloud.send(Command::LoadDevices);
        }
    });

    rsx! {
        section { class: "card",
            h2 { {tx::TITLE} }
            if !status.configured {
                p { class: "muted small", {tx::NOT_CONFIGURED} }
            } else if signed_in {
                SignedIn { status: status.clone() }
            } else {
                if *cloud.signing_in.read() {
                    p { class: "muted", aria_live: "polite", {tx::SIGNING_IN} }
                } else if status.finishing_sign_in {
                    p { class: "muted", aria_live: "polite", {tx::FINISHING} }
                    button { class: "btn btn-primary", onclick: move |_| cloud.send(Command::Kick), {tx::TRY_NOW} }
                } else {
                    button { class: "btn btn-primary", onclick: move |_| cloud.send(Command::SignInGoogle), {tx::SIGN_IN} }
                }
            }
            if let Some(p) = status.problem {
                p { class: "small", role: "status", {problem_line(p)} }
            }
        }
        if status.configured && !signed_in {
            PasswordSignIn {}
        }
        if signed_in && status.held > 0 {
            super::held_ui::HeldCard { held: status.held }
        }
        if signed_in {
            LoginCard {}
            DevicesCard {}
        }
    }
}

/// Signed out: username + password, for a device without Google.
#[component]
fn PasswordSignIn() -> Element {
    let cloud = use_context::<Cloud>();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut theirs = use_signal(|| false);
    let busy = *cloud.signing_in.read();
    let ready = !username.read().trim().is_empty() && !password.read().is_empty();
    let mut submit = move || {
        if !ready || busy {
            return;
        }
        cloud.send(Command::SignInPassword {
            username: username.peek().clone(),
            password: password.peek().clone(),
            remember: !*theirs.peek(),
        });
        password.set(String::new());
    };
    rsx! {
        section { class: "card pw-card",
            h2 { {tx::PW_TITLE} }
            p { class: "muted small", {tx::PW_INTRO} }
            form { class: "pw-form",
                onsubmit: move |e| {
                    e.prevent_default();
                    submit();
                },
                label { class: "field",
                    span { {tx::USERNAME} }
                    input {
                        r#type: "text", name: "username", dir: "ltr", lang: "en",
                        autocomplete: "username", autocapitalize: "none", spellcheck: "false",
                        value: "{username}",
                        oninput: move |e| username.set(e.value()),
                    }
                }
                label { class: "field",
                    span { {tx::PASSWORD} }
                    input {
                        r#type: "password", name: "password", dir: "ltr",
                        autocomplete: "current-password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }
                label { class: "pw-theirs",
                    input {
                        r#type: "checkbox",
                        checked: theirs(),
                        onchange: move |e: FormEvent| theirs.set(e.checked()),
                    }
                    span {
                        {tx::NOT_MINE}
                        small { class: "muted", {tx::NOT_MINE_HINT} }
                    }
                }
                div { class: "sync-actions",
                    button { class: "btn btn-primary", r#type: "submit", disabled: !ready || busy,
                        if busy { {tx::PW_CHECKING} } else { {tx::PW_SIGN_IN} }
                    }
                }
            }
        }
    }
}

/// Signed in: the username and password this account can sign in with
/// elsewhere, set or changed here.
#[component]
fn LoginCard() -> Element {
    let cloud = use_context::<Cloud>();
    let login = cloud.login.read().clone();
    let state = use_memo(move || cloud.status.read().sync);
    use_effect(move || {
        if matches!(state(), SyncState::Live | SyncState::CatchingUp) && cloud.login.peek().username.is_none()
        {
            cloud.send(Command::LoadLogin);
        }
    });
    let current = login.username.clone().flatten();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut again = use_signal(String::new);
    // A session that signed in with the password gives the current one to
    // change or remove it (Batna Cloud checks it, #721).
    let mut old = use_signal(String::new);
    let needs_current = login.needs_current && current.is_some();
    let given = move || if needs_current { Some(old.peek().clone()) } else { None };
    let mut mismatch = use_signal(|| false);
    // The form starts from the username the account has.
    let known = current.clone();
    use_effect(use_reactive!(|known| {
        if let Some(k) = known
            && username.peek().is_empty()
        {
            username.set(k);
        }
    }));
    let mut editing = use_signal(|| false);
    let open = editing() || (login.username == Some(None));
    let saved = login.saved;
    use_effect(use_reactive!(|saved| {
        if saved {
            password.set(String::new());
            again.set(String::new());
            old.set(String::new());
            editing.set(false);
        }
    }));
    let ready = !username.read().trim().is_empty()
        && !password.read().is_empty()
        && !again.read().is_empty()
        && (!needs_current || !old.read().is_empty());
    let current_field = rsx! {
        label { class: "field",
            span { {tx::CURRENT_PASSWORD} }
            input {
                r#type: "password", name: "current-password", dir: "ltr", autocomplete: "current-password",
                value: "{old}",
                oninput: move |e| old.set(e.value()),
            }
            span { class: "field-hint", {tx::CURRENT_HINT} }
        }
    };
    let problem = if mismatch() { Some(Problem::Mismatch) } else { login.problem };
    rsx! {
        section { class: "card pw-card",
            h2 { {tx::PW_TITLE} }
            match &login.username {
                None => rsx! { p { class: "muted", {tx::LOADING} } },
                Some(Some(name)) => rsx! {
                    div { class: "sync-who",
                        span { class: "row-icon", aria_hidden: "true", {crate::icons::section("sync")} }
                        div { class: "row-copy",
                            small { {tx::PW_YOURS} }
                            bdi { class: "sync-email", dir: "ltr", "{name}" }
                        }
                    }
                    if saved {
                        p { class: "small", role: "status", {tx::PW_SAVED} }
                    }
                    p { class: "muted small", {tx::PW_HOW} }
                },
                Some(None) => rsx! { p { class: "muted small", {tx::PW_NONE} } },
            }
            if open {
                form { class: "pw-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        if *password.peek() != *again.peek() {
                            mismatch.set(true);
                            return;
                        }
                        mismatch.set(false);
                        cloud.send(Command::SetLogin {
                            username: username.peek().clone(),
                            password: password.peek().clone(),
                            current: given(),
                        });
                    },
                    if needs_current { {current_field.clone()} }
                    label { class: "field",
                        span { {tx::USERNAME} }
                        input {
                            r#type: "text", name: "username", dir: "ltr", lang: "en",
                            autocomplete: "username", autocapitalize: "none", spellcheck: "false",
                            value: "{username}",
                            oninput: move |e| username.set(e.value()),
                        }
                        span { class: "field-hint", {tx::USERNAME_HINT} }
                    }
                    label { class: "field",
                        span { {tx::NEW_PASSWORD} }
                        input {
                            r#type: "password", name: "new-password", dir: "ltr", autocomplete: "new-password",
                            value: "{password}",
                            oninput: move |e| { password.set(e.value()); mismatch.set(false); },
                        }
                        span { class: "field-hint", {tx::PASSWORD_HINT} }
                    }
                    label { class: "field",
                        span { {tx::PASSWORD_AGAIN} }
                        input {
                            r#type: "password", name: "new-password-again", dir: "ltr", autocomplete: "new-password",
                            value: "{again}",
                            oninput: move |e| { again.set(e.value()); mismatch.set(false); },
                        }
                    }
                    div { class: "sync-actions",
                        if current.is_some() {
                            button { class: "btn btn-ghost", r#type: "button", onclick: move |_| editing.set(false), {tx::CANCEL} }
                        }
                        button { class: "btn btn-primary", r#type: "submit", disabled: !ready || login.saving,
                            if login.saving { {tx::PW_SAVING} } else { {tx::PW_SAVE} }
                        }
                    }
                }
            } else if current.is_some() {
                if needs_current { {current_field} }
                div { class: "sync-actions",
                    button { class: "btn btn-danger",
                        disabled: login.saving || (needs_current && old.read().is_empty()),
                        onclick: move |_| cloud.send(Command::RemoveLogin { current: given() }),
                        {tx::PW_REMOVE}
                    }
                    button { class: "btn btn-secondary", onclick: move |_| editing.set(true), {tx::PW_CHANGE} }
                }
            }
            if let Some(p) = problem {
                p { class: "small", role: "status", {problem_line(p)} }
            }
        }
    }
}

#[component]
fn SignedIn(status: CloudStatus) -> Element {
    let cloud = use_context::<Cloud>();
    let email = status.email.clone().unwrap_or_default();
    let last = status.last_sync_ms.map(|ms| ago(ms, now_ms()));
    rsx! {
        div { class: "sync-who",
            span { class: "row-icon", aria_hidden: "true", {crate::icons::section("sync")} }
            div { class: "row-copy",
                small { {tx::SIGNED_IN_AS} }
                bdi { class: "sync-email", dir: "ltr", "{email}" }
            }
        }
        if status.temporary {
            p { class: "small pw-temporary", role: "note", {tx::TEMPORARY} }
        }
        p { class: "sync-state muted small", aria_live: "polite",
            if let Some(state) = state_text(status.sync) {
                span { class: "chip", "data-tone": if status.sync == SyncState::Live { "good" } else { "accent" }, {state} }
            }
            if status.pending > 0 {
                span { {tx::pending(status.pending)} }
            }
            if let Some(last) = last {
                span { "{tx::LAST}: {last}" }
            }
        }
        div { class: "sync-actions",
            button { class: "btn btn-secondary", onclick: move |_| cloud.send(Command::SignOut), {tx::SIGN_OUT} }
        }
    }
}

#[component]
fn DevicesCard() -> Element {
    let cloud = use_context::<Cloud>();
    let devices = cloud.devices.read().clone();
    let now = now_ms();
    rsx! {
        section { class: "card sync-devices",
            div { class: "section-head",
                h2 { {tx::DEVICES} }
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| cloud.send(Command::LoadDevices),
                    {crate::icons::refresh()}
                    {tx::REFRESH}
                }
            }
            match devices {
                None => rsx! { p { class: "muted", {tx::LOADING} } },
                Some(Err(p)) => rsx! { p { class: "small", {problem_line(p)} } },
                Some(Ok(list)) => rsx! {
                    ul { class: "device-list",
                        for d in list {
                            li { key: "{d.device_id}", class: if d.revoked { "device-row revoked" } else { "device-row" },
                                span { class: "row-icon", aria_hidden: "true", {platform_icon(d.platform, &d.name)} }
                                div { class: "row-copy",
                                    span { class: "device-title",
                                        bdi { class: "device-name", dir: "auto", "{d.name}" }
                                        if d.current {
                                            span { class: "chip", "data-tone": "accent", {tx::THIS_DEVICE} }
                                        }
                                    }
                                    small {
                                        if d.revoked {
                                            {tx::DEVICE_OUT}
                                        } else if let Some(ms) = d.last_sync_ms {
                                            "{tx::LAST}: {ago(ms, now)}"
                                        } else {
                                            {tx::NEVER_SYNCED}
                                        }
                                    }
                                }
                                if !d.current && !d.revoked {
                                    div { class: "row-end",
                                        button {
                                            class: "btn btn-danger device-out",
                                            onclick: {
                                                let id = d.device_id.clone();
                                                move |_| cloud.send(Command::RevokeDevice(id.clone()))
                                            },
                                            {tx::SIGN_OUT_DEVICE}
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// Galaxy Tab model numbers start SM-X or SM-T; other Android devices are phones.
fn is_tablet(name: &str) -> bool {
    name.starts_with("SM-X") || name.starts_with("SM-T")
}

fn platform_icon(p: Platform, name: &str) -> Element {
    let d = match p {
        Platform::Android if is_tablet(name) => {
            "M6 3h12a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM11 18h2"
        }
        Platform::Android => "M8 3h8v18H8zM11 18h2",
        // laptop
        Platform::Macos => "M5 6h14v9H5zM3 18h18",
        Platform::Web => "M4 5h16v14H4zM4 9h16M7 7h.01M9.5 7h.01",
        Platform::Other => "M4 5h16v11H4zM9 20h6M12 16v4",
    };
    rsx! {
        svg {
            width: "22",
            height: "22",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    use super::*;

    const MIN: u64 = 60_000;

    struct Case {
        status: CloudStatus,
        devices: Option<Result<Vec<DeviceSummary>, Problem>>,
        signing_in: bool,
        login: LoginStatus,
        sent: Option<UnboundedSender<Command>>,
    }

    thread_local! {
        static CASE: RefCell<Option<Case>> = const { RefCell::new(None) };
        static CLOUD: RefCell<Option<Cloud>> = const { RefCell::new(None) };
    }

    fn fixture() -> Element {
        let case = CASE.with(|c| c.borrow_mut().take()).unwrap();
        let cloud = use_context_provider(|| Cloud {
            status: Signal::new(case.status),
            devices: Signal::new(case.devices),
            signing_in: Signal::new(case.signing_in),
            login: Signal::new(case.login),
            commands: Signal::new(case.sent),
        });
        CLOUD.with(|c| *c.borrow_mut() = Some(cloud));
        rsx! { SyncSection {} }
    }

    fn status(sync: SyncState) -> CloudStatus {
        CloudStatus {
            configured: true,
            email: (sync != SyncState::SignedOut).then(|| "mohamed@example.com".into()),
            device_id: "phone-1".into(),
            sync,
            pending: 0,
            held: 0,
            last_sync_ms: None,
            problem: None,
            finishing_sign_in: false,
            temporary: false,
        }
    }

    /// Renders Settings › Sync and returns the HTML and what it asked the engine.
    fn render(
        status: CloudStatus,
        devices: Option<Result<Vec<DeviceSummary>, Problem>>,
    ) -> (String, Vec<Command>) {
        render_with(status, devices, false)
    }

    fn render_with(
        status: CloudStatus,
        devices: Option<Result<Vec<DeviceSummary>, Problem>>,
        signing_in: bool,
    ) -> (String, Vec<Command>) {
        let (tx, mut rx): (_, UnboundedReceiver<Command>) = unbounded_channel();
        CASE.with(|c| {
            *c.borrow_mut() =
                Some(Case { status, devices, signing_in, login: LoginStatus::default(), sent: Some(tx) })
        });
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        // Let effects run once.
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(20), dom.wait_for_work()).await;
        });
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus::ssr::render(&dom);
        let mut sent = Vec::new();
        while let Ok(c) = rx.try_recv() {
            sent.push(c);
        }
        (html, sent)
    }

    fn device(id: &str, name: &str, platform: Platform) -> DeviceSummary {
        DeviceSummary {
            device_id: id.into(),
            name: name.into(),
            model: None,
            platform,
            signed_in_at_ms: 1,
            last_seen_ms: 1,
            last_sync_ms: None,
            current: false,
            revoked: false,
        }
    }

    #[test]
    fn a_build_without_a_server_says_sync_is_off() {
        let (html, sent) = render(CloudStatus::not_configured("phone-1".into()), None);
        assert!(html.contains(tx::NOT_CONFIGURED), "{html}");
        assert!(!html.contains(tx::SIGN_IN) && !html.contains("sync-devices"), "{html}");
        assert!(sent.is_empty());
    }

    #[test]
    fn signed_out_offers_google_and_shows_each_step_of_signing_in() {
        let (html, _) = render(status(SyncState::SignedOut), None);
        assert!(html.contains(tx::SIGN_IN), "{html}");
        assert!(!html.contains("sync-devices"), "no devices card while signed out");

        let (html, _) = render_with(status(SyncState::SignedOut), None, true);
        assert!(html.contains(tx::SIGNING_IN) && !html.contains(tx::SIGN_IN), "{html}");

        let waiting = CloudStatus {
            finishing_sign_in: true,
            problem: Some(Problem::SignInWaiting),
            ..status(SyncState::SignedOut)
        };
        let (html, _) = render(waiting, None);
        assert!(html.contains(tx::FINISHING) && html.contains(tx::TRY_NOW), "{html}");
        assert!(html.contains(tx::P_SIGN_IN_WAITING));
    }

    #[test]
    fn engine_events_reach_the_screen() {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_scope(ScopeId::ROOT, || {
            let cloud = Cloud {
                status: Signal::new(CloudStatus::not_configured("p".into())),
                devices: Signal::new(None),
                signing_in: Signal::new(false),
                login: Signal::new(LoginStatus::default()),
                commands: Signal::new(None),
            };
            assert!(!mirror(Event::SigningIn(true), cloud));
            assert!(*cloud.signing_in.peek());
            assert!(!mirror(Event::Status(status(SyncState::Live)), cloud));
            assert_eq!(cloud.status.peek().sync, SyncState::Live);
            assert!(!mirror(Event::Devices(Err(Problem::Server)), cloud));
            assert_eq!(*cloud.devices.peek(), Some(Err(Problem::Server)));
            let tables = ["settings".to_string()].into_iter().collect();
            assert!(mirror(Event::Changed(tables), cloud), "a change from another device reloads");
            // Sending without an engine (this build has none) does nothing.
            cloud.send(Command::Kick);
        });
    }

    /// Renders, presses `label`, and returns what reached the engine.
    fn press(
        status: CloudStatus,
        devices: Option<Result<Vec<DeviceSummary>, Problem>>,
        label: &str,
    ) -> Vec<String> {
        let mut screen = Screen::new(status, devices, LoginStatus::default());
        crate::testing::click(&mut screen.dom, label);
        screen.sent()
    }

    /// Settings › Sync, live, to type into and press.
    struct Screen {
        dom: VirtualDom,
        rx: UnboundedReceiver<Command>,
    }

    impl Screen {
        fn new(
            status: CloudStatus,
            devices: Option<Result<Vec<DeviceSummary>, Problem>>,
            login: LoginStatus,
        ) -> Self {
            let (tx, rx) = unbounded_channel();
            CASE.with(|c| {
                *c.borrow_mut() = Some(Case { status, devices, signing_in: false, login, sent: Some(tx) })
            });
            let mut dom = VirtualDom::new(fixture);
            dom.rebuild_in_place();
            Self { dom, rx }
        }

        fn html(&self) -> String {
            crate::testing::html(&self.dom)
        }

        /// What it asked the engine since last time.
        fn sent(&mut self) -> Vec<String> {
            let mut sent = Vec::new();
            while let Ok(c) = self.rx.try_recv() {
                sent.push(format!("{c:?}"));
            }
            sent
        }

        /// The engine's answer about the username arrives.
        fn answer(&mut self, login: LoginStatus) {
            let mut signal = CLOUD.with(|c| *c.borrow()).unwrap().login;
            self.dom.in_runtime(|| signal.set(login));
            self.settle();
        }

        /// Lets effects run.
        fn settle(&mut self) {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async {
                let _ = tokio::time::timeout(std::time::Duration::from_millis(20), self.dom.wait_for_work())
                    .await;
            });
            self.dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
    }

    fn login(username: Option<&str>) -> LoginStatus {
        LoginStatus { username: Some(username.map(Into::into)), ..LoginStatus::default() }
    }

    #[test]
    fn a_session_on_someone_elses_device_says_it_ends_with_the_app() {
        let (html, _) = render(CloudStatus { temporary: true, ..status(SyncState::Live) }, None);
        assert!(html.contains(tx::TEMPORARY), "{html}");
        let (html, _) = render(status(SyncState::Live), None);
        assert!(!html.contains(tx::TEMPORARY));
    }

    #[test]
    fn a_device_without_google_signs_in_with_a_username_and_password() {
        let mut screen = Screen::new(status(SyncState::SignedOut), None, LoginStatus::default());
        let html = screen.html();
        assert!(html.contains(tx::PW_TITLE) && html.contains(tx::NOT_MINE), "{html}");
        // Nothing to send until both are filled in.
        crate::testing::submit(&mut screen.dom, tx::PW_SIGN_IN);
        assert!(screen.sent().is_empty());
        crate::testing::type_into(&mut screen.dom, "text", "Mo.Refaat");
        crate::testing::type_into(&mut screen.dom, "password", "correct horse battery");
        crate::testing::type_into(&mut screen.dom, "checkbox", "true");
        crate::testing::submit(&mut screen.dom, tx::PW_SIGN_IN);
        assert_eq!(screen.sent(), ["SignInPassword { username: \"Mo.Refaat\", remember: false }"]);
        assert!(!screen.html().contains("correct horse"), "the password field empties");

        let (html, _) = render_with(status(SyncState::SignedOut), None, true);
        assert!(html.contains(tx::PW_CHECKING), "{html}");
    }

    #[test]
    fn signed_in_he_sets_changes_and_removes_his_password() {
        let live = status(SyncState::Live);
        let mut screen = Screen::new(live.clone(), None, LoginStatus::default());
        screen.settle();
        assert!(screen.sent().contains(&"LoadLogin".to_string()), "it asks once it's live");
        assert!(screen.html().contains(tx::LOADING));

        // No username yet: the form is open.
        screen.answer(login(None));
        assert!(screen.html().contains(tx::PW_NONE));
        crate::testing::type_into(&mut screen.dom, "text", "mo.refaat");
        crate::testing::type_nth(&mut screen.dom, "password", 0, "correct horse battery");
        crate::testing::type_nth(&mut screen.dom, "password", 1, "correct horse batterY");
        crate::testing::submit(&mut screen.dom, tx::PW_SAVE);
        assert!(screen.sent().is_empty(), "the two passwords differ");
        assert!(screen.html().contains(&problem_line(Problem::Mismatch)));
        crate::testing::type_nth(&mut screen.dom, "password", 1, "correct horse battery");
        assert!(!screen.html().contains(&problem_line(Problem::Mismatch)), "typing again clears it");
        crate::testing::submit(&mut screen.dom, tx::PW_SAVE);
        assert_eq!(screen.sent(), ["SetLogin { username: \"mo.refaat\" }"], "the password stays out of logs");

        screen.answer(LoginStatus { saving: true, ..login(None) });
        assert!(screen.html().contains(tx::PW_SAVING));
        screen.answer(LoginStatus { saved: true, ..login(Some("mo.refaat")) });
        let html = screen.html();
        assert!(html.contains(tx::PW_SAVED) && html.contains("mo.refaat"), "{html}");
        assert!(!html.contains(tx::PW_SAVE) && html.contains(tx::PW_CHANGE), "the form closes");

        // Change opens the form with his username; Cancel closes it.
        crate::testing::click(&mut screen.dom, tx::PW_CHANGE);
        assert!(screen.html().contains(tx::PW_SAVE) && screen.html().contains(r#"value="mo.refaat""#));
        crate::testing::click(&mut screen.dom, tx::CANCEL);
        assert!(!screen.html().contains(tx::PW_SAVE));
        crate::testing::click(&mut screen.dom, tx::PW_REMOVE);
        assert_eq!(screen.sent(), ["RemoveLogin"]);

        screen.answer(LoginStatus { problem: Some(Problem::Locked(3)), ..login(Some("mo.refaat")) });
        assert!(screen.html().contains(&tx::locked(3)));
    }

    #[test]
    fn each_button_asks_the_engine_for_its_own_thing() {
        assert_eq!(press(status(SyncState::SignedOut), None, tx::SIGN_IN), ["SignInGoogle"]);
        let waiting = CloudStatus { finishing_sign_in: true, ..status(SyncState::SignedOut) };
        assert_eq!(press(waiting, None, tx::TRY_NOW), ["Kick"]);
        assert_eq!(press(status(SyncState::Offline), None, tx::SIGN_OUT), ["SignOut"]);
        assert_eq!(press(status(SyncState::Offline), None, tx::REFRESH), ["LoadDevices"]);
        let tab = device("tab-1", "SM-X730", Platform::Android);
        let sent = press(status(SyncState::Offline), Some(Ok(vec![tab])), tx::SIGN_OUT_DEVICE);
        assert_eq!(sent, ["RevokeDevice(\"tab-1\")"]);
    }

    #[test]
    fn every_problem_has_its_own_words() {
        let all = [
            Problem::NotConfigured,
            Problem::NotAllowed,
            Problem::SignInCancelled,
            Problem::SignInFailed,
            Problem::SignedOutRemotely,
            Problem::Network,
            Problem::SignInWaiting,
            Problem::Server,
            Problem::Outdated,
            Problem::ClockOff,
        ];
        let words: std::collections::BTreeSet<&str> = all.iter().map(|p| problem_text(*p)).collect();
        assert_eq!(words.len(), all.len());
        let (html, _) =
            render(CloudStatus { problem: Some(Problem::ClockOff), ..status(SyncState::Live) }, None);
        assert!(html.contains(tx::P_CLOCK) && html.contains("role=\"status\""), "{html}");
    }

    /// Refused changes are named, with a way to send them again or let
    /// them go (audit F12).
    #[test]
    fn held_changes_get_their_own_card() {
        let (html, _) = render(status(SyncState::Live), None);
        assert!(!html.contains("sync-held"), "nothing held, no card: {html}");
        let (html, _) = render(CloudStatus { held: 3, ..status(SyncState::Live) }, None);
        assert!(html.contains("sync-held") && html.contains("3 تغييرات من الجهاز ده"), "{html}");
        assert!(html.contains(tx::HELD_SEND_AGAIN) && html.contains(tx::HELD_FORGET), "{html}");
        assert!(!html.contains(tx::HELD_FORGET_SURE), "letting go asks first: {html}");
    }

    #[test]
    fn signed_in_shows_who_where_and_what_waits() {
        let s = CloudStatus { pending: 3, last_sync_ms: Some(now_ms() - 5 * MIN), ..status(SyncState::Live) };
        let (html, sent) = render(s, None);
        assert!(html.contains("mohamed@example.com") && html.contains("dir=\"ltr\""), "{html}");
        assert!(html.contains(tx::LIVE) && html.contains("data-tone=\"good\""), "{html}");
        assert!(html.contains("3 تغييرات مستنية تترفع"), "{html}");
        assert!(html.contains(&format!("{}: من 5 دقايق", tx::LAST)), "{html}");
        assert!(html.contains(tx::SIGN_OUT));
        // Opening it while live asks for the device list, which shows as loading.
        assert!(html.contains(tx::LOADING));
        assert!(sent.iter().any(|c| matches!(c, Command::LoadDevices)), "{sent:?}");

        for (sync, words) in [
            (SyncState::Connecting, tx::CONNECTING),
            (SyncState::CatchingUp, tx::CATCHING_UP),
            (SyncState::Offline, tx::OFFLINE),
            (SyncState::Stopped, tx::STOPPED),
        ] {
            let (html, sent) = render(status(sync), Some(Ok(Vec::new())));
            assert!(html.contains(words) && html.contains("data-tone=\"accent\""), "{sync:?}: {html}");
            assert_eq!(sent.iter().any(|c| matches!(c, Command::LoadDevices)), sync == SyncState::CatchingUp);
        }
    }

    #[test]
    fn the_devices_card_lists_each_device_with_its_state() {
        let mut me = device("phone-1", "Galaxy S24 Ultra", Platform::Android);
        me.current = true;
        me.last_sync_ms = Some(now_ms() - 2 * 60 * MIN);
        let tab = device("tab-1", "SM-X730", Platform::Android);
        let mut mac = device("mac-1", "Mohamed’s MacBook Air", Platform::Macos);
        mac.revoked = true;
        let web = device("web-1", "Chrome", Platform::Web);
        let dev = device("dev-1", "Batna (dev)", Platform::Other);
        let (html, _) = render(status(SyncState::Offline), Some(Ok(vec![me, tab, mac, web, dev])));
        assert!(html.contains(tx::THIS_DEVICE));
        assert!(html.contains(&format!("{}: من ساعتين", tx::LAST)), "{html}");
        assert!(html.contains(tx::NEVER_SYNCED) && html.contains(tx::DEVICE_OUT));
        assert!(html.contains("device-row revoked"));
        // Only other, still signed-in devices can be signed out from here.
        assert_eq!(html.matches(tx::SIGN_OUT_DEVICE).count(), 3, "{html}");
        // Phone, tablet, laptop, browser and other icons are all different.
        let icons = ["M8 3h8v18H8z", "M6 3h12a1", "M5 6h14v9H5z", "M4 5h16v14H4z", "M4 5h16v11H4z"];
        for d in icons {
            assert!(html.contains(d), "missing icon {d}");
        }
        let (html, _) = render(status(SyncState::Offline), Some(Err(Problem::Network)));
        assert!(html.contains(tx::P_NETWORK));
        assert_eq!(html.matches(tx::LOADING).count(), 1, "only the username card still waits");
    }

    #[test]
    fn galaxy_tabs_get_the_tablet_icon() {
        assert!(is_tablet("SM-X730"));
        assert!(!is_tablet("SM-S928B"));
        assert!(!is_tablet("Mohamed’s MacBook Air"));
    }

    #[test]
    fn relative_times_use_egyptian_number_forms() {
        let now = 100 * 86_400_000;
        assert_eq!(ago(now - 10_000, now), "دلوقتي");
        assert_eq!(ago(now - MIN, now), "من دقيقة");
        assert_eq!(ago(now - 2 * MIN, now), "من دقيقتين");
        assert_eq!(ago(now - 5 * MIN, now), "من 5 دقايق");
        assert_eq!(ago(now - 25 * MIN, now), "من 25 دقيقة");
        assert_eq!(ago(now - 3 * 60 * MIN, now), "من 3 ساعات");
        assert_eq!(ago(now - 2 * 86_400_000, now), "من يومين");
    }
}
