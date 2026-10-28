//! Settings › Updates, the gentle "new Batna is ready" card above every
//! screen, and the hooks that follow the updater from the UI.
//!
//! Adapted from kabar-app main: crates/app/src/update/view.rs (one
//! language, Batna's settings rows and screens).

use batna_core::update::Channel;
use dioxus::prelude::*;

use super::{Failure, Mode, Off, Offer, Phase, Status, Updater};
use crate::i18n::update as tx;
use crate::settings::{DeviceOnly, Section, SettingsNav};
use crate::{Screen, Ui};

const UPDATE_CSS: &str = r#"
.update-state { display: grid; gap: var(--sp-2); padding: var(--sp-3);
  border-radius: var(--radius); background: var(--accent-soft); }
.update-state progress { inline-size: 100%; accent-color: var(--accent); }
.update-notes { white-space: pre-line; }
.update-primary { background: var(--accent); color: var(--surface); border-color: transparent; font-weight: 600; }
.update-banner { flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--sp-2) var(--sp-3);
  border-inline-start: 4px solid var(--accent); margin-block-end: var(--sp-3); }
.update-banner .ub-text { flex: 1 1 14rem; min-inline-size: 0; display: grid; gap: var(--sp-1); }
.update-toggle { display: flex; align-items: center; justify-content: space-between; gap: var(--sp-3); }
.update-toggle-text { display: grid; gap: var(--sp-1); }
"#;

/// The updater's status, following its thread.
pub fn use_update_status() -> Signal<Option<Status>> {
    let mut status = use_signal(|| super::get().map(Updater::status));
    use_future(move || async move {
        let Some(u) = super::get() else { return };
        let mut rx = u.subscribe();
        while rx.changed().await.is_ok() {
            let now = rx.borrow_and_update().clone();
            if status.peek().as_ref() != Some(&now) {
                status.set(Some(now));
            }
        }
    });
    status
}

/// Runs a blocking updater call off the UI thread.
fn work(f: impl FnOnce(&'static Updater) + Send + 'static) {
    if let Some(u) = super::get() {
        let _ = std::thread::Builder::new().name("batna-update".into()).spawn(move || f(u));
    }
}

fn notes(o: &Offer) -> String {
    super::notes_of(o)
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1} {}", bytes as f64 / (1024.0 * 1024.0), tx::MB)
}

fn channel_label(c: Channel) -> &'static str {
    match c {
        Channel::Preview => tx::PREVIEW,
        Channel::Alpha => tx::ALPHA,
        Channel::Beta => tx::BETA,
        Channel::Stable => tx::STABLE,
    }
}

fn mode_label(m: Mode) -> (&'static str, &'static str) {
    match m {
        Mode::Auto => (tx::MODE_AUTO, tx::MODE_AUTO_SUB),
        Mode::Notify => (tx::MODE_NOTIFY, tx::MODE_NOTIFY_SUB),
        Mode::Off => (tx::MODE_OFF, tx::MODE_OFF_SUB),
    }
}

/// The button that moves an offer forward, for the card and the banner.
fn primary_action(phase: &Phase) -> Option<(&'static str, fn())> {
    fn download() {
        work(Updater::download);
    }
    fn install() {
        work(Updater::install);
    }
    fn restart() {
        #[cfg(target_os = "macos")]
        super::restart();
    }
    match phase {
        Phase::Available(_) => Some((tx::DOWNLOAD, download)),
        Phase::Ready(_) if cfg!(target_os = "macos") => Some((tx::INSTALL_RESTART, install)),
        Phase::Ready(_) => Some((tx::INSTALL_NOW, install)),
        Phase::NeedsPermission(_) => Some((tx::TRY_AGAIN, install)),
        Phase::Installed(_) => Some((tx::RESTART_NOW, restart)),
        Phase::Failed { offer: Some(_), why: Failure::Install(_) } => Some((tx::TRY_AGAIN, install)),
        Phase::Failed { offer: Some(_), why: Failure::Network | Failure::Verify(_) } => {
            Some((tx::TRY_AGAIN, download))
        }
        _ => None,
    }
}

