//! Settings › Updates, the gentle "new Batna is ready" card above every
//! screen, and the hooks that follow the updater from the UI.
//!
//! Adapted from kabar-app main: crates/app/src/update/view.rs (one
//! language, Batna's settings rows and screens).

use batna_core::update::Channel;
use dioxus::prelude::*;

use super::sheet::{SheetMode, UpdateSheetNav};
use super::stage::{Act, Stage, stage};
use super::{ContentCheck, Failure, Mode, Off, Offer, Phase, Status, Updater};
use crate::i18n::update as tx;
use crate::settings::{DeviceOnly, Section, SettingsNav};
use crate::{Screen, Ui};

pub(super) const UPDATE_CSS: &str = r#"
.update-state { display: grid; gap: var(--sp-2); padding: var(--sp-3);
  border-radius: var(--radius); background: var(--accent-soft); }
.update-state progress { inline-size: 100%; accent-color: var(--accent); }
.update-notes { white-space: pre-line; }
.update-banner { flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--sp-2) var(--sp-3);
  border-inline-start: 4px solid var(--accent); margin-block-end: var(--sp-3); }
.update-banner .ub-text { flex: 1 1 14rem; min-inline-size: 0; display: grid; gap: var(--sp-1); }
.content-note { flex-direction: row; flex-wrap: wrap; align-items: center; gap: var(--sp-2);
  border-inline-start: 4px solid var(--accent); margin-block-end: var(--sp-3); padding-block: var(--sp-2); }
/* The news gets a line of its own when narrow (the buttons go under it,
   side by side) and at most two lines. */
.content-note p { flex: 1 1 14rem; min-inline-size: 0; margin: 0;
  display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; overflow: hidden; }
.content-note .btn, .content-note .icon-button { flex: none; }
.ur-card { padding: var(--sp-1) 0; gap: 0; }
.ur-row { display: flex; align-items: center; gap: var(--sp-3); inline-size: 100%; min-block-size: var(--tap-lg);
  padding: var(--sp-2) var(--sp-3); background: none; border: 0; color: inherit; text-align: start; font: inherit; cursor: pointer; }
.ur-row + .ur-row { border-block-start: 1px solid var(--line); }
.ur-row:disabled { cursor: default; }
.ur-row:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
.ur-text { flex: 1; min-inline-size: 0; display: grid; gap: 2px; }
.ur-title { font-weight: 600; }
.ur-sub { color: var(--muted); font-size: var(--fs-sm); overflow-wrap: anywhere; }
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

pub(super) use super::stage::work;

/// What's new in `o`: up to `max` of its changes (new things first), or
/// the feed's notes from before the changelog.
fn notes(o: &Offer, max: usize) -> String {
    if o.changes.is_empty() {
        super::notes_of(o)
    } else {
        super::changelog::lines_upto(&o.changes, tx::notice_more, max)
    }
}

