//! First run: a few calm questions that set up the planner, the session
//! length, the reminders and the colours. Every step can be skipped, and
//! Settings › Study opens it again.

use batna_core::plan::{Day, PlanSettings};
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
.onboard { max-inline-size: 36rem; min-block-size: 100%; margin-inline: auto; padding: var(--sp-4); display: flex; flex-direction: column; gap: var(--sp-3); }
.onboard-top { display: flex; align-items: center; justify-content: space-between; gap: var(--sp-2); min-block-size: var(--tap); }
.onboard-body { display: flex; flex-direction: column; gap: var(--sp-3); }
.onboard-body h1 { font-size: 1.55rem; font-weight: 600; line-height: 1.6; margin: 0; }
.onboard-body p { margin: 0; }
/* A quiet picture: the app's pulse in a soft panel with faint rings. */
.onboard-art { position: relative; overflow: hidden; display: grid; place-items: center; block-size: 10rem;
  border-radius: var(--radius-sheet, 22px); background: var(--accent-soft); color: var(--accent-ink); }
.onboard-art::before, .onboard-art::after { content: ""; position: absolute; border-radius: 50%; border: 1px solid var(--line);
  inline-size: 10rem; block-size: 10rem; inset-inline-end: -3rem; inset-block-start: -5.5rem; }
.onboard-art::after { inline-size: 16rem; block-size: 16rem; inset-inline-end: -5rem; inset-block-start: -8.5rem; }
.onboard-art svg { inline-size: 4.5rem; block-size: 4.5rem; stroke-width: 1.2; }
.onboard-art-done { block-size: 6rem; background: var(--good-soft); color: var(--good); }
.onboard-art-done svg { inline-size: 3rem; block-size: 3rem; }
.onboard-body .segmented { align-self: stretch; }
.onboard-body .segmented > button { flex: 1; }
.onboard-chips { display: flex; flex-wrap: wrap; gap: var(--sp-2); }
.onboard-chips button { min-block-size: 44px; padding-inline: var(--sp-3); border-radius: var(--radius-pill); border: 1px solid var(--line); background: var(--surface); }
.onboard-chips button[aria-pressed="true"] { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); font-weight: 700; }
/* The action stays at the foot; the last line of a long step scrolls clear of it. */
.onboard-actions { display: flex; flex-direction: column; gap: var(--sp-1); margin-block-start: auto;
  position: sticky; inset-block-end: 0; padding-block: var(--sp-2) calc(var(--sp-2) + env(safe-area-inset-bottom, 0px)); background: var(--bg); }
.onboard-actions .btn { justify-content: center; gap: var(--sp-2); }
.onboard-lectures { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--sp-1); }
.onboard-lectures li { display: flex; align-items: center; gap: var(--sp-2); }
.onboard-lecture { flex: 1; display: flex; align-items: center; gap: var(--sp-2); min-block-size: 44px; padding-inline: var(--sp-3);
  text-align: start; border-radius: var(--radius); border: 1px solid var(--line); background: var(--surface); }
.onboard-lecture[aria-pressed="true"] { border-color: var(--accent); background: var(--accent-soft); }
.onboard-check { flex: none; inline-size: 1.1rem; block-size: 1.1rem; border-radius: 4px; border: 2px solid var(--line); }
.onboard-lecture[aria-pressed="true"] .onboard-check { background: var(--accent); border-color: var(--accent); }
.onboard-num { flex: none; font-variant-numeric: tabular-nums; color: var(--muted); }
.onboard-lectures li.onboard-course { margin-block-start: var(--sp-2); font-weight: 700; color: var(--accent); }
.onboard-upto { flex: none; min-block-size: 44px; padding-inline: var(--sp-2); border: 0; background: none; color: var(--accent); }
/* Six themes with long names: whole-word rows, two columns once there is room (#232). */
.onboard-themes { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 12rem), 1fr)); gap: var(--sp-2); }
.onboard-theme { display: flex; align-items: center; gap: var(--sp-2); min-block-size: 44px; padding-inline: var(--sp-3);
  text-align: start; border-radius: var(--radius); border: 1px solid var(--line); background: var(--surface); }
.onboard-theme[aria-checked="true"] { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); font-weight: 700; }
.onboard-theme .onboard-swatch { flex: none; margin-inline-end: 0; }
.onboard-swatch { display: inline-block; vertical-align: middle; inline-size: 1.1rem; block-size: 1.1rem; margin-inline-end: var(--sp-1);
  border-radius: 50%; border: 1px solid var(--line); background: linear-gradient(135deg, var(--sw-a) 50%, var(--sw-b) 50%); }
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
        crate::store::save_setting(Self::KEY, self);
    }
}

