//! First run: a few calm questions that set up the planner, the session
//! length, the reminders and the colours. Every step can be skipped, and
//! Settings › Study opens it again.

use batna_core::plan::Day;
use batna_core::theme::Theme;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Ui;
use crate::content::Library;
use crate::i18n::onboard as tx;
use crate::reminders::{MAX_TIMES, RemindPrefs, hhmm, parse_hhmm};
use crate::store::{now_ms, shared};

const CSS: &str = r#"
.onboard-backdrop { position: fixed; inset: 0; z-index: 50; background: var(--bg); overflow-y: auto; }
.onboard { max-inline-size: 36rem; margin-inline: auto; padding: var(--sp-4); display: grid; gap: var(--sp-3); }
.onboard-steps { display: flex; gap: var(--sp-1); }
.onboard-steps span { flex: 1; block-size: 4px; border-radius: 2px; background: var(--line); }
.onboard-steps span[data-done="true"] { background: var(--accent); }
.onboard-chips { display: flex; flex-wrap: wrap; gap: var(--sp-2); }
.onboard-chips button { min-block-size: 44px; padding-inline: var(--sp-3); border-radius: var(--radius-pill); border: 1px solid var(--line); background: var(--surface); }
.onboard-chips button[aria-pressed="true"] { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); font-weight: 700; }
.onboard-actions { display: flex; gap: var(--sp-2); justify-content: space-between; flex-wrap: wrap; }
"#;

/// Session lengths offered, in minutes (the study screen's choices).
pub const MINUTE_CHOICES: [u32; 4] = [5, 10, 15, 20];

/// His answers that no other settings own, synced as `onboard`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Answers {
    /// Minutes a day; the study screen's default session length.
    pub minutes: Option<u32>,
    /// Lectures he had already studied before Batna (`hb.L01`, ...).
    pub studied: Vec<String>,
}

impl Answers {
    const KEY: &str = "onboard";

    pub fn load() -> Self {
        shared().setting(Self::KEY).unwrap_or_default()
    }

    fn save(&self) {
        if let Err(e) = shared().put_setting(Self::KEY, self) {
            eprintln!("saving first-run answers: {e}");
        }
    }
}

/// Default study session length: his first-run answer, else 10 minutes.
pub fn session_minutes() -> u32 {
    Answers::load().minutes.filter(|m| MINUTE_CHOICES.contains(m)).unwrap_or(10)
}

/// Shown on this device (device-only: each device asks once, or never
/// once he has skipped or finished it). BATNA_ONBOARD=1 forces it open;
/// a launch that previews another screen or Settings skips it.
pub static OPEN: GlobalSignal<bool> = Signal::global(|| {
    let env = |k: &str| std::env::var_os(k).is_some();
    if env("BATNA_ONBOARD") {
        return true;
    }
    let previewing = ["BATNA_OPEN_SCREEN", "BATNA_OPEN_SETTINGS", "BATNA_STATS_TAB", "BATNA_PLAN_EXAM"];
    !previewing.iter().any(|k| env(k)) && !shared().local_setting::<bool>(SEEN).unwrap_or(false)
});

const SEEN: &str = "onboarded";

fn close() {
    if let Err(e) = shared().put_local_setting(SEEN, &true) {
        eprintln!("saving first-run flag: {e}");
    }
    *OPEN.write() = false;
}

/// Opens the questions again (Settings › Study).
pub fn reopen() {
    *OPEN.write() = true;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Exam,
    Days,
    Minutes,
    Studied,
    Reminders,
    Theme,
}

const STEPS: [Step; 6] = [Step::Exam, Step::Days, Step::Minutes, Step::Studied, Step::Reminders, Step::Theme];

/// The first-run sheet; renders nothing once seen.
#[component]
pub fn Onboarding() -> Element {
    if !OPEN() {
        return rsx! {};
    }
    rsx! { Questions {} }
}

