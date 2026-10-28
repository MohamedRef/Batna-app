//! Settings › Sync: Google sign-in, the sync state, and the Devices list
//! with remote sign-out. `use_provide_cloud` starts the engine once (when
//! this build knows a Batna Cloud URL) and mirrors its events into signals.
// adapted from kabar-app main: crates/app/src/account.rs and main.rs (use_cloud)

use batna_protocol::cloud::{DeviceSummary, Platform};
use dioxus::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use super::{CloudStatus, Command, Event, Problem, SyncState};
use crate::i18n::cloud as tx;

/// Batna Cloud state for the UI, shared through context.
#[derive(Clone, Copy)]
pub struct Cloud {
    pub status: Signal<CloudStatus>,
    pub devices: Signal<Option<Result<Vec<DeviceSummary>, Problem>>>,
    pub signing_in: Signal<bool>,
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
        commands: Signal::new(None),
    });
    let review = use_context::<crate::store::Review>();
    use_hook(move || {
        let Some(config) = super::CloudConfig::from_env() else { return };
        // An in-memory store (the data folder couldn't be opened) has nothing to sync.
        let Some(path) = store.path.clone() else { return };
        let device = super::device_info(device_id);
        let vault = Box::new(super::session::KeyringVault::new());
        let mut handle = match super::start(config, path, device, vault) {
            Ok(h) => h,
            Err(e) => {
                crate::diag::report!("sync didn't start: {e}");
                return;
            }
        };
        let mut commands = cloud.commands;
        commands.set(Some(handle.commands.clone()));
        let review = review.clone();
        spawn(async move {
            let (mut status, mut devices, mut signing_in) = (cloud.status, cloud.devices, cloud.signing_in);
            while let Some(event) = handle.events.recv().await {
                match event {
                    Event::Status(s) => {
                        if *status.peek() != s {
                            status.set(s);
                        }
                    }
                    Event::Devices(d) => devices.set(Some(d)),
                    Event::SigningIn(on) => signing_in.set(on),
                    Event::Changed(_) => {
                        // Another device answered, judged or changed a setting.
                        let _ = crate::store::shared().observe_stored();
                        review.backend.borrow_mut().reload_and_notify();
                    }
                }
            }
        });
    });
    // Every answer and verdict is in the outbox: push it now.
    let revision = use_context::<crate::store::Review>().revision;
    use_effect(move || {
        revision.read();
        cloud.send(Command::Kick);
    });
}

fn problem_text(p: Problem) -> &'static str {
    match p {
        Problem::NotConfigured => tx::P_NOT_CONFIGURED,
        Problem::NotAllowed => tx::P_NOT_ALLOWED,
        Problem::SignInCancelled => tx::P_CANCELLED,
        Problem::SignInFailed => tx::P_FAILED,
        Problem::SignedOutRemotely => tx::P_REMOTE,
        Problem::Network => tx::P_NETWORK,
        Problem::Server => tx::P_SERVER,
        Problem::Outdated => tx::P_OUTDATED,
        Problem::ClockOff => tx::P_CLOCK,
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
    let (n, unit) = match secs {
        s if s < 3600 => (s / 60, 0),
        s if s < 86_400 => (s / 3600, 1),
        s => (s / 86_400, 2),
    };
    // [one, two, 3–10, 11+]
    let forms = [
        ["دقيقة", "دقيقتين", "دقايق", "دقيقة"],
        ["ساعة", "ساعتين", "ساعات", "ساعة"],
        ["يوم", "يومين", "أيام", "يوم"],
    ][unit];
    match n {
        1 => format!("من {}", forms[0]),
        2 => format!("من {}", forms[1]),
        3..=10 => format!("من {n} {}", forms[2]),
        _ => format!("من {n} {}", forms[3]),
    }
}

fn now_ms() -> u64 {
    super::engine::now_ms()
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
                p { class: "muted small", {tx::INTRO} }
                if *cloud.signing_in.read() {
                    p { class: "muted", aria_live: "polite", {tx::SIGNING_IN} }
                } else {
                    button { class: "primary", onclick: move |_| cloud.send(Command::SignInGoogle), {tx::SIGN_IN} }
                }
            }
            if let Some(p) = status.problem {
                p { class: "small", role: "status", {problem_text(p)} }
            }
        }
        if signed_in {
            DevicesCard {}
        }
    }
}

#[component]
fn SignedIn(status: CloudStatus) -> Element {
    let cloud = use_context::<Cloud>();
    let email = status.email.clone().unwrap_or_default();
    let last = status.last_sync_ms.map(|ms| ago(ms, now_ms()));
    rsx! {
        p { "{tx::SIGNED_IN_AS} " bdi { dir: "ltr", "{email}" } }
        p { class: "muted small", aria_live: "polite",
            if let Some(state) = state_text(status.sync) {
                span { class: if status.sync == SyncState::Live { "chip ok" } else { "chip" }, {state} }
                " "
            }
            if status.pending > 0 {
                "{status.pending} {tx::PENDING} · "
            }
            if let Some(last) = last {
                "{tx::LAST}: {last}"
            }
        }
        button { onclick: move |_| cloud.send(Command::SignOut), {tx::SIGN_OUT} }
    }
}

#[component]
fn DevicesCard() -> Element {
    let cloud = use_context::<Cloud>();
    let devices = cloud.devices.read().clone();
    let now = now_ms();
    rsx! {
        section { class: "card",
            h2 { {tx::DEVICES} }
            match devices {
                None => rsx! { p { class: "muted", {tx::LOADING} } },
                Some(Err(p)) => rsx! { p { class: "small", {problem_text(p)} } },
                Some(Ok(list)) => rsx! {
                    ul { class: "devices",
                        for d in list {
                            li { key: "{d.device_id}", class: if d.revoked { "device revoked" } else { "device" },
                                span { class: "device-icon", aria_hidden: "true", {platform_icon(d.platform)} }
                                span { class: "device-main",
                                    bdi { class: "device-name", dir: "auto", "{d.name}" }
                                    if d.current {
                                        " "
                                        span { class: "chip", {tx::THIS_DEVICE} }
                                    }
                                    br {}
                                    span { class: "muted small",
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
                                    button {
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
                },
            }
            button { onclick: move |_| cloud.send(Command::LoadDevices), {tx::REFRESH} }
        }
    }
}

fn platform_icon(p: Platform) -> Element {
    let d = match p {
        // phone or tablet
        Platform::Android => "M8 3h8v18H8zM11 18h2",
        // laptop
        Platform::Macos => "M5 6h14v9H5zM3 18h18",
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
    use super::*;

    const MIN: u64 = 60_000;

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