/// Default study session length: his first-run answer, else 10 minutes.
pub fn session_minutes() -> u32 {
    let _ = MINUTES_REV();
    Answers::load().minutes.filter(|m| MINUTE_CHOICES.contains(m)).unwrap_or(10)
}

/// Bumped when Today's chips change the session length, so readers re-read it.
static MINUTES_REV: GlobalSignal<u64> = Signal::global(|| 0);

/// Today's session-length chips: saves the new default length.
pub fn set_session_minutes(m: u32) {
    let mut a = Answers::load();
    if a.minutes != Some(m) {
        a.minutes = Some(m);
        a.save();
        *MINUTES_REV.write() += 1;
    }
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
        crate::diag::report!("saving first-run flag: {e}");
    }
    *OPEN.write() = false;
}

/// Opens the questions again (Settings › Study).
pub fn reopen() {
    *OPEN.write() = true;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Welcome,
    /// Session length and, if he knows it, the exam day.
    Time,
    /// The end: start, or set any of the optional details.
    Ready,
    Days,
    Studied,
    Reminders,
    Theme,
}

/// The quick path; the rest open from [`Step::Ready`].
const STEPS: [Step; 3] = [Step::Welcome, Step::Time, Step::Ready];

/// The first-run sheet; renders nothing once seen.
#[component]
pub fn Onboarding() -> Element {
    if !OPEN() {
        return rsx! {};
    }
    // BATNA_ONBOARD=3 opens on that screen (previews and screenshots).
    let start = start_step(std::env::var("BATNA_ONBOARD").ok().as_deref());
    rsx! { Questions { start, detail: None } }
}

/// The screen BATNA_ONBOARD names (1 to 3), else the first.
fn start_step(var: Option<&str>) -> usize {
    let n = var.and_then(|v| v.parse::<usize>().ok()).unwrap_or(1);
    n.clamp(1, STEPS.len()) - 1
}

