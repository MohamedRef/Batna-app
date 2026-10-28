//! Oral mode: the structured oral exam, hands-free. The app reads a question
//! aloud, gives Mohamed time to answer out loud, reads the model answer, and
//! he rates himself Again or Good. Again comes back later in the session.
//! No recording and no speech recognition: nothing leaves the device.

mod session;

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use std::collections::BTreeSet;

use batna_core::review::{Grade, Origin, ReviewEvent, StudyStoreHandle};
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::count::{QUESTION, SECOND, count};
use crate::i18n::oral as tx;
use crate::platform::background::{self, Labels, Remote};
use crate::platform::tts;
use session::{Entry, Pick, Session};

const ORAL_CSS: &str = include_str!("oral.css");

/// Think-timer choices, in seconds (the brief's 10–30 s range).
const THINK_MIN: u32 = 10;
const THINK_MAX: u32 = 30;
const SIZES: [usize; 3] = [10, 20, 30];
/// With auto-continue on, how long the answer waits for a rating.
const AUTO_WAIT: Duration = Duration::from_secs(8);
/// After a rating, "show answer" waits this long, so a double tap or a held
/// key cannot flash the next question's answer.
const SETTLE: Duration = Duration::from_millis(700);

#[derive(Clone, Debug, PartialEq)]
struct Settings {
    lecture: Option<String>,
    size: usize,
    think: u32,
    voice: bool,
    /// Index into [`tts::RATES`].
    rate: usize,
    auto_continue: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { lecture: None, size: 10, think: 15, voice: true, rate: 1, auto_continue: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    /// Reading the question.
    Asking,
    /// The think timer: seconds left.
    Thinking(u32),
    /// "And what else?", then a shorter timer.
    WhatElse(u32),
    /// Reading the answer, then waiting for a rating.
    Answer,
}

// Kept across screen changes, for this run of the app. Ratings go to the
// review store (FSRS), like the study player's.
static SETTINGS: GlobalSignal<Settings> = Signal::global(Settings::default);
static SESSION: GlobalSignal<Option<Session>> = Signal::global(|| None);

#[component]
pub fn Oral() -> Element {
    use_drop(tts::stop);
    let active = SESSION.read().as_ref().map(|s| s.current.is_some() || s.left() > 0);
    rsx! {
        style { {ORAL_CSS} }
        match active {
            None => rsx! { Setup {} },
            Some(true) => rsx! { Runner {} },
            Some(false) => rsx! { Summary {} },
        }
    }
}

/// The review store the app provides (FSRS, Mohamed's ✓/✎/✗).
fn use_store() -> CopyValue<StudyStoreHandle> {
    use_hook(|| CopyValue::new(consume_context::<StudyStoreHandle>()))
}

/// Askable items, minus what Mohamed marked wrong, with his edits applied.
fn pool(store: &StudyStoreHandle, lecture: Option<&str>) -> Vec<session::Prompt> {
    let s = store.0.borrow();
    session::pool_with(Library::get().lectures.values(), lecture, |item| {
        (!s.is_hidden(&item.id)).then(|| s.effective(item))
    })
}

fn start(store: &StudyStoreHandle) {
    let settings = SETTINGS.peek().clone();
    let pool = pool(store, settings.lecture.as_deref());
    let due: BTreeSet<String> = store.0.borrow().due(now_ms()).into_iter().collect();
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
    let mut s = Session::new(pool, &Pick { lecture: settings.lecture, size: settings.size, seed, due });
    s.advance();
    *SESSION.write() = Some(s);
}

#[component]
fn Setup() -> Element {
    let store = use_store();
    let settings = SETTINGS.read().clone();
    let lib = Library::get();
    let all = pool(&store.read(), None);
    let lectures: Vec<(String, String)> = lib
        .index
        .lectures
        .iter()
        .filter(|e| all.iter().any(|p| p.lecture == e.id))
        .map(|e| (e.id.clone(), e.title.clone()))
        .collect();
    let available = all.iter().filter(|p| settings.lecture.as_ref().is_none_or(|l| *l == p.lecture)).count();

    rsx! {
        // Tablet landscape and the Mac: what oral mode is and Start on one
        // side, its settings on the other.
        div { class: "tab-cols oral-cols",
            div { class: "tab-main tab-sticky",
                section { class: "card oral-setup",
                    h1 { {tx::TITLE} }
                    p { class: "muted", {tx::INTRO} }
                    if available > 0 {
                        p { class: "muted small", "{tx::AVAILABLE} " {count(available, &QUESTION)} }
                        button {
                            class: "oral-start",
                            "aria-keyshortcuts": "Enter",
                            onclick: move |_| start(&store.read()),
                            {tx::START}
                        }
                    }
                }
            }
            div { class: "tab-aside",
                if lectures.is_empty() {
                    p { class: "note", {tx::EMPTY} }
                } else {
                    section { class: "card oral-setup",
                        label { class: "oral-field",
                            span { {tx::LECTURE} }
                            select {
                                onchange: move |e| {
                                    let v = e.value();
                                    SETTINGS.write().lecture = (!v.is_empty()).then_some(v);
                                },
                                option { value: "", selected: settings.lecture.is_none(), {tx::ALL_LECTURES} }
                                for (id, title) in lectures {
                                    option {
                                        key: "{id}",
                                        value: "{id}",
                                        selected: settings.lecture.as_deref() == Some(id.as_str()),
                                        "{title}"
                                    }
                                }
                            }
                        }
                        div { class: "oral-field",
                            span { {tx::SIZE} }
                            div { class: "segmented", role: "radiogroup",
                                for size in SIZES {
                                    button {
                                        key: "{size}",
                                        role: "radio",
                                        "aria-checked": "{settings.size == size}",
                                        onclick: move |_| SETTINGS.write().size = size,
                                        "{size}"
                                    }
                                }
                            }
                        }
                        label { class: "oral-field",
                            span { "{tx::THINK}: " {count(settings.think, &SECOND)} }
                            input {
                                r#type: "range",
                                min: "{THINK_MIN}",
                                max: "{THINK_MAX}",
                                step: "5",
                                value: "{settings.think}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        SETTINGS.write().think = v.clamp(THINK_MIN, THINK_MAX);
                                    }
                                },
                            }
                        }
                        label { class: "oral-check",
                            input {
                                r#type: "checkbox",
                                checked: settings.voice,
                                onchange: move |e| SETTINGS.write().voice = e.checked(),
                            }
                            span { {tx::VOICE} }
                        }
                        if settings.voice {
                            div { class: "oral-field",
                                span { {tx::RATE} }
                                div { class: "segmented", role: "radiogroup",
                                    for (i, name) in tx::RATES.iter().enumerate() {
                                        button {
                                            key: "{i}",
                                            role: "radio",
                                            "aria-checked": "{settings.rate == i}",
                                            onclick: move |_| SETTINGS.write().rate = i,
                                            {*name}
                                        }
                                    }
                                }
                            }
                        }
                        label { class: "oral-check",
                            input {
                                r#type: "checkbox",
                                checked: settings.auto_continue,
                                onchange: move |e| SETTINGS.write().auto_continue = e.checked(),
                            }
                            span { {tx::AUTO} }
                        }
                        p { class: "muted small", {tx::AUTO_HINT} }
                    }
                }
            }
        }
    }
}

