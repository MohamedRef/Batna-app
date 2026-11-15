//! What one update phase says and offers: its headline, the detail under
//! it, a hint, manual steps and the buttons. Settings › Updates, the banner
//! and the "what's new" sheet all read it from here, so a failure can no
//! longer show in one and hide in another (#1338).

use dioxus::prelude::*;

use super::{Failure, Offer, Phase, Status, Updater};
use crate::i18n::update as tx;

/// A manual download asked for on mobile data: the question is open
/// (`MeteredConfirm`, once in the shell, asks it).
pub static METERED_ASK: GlobalSignal<bool> = Signal::global(|| false);

#[cfg(test)]
thread_local! {
    /// Tests pretend to be on mobile data.
    pub static FAKE_METERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn metered() -> bool {
    #[cfg(test)]
    if FAKE_METERED.with(std::cell::Cell::get) {
        return true;
    }
    crate::platform::network::metered()
}

/// A button a phase offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Download,
    Install,
    /// Mac: start the new Batna.
    Restart,
    /// Android: the "install unknown apps" switch for Batna.
    OpenPermission,
    /// The newest preview APK, installed by hand (a renewed preview key).
    GetPreview,
    /// The channel's release page, to install one build by hand.
    OpenReleases,
}

impl Act {
    pub fn label(self, phase: &Phase) -> &'static str {
        match self {
            Self::Download if matches!(phase, Phase::Failed { .. }) => tx::TRY_AGAIN,
            Self::Download => tx::DOWNLOAD,
            Self::Install => match phase {
                Phase::Ready(_) if cfg!(target_os = "macos") => tx::INSTALL_RESTART,
                Phase::Ready(_) => tx::INSTALL_NOW,
                _ => tx::TRY_AGAIN,
            },
            Self::Restart => tx::RESTART_NOW,
            Self::OpenPermission => tx::OPEN_PERMISSION,
            Self::GetPreview => tx::GET_PREVIEW,
            Self::OpenReleases => tx::OPEN_RELEASES,
        }
    }

    /// Does it, off the UI thread where it blocks.
    pub fn run(self) {
        match self {
            // On mobile data he decides first: the size, now or on Wi-Fi.
            Self::Download if metered() => *METERED_ASK.write() = true,
            Self::Download => work(Updater::download),
            Self::Install => work(Updater::install),
            Self::Restart => {
                #[cfg(target_os = "macos")]
                super::restart();
            }
            Self::OpenPermission =>
            {
                #[cfg(target_os = "android")]
                if let Err(e) = crate::platform::install::open_install_permission() {
                    crate::diag::report!("{e}");
                }
            }
            Self::GetPreview => open(&preview_apk_url()),
            Self::OpenReleases => open(&releases_url()),
        }
    }
}

/// Runs a blocking updater call off the UI thread.
pub(super) fn work(f: impl FnOnce(&'static Updater) + Send + 'static) {
    if let Some(u) = super::get() {
        let _ = std::thread::Builder::new().name("batna-update".into()).spawn(move || f(u));
    }
}

fn open(url: &str) {
    if let Err(e) = crate::cloud::open_url(url) {
        crate::diag::report!("{e}");
    }
}

/// The newest preview APK, next to the preview feed.
pub(super) fn preview_apk_url() -> String {
    let feed = batna_core::update::Channel::Preview.feed_url();
    let base = feed.rsplit_once('/').map_or(feed.as_str(), |(b, _)| b);
    format!("{base}/batna-preview.apk")
}

/// The release page of the channel he follows.
pub(super) fn releases_url() -> String {
    let base = batna_core::update::FEED_BASE.replace("/releases/download", "/releases/tag");
    match super::get() {
        Some(u) => format!("{base}/{}", u.installed().channel.release_tag()),
        None => base.trim_end_matches("/tag").to_owned(),
    }
}

/// Everything a phase shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Stage {
    pub headline: &'static str,
    /// The offer it is about (a failure may have none).
    pub offer: Option<Offer>,
    pub failed: bool,
    /// The system's own words for a failure (LTR, small).
    pub detail: Option<String>,
    /// One calm line under the headline.
    pub hint: Option<&'static str>,
    /// Steps to do by hand.
    pub steps: &'static [&'static str],
    /// (done, total) bytes while downloading.
    pub progress: Option<(u64, u64)>,
    /// The first is the main button.
    pub acts: Vec<Act>,
}