/// `start` is the quick path's screen; `detail` an optional one opened from
/// the last screen.
#[component]
fn Questions(start: usize, detail: Option<Step>) -> Element {
    let mut ui = use_context::<Ui>();
    let settings = use_context::<crate::settings::SettingsNav>();
    let mut step = use_signal(|| start);
    let mut detail = use_signal(|| detail);
    let plan = crate::plan::SETTINGS.peek().clone();
    let answers = Answers::load();
    let today = crate::store::zone().study_day(now_ms());
    let mut exam = use_signal(|| plan.exam.map(Day::iso).unwrap_or_default());
    let mut weekdays = use_signal(|| plan.weekdays);
    let mut minutes = use_signal(|| answers.minutes.unwrap_or(10));
    let mut studied = use_signal(|| answers.studied.clone());
    // Reminder times as they stand: none while reminders are off, so the
    // last screen never shows a time that isn't set.
    let remind_now = || {
        let p = crate::reminders::PREFS.peek();
        if p.on { p.times.clone() } else { Vec::new() }
    };
    let mut times = use_signal(remind_now);
    // What each step showed at first. A step he clicks through unchanged
    // saves nothing, so a new device that hasn't synced yet can't overwrite
    // the real settings with its defaults (#32).
    let initial = use_hook(|| (plan.clone(), answers.clone(), remind_now()));
    let mut error = use_signal(|| None::<&'static str>);

    let current = detail().unwrap_or(STEPS[step()]);
    // Saves one step's answer; false keeps him on it with a gentle note.
    let mut commit = move |s: Step| -> bool {
        let picks = Picks {
            exam: exam.read().clone(),
            weekdays: *weekdays.read(),
            minutes: minutes(),
            studied: studied.read().clone(),
            times: times.read().clone(),
        };
        match changes(s, &picks, &initial, today) {
            Ok(c) => {
                apply(c);
                error.set(None);
                true
            }
            Err(note) => {
                error.set(Some(note));
                false
            }
        }
    };

    let body = match current {
        Step::Welcome => rsx! {
            div { class: "onboard-art", aria_hidden: "true", {crate::icons::pulse()} }
            h1 { {tx::WELCOME_TITLE} }
            p { class: "muted", {tx::WELCOME_BODY} }
            p { class: "note", {tx::WELCOME_NOTE} }
            // A new device of his: sign in and his plan and lessons sync
            // down instead of being asked again (Settings › Sync).
            p {
                button { class: "btn btn-secondary",
                    onclick: move |_| {
                        close();
                        settings.show(Some(crate::settings::Section::Sync));
                    },
                    {tx::HAVE_ACCOUNT}
                }
            }
        },
        Step::Time => rsx! {
            h1 { {tx::TIME_TITLE} }
            p { class: "muted", {tx::TIME_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::MINUTES_TITLE,
                for m in MINUTE_CHOICES {
                    button {
                        key: "{m}",
                        role: "radio",
                        "aria-checked": minutes() == m,
                        onclick: move |_| minutes.set(m),
                        {crate::i18n::count::count(m, &crate::i18n::count::MINUTE)}
                    }
                }
            }
            label { class: "field", "data-invalid": error().is_some(),
                span { {tx::EXAM_OPTIONAL} }
                input {
                    r#type: "date",
                    min: "{today.plus(1).iso()}",
                    max: "{today.plus(batna_core::plan::MAX_DAYS_AHEAD).iso()}",
                    value: "{exam}",
                    oninput: move |e| {
                        exam.set(e.value());
                        error.set(None);
                    },
                }
                span { class: "field-hint", {tx::EXAM_HINT} }
            }
        },
        Step::Ready => {
            let days = weekdays.read().iter().filter(|d| **d).count();
            let theme = theme_name(ui.prefs.read().theme);
            let reminders = if times.read().is_empty() {
                tx::REMIND_OFF.to_string()
            } else {
                times.read().iter().map(|t| hhmm(*t)).collect::<Vec<_>>().join(" · ")
            };
            rsx! {
                div { class: "onboard-art onboard-art-done", aria_hidden: "true", {crate::icons::check()} }
                h1 { {tx::READY_TITLE} }
                p { class: "muted", {tx::READY_BODY} }
                ul { class: "tap-list",
                    for (s , title , sub) in [
                        (Step::Days, tx::DAYS_ROW, tx::days_sub(days)),
                        (Step::Studied, tx::STUDIED_ROW, tx::studied_sub(studied.read().len())),
                        (Step::Reminders, tx::REMIND_ROW, reminders),
                        (Step::Theme, tx::THEME_ROW, theme.to_string()),
                    ]
                    {
                        li { key: "{title}",
                            button { class: "tap tap-row", onclick: move |_| detail.set(Some(s)),
                                span { class: "tap-row-main",
                                    strong { {title} }
                                    span { class: "muted small", bdi { {sub} } }
                                }
                                span { class: "chev", aria_hidden: "true" }
                            }
                        }
                    }
                }
            }
        }
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
        Step::Studied => {
            let ids: Vec<String> = Library::get().index.lectures.iter().map(|l| l.id.clone()).collect();
            let all = ids.clone();
            rsx! {
                h1 { {tx::STUDIED_TITLE} }
                p { class: "muted", {tx::STUDIED_BODY} }
                div { class: "row",
                    button { class: "btn btn-secondary", onclick: move |_| studied.set(all.clone()), {tx::ALL} }
                    button { class: "btn btn-secondary", onclick: move |_| studied.set(Vec::new()), {tx::NONE} }
                }
                ul { class: "onboard-lectures", aria_label: tx::STUDIED_TITLE,
                    for group in Library::get().by_course(Library::get().index.lectures.iter().enumerate().collect(), |(_, l)| l.id.as_str()) {
                        li { key: "{group.id}", class: "onboard-course", "{group.short()}" }
                        for (n , l) in group.rows {
                            li { key: "{l.id}",
                                button {
                                    class: "onboard-lecture",
                                    "aria-pressed": if studied.read().contains(&l.id) { "true" } else { "false" },
                                    onclick: {
                                        let id = l.id.clone();
                                        move |_| toggle(&mut studied.write(), &id)
                                    },
                                    span { class: "onboard-check", aria_hidden: "true" }
                                    bdi { class: "onboard-num", "{crate::content::lecture_no(&l.id)}" }
                                    bdi { {l.title.clone()} }
                                }
                                button {
                                    class: "onboard-upto small",
                                    title: tx::UP_TO_HERE,
                                    onclick: {
                                        let ids = ids.clone();
                                        move |_| up_to_here(&mut studied.write(), &ids, n)
                                    },
                                    {tx::UP_TO_HERE}
                                }
                            }
                        }
                    }
                }
            }
        }
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
                    button { class: "btn btn-secondary", onclick: move |_| { times.write().remove(i); }, {tx::REMOVE} }
                }
            }
            if times.read().len() < MAX_TIMES {
                button {
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        let next = next_time(&times.read());
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
            div { class: "onboard-themes", role: "radiogroup", aria_label: tx::THEME_TITLE,
                for t in Theme::ALL {
                    button {
                        key: "{t.as_str()}",
                        class: "onboard-theme",
                        role: "radio",
                        "aria-checked": ui.prefs.read().theme == t,
                        onclick: move |_| {
                            ui.prefs.write().theme = t;
                            ui.prefs.peek().save();
                        },
                        span { class: "onboard-swatch", aria_hidden: "true", style: swatch_style(t) }
                        {theme_name(t)}
                    }
                }
            }
        },
    };

    let is_detail = detail().is_some();
    let last = step() + 1 == STEPS.len();
    rsx! {
        style { {CSS} }
        div { class: "onboard-backdrop",
            div { class: "onboard", role: "dialog", aria_modal: "true", aria_label: tx::TITLE,
                div { class: "onboard-top",
                    span { class: "chip", {tx::progress(step() + 1, STEPS.len())} }
                    if !last {
                        button { class: "btn btn-ghost", "data-back": if step() == 0 && detail().is_none() { "true" } else { "false" }, onclick: move |_| close(), {tx::LATER} }
                    }
                }
                section { class: "onboard-body", {body} }
                if let Some(msg) = error() {
                    p { class: "field-error", role: "alert", {msg} }
                }
                div { class: "onboard-actions",
                    if is_detail {
                        // A detail returns to the last screen once saved.
                        button {
                            class: "btn btn-primary btn-lg btn-block",
                            onclick: move |_| {
                                if commit(current) {
                                    detail.set(None);
                                }
                            },
                            {tx::DETAIL_DONE}
                        }
                    } else if last {
                        button {
                            class: "btn btn-primary btn-lg btn-block",
                            onclick: move |_| {
                                close();
                                match crate::learn::next_lesson(&[]) {
                                    Some(l) => crate::learn::open(ui, &l.id),
                                    None => ui.screen.set(crate::Screen::Learn),
                                }
                            },
                            {tx::START_LESSON}
                            {crate::icons::arrow_go()}
                        }
                        button {
                            class: "btn btn-ghost btn-block",
                            onclick: move |_| {
                                close();
                                ui.screen.set(crate::Screen::Today);
                            },
                            {tx::START_TODAY}
                        }
                    } else {
                        button {
                            class: "btn btn-primary btn-lg btn-block",
                            onclick: move |_| {
                                if commit(current) {
                                    step += 1;
                                }
                            },
                            {tx::CONTINUE}
                            {crate::icons::arrow_go()}
                        }
                    }
                    if step() > 0 && !is_detail {
                        button { class: "btn btn-ghost btn-block", "data-back": "true", onclick: move |_| { error.set(None); step -= 1; }, {tx::BACK} }
                    } else if is_detail {
                        button { class: "btn btn-ghost btn-block", "data-back": "true", onclick: move |_| { error.set(None); detail.set(None); }, {tx::BACK} }
                    }
                }
            }
        }
    }
}

/// Ticks a lecture he studied, or unticks it.
fn toggle(studied: &mut Vec<String>, id: &str) {
    match studied.iter().position(|x| x == id) {
        Some(i) => {
            studied.remove(i);
        }
        None => studied.push(id.to_string()),
    }
}

/// "Up to here": every lecture of that course up to `ids[n]`, in order;
/// other courses keep their picks.
fn up_to_here(studied: &mut Vec<String>, ids: &[String], n: usize) {
    let course = batna_core::program::course_of(&ids[n]);
    studied.retain(|x| batna_core::program::course_of(x) != course);
    studied.extend(ids[..=n].iter().filter(|x| batna_core::program::course_of(x) == course).cloned());
}

/// A new reminder: 7 pm, or two hours after the last one, never after 11 pm.
fn next_time(times: &[u16]) -> u16 {
    times.last().map_or(19 * 60, |t| (t + 120).min(23 * 60))
}

/// What he picked on the questions, as they stand.
struct Picks {
    exam: String,
    weekdays: [bool; 7],
    minutes: u32,
    studied: Vec<String>,
    times: Vec<u16>,
}

/// What the questions showed at first: the plan, his answers, the reminder times.
type Initial = (PlanSettings, Answers, Vec<u16>);

/// What one step's answer changes. Nothing changes for a step he clicked
/// through as it was, so a new device that hasn't synced yet can't
/// overwrite the real settings with its defaults (#32).
#[derive(Debug, Default, PartialEq)]
struct Changes {
    exam: Option<Day>,
    minutes: Option<u32>,
    weekdays: Option<[bool; 7]>,
    studied: Option<Vec<String>>,
    remind: Option<RemindPrefs>,
}

/// Checks one step's answer: `Err` is the gentle note that keeps him on it.
fn changes(step: Step, picks: &Picks, initial: &Initial, today: Day) -> Result<Changes, &'static str> {
    let mut c = Changes::default();
    match step {
        Step::Time => {
            // The exam day first: a wrong date keeps him here.
            let text = picks.exam.as_str();
            if !text.is_empty() && Some(text) != initial.0.exam.map(Day::iso).as_deref() {
                let day = Day::parse(text).filter(|d| *d > today).ok_or(tx::PICK_FUTURE)?;
                if !batna_core::plan::exam_day_ok(today, day) {
                    return Err(tx::PICK_FAR);
                }
                c.exam = Some(day);
            }
            let m = picks.minutes;
            if Some(m) != initial.1.minutes && (initial.1.minutes.is_some() || m != 10) {
                c.minutes = Some(m);
            }
        }
        Step::Days => {
            if !picks.weekdays.contains(&true) {
                return Err(tx::PICK_A_DAY);
            }
            if picks.weekdays != initial.0.weekdays {
                c.weekdays = Some(picks.weekdays);
            }
        }
        Step::Studied => {
            if picks.studied != initial.1.studied {
                c.studied = Some(picks.studied.clone());
            }
        }
        Step::Reminders => {
            if picks.times != initial.2 {
                let on = !picks.times.is_empty();
                let times = if on { picks.times.clone() } else { RemindPrefs::default().times };
                c.remind = Some(RemindPrefs { on, times });
            }
        }
        Step::Welcome | Step::Ready | Step::Theme => {}
    }
    Ok(c)
}