/// What the driver should do next for the current question.
#[derive(Clone, Copy, PartialEq)]
enum Step {
    Ask,
    Reveal,
}

#[component]
fn Runner() -> Element {
    let store = use_store();
    let mut phase = use_signal(|| Phase::Asking);
    let no_voice = use_signal(|| false);
    // Bumped by every new step, so a stale timer or voice stops on its own.
    let mut run = use_signal(|| 0u64);
    let mut asked_at = use_signal(Instant::now);
    let mut rated_at = use_signal(|| None::<Instant>);
    let settled = move || rated_at.peek().is_none_or(|at| at.elapsed() >= SETTLE);
    // Rates question `turn` once and shows the next one as unasked right away
    // (the driver's own task runs later), so a second tap finds nothing to grade.
    let mut grade_turn = move |turn: u64, grade: Option<Grade>| {
        if rate_current(&store.read(), turn, grade, *asked_at.peek()) {
            phase.set(Phase::Asking);
            rated_at.set(Some(Instant::now()));
            tts::stop();
        }
    };

    let mut drive = move |step: Step| {
        let id = *run.peek() + 1;
        run.set(id);
        tts::stop();
        if step == Step::Reveal && !settled() {
            return;
        }
        let Some((entry, turn)) = SESSION.peek().as_ref().and_then(|s| Some((s.current.clone()?, s.asked)))
        else {
            return;
        };
        let Settings { think, voice, rate, auto_continue, .. } = SETTINGS.peek().clone();
        let rate = tts::RATES[rate.min(tts::RATES.len() - 1)];
        let say = move |text: String| speak(text, voice, rate, no_voice);
        spawn(async move {
            let live = move || *run.peek() == id;
            if step == Step::Ask {
                asked_at.set(Instant::now());
                phase.set(Phase::Asking);
                say(spoken_question(&entry)).await;
                if !live() || !countdown(think, Phase::Thinking, phase, live).await {
                    return;
                }
                if entry.prompt.what_else {
                    let more = (think / 2).max(THINK_MIN);
                    phase.set(Phase::WhatElse(more));
                    say("And what else?".into()).await;
                    if !live() || !countdown(more, Phase::WhatElse, phase, live).await {
                        return;
                    }
                }
            }
            phase.set(Phase::Answer);
            say(entry.prompt.spoken_answer()).await;
            if auto_continue && live() {
                tokio::time::sleep(AUTO_WAIT).await;
                if live() {
                    // Moving on changes the session, and the effect below asks the next one.
                    grade_turn(turn, None);
                }
            }
        });
    };

    // Ask the first question, and the next one each time the session moves on.
    let asked_now = SESSION.read().as_ref().map_or(0, |s| s.asked);
    let mut asked = use_signal(|| 0u64);
    use_effect(use_reactive!(|asked_now| {
        if asked_now != 0 && *asked.peek() != asked_now {
            asked.set(asked_now);
            drive(Step::Ask);
        }
    }));

    // Screen off (Android): a foreground service keeps the session going,
    // shows it on the lock screen and passes on headset and notification buttons.
    use_hook(|| {
        background::start(&Labels { show: tx::SHOW, again: tx::AGAIN, good: tx::GOOD, end: tx::END });
    });
    use_drop(background::stop);
    let revealed_now = use_memo(move || *phase.read() == Phase::Answer);
    use_effect(move || {
        let revealed = revealed_now();
        let session = SESSION.read();
        let Some(entry) = session.as_ref().and_then(|s| s.current.as_ref()) else { return };
        let title = format!("{} · {} {}", tx::TITLE, tx::LEFT, session.as_ref().map_or(0, Session::left));
        let lines = if revealed { &entry.prompt.answer } else { &entry.prompt.question };
        background::update(&title, &lines.join(" · "), revealed);
    });
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let Some(remote) = background::take() else { continue };
            let revealed = revealed_now();
            let turn = SESSION.peek().as_ref().map_or(0, |s| s.asked);
            match remote {
                Remote::End => end_session(),
                Remote::Again if revealed => grade_turn(turn, Some(Grade::Again)),
                Remote::Main | Remote::Good if revealed => grade_turn(turn, Some(Grade::Good)),
                Remote::Main | Remote::Good => drive(Step::Reveal),
                Remote::Again => {}
            }
        }
    });

    let session = SESSION.read();
    let Some(s) = session.as_ref() else { return rsx! {} };
    let Some(entry) = s.current.clone() else { return rsx! {} };
    let left = s.left();
    let turn = s.asked;
    drop(session);
    let phase_now = *phase.read();
    let revealed = phase_now == Phase::Answer;
    let p = &entry.prompt;
    let lecture = p.lecture.rsplit('.').next().unwrap_or(&p.lecture);

    rsx! {
        div {
            class: "oral-run",
            "data-phase": if revealed { "answer" } else { "ask" },
            header { class: "oral-top",
                span { class: "muted small", "{tx::LEFT} " bdi { "{left}" } }
                if entry.follow_up {
                    span { class: "chip accent", {tx::FOLLOW_UP} }
                }
                if entry.again {
                    span { class: "chip", {tx::AGAIN_BADGE} }
                }
                span { class: "muted small", bdi { "{lecture}" } }
                crate::hover::BookPage { page: p.page, quote: p.quote.clone() }
                button {
                    class: "oral-end",
                    "aria-keyshortcuts": "Escape",
                    onclick: move |_| end_session(),
                    {tx::END}
                }
            }
            if *no_voice.read() {
                p { class: "note small", {tx::NO_VOICE} }
            }
            crate::desk::Panes {
                primary: rsx! {
                    section { class: "card oral-question",
                        h2 { class: "muted small", {tx::LISTEN} }
                        div { class: "en", dir: "ltr", lang: "en",
                            for (i, line) in p.question.iter().enumerate() {
                                p { key: "{i}", class: if i + 1 == p.question.len() { "oral-ask" } else { "" }, "{line}" }
                            }
                        }
                        if let Some((pass, of)) = p.pass {
                            span { class: "chip", "{tx::PASS} " bdi { "{pass}" } " {tx::OF} " bdi { "{of}" } }
                        }
                    }
                    match phase_now {
                        Phase::Asking => rsx! {
                            section { class: "oral-timer", "aria-live": "polite",
                                span { class: "oral-status", {tx::LISTEN} }
                            }
                        },
                        Phase::Thinking(n) | Phase::WhatElse(n) => {
                            let what_else = matches!(phase_now, Phase::WhatElse(_));
                            let full = if what_else { (SETTINGS.peek().think / 2).max(THINK_MIN) } else { SETTINGS.peek().think };
                            let pct = 100 * n / full.max(1);
                            rsx! {
                                section { class: "oral-timer", "aria-live": "polite",
                                    div { class: "oral-ring", style: "--p: {pct}", span { "{n}" } }
                                    span { class: "oral-status",
                                        if what_else { {tx::WHAT_ELSE} } else { {tx::THINKING} }
                                    }
                                }
                            }
                        }
                        Phase::Answer => rsx! {},
                    }
                },
                secondary: revealed.then(|| rsx! {
                    section { class: "card oral-answer",
                        h2 { class: "small", {tx::ANSWER} }
                        ul { class: "en", dir: "ltr", lang: "en",
                            for (i, line) in p.answer.iter().enumerate() {
                                li { key: "{i}", "{line}" }
                            }
                        }
                    }
                }),
            }
            if revealed {
                crate::verify::VerifyBar { item_id: p.item_id.clone() }
            }
            div { class: "oral-actions",
                if revealed {
                    button {
                        class: "oral-big again",
                        "aria-keyshortcuts": "1",
                        onclick: move |_| grade_turn(turn, Some(Grade::Again)),
                        span { class: "big-label", {tx::AGAIN} }
                        span { class: "big-hint", {tx::AGAIN_HINT} }
                    }
                    button {
                        class: "oral-big good",
                        "aria-keyshortcuts": "3 2 Space",
                        onclick: move |_| grade_turn(turn, Some(Grade::Good)),
                        span { class: "big-label", {tx::GOOD} }
                        span { class: "big-hint", {tx::GOOD_HINT} }
                    }
                } else {
                    button { class: "oral-small", onclick: move |_| drive(Step::Ask), {tx::REPEAT} }
                    button {
                        class: "oral-big show",
                        "aria-keyshortcuts": "Space Enter",
                        onclick: move |_| drive(Step::Reveal),
                        span { class: "big-label", {tx::SHOW} }
                    }
                }
            }
        }
    }
}