/// Settings › Updates.
#[component]
pub fn UpdatesSection() -> Element {
    let status = use_update_status();
    let s = status.read().clone();
    let build = super::build_number();
    let commit = super::commit();
    let busy = s.as_ref().is_some_and(|s| {
        matches!(s.phase, Phase::Checking | Phase::Downloading { .. } | Phase::Installing(_))
    });
    // Updates that are off for a reason this build already knows (no data
    // folder at all counts as a development build).
    let off = s.as_ref().map_or(Some(Off::DevBuild), |s| s.off);

    rsx! {
        style { {UPDATE_CSS} }
        section { class: "card", aria_labelledby: "updates-title",
            h2 { id: "updates-title", {tx::TITLE} }
            p {
                span { class: "muted", {tx::THIS_BUILD} ": " }
                bdi { dir: "ltr",
                    {env!("CARGO_PKG_VERSION")}
                    if build > 0 { " · {tx::BUILD} {build}" }
                    if let Some(c) = commit { " · {c}" }
                }
            }
            if let Some(off) = off {
                p { class: "muted small", role: "status",
                    {match off {
                        Off::DevBuild => tx::OFF_DEV,
                        Off::NoKey => tx::OFF_KEY,
                        Off::NoPlatform => tx::OFF_PLATFORM,
                        Off::NotInBundle => tx::OFF_BUNDLE,
                    }}
                }
            } else if let Some(s) = s.clone() {
                PhaseView { phase: s.phase.clone(), auto: s.prefs.mode == Mode::Auto }
                if s.content_waiting {
                    p { class: "note", role: "status", {tx::CONTENT_WAITING} }
                }
                div { class: "row",
                    button {
                        class: "button",
                        disabled: busy,
                        onclick: move |_| work(|u| u.check(false)),
                        {tx::CHECK_NOW}
                    }
                }
            }
        }
        if let Some(s) = s.filter(|s| s.off.is_none()) {
            section { class: "card",
                h2 { {tx::MODE} DeviceOnly {} }
                div { class: "choice-list", role: "radiogroup", aria_label: tx::MODE,
                    for m in Mode::ALL {
                        button {
                            key: "{m:?}",
                            class: "choice",
                            role: "radio",
                            aria_checked: if s.prefs.mode == m { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(u) = super::get() {
                                    u.set_mode(m);
                                }
                            },
                            span { class: "choice-text",
                                span { class: "choice-title", {mode_label(m).0} }
                                span { class: "muted small", {mode_label(m).1} }
                            }
                        }
                    }
                }
            }
            if s.prefs.mode != Mode::Off {
                section { class: "card",
                    label { class: "update-toggle",
                        span { class: "update-toggle-text",
                            span { {tx::NOTIFY} DeviceOnly {} }
                            span { class: "muted small", {tx::NOTIFY_SUB} }
                        }
                        input {
                            r#type: "checkbox",
                            role: "switch",
                            checked: s.prefs.notify,
                            onchange: move |e: FormEvent| {
                                if let Some(u) = super::get() {
                                    u.set_notify(e.checked());
                                }
                            },
                        }
                    }
                }
            }
            section { class: "card",
                h2 { {tx::CHANNEL} DeviceOnly {} }
                if s.preview {
                    p { span { class: "chip", {tx::PREVIEW} } }
                    p { class: "muted small", {tx::PREVIEW_SUB} }
                } else {
                    div { class: "segmented", role: "radiogroup", aria_label: tx::CHANNEL,
                        style: "grid-template-columns: repeat(3, 1fr);",
                        for c in Channel::RELEASE {
                            button {
                                key: "{c.as_str()}",
                                role: "radio",
                                aria_checked: if s.channel == c { "true" } else { "false" },
                                disabled: busy,
                                onclick: move |_| work(move |u| u.set_channel(c)),
                                {channel_label(c)}
                            }
                        }
                    }
                    p { class: "muted small", {tx::CHANNEL_SUB} }
                }
            }
        }
        section { class: "card",
            p { class: "muted small", {tx::SAFETY} }
            p { class: "muted small", {tx::CONTENT} }
        }
    }
}