/// Saves what [`changes`] decided.
fn apply(c: Changes) {
    if c.exam.is_some() || c.weekdays.is_some() {
        let mut s = crate::plan::SETTINGS.write();
        if let Some(day) = c.exam {
            s.exam = Some(day);
        }
        if let Some(w) = c.weekdays {
            s.weekdays = w;
        }
        crate::plan::save(&s);
    }
    if let Some(m) = c.minutes {
        set_session_minutes(m);
    }
    if let Some(studied) = c.studied {
        let mut a = Answers::load();
        a.studied = studied;
        a.save();
    }
    if let Some(r) = c.remind {
        let on = r.on;
        r.save();
        if on && crate::reminders::HERE() {
            crate::platform::reminders::request_permission();
        }
    }
}

/// The page and accent colours (System themes: the day and night page), as
/// in Settings › Look.
fn swatch_style(t: Theme) -> String {
    let (a, b) = match t.palette() {
        Some(p) => (p.bg, p.accent),
        None => (t.day_and_night().0.bg, t.day_and_night().1.bg),
    };
    format!("--sw-a:{a};--sw-b:{b};")
}

fn theme_name(t: Theme) -> &'static str {
    use crate::i18n::settings as s;
    match t {
        Theme::System => s::THEME_SYSTEM,
        Theme::SystemBlack => s::THEME_SYSTEM_BLACK,
        Theme::Paper => s::THEME_PAPER,
        Theme::Night => s::THEME_NIGHT,
        Theme::Black => s::THEME_BLACK,
        Theme::NightShift => s::THEME_SHIFT,
    }
}