#[component]
fn Questions() -> Element {
    let mut ui = use_context::<Ui>();
    let mut step = use_signal(|| 0usize);
    let plan = crate::plan::SETTINGS.peek().clone();
    let answers = Answers::load();
    let today = crate::store::zone().study_day(now_ms());
    let mut exam = use_signal(|| plan.exam.map(Day::iso).unwrap_or_default());
    let mut weekdays = use_signal(|| plan.weekdays);
    let mut minutes = use_signal(|| answers.minutes.unwrap_or(10));
    let mut studied = use_signal(|| answers.studied.clone());
    let mut times = use_signal(|| crate::reminders::PREFS.peek().times.clone());
    let mut error = use_signal(|| None::<&'static str>);

    let current = STEPS[step()];
    let last = step() + 1 == STEPS.len();
    // Saves this step's answer; false keeps him on it with a gentle note.
    let mut commit = move || -> bool {
        match current {
            Step::Exam => {
                let text = exam.read().clone();
                if text.is_empty() {
                    return true;
                }
                let Some(day) = Day::parse(&text).filter(|d| *d > today) else {
                    error.set(Some(tx::PICK_FUTURE));
                    return false;
                };
                let mut s = crate::plan::SETTINGS.write();
                s.exam = Some(day);
                crate::plan::save(&s);
            }
            Step::Days => {
                if !weekdays.read().contains(&true) {
                    error.set(Some(tx::PICK_A_DAY));
                    return false;
                }
                let mut s = crate::plan::SETTINGS.write();
                s.weekdays = *weekdays.read();
                crate::plan::save(&s);
            }
            Step::Minutes | Step::Studied => {
                Answers { minutes: Some(minutes()), studied: studied.read().clone() }.save();
            }
            Step::Reminders => {
                let t = times.read().clone();
                let on = !t.is_empty();
                RemindPrefs { on, times: if on { t } else { RemindPrefs::default().times } }.save();
                if on && crate::reminders::HERE() {
                    crate::platform::reminders::request_permission();
                }
            }
            Step::Theme => {}
        }
        error.set(None);
        true
    };

    let body = match current {
        Step::Exam => rsx! {
            h1 { {tx::EXAM_TITLE} }
            p { class: "muted", {tx::EXAM_BODY} }
            label { class: "field",
                span { {tx::EXAM_DAY} }
                input {
                    r#type: "date",
                    min: "{today.plus(1).iso()}",
                    value: "{exam}",
                    oninput: move |e| {
                        exam.set(e.value());
                        error.set(None);
                    },
                }
            }
        },
        Step::Days => rsx! {
            h1 { {tx::DAYS_TITLE} }
            p { class: "muted", {tx::DAYS_BODY} }
            div { class: "onboard-chips", role: "group", aria_label: tx::DAYS_TITLE,
                for (w , name) in crate::i18n::plan::WEEKDAYS.iter().enumerate() {
                    button {
                        key: "{w}",
                        "aria-pressed": if weekdays.read()[w] { "true" } else { "false" },
                        onclick: move |_| {
                            weekdays.write()[w] ^= true;
                            error.set(None);
                        },
                        {*name}
                    }
                }
            }
        },
        Step::Minutes => rsx! {
            h1 { {tx::MINUTES_TITLE} }
            p { class: "muted", {tx::MINUTES_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::MINUTES_TITLE,
                for m in MINUTE_CHOICES {
                    button {
                        key: "{m}",
                        role: "radio",
                        "aria-checked": minutes() == m,
                        onclick: move |_| minutes.set(m),
                        "{m} {tx::MIN}"
                    }
                }
            }
        },
        Step::Studied => rsx! {
            h1 { {tx::STUDIED_TITLE} }
            p { class: "muted", {tx::STUDIED_BODY} }
            div { class: "onboard-chips", role: "group", aria_label: tx::STUDIED_TITLE,
                for l in Library::get().index.lectures.iter() {
                    button {
                        key: "{l.id}",
                        "aria-pressed": if studied.read().contains(&l.id) { "true" } else { "false" },
                        onclick: {
                            let id = l.id.clone();
                            move |_| {
                                let mut s = studied.write();
                                match s.iter().position(|x| *x == id) {
                                    Some(i) => {
                                        s.remove(i);
                                    }
                                    None => s.push(id.clone()),
                                }
                            }
                        },
                        bdi { {l.title.clone()} }
                    }
                }
            }
        },
        Step::Reminders => rsx! {
            h1 { {tx::REMIND_TITLE} }
            p { class: "muted", {tx::REMIND_BODY} }
            for (i , t) in times.read().iter().copied().enumerate() {
                div { key: "{i}-{t}", class: "row",
                    input {
                        r#type: "time",
                        value: hhmm(t),
                        aria_label: tx::REMIND_TITLE,
                        onchange: move |e: FormEvent| {
                            if let Some(v) = parse_hhmm(&e.value()) {
                                times.write()[i] = v;
                            }
                        },
                    }
                    button { class: "secondary", onclick: move |_| { times.write().remove(i); }, {tx::REMOVE} }
                }
            }
            if times.read().len() < MAX_TIMES {
                button {
                    class: "secondary",
                    onclick: move |_| {
                        let next = times.read().last().map_or(19 * 60, |t| (t + 120).min(23 * 60));
                        times.write().push(next);
                    },
                    {tx::ADD_TIME}
                }
            }
            if times.read().is_empty() {
                p { class: "muted small", {tx::NO_REMINDERS} }
            }
        },
        Step::Theme => rsx! {
            h1 { {tx::THEME_TITLE} }
            p { class: "muted", {tx::THEME_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::THEME_TITLE,
                for t in Theme::ALL {
                    button {
                        key: "{t.as_str()}",
                        role: "radio",
                        "aria-checked": ui.prefs.read().theme == t,
                        onclick: move |_| {
                            ui.prefs.write().theme = t;
                            ui.prefs.peek().save();
                        },
                        {theme_name(t)}
                    }
                }
            }
        },
    };

    rsx! {
        style { {CSS} }
        div { class: "onboard-backdrop",
            div { class: "onboard", role: "dialog", aria_modal: "true", aria_label: tx::TITLE,
                div { class: "onboard-steps", aria_hidden: "true",
                    for i in 0..STEPS.len() {
                        span { key: "{i}", "data-done": if i <= step() { "true" } else { "false" } }
                    }
                }
                p { class: "muted small", "{tx::STEP} {step() + 1} {tx::OF} {STEPS.len()}" }
                section { class: "card", {body} }
                if let Some(msg) = error() {
                    p { class: "error", role: "alert", {msg} }
                }
                div { class: "onboard-actions",
                    button { class: "secondary", onclick: move |_| close(), {tx::SKIP_ALL} }
                    div { class: "row",
                        if step() > 0 {
                            button { class: "secondary", onclick: move |_| { error.set(None); step -= 1; }, {tx::BACK} }
                        }
                        button {
                            class: "primary",
                            onclick: move |_| {
                                if commit() {
                                    if last { close() } else { step += 1 }
                                }
                            },
                            if last { {tx::DONE} } else { {tx::NEXT} }
                        }
                    }
                }
            }
        }
    }
}

fn theme_name(t: Theme) -> &'static str {
    use crate::i18n::settings as s;
    match t {
        Theme::System => s::THEME_SYSTEM,
        Theme::Paper => s::THEME_PAPER,
        Theme::Night => s::THEME_NIGHT,
        Theme::NightShift => s::THEME_SHIFT,
    }
}

/// Settings › Study: go through the first-run questions again.
#[component]
pub fn ReopenCard() -> Element {
    rsx! {
        section { class: "card",
            h2 { {tx::REOPEN_TITLE} }
            p { class: "muted small", {tx::REOPEN_BODY} }
            button { class: "secondary", onclick: move |_| reopen(), {tx::REOPEN} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_tolerate_missing_fields() {
        let a: Answers = serde_json::from_str(r#"{"minutes":15}"#).unwrap();
        assert_eq!(a, Answers { minutes: Some(15), studied: Vec::new() });
    }
}