#[component]
fn PhaseView(phase: Phase, auto: bool) -> Element {
    let action = primary_action(&phase);
    let offer: Option<Offer> = match &phase {
        Phase::Available(o)
        | Phase::Ready(o)
        | Phase::Installing(o)
        | Phase::NeedsPermission(o)
        | Phase::Installed(o)
        | Phase::Downloading { offer: o, .. } => Some(o.clone()),
        Phase::Failed { offer, .. } => offer.clone(),
        _ => None,
    };
    let headline = match &phase {
        Phase::Idle => return rsx! {},
        Phase::Checking => tx::CHECKING,
        Phase::UpToDate => tx::UP_TO_DATE,
        Phase::NotForThisDevice => tx::NOT_FOR_DEVICE,
        Phase::Available(_) => tx::AVAILABLE,
        Phase::Downloading { .. } => tx::DOWNLOADING,
        Phase::Ready(_) => tx::READY,
        Phase::Installing(_) => tx::INSTALLING,
        Phase::NeedsPermission(_) => tx::NEEDS_PERMISSION,
        Phase::Installed(_) => tx::INSTALLED,
        Phase::Failed { why: Failure::Network, .. } => tx::FAILED_NETWORK,
        Phase::Failed { why: Failure::Verify(_), .. } => tx::FAILED_VERIFY,
        Phase::Failed { why: Failure::Install(_), .. } => tx::FAILED_INSTALL,
        Phase::Failed { why: Failure::TooNew, .. } => tx::TOO_NEW,
        Phase::Failed { why: Failure::KeyChanged, .. } => tx::KEY_CHANGED,
    };
    let failed = matches!(phase, Phase::Failed { .. });
    let detail = match &phase {
        Phase::Failed { why: Failure::Verify(d) | Failure::Install(d), .. } => Some(d.clone()),
        _ => None,
    };
    let progress = match &phase {
        Phase::Downloading { offer, done } => Some((*done, offer.asset.size.max(1))),
        _ => None,
    };
    let what_new = offer.as_ref().map(notes).filter(|n| !n.is_empty() && !failed);
    rsx! {
        div { class: "update-state", role: "status", aria_live: "polite",
            p {
                strong {
                    if matches!(phase, Phase::UpToDate) { "✓ " }
                    {headline}
                }
                if let Some(o) = offer.as_ref().filter(|_| !failed) {
                    " "
                    bdi { dir: "ltr", "{o.label()}" }
                    if matches!(phase, Phase::Available(_)) {
                        span { class: "muted", " · " bdi { dir: "ltr", "{megabytes(o.asset.size)}" } }
                    }
                }
            }
            if let Some((done, total)) = progress {
                progress { max: "{total}", value: "{done}" }
            }
            if matches!(phase, Phase::Installing(_)) && cfg!(target_os = "android") {
                p { class: "muted small", {tx::INSTALLING_ANDROID} }
            }
            if matches!(phase, Phase::Available(_)) && auto && cfg!(target_os = "android") {
                p { class: "muted small", {tx::WAITING_WIFI} }
            }
            if matches!(phase, Phase::Installed(_)) {
                p { class: "muted small", {tx::NEXT_LAUNCH} }
            }
            if let Some(n) = what_new {
                p { class: "update-notes",
                    span { class: "muted", {tx::WHATS_NEW} ": " }
                    "{n}"
                }
            }
            if let Some(d) = detail {
                p { class: "muted small", dir: "ltr", bdi { "{d}" } }
            }
            if let Some((label, run)) = action {
                div { class: "row",
                    button { class: "button update-primary", onclick: move |_| run(), {label} }
                    if matches!(phase, Phase::NeedsPermission(_)) {
                        button { class: "button", onclick: move |_| open_install_permission(), {tx::OPEN_PERMISSION} }
                    }
                }
            }
        }
    }
}

fn open_install_permission() {
    #[cfg(target_os = "android")]
    if let Err(e) = crate::platform::install::open_install_permission() {
        crate::diag::report!("{e}");
    }
}