/// What `phase` shows; `None` before any check. `android` picks the
/// Android-only hint (a parameter, so tests can ask for it).
pub fn stage(phase: &Phase, android: bool) -> Option<Stage> {
    let offer = match phase {
        Phase::Available(o)
        | Phase::Ready(o)
        | Phase::Installing(o)
        | Phase::NeedsPermission(o)
        | Phase::Installed(o)
        | Phase::Downloading { offer: o, .. } => Some(o.clone()),
        Phase::Failed { offer, .. } => offer.clone(),
        _ => None,
    };
    let (headline, hint, steps, acts): (_, _, &'static [&'static str], _) = match phase {
        Phase::Idle => return None,
        Phase::Checking => (tx::CHECKING, None, &[], vec![]),
        Phase::UpToDate => (tx::UP_TO_DATE, None, &[], vec![]),
        Phase::NotForThisDevice => (tx::NOT_FOR_DEVICE, None, &[], vec![]),
        Phase::Available(_) => (tx::AVAILABLE, None, &[], vec![Act::Download]),
        Phase::Downloading { .. } => (tx::DOWNLOADING, None, &[], vec![]),
        Phase::Ready(_) => (tx::READY, None, &[], vec![Act::Install]),
        Phase::Installing(_) => (tx::INSTALLING, android.then_some(tx::INSTALLING_ANDROID), &[], vec![]),
        Phase::NeedsPermission(_) => {
            (tx::NEEDS_PERMISSION, Some(tx::PERMISSION_HINT), &[], vec![Act::OpenPermission, Act::Install])
        }
        Phase::Installed(_) => (tx::INSTALLED, Some(tx::NEXT_LAUNCH), &[], vec![Act::Restart]),
        Phase::Failed { why, offer } => {
            let retry = |a: Act| if offer.is_some() { vec![a] } else { vec![] };
            match why {
                Failure::Network => (tx::FAILED_NETWORK, None, &[], retry(Act::Download)),
                Failure::Verify(_) => (tx::FAILED_VERIFY, None, &[], retry(Act::Download)),
                Failure::Install(_) => (tx::FAILED_INSTALL, None, &[], retry(Act::Install)),
                Failure::TooNew => (tx::TOO_NEW, None, &[], vec![Act::OpenReleases]),
                Failure::KeyChanged => (tx::KEY_CHANGED, None, &tx::KEY_CHANGED_STEPS, vec![Act::GetPreview]),
            }
        }
    };
    let detail = match phase {
        Phase::Failed { why: Failure::Verify(d) | Failure::Install(d), .. } => Some(d.clone()),
        _ => None,
    };
    let progress = match phase {
        Phase::Downloading { offer, done } => Some((*done, offer.asset.size.max(1))),
        _ => None,
    };
    Some(Stage {
        headline,
        offer,
        failed: matches!(phase, Phase::Failed { .. }),
        detail,
        hint,
        steps,
        progress,
        acts,
    })
}

/// A button's text; a download names its size («نزّلها · 46.6 ميجا»), the
/// number kept LTR.
#[component]
pub fn ActLabel(act: Act, phase: Phase, size: Option<u64>) -> Element {
    let size = size.filter(|_| act == Act::Download).map(super::view::megabytes);
    rsx! {
        {act.label(&phase)}
        if let Some(mb) = size {
            " · "
            bdi { dir: "ltr", class: "num-ltr", "{mb}" }
        }
    }
}

/// The mobile-data question, once in the shell.
#[component]
pub fn MeteredConfirm() -> Element {
    let status = super::view::use_update_status();
    rsx! { MeteredConfirmView { status } }
}

/// The question for a given status (what [`MeteredConfirm`] follows).
#[component]
pub fn MeteredConfirmView(status: Signal<Option<Status>>) -> Element {
    if !METERED_ASK() {
        return rsx! {};
    }
    let size = status
        .read()
        .as_ref()
        .and_then(|s| stage(&s.phase, false))
        .and_then(|st| st.offer)
        .map(|o| o.asset.size);
    let close = move || *METERED_ASK.write() = false;
    rsx! {
        style { {super::view::UPDATE_CSS} }
        div { class: "um-backdrop", onclick: move |_| close() }
        div { class: "um-dialog", role: "alertdialog", aria_modal: "true", aria_labelledby: "um-title",
            h2 { id: "um-title", {tx::METERED_TITLE} }
            p {
                {tx::METERED_BODY}
                if let Some(b) = size {
                    " ("
                    bdi { dir: "ltr", class: "num-ltr", "{super::view::megabytes(b)}" }
                    ")"
                }
            }
            p { class: "muted small", {tx::METERED_NOTE} }
            div { class: "row",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        close();
                        work(Updater::download);
                    },
                    {tx::METERED_NOW}
                }
                button {
                    class: "btn btn-secondary",
                    "data-back": "true",
                    onclick: move |_| {
                        close();
                        work(|u| u.download_on_wifi());
                    },
                    {tx::METERED_WAIT}
                }
            }
        }
    }
}