pub(super) fn megabytes(bytes: u64) -> String {
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

/// Settings › Updates.
#[component]
pub fn UpdatesSection() -> Element {
    let status = use_update_status();
    rsx! { UpdatesView { status } }
}

/// Settings › Updates for a given status (what [`UpdatesSection`] follows).
#[component]
pub(super) fn UpdatesView(status: Signal<Option<Status>>) -> Element {
    let s = status.read().clone();
    let build = super::build_number();
    let commit = super::commit();
    let busy = s.as_ref().is_some_and(|s| {
        matches!(s.phase, Phase::Checking | Phase::Downloading { .. } | Phase::Installing(_))
    });
    // Updates that are off for a reason this build already knows (no data
    // folder at all counts as a development build).
    let off = s.as_ref().map_or(Some(Off::DevBuild), |s| s.off);
    let has_offer = s.as_ref().is_some_and(|s| {
        matches!(
            s.phase,
            Phase::Available(_)
                | Phase::Downloading { .. }
                | Phase::Ready(_)
                | Phase::NeedsPermission(_)
                | Phase::Installed(_)
        )
    });
    let sheet = try_use_context::<UpdateSheetNav>();

    // The content row opens what the content update brought while it is
    // new to him, else the whole changelog.
    let content_news = s.as_ref().is_some_and(Status::content_news);
    let open_changes = move |mode: SheetMode| {
        move |_| {
            if let Some(sheet) = sheet {
                sheet.open(mode);
            }
        }
    };
    let content_mode = if content_news { SheetMode::Updated } else { SheetMode::History };
    // Always a line under the content row, so he sees where the lessons
    // stand: a content-only update ahead of the app, the app's own lessons,
    // and a newer pack on its way.
    let fetching = s.as_ref().is_some_and(|s| s.content_check == Some(ContentCheck::Fetching));
    let content_sub = match s.as_ref().and_then(Status::content_ahead) {
        Some(c) => {
            let when = s.as_ref().and_then(|s| s.content_at).map(|at| {
                let now = super::now_ms();
                format!(" · {} {}", tx::CONTENT_PUBLISHED, crate::cloud::ago((at.max(0) as u64) * 1000, now))
            });
            rsx! {
                {tx::BUILD} " "
                bdi { dir: "ltr", "{c}" }
                {when.unwrap_or_default()}
            }
        }
        None => rsx! { {tx::CONTENT_WITH_APP} },
    };
    let last_check = s.as_ref().and_then(|s| s.prefs.last_check_ms);
    let feed_down = s.as_ref().is_some_and(|s| {
        matches!(s.phase, Phase::Failed { why: Failure::Network, offer: None })
            && s.content_check == Some(ContentCheck::Failed)
            && !s.content_waiting
    });

    rsx! {
        style { {UPDATE_CSS} }
        // What is on this device: the app, and the lessons/questions/look
        // apart from it, since a content-only update changes only the second.
        div { class: "section-head", h2 { id: "updates-title", {tx::GROUP_HAVE} } }
        crate::ui::Card { class: "ur-card", aria_labelledby: "updates-title",
            button {
                class: "ur-row",
                disabled: sheet.is_none(),
                onclick: open_changes(SheetMode::History),
                span { class: "row-icon", aria_hidden: "true", {crate::icons::section("updates")} }
                span { class: "ur-text",
                    span { class: "ur-title", {tx::ROW_APP} }
                    span { class: "ur-sub",
                        // Each Latin piece isolated on its own, so the Arabic
                        // word "build" doesn't reorder the numbers around it.
                        bdi { dir: "ltr", {env!("CARGO_PKG_VERSION")} }
                        if build > 0 {
                            " · {tx::BUILD} "
                            bdi { dir: "ltr", "{build}" }
                        }
                        if let Some(c) = commit {
                            " · "
                            bdi { dir: "ltr", "{c}" }
                        }
                    }
                }
                span { class: "chev", aria_hidden: "true" }
            }
            button {
                class: "ur-row",
                disabled: sheet.is_none(),
                onclick: open_changes(content_mode),
                span { class: "row-icon", aria_hidden: "true", {crate::icons::book()} }
                span { class: "ur-text",
                    span { class: "ur-title", {tx::ROW_CONTENT} }
                    span { class: "ur-sub",
                        {content_sub}
                        if fetching {
                            " · " {tx::CONTENT_ROW_FETCHING}
                        }
                    }
                }
                if content_news {
                    span { class: "chip", "data-tone": "good", {tx::BADGE_NEW} }
                }
                span { class: "chev", aria_hidden: "true" }
            }
        }
        div { class: "section-head", h2 { {tx::GROUP_CHECK} } }
        crate::ui::Card {
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
                // The feed itself out of reach is one line, not two.
                if feed_down {
                    PhaseView { phase: s.phase.clone(), auto: s.prefs.mode == Mode::Auto, label: false }
                } else {
                    PhaseView { phase: s.phase.clone(), auto: s.prefs.mode == Mode::Auto, label: true }
                    ContentResult { status: s.clone() }
                }
                p { class: "muted small ur-last",
                    match last_check {
                        Some(t) => rsx! { {tx::LAST_CHECK} " " {crate::cloud::ago(t, super::now_ms())} },
                        None => rsx! { {tx::NEVER_CHECKED} },
                    }
                }
                div { class: "row",
                    button {
                        class: "btn btn-primary",
                        disabled: busy,
                        onclick: move |_| work(|u| u.check(false)),
                        {tx::CHECK_NOW}
                    }
                    if let Some(sheet) = sheet {
                        if has_offer {
                            button { class: "btn btn-secondary", onclick: move |_| sheet.open(SheetMode::Offer), {tx::WHATS_NEW_ASK} }
                        }
                        button { class: "btn btn-ghost", onclick: move |_| sheet.open(SheetMode::History), {tx::HISTORY} }
                    }
                }
            }
        }
        if let Some(s) = s.filter(|s| s.off.is_none()) {
            crate::ui::Card {
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
                crate::ui::Card {
                    label { class: "update-toggle",
                        span { class: "update-toggle-text",
                            span { {tx::NOTIFY} DeviceOnly {} }
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
            crate::ui::Card {
                h2 { {tx::CHANNEL} DeviceOnly {} }
                div { class: "segmented", role: "radiogroup", aria_label: tx::CHANNEL,
                    style: "grid-template-columns: repeat(4, 1fr);",
                    for c in Channel::ALL {
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
                if s.channel == Channel::Preview {
                    p { class: "muted small", {tx::PREVIEW_CHANNEL_NOTE} }
                }
            }
        }
    }
}

/// The Settings list's line for Updates: what is going on, or when it last
/// looked. `None` keeps the section's own summary (updates off, never looked).
pub fn summary(s: &Status) -> Option<String> {
    if s.off.is_some() {
        return None;
    }
    let line = match &s.phase {
        Phase::Checking => tx::CHECKING,
        Phase::Available(_) | Phase::Downloading { .. } => tx::AVAILABLE,
        Phase::Ready(_) | Phase::NeedsPermission(_) => tx::READY,
        _ if s.content_waiting => tx::SUMMARY_CONTENT_WAITING,
        _ if s.content_news() => tx::CONTENT_NOTE,
        Phase::Failed { .. } => tx::SUMMARY_FAILED,
        _ => {
            let t = s.prefs.last_check_ms?;
            return Some(format!("{} {}", tx::LAST_CHECK, crate::cloud::ago(t, super::now_ms())));
        }
    };
    Some(line.to_owned())
}

/// What the last check found for the lessons and questions. A waiting
/// pack says so whatever came after; nothing shows before the first check.
#[component]
fn ContentResult(status: Status) -> Element {
    let (mark, line) = match (status.content_waiting, status.content_check) {
        (true, _) | (_, Some(ContentCheck::New)) => ("", tx::CONTENT_NEW),
        (false, Some(ContentCheck::Current)) => ("✓ ", tx::CONTENT_CURRENT),
        (false, Some(ContentCheck::Failed)) => ("", tx::CONTENT_FAILED),
        (false, Some(ContentCheck::Fetching)) => ("", tx::CONTENT_FETCHING),
        (false, None) => return rsx! {},
    };
    // The same box as the app's half above, so the two read as a pair.
    rsx! {
        div { class: "update-state", role: "status", aria_live: "polite",
            p {
                strong { {tx::RESULT_CONTENT} ": " }
                strong { "{mark}" {line} }
            }
        }
    }
}

#[component]
fn PhaseView(phase: Phase, auto: bool, label: bool) -> Element {
    let _ = auto;
    let Some(st) = stage(&phase, cfg!(target_os = "android")) else { return rsx! {} };
    let failed = st.failed;
    let what_new = st.offer.as_ref().map(|o| notes(o, 3)).filter(|n| !n.is_empty() && !failed);
    // Changelog lines are bullets already; old notes get a "What's new:".
    let prefix = st.offer.as_ref().is_some_and(|o| o.changes.is_empty());
    rsx! {
        div { class: "update-state", role: "status", aria_live: "polite",
            p {
                // The content's half shows right under it.
                if label {
                    strong { {tx::RESULT_APP} ": " }
                }
                strong {
                    if matches!(phase, Phase::UpToDate) { "✓ " }
                    {st.headline}
                }
                if let Some(o) = st.offer.as_ref().filter(|_| !failed) {
                    " "
                    bdi { dir: "ltr", "{o.label()}" }
                    if matches!(phase, Phase::Available(_)) {
                        span { class: "muted", " · " bdi { dir: "ltr", "{megabytes(o.asset.size)}" } }
                    }
                }
            }
            if let Some((done, total)) = st.progress {
                progress { max: "{total}", value: "{done}" }
            }
            StageExtras { stage: st.clone() }
            if let Some(n) = what_new {
                p { class: "update-notes",
                    if prefix {
                        span { class: "muted", {tx::WHATS_NEW} ": " }
                    }
                    "{n}"
                }
            }
            StageButtons { phase: phase.clone(), acts: st.acts.clone(), class: "row" }
        }
    }
}

/// A stage's hint, the system's detail and any steps by hand: the same in
/// Settings and in the sheet.
#[component]
pub(super) fn StageExtras(stage: Stage) -> Element {
    rsx! {
        if let Some(h) = stage.hint {
            p { class: "muted small us-hint", {h} }
        }
        if let Some(d) = stage.detail {
            p { class: "muted small", dir: "ltr", bdi { "{d}" } }
        }
        if !stage.steps.is_empty() {
            ol { class: "small",
                for (i, step) in stage.steps.iter().enumerate() {
                    li { key: "{i}", {*step} }
                }
            }
        }
    }
}

/// A stage's buttons, the first one primary.
#[component]
pub(super) fn StageButtons(phase: Phase, acts: Vec<Act>, class: &'static str) -> Element {
    if acts.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "{class}",
            for (i, act) in acts.into_iter().enumerate() {
                button {
                    key: "{i}",
                    class: if i == 0 { "btn btn-primary" } else { "btn btn-secondary" },
                    onclick: move |_| act.run(),
                    {act.label(&phase)}
                }
            }
        }
    }
}

/// The card above the screen once there's something to do: a download to
/// start ("tell me first"), an install, or a restart. "Later" hides it
/// until the next version.
#[component]
pub fn UpdateBanner() -> Element {
    let status = use_update_status();
    rsx! { BannerView { status } }
}

#[component]
pub(super) fn BannerView(status: Signal<Option<Status>>) -> Element {
    let Some(s) = status.read().clone() else { return rsx! {} };
    let Some(offer) = s.banner().cloned() else { return rsx! {} };
    let Some(st) = stage(&s.phase, cfg!(target_os = "android")) else { return rsx! {} };
    if st.acts.is_empty() {
        return rsx! {};
    }
    let headline = st.headline;
    // The banner stays short: two lines, the sheet has the rest.
    let what_new = notes(&offer, 2);
    let sheet = try_use_context::<UpdateSheetNav>();
    rsx! {
        style { {UPDATE_CSS} }
        crate::ui::Card { class: "update-banner", role: "status",
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
                for (i, act) in st.acts.iter().copied().enumerate() {
                    button {
                        key: "{i}",
                        class: if i == 0 { "btn btn-primary" } else { "btn btn-secondary" },
                        onclick: move |_| act.run(),
                        {act.label(&s.phase)}
                    }
                }
                if let Some(sheet) = sheet {
                    button { class: "btn btn-secondary", onclick: move |_| sheet.open(SheetMode::Offer), {tx::WHATS_NEW_ASK} }
                }
                button {
                    class: "btn btn-ghost",
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

/// After a content-only update: one quiet line at the top of Today with the
/// update's first change, instead of the sheet opening by itself. A tap
/// opens "what's new"; ✕ marks it seen (#336).
#[component]
pub fn ContentNote() -> Element {
    let status = use_update_status();
    rsx! { ContentNoteView { status } }
}

#[component]
pub(super) fn ContentNoteView(status: Signal<Option<Status>>) -> Element {
    let Some(s) = status.read().clone() else { return rsx! {} };
    if !s.content_only_news() {
        return rsx! {};
    }
    let first = s
        .whats_new()
        .iter()
        .flat_map(|b| b.changes.iter())
        .map(|c| if c.text.ar_eg.is_empty() { c.text.en.clone() } else { c.text.ar_eg.clone() })
        .next();
    let sheet = try_use_context::<UpdateSheetNav>();
    rsx! {
        style { {UPDATE_CSS} }
        crate::ui::Card { class: "content-note", role: "status",
            p {
                strong { {tx::CONTENT_NOTE} }
                if let Some(f) = first {
                    ": {f}"
                }
            }
            if let Some(sheet) = sheet {
                button { class: "btn btn-secondary", onclick: move |_| sheet.open(SheetMode::Updated), {tx::WHATS_NEW_ASK} }
            }
            button {
                class: "icon-button",
                aria_label: "{tx::CLOSE}",
                onclick: move |_| {
                    if let Some(u) = super::get() {
                        u.close_whats_new();
                    }
                },
                span { aria_hidden: "true", "✕" }
            }
        }
    }
}

/// Follows a tap on the "new version" notification (Android): the "what's
/// new" sheet opens, and the notification's button starts the download or
/// the install. Mounted once in the shell.
///
/// The tap's trampoline activity is translucent, so a tap while Batna is
/// open may not hide the page at all: besides visibilitychange, it looks on
/// focus, pageshow and every two seconds while visible (Batna-app#165). The
/// Kotlin side drops taps over a minute old.
pub fn use_update_notice_taps(nav: SettingsNav) {
    let sheet = try_use_context::<UpdateSheetNav>();
    use_future(move || async move {
        if !cfg!(target_os = "android") || super::get().is_none() {
            return;
        }
        let mut eval = document::eval(
            r#"
            const send = () => { if (!document.hidden) dioxus.send(true); };
            document.addEventListener("visibilitychange", send);
            window.addEventListener("focus", send);
            window.addEventListener("pageshow", send);
            setInterval(send, 2000);
            send();
            await new Promise(() => {});
            "#,
        );
        while eval.recv::<bool>().await.is_ok() {
            let Some(tap) = crate::platform::update_notice::take_tap() else { continue };
            // The sheet says what's new and holds the button; without it
            // (never in the shell), Settings › Updates does.
            match sheet {
                Some(sheet) => sheet.open_offer(),
                None => nav.show(Some(Section::Updates)),
            }
            match tap_asks(&tap) {
                Some(Asked::Download) => work(Updater::download),
                Some(Asked::Install) => work(Updater::install),
                None => {}
            }
        }
    });
}

/// What a notification button asked the updater for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Asked {
    Download,
    Install,
}

/// The notification's button (`open` only opens the sheet).
fn tap_asks(tap: &str) -> Option<Asked> {
    match tap {
        "download" => Some(Asked::Download),
        "install" => Some(Asked::Install),
        _ => None,
    }
}

/// Whether installing now could cut a study session short: never in the
/// middle of studying, a bank block, a case or oral mode.
fn calm(screen: Screen) -> bool {
    matches!(screen, Screen::Today | Screen::Stats | Screen::Settings)
}

/// Whether something open over a calm screen would be lost by the restart:
/// a lecture he is reading (and where he scrolled to), a card he is
/// writing, or the first-run questions.
fn overlay_open() -> bool {
    crate::lecture::is_open() || crate::own::drafting() || *crate::onboarding::OPEN.peek()
}

/// The startup guard must not certify a pack until the process has opened it.
/// The first tick after a real load marks two seconds of stable operation.
fn should_settle_content(loaded: bool, settled_ticks: u32) -> bool {
    loaded && settled_ticks == 0
}

/// A waiting content pack switches in only between sessions, with nothing
/// open that the redraw would lose (`overlay` is asked only then).
fn should_switch(waiting: bool, screen: Screen, overlay: impl FnOnce() -> bool) -> bool {
    waiting && calm(screen) && !overlay()
}

/// Switches to a content pack that arrived while Batna is open, once he is
/// between sessions (the calm moments a quiet install waits for too): the
/// lectures, lessons and look change under him with no install, and the
/// screens are drawn again. Mounted once in the shell.
pub fn use_content_switch(ui: Ui) {
    let status = use_update_status();
    let review = try_use_context::<crate::store::Review>();
    use_future(move || {
        let review = review.clone();
        async move {
            // Ticks since the pack in use was opened: once Batna has drawn
            // its first screen and run a couple of seconds on it, the
            // crash-loop guard starts counting again (#344).
            let mut loaded_for = 0u32;
            loop {
                crate::platform::time::sleep(std::time::Duration::from_secs(2)).await;
                // Count calm time only after the library is genuinely loaded.
                // Otherwise a slow first render can mark a bad pack as good.
                if should_settle_content(crate::content::Library::is_initialized(), loaded_for) {
                    loaded_for += 1;
                    super::pack_settled();
                }
                if super::get().is_none() {
                    continue;
                }
                let waiting = status.peek().as_ref().is_some_and(|s| s.content_waiting);
                if should_switch(waiting, *ui.screen.peek(), overlay_open)
                    && super::get().is_some_and(|u| u.apply_content())
                {
                    if let Some(r) = &review {
                        r.backend.borrow_mut().library_replaced();
                    }
                    *crate::content::REV.write() += 1;
                    loaded_for = 0;
                }
            }
        }
    });
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
            let now_calm = || calm(*ui.screen.peek()) && !overlay_open();
            match install_step(&s, hidden, now_calm, install_allowed, install_quietly) {
                InstallStep::Poll { front } => {
                    #[cfg(target_os = "android")]
                    if let Some(u) = super::get() {
                        // Back in front: a confirmation that arrived in the
                        // background (shown as a notification) opens now.
                        if front {
                            let _ = crate::platform::install::show_pending_confirm();
                        }
                        u.poll_install();
                    }
                    #[cfg(not(target_os = "android"))]
                    let _ = front;
                }
                InstallStep::Install => work(Updater::install),
                InstallStep::Wait => {}
            }
        }
    });
}

/// What one tick of the Android install watcher does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstallStep {
    /// An install is running: follow it (`front`: Batna just came back).
    Poll {
        front: bool,
    },
    Install,
    Wait,
}

/// `hidden`: the page just went to the background (`Some(true)`), came
/// back (`Some(false)`), or neither (a timer tick). The checks that ask
/// Android run only when they matter.
fn install_step(
    s: &Status,
    hidden: Option<bool>,
    calm_now: impl FnOnce() -> bool,
    allowed: impl FnOnce() -> bool,
    quiet: impl FnOnce() -> bool,
) -> InstallStep {
    match (&s.phase, hidden) {
        (Phase::Installing(_), _) => InstallStep::Poll { front: hidden == Some(false) },
        // Back from "Install unknown apps" with the permission granted.
        (Phase::NeedsPermission(_), Some(false)) if allowed() => InstallStep::Install,
        // Automatic mode installs a ready update quietly once Batna is in the
        // background, never mid-session and never twice after a failure.
        (Phase::Ready(o), Some(true)) if s.prefs.mode == Mode::Auto => {
            let tried = s.prefs.auto_failed.as_deref() == Some(o.key().as_str());
            if !tried && calm_now() && quiet() { InstallStep::Install } else { InstallStep::Wait }
        }
        _ => InstallStep::Wait,
    }
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

#[cfg(test)]
mod settle_tests {
    use super::should_settle_content;

    #[test]
    fn a_timer_before_library_load_never_settles_the_pack() {
        assert!(!should_settle_content(false, 0));
        assert!(!should_settle_content(false, 1));
        assert!(should_settle_content(true, 0));
        assert!(!should_settle_content(true, 1));
    }
}

#[cfg(test)]
mod watcher_tests {
    use super::super::tests::{offer, status};
    use super::*;

    const YES: fn() -> bool = || true;
    const NO: fn() -> bool = || false;

    #[test]
    fn the_install_watcher_follows_installs_and_waits_for_calm() {
        let o = offer("0.2.0", 50);
        let installing = status(Phase::Installing(o.clone()), Mode::Notify);
        assert_eq!(install_step(&installing, Some(false), NO, NO, NO), InstallStep::Poll { front: true });
        assert_eq!(install_step(&installing, None, NO, NO, NO), InstallStep::Poll { front: false });

        let asked = status(Phase::NeedsPermission(o.clone()), Mode::Notify);
        assert_eq!(install_step(&asked, Some(false), NO, YES, NO), InstallStep::Install, "allowed meanwhile");
        assert_eq!(install_step(&asked, Some(false), NO, NO, NO), InstallStep::Wait);
        assert_eq!(install_step(&asked, Some(true), NO, YES, NO), InstallStep::Wait, "only on coming back");

        let mut ready = status(Phase::Ready(o.clone()), Mode::Auto);
        assert_eq!(
            install_step(&ready, Some(true), YES, NO, YES),
            InstallStep::Install,
            "quietly, in the background"
        );
        assert_eq!(install_step(&ready, Some(true), NO, NO, YES), InstallStep::Wait, "mid-session");
        assert_eq!(install_step(&ready, Some(true), YES, NO, NO), InstallStep::Wait, "Android would ask");
        assert_eq!(
            install_step(&ready, Some(false), YES, NO, YES),
            InstallStep::Wait,
            "in front: the banner asks"
        );
        ready.prefs.auto_failed = Some(o.key());
        assert_eq!(
            install_step(&ready, Some(true), YES, NO, YES),
            InstallStep::Wait,
            "never twice after a failure"
        );
        let notify = status(Phase::Ready(o), Mode::Notify);
        assert_eq!(
            install_step(&notify, Some(true), YES, NO, YES),
            InstallStep::Wait,
            "he chose to be asked"
        );
        assert_eq!(
            install_step(&status(Phase::UpToDate, Mode::Auto), None, YES, YES, YES),
            InstallStep::Wait
        );
        // Away from Android nothing may install itself.
        assert_eq!(
            install_step(&asked, Some(false), YES, install_allowed, install_quietly),
            InstallStep::Wait
        );
        let ready = status(Phase::Ready(offer("0.3.0", 60)), Mode::Auto);
        assert_eq!(
            install_step(&ready, Some(true), YES, install_allowed, install_quietly),
            InstallStep::Wait
        );
    }

    #[test]
    fn content_switches_between_sessions_and_taps_name_their_step() {
        assert!(should_switch(true, Screen::Today, || false));
        assert!(!should_switch(true, Screen::Study, || false), "never mid-session");
        assert!(!should_switch(true, Screen::Today, || true), "a lecture or a draft is open");
        assert!(!should_switch(false, Screen::Today, || panic!("not asked without a pack")));
        assert_eq!(tap_asks("download"), Some(Asked::Download));
        assert_eq!(tap_asks("install"), Some(Asked::Install));
        assert_eq!(tap_asks("open"), None);
    }

    #[test]
    fn the_shell_hooks_mount_quietly_without_an_updater() {
        crate::store::use_memory_store();
        fn app() -> Element {
            let nav = use_context_provider(crate::settings::SettingsNav::new);
            use_context_provider(super::super::sheet::UpdateSheetNav::new);
            let ui = use_context_provider(|| Ui {
                screen: Signal::new(Screen::Today),
                prefs: Signal::new(Default::default()),
                focus: Signal::new(false),
            });
            crate::store::use_provide_review();
            use_update_notice_taps(nav);
            use_content_switch(ui);
            use_android_install(ui);
            rsx! {
                UpdatesSection {}
                UpdateBanner {}
                ContentNote {}
                super::super::sheet::UpdateSheet {}
            }
        }
        let t = crate::tap_dom::TestDom::of(app);
        let page = t.html();
        assert!(page.contains(tx::OFF_DEV), "no updater: a development build says so: {page}");
        assert!(!page.contains("role=\"dialog\""), "nothing to tell: {page}");
    }
}