/// Settings › Study: go through the first-run questions again.
#[component]
pub fn ReopenCard() -> Element {
    rsx! {
        crate::ui::Card { class: "onboard-reopen",
            h2 { {tx::REOPEN_TITLE} }
            button { class: "btn btn-secondary", onclick: move |_| reopen(), {tx::REOPEN} }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::prefs::DevicePrefs;

    thread_local! {
        static AT: Cell<(usize, Option<Step>)> = const { Cell::new((0, None)) };
        static UI: Cell<Option<Ui>> = const { Cell::new(None) };
        static NAV: Cell<Option<crate::settings::SettingsNav>> = const { Cell::new(None) };
    }

    fn fixture() -> Element {
        let (start, detail) = AT.with(Cell::get);
        let ui = use_context_provider(|| Ui {
            screen: Signal::new(crate::Screen::Today),
            prefs: Signal::new(DevicePrefs { theme: Theme::Night, ..DevicePrefs::default() }),
            focus: Signal::new(false),
        });
        UI.with(|u| u.set(Some(ui)));
        let nav = use_context_provider(crate::settings::SettingsNav::new);
        NAV.with(|n| n.set(Some(nav)));
        rsx! { Questions { start, detail } }
    }

    fn dom(start: usize, detail: Option<Step>) -> VirtualDom {
        crate::store::use_memory_store();
        AT.with(|a| a.set((start, detail)));
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        dom
    }

    fn render(start: usize, detail: Option<Step>) -> String {
        AT.with(|a| a.set((start, detail)));
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    fn today() -> Day {
        Day::parse("2026-11-04").unwrap()
    }

    fn picks() -> Picks {
        Picks {
            exam: String::new(),
            weekdays: [true; 7],
            minutes: 10,
            studied: Vec::new(),
            times: Vec::new(),
        }
    }

    fn initial() -> Initial {
        (PlanSettings { weekdays: [true; 7], ..PlanSettings::default() }, Answers::default(), Vec::new())
    }

    #[test]
    fn clicking_through_unchanged_saves_nothing() {
        for step in
            [Step::Welcome, Step::Time, Step::Ready, Step::Days, Step::Studied, Step::Reminders, Step::Theme]
        {
            assert_eq!(changes(step, &picks(), &initial(), today()), Ok(Changes::default()), "{step:?}");
        }
        // The exam day he already had, shown again, is not a change either.
        let mut init = initial();
        init.0.exam = Day::parse("2027-01-10");
        let p = Picks { exam: "2027-01-10".into(), ..picks() };
        assert_eq!(changes(Step::Time, &p, &init, today()), Ok(Changes::default()));
    }

    #[test]
    fn the_exam_day_must_be_ahead_and_within_three_years() {
        let at =
            |exam: &str| changes(Step::Time, &Picks { exam: exam.into(), ..picks() }, &initial(), today());
        assert_eq!(at("2026-11-04"), Err(tx::PICK_FUTURE), "today is too late");
        assert_eq!(at("2025-01-01"), Err(tx::PICK_FUTURE));
        assert_eq!(at("not a date"), Err(tx::PICK_FUTURE));
        assert_eq!(at("2031-01-01"), Err(tx::PICK_FAR));
        let ok = at("2027-01-10").unwrap();
        assert_eq!(ok.exam, Day::parse("2027-01-10"));
        assert_eq!(ok.minutes, None);
    }

    #[test]
    fn session_length_is_saved_only_when_he_changes_it() {
        let time = |minutes, before| {
            let mut init = initial();
            init.1.minutes = before;
            changes(Step::Time, &Picks { minutes, ..picks() }, &init, today()).unwrap().minutes
        };
        assert_eq!(time(10, None), None, "the default, untouched");
        assert_eq!(time(15, None), Some(15));
        assert_eq!(time(10, Some(20)), Some(10), "back to ten from an earlier choice");
        assert_eq!(time(20, Some(20)), None);
    }

    #[test]
    fn days_need_at_least_one_and_studied_lectures_are_kept() {
        let mut p = picks();
        p.weekdays = [false; 7];
        assert_eq!(changes(Step::Days, &p, &initial(), today()), Err(tx::PICK_A_DAY));
        p.weekdays[6] = true;
        assert_eq!(changes(Step::Days, &p, &initial(), today()).unwrap().weekdays, Some(p.weekdays));
        let p = Picks { studied: vec!["hb.L01".into(), "hb.L02".into()], ..picks() };
        let c = changes(Step::Studied, &p, &initial(), today()).unwrap();
        assert_eq!(c.studied.as_deref(), Some(&["hb.L01".to_string(), "hb.L02".to_string()][..]));
    }

    #[test]
    fn reminder_times_turn_reminders_on_and_none_turns_them_off() {
        let p = Picks { times: vec![19 * 60, 21 * 60], ..picks() };
        let r = changes(Step::Reminders, &p, &initial(), today()).unwrap().remind.unwrap();
        assert!(r.on);
        assert_eq!(r.times, [19 * 60, 21 * 60]);
        // He removed the time he had: off, keeping the usual times for later.
        let mut init = initial();
        init.2 = vec![20 * 60];
        let r = changes(Step::Reminders, &picks(), &init, today()).unwrap().remind.unwrap();
        assert!(!r.on);
        assert_eq!(r.times, RemindPrefs::default().times);
    }

    #[test]
    fn lectures_tick_one_by_one_or_up_to_one_in_its_course() {
        let mut s = vec!["hb.L03".to_string()];
        toggle(&mut s, "hb.L05");
        assert_eq!(s, ["hb.L03", "hb.L05"]);
        toggle(&mut s, "hb.L03");
        assert_eq!(s, ["hb.L05"]);
        let ids: Vec<String> =
            ["hb.L01", "hb.L02", "hb.L03", "geri.L01", "geri.L02"].map(String::from).to_vec();
        let mut s = vec!["geri.L02".to_string(), "hb.L03".to_string()];
        up_to_here(&mut s, &ids, 1);
        assert_eq!(s, ["geri.L02", "hb.L01", "hb.L02"], "hb is replaced, geri kept");
        up_to_here(&mut s, &ids, 3);
        assert_eq!(s, ["hb.L01", "hb.L02", "geri.L01"]);
    }

    #[test]
    fn a_new_reminder_comes_two_hours_after_the_last_but_not_past_eleven() {
        assert_eq!(next_time(&[]), 19 * 60);
        assert_eq!(next_time(&[19 * 60]), 21 * 60);
        assert_eq!(next_time(&[19 * 60, 22 * 60]), 23 * 60);
        assert_eq!(next_time(&[23 * 60]), 23 * 60);
    }

    #[test]
    fn a_preview_can_open_any_quick_screen() {
        assert_eq!(start_step(None), 0);
        assert_eq!(start_step(Some("2")), 1);
        assert_eq!(start_step(Some("9")), 2);
        assert_eq!(start_step(Some("0")), 0);
        assert_eq!(start_step(Some("x")), 0);
    }

    #[test]
    fn the_quick_path_reads_well_on_every_screen() {
        let html = render(0, None);
        assert!(html.contains(tx::WELCOME_TITLE) && html.contains(tx::LATER), "{html}");
        assert!(html.contains(&tx::progress(1, 3)) && html.contains(tx::CONTINUE));
        assert!(!html.contains(&format!(">{}<", tx::BACK)), "no back on the first screen");

        let html = render(1, None);
        assert!(html.contains(tx::TIME_TITLE) && html.contains(tx::EXAM_OPTIONAL), "{html}");
        // Four session lengths, the ten-minute default picked.
        assert_eq!(html.matches("role=\"radio\"").count(), 4);
        assert!(html.contains("aria-checked=true") || html.contains("aria-checked=\"true\""), "{html}");
        assert!(html.contains("type=\"date\"") && html.contains("min=\"") && html.contains(tx::BACK));

        let html = render(2, None);
        assert!(
            html.contains(tx::READY_TITLE)
                && html.contains(tx::START_LESSON)
                && html.contains(tx::START_TODAY)
        );
        for row in [tx::DAYS_ROW, tx::STUDIED_ROW, tx::REMIND_ROW, tx::THEME_ROW] {
            assert!(html.contains(row), "missing {row}");
        }
        // His theme, by name; no "later" on the last screen.
        assert!(html.contains(crate::i18n::settings::THEME_NIGHT));
        assert!(!html.contains(tx::LATER));
    }

    #[test]
    fn each_optional_detail_opens_from_the_last_screen() {
        let html = render(2, Some(Step::Days));
        assert!(html.contains(tx::DAYS_TITLE) && html.contains(tx::DETAIL_DONE), "{html}");
        for day in crate::i18n::plan::WEEKDAYS {
            assert!(html.contains(day), "missing {day}");
        }

        let html = render(2, Some(Step::Studied));
        assert!(html.contains(tx::STUDIED_TITLE) && html.contains(tx::ALL) && html.contains(tx::NONE));
        let lectures = Library::get().index.lectures.len();
        assert_eq!(html.matches("class=\"onboard-lecture\"").count(), lectures);
        assert_eq!(html.matches(tx::UP_TO_HERE).count(), lectures * 2, "a label and a title each");
        assert!(html.contains("onboard-course"));

        let html = render(2, Some(Step::Reminders));
        assert!(html.contains(tx::REMIND_TITLE), "{html}");
        let times = html.matches("type=\"time\"").count();
        assert_eq!(html.contains(tx::NO_REMINDERS), times == 0);
        assert_eq!(html.contains(tx::ADD_TIME), times < MAX_TIMES);

        let html = render(2, Some(Step::Theme));
        assert_eq!(html.matches("class=\"onboard-theme\"").count(), Theme::ALL.len());
        assert!(html.contains("--sw-a:") && html.contains(crate::i18n::settings::THEME_SHIFT));
    }

    #[test]
    fn continue_and_back_walk_the_quick_path() {
        use crate::testing::{click, html};
        let mut d = dom(0, None);
        click(&mut d, tx::CONTINUE);
        assert!(html(&d).contains(tx::TIME_TITLE));
        click(&mut d, tx::BACK);
        assert!(html(&d).contains(tx::WELCOME_TITLE));
        click(&mut d, tx::CONTINUE);
        // Twenty minutes, then on: the last screen.
        click(&mut d, "20");
        assert!(html(&d).contains("aria-checked=true"), "{}", html(&d));
        click(&mut d, tx::CONTINUE);
        let h = html(&d);
        assert!(h.contains(tx::READY_TITLE) && h.contains(&tx::progress(3, 3)), "{h}");
        let saved = d.in_runtime(session_minutes);
        assert_eq!(saved, 20);
    }

    #[test]
    fn later_and_both_starts_close_the_questions() {
        use crate::testing::click;
        let mut d = dom(0, None);
        d.in_runtime(reopen);
        click(&mut d, tx::LATER);
        assert!(!d.in_runtime(|| OPEN.cloned()));

        let mut d = dom(2, None);
        let ui = UI.with(Cell::get).unwrap();
        d.in_runtime(|| ui.screen.clone().set(crate::Screen::Stats));
        click(&mut d, tx::START_TODAY);
        assert_eq!(d.in_runtime(|| *ui.screen.peek()), crate::Screen::Today);

        let mut d = dom(2, None);
        let ui = UI.with(Cell::get).unwrap();
        d.in_runtime(reopen);
        click(&mut d, tx::START_LESSON);
        assert!(!d.in_runtime(|| OPEN.cloned()));
        assert_ne!(d.in_runtime(|| *ui.screen.peek()), crate::Screen::Today, "a lesson or the lessons list");
    }

    #[test]
    fn a_new_device_of_his_goes_to_sync_instead_of_the_questions() {
        let mut d = dom(0, None);
        d.in_runtime(reopen);
        crate::testing::click(&mut d, tx::HAVE_ACCOUNT);
        assert!(!d.in_runtime(|| OPEN.cloned()));
        let nav = NAV.with(Cell::get).unwrap();
        assert!(d.in_runtime(|| *nav.open.peek()));
        assert_eq!(d.in_runtime(|| *nav.section.peek()), Some(crate::settings::Section::Sync));
    }

    #[test]
    fn a_past_exam_day_keeps_him_on_the_step_with_a_note() {
        use crate::testing::{click, html, type_into};
        let mut d = dom(1, None);
        type_into(&mut d, "date", "2020-02-02");
        click(&mut d, tx::CONTINUE);
        let h = html(&d);
        assert!(
            h.contains(tx::TIME_TITLE) && h.contains(tx::PICK_FUTURE) && h.contains("role=\"alert\""),
            "{h}"
        );
        // Typing again clears the note.
        type_into(&mut d, "date", "");
        assert!(!html(&d).contains(tx::PICK_FUTURE));
    }

    #[test]
    fn details_open_from_the_last_screen_and_come_back() {
        use crate::testing::{click, click_nth, html};
        let mut d = dom(2, None);
        click(&mut d, tx::DAYS_ROW);
        assert!(html(&d).contains(tx::DAYS_TITLE));
        // Untick every day: "done" keeps him here and says why.
        for _ in 0..7 {
            let h = html(&d);
            let Some(day) = crate::i18n::plan::WEEKDAYS
                .iter()
                .find(|day| h.contains(&format!("aria-pressed=\"true\">{day}<")))
            else {
                break;
            };
            click(&mut d, day);
        }
        click(&mut d, tx::DETAIL_DONE);
        assert!(html(&d).contains(tx::PICK_A_DAY), "{}", html(&d));
        click_nth(&mut d, tx::BACK, 0);
        assert!(html(&d).contains(tx::READY_TITLE));

        click(&mut d, tx::STUDIED_ROW);
        click(&mut d, tx::ALL);
        let lectures = Library::get().index.lectures.len();
        let ticked =
            |d: &VirtualDom| html(d).matches("class=\"onboard-lecture\" aria-pressed=\"true\"").count();
        assert_eq!(ticked(&d), lectures);
        click(&mut d, tx::NONE);
        assert_eq!(ticked(&d), 0);
        // "Up to here" on the third lecture ticks the course's first three.
        click_nth(&mut d, tx::UP_TO_HERE, 2);
        assert_eq!(ticked(&d), 3);
        // Tapping one of them unticks it (a title no other title contains).
        let lib = Library::get();
        let shown = lib.by_course(lib.index.lectures.iter().enumerate().collect(), |(_, l)| l.id.as_str());
        let titles: Vec<&str> = lib.index.lectures.iter().map(|l| l.title.as_str()).collect();
        let one = shown[0].rows[..3]
            .iter()
            .map(|(_, l)| l.title.as_str())
            .find(|t| titles.iter().filter(|o| o.contains(*t)).count() == 1)
            .unwrap();
        click(&mut d, one);
        assert_eq!(ticked(&d), 2);
        click(&mut d, tx::BACK);

        click(&mut d, tx::REMIND_ROW);
        let before = html(&d).matches("type=\"time\"").count();
        if before < MAX_TIMES {
            click(&mut d, tx::ADD_TIME);
            assert_eq!(html(&d).matches("type=\"time\"").count(), before + 1);
        }
        click(&mut d, tx::REMOVE);
        assert_eq!(
            html(&d).matches("type=\"time\"").count(),
            before.max(1) - 1 + usize::from(before < MAX_TIMES)
        );
        click(&mut d, tx::BACK);
        assert!(html(&d).contains(tx::READY_TITLE));
    }

    #[test]
    fn every_theme_has_a_name_and_two_swatch_colours() {
        let names: std::collections::BTreeSet<_> = Theme::ALL.iter().map(|t| theme_name(*t)).collect();
        assert_eq!(names.len(), Theme::ALL.len());
        for t in Theme::ALL {
            let style = swatch_style(t);
            assert!(style.starts_with("--sw-a:#") && style.contains(";--sw-b:#"), "{style}");
        }
    }

    #[test]
    fn settings_can_reopen_the_questions_and_closing_remembers_it() {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_scope(ScopeId::ROOT, || {
            reopen();
            assert!(OPEN());
            let html = dioxus::ssr::render_element(rsx! { ReopenCard {} });
            assert!(html.contains(tx::REOPEN_TITLE) && html.contains(tx::REOPEN));
            close();
            assert!(!OPEN());
            assert_eq!(shared().local_setting::<bool>(SEEN), Some(true));
        });
    }

    #[test]
    fn answers_tolerate_missing_fields() {
        let a: Answers = serde_json::from_str(r#"{"minutes":15}"#).unwrap();
        assert_eq!(a, Answers { minutes: Some(15), studied: Vec::new() });
    }
}