/// The card above the screen once there's something to do: a download to
/// start ("tell me first"), an install, or a restart. "Later" hides it
/// until the next version.
#[component]
pub fn UpdateBanner() -> Element {
    let status = use_update_status();
    let Some(s) = status.read().clone() else { return rsx! {} };
    let Some(offer) = s.banner().cloned() else { return rsx! {} };
    let Some((label, run)) = primary_action(&s.phase) else { return rsx! {} };
    let headline = match s.phase {
        Phase::Available(_) => tx::AVAILABLE,
        Phase::Installed(_) => tx::INSTALLED,
        Phase::NeedsPermission(_) => tx::NEEDS_PERMISSION,
        _ => tx::READY,
    };
    let what_new = notes(&offer);
    rsx! {
        style { {UPDATE_CSS} }
        section { class: "card update-banner", role: "status",
            div { class: "ub-text",
                p {
                    strong { {headline} }
                    " "
                    bdi { dir: "ltr", "{offer.label()}" }
                }
                if !what_new.is_empty() {
                    p { class: "muted small update-notes", "{what_new}" }
                }
            }
            div { class: "row",
                button { class: "button update-primary", onclick: move |_| run(), {label} }
                button {
                    class: "button",
                    onclick: move |_| {
                        if let Some(u) = super::get() {
                            u.dismiss();
                        }
                    },
                    {tx::LATER}
                }
            }
        }
    }
}

/// Follows a tap on the "new version" notification (Android): Settings ›
/// Updates opens, and the notification's button starts the download or the
/// install. Mounted once in the shell.
pub fn use_update_notice_taps(nav: SettingsNav) {
    use_future(move || async move {
        if !cfg!(target_os = "android") || super::get().is_none() {
            return;
        }
        let mut eval = document::eval(
            r#"
            const send = () => { if (!document.hidden) dioxus.send(true); };
            document.addEventListener("visibilitychange", send);
            send();
            await new Promise(() => {});
            "#,
        );
        while eval.recv::<bool>().await.is_ok() {
            let Some(tap) = crate::platform::update_notice::take_tap() else { continue };
            nav.show(Some(Section::Updates));
            match tap.as_str() {
                "download" => work(Updater::download),
                "install" => work(Updater::install),
                _ => {}
            }
        }
    });
}

/// Whether installing now could cut a study session short: never in the
/// middle of studying, a bank block, a case or oral mode.
fn calm(screen: Screen) -> bool {
    matches!(screen, Screen::Today | Screen::Stats | Screen::Settings)
}

/// Android: follows an install in progress, picks up after "Install unknown
/// apps" was allowed, and, in automatic mode, installs a ready update
/// quietly while Batna is in the background (never mid-session, and only
/// where Android needs no confirmation). Mounted once in the shell.
pub fn use_android_install(ui: Ui) {
    let status = use_update_status();
    use_future(move || async move {
        if !cfg!(target_os = "android") || super::get().is_none() {
            return;
        }
        let mut eval = document::eval(
            r#"
            const send = () => dioxus.send(document.hidden);
            document.addEventListener("visibilitychange", send);
            setInterval(() => dioxus.send(null), 1000);
            await new Promise(() => {});
            "#,
        );
        while let Ok(hidden) = eval.recv::<Option<bool>>().await {
            let Some(s) = status.peek().clone() else { continue };
            match (&s.phase, hidden) {
                (Phase::Installing(_), _) => {
                    #[cfg(target_os = "android")]
                    if let Some(u) = super::get() {
                        // Back in front: a confirmation that arrived in the
                        // background (shown as a notification) opens now.
                        if hidden == Some(false) {
                            let _ = crate::platform::install::show_pending_confirm();
                        }
                        u.poll_install();
                    }
                }
                (Phase::NeedsPermission(_), Some(false)) if install_allowed() => work(Updater::install),
                (Phase::Ready(o), Some(true)) if s.prefs.mode == Mode::Auto => {
                    let tried = s.prefs.auto_failed.as_deref() == Some(o.key().as_str());
                    if calm(*ui.screen.peek()) && !tried && install_quietly() {
                        work(Updater::install);
                    }
                }
                _ => {}
            }
        }
    });
}

fn install_allowed() -> bool {
    #[cfg(target_os = "android")]
    {
        crate::platform::install::can_request_installs().unwrap_or(false)
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}

fn install_quietly() -> bool {
    #[cfg(target_os = "android")]
    {
        install_allowed() && crate::platform::install::can_install_quietly().unwrap_or(false)
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_installs_wait_for_a_screen_without_a_session() {
        assert!(calm(Screen::Today));
        assert!(calm(Screen::Stats));
        for busy in [Screen::Study, Screen::Bank, Screen::Cases, Screen::Oral] {
            assert!(!calm(busy), "{busy:?}");
        }
    }
}