/// Counts down `secs`, showing each second. `false` if the step was replaced.
async fn countdown(
    secs: u32,
    show: fn(u32) -> Phase,
    mut phase: Signal<Phase>,
    live: impl Fn() -> bool,
) -> bool {
    for n in (1..=secs).rev() {
        phase.set(show(n));
        tokio::time::sleep(Duration::from_secs(1)).await;
        if !live() {
            return false;
        }
    }
    true
}

/// Says `text` when the voice is on and working; on a missing voice, turns
/// it off for the session and shows why (the text is on screen anyway).
async fn speak(text: String, voice: bool, rate: f32, mut no_voice: Signal<bool>) {
    if !voice || *no_voice.peek() {
        return;
    }
    if let Err(e) = tts::say(&text, rate).await {
        if !matches!(e, tts::SpeakError::Failed(_)) {
            no_voice.set(true);
        }
        crate::diag::report!("oral: speech failed: {e:?}");
    }
}

/// The question as the examiner says it.
fn spoken_question(entry: &Entry) -> String {
    let lead = if entry.follow_up {
        "Following on from that. "
    } else if entry.again {
        "Once more. "
    } else {
        ""
    };
    format!("{lead}{}", entry.prompt.spoken_question())
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// Stops early; the summary shows what was done.
fn end_session() {
    tts::stop();
    if let Some(s) = SESSION.write().as_mut() {
        s.end();
    }
}

/// Records the rating of question `turn` (none when heard without one) and
/// moves on. `false` when that question is already past (a second tap).
fn rate_current(store: &StudyStoreHandle, turn: u64, grade: Option<Grade>, asked_at: Instant) -> bool {
    let mut guard = SESSION.write();
    let Some(s) = guard.as_mut() else { return false };
    let Some(prompt) = s.rate_turn(turn, grade) else { return false };
    if let Some(grade) = grade {
        store.0.borrow_mut().record(ReviewEvent {
            item_id: prompt.item_id,
            grade,
            origin: Origin::Oral,
            at_ms: now_ms(),
            took_ms: asked_at.elapsed().as_millis().min(u32::MAX as u128) as u32,
        });
    }
    true
}

#[component]
fn Summary() -> Element {
    let store = use_store();
    let session = SESSION.read();
    let Some(s) = session.as_ref() else { return rsx! {} };
    let t = s.tally.clone();
    drop(session);
    rsx! {
        section { class: "card oral-summary",
            h1 { {tx::DONE} }
            dl { class: "oral-tally",
                div { dt { {tx::DONE_GOOD} } dd { "{t.good}" } }
                div { dt { {tx::DONE_AGAIN} } dd { "{t.again}" } }
                if t.heard > 0 {
                    div { dt { {tx::DONE_HEARD} } dd { "{t.heard}" } }
                }
            }
        }
        section { class: "card",
            h2 { {tx::MISSED} }
            if t.missed.is_empty() {
                p { class: "muted", {tx::NONE_MISSED} }
            } else {
                ul { class: "en oral-missed", dir: "ltr", lang: "en",
                    for p in t.missed {
                        li { key: "{p.item_id}",
                            span { "{p.question.last().cloned().unwrap_or_default()}" }
                            span { class: "muted small", " (p. {p.page})" }
                        }
                    }
                }
            }
        }
        div { class: "oral-actions",
            button { class: "oral-small", onclick: move |_| *SESSION.write() = None, {tx::BACK} }
            button {
                class: "oral-big good",
                "aria-keyshortcuts": "Enter",
                onclick: move |_| start(&store.read()),
                span { class: "big-label", {tx::ANOTHER} }
            }
        }
    }
}
