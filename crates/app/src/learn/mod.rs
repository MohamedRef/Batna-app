//! Learn (الدروس): short interactive lessons that teach each lecture before
//! its cards are reviewed. Explanations in Egyptian Arabic, drawn figures,
//! mnemonics and small checks between them (`batna_core::lesson`).
//!
//! The list shows every lecture's lessons; a lesson opens in the player
//! (`player.rs`). Finishing one is saved as a synced setting
//! `lesson.<id>` ([`Mark`]), and its practice items start a study session.

mod player;
mod practice;
mod steps;
mod text;

use batna_core::lesson::Lesson;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::Library;
use crate::i18n::count::{MINUTE, QUESTION, count};
use crate::i18n::learn as tx;
use crate::look::{Css, css};
use crate::parts::{Collection, Layout, Sort, ViewBar, sort_by};
use crate::{Screen, Ui};

pub(crate) const CSS: Css = css!("src/learn/learn.css");

/// The lesson open in the player; set it, then switch to [`Screen::Learn`].
/// `BATNA_OPEN_LESSON=hb.L01.lesson1` opens one at launch (previews, screenshots).
pub static OPEN: GlobalSignal<Option<String>> = Signal::global(|| std::env::var("BATNA_OPEN_LESSON").ok());

/// Where the open lesson was opened from: closing it goes back there.
static FROM: GlobalSignal<Screen> = Signal::global(|| Screen::Learn);

/// Opens a lesson from anywhere (Today, a study item, the planner).
pub fn open(mut ui: Ui, id: &str) {
    *FROM.write() = *ui.screen.peek();
    *OPEN.write() = Some(id.to_string());
    ui.screen.set(Screen::Learn);
}

/// Closes the player, back to where the lesson was opened from.
pub fn close(mut ui: Ui) {
    *OPEN.write() = None;
    let from = std::mem::replace(&mut *FROM.write(), Screen::Learn);
    if from != Screen::Learn {
        ui.screen.set(from);
    }
}

/// Lets go of the open lesson without switching screen, for a jump
/// elsewhere (a book page's "study this page"): the Learn tab then shows the
/// list, not a half-finished lesson.
pub fn forget() {
    *OPEN.write() = None;
    *FROM.write() = Screen::Learn;
}

/// A finished lesson, saved as the synced setting `lesson.<id>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    pub done_ms: i64,
    /// Questions right on the first try, out of `asked`.
    pub right: u32,
    pub asked: u32,
}

const PREFIX: &str = "lesson.";

/// Bumped when a lesson is finished or a sync lands, so lists re-read the marks.
static MARKS_REV: GlobalSignal<u64> = Signal::global(|| 0);

pub fn mark(id: &str) -> Option<Mark> {
    let _ = MARKS_REV();
    crate::store::shared().setting(&format!("{PREFIX}{id}"))
}

fn save_mark(id: &str, m: Mark) {
    if let Err(e) = crate::store::shared().put_setting(&format!("{PREFIX}{id}"), &m) {
        crate::diag::report!("saving lesson mark {id} failed: {e}");
    }
    reload();
}

/// Re-reads the marks and the lesson view (after a sync or a restore).
pub fn reload() {
    *MARKS_REV.write() += 1;
    let fresh = load_view();
    if *VIEW.peek() != fresh {
        *VIEW.write() = fresh;
    }
}

/// How a lesson is shown: one step at a time with Back and Continue
/// (`Steps`), or as book pages turned by swiping, with small arrows
/// (`Pages`). Swiping works in both. A synced setting, `lesson_view`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Steps,
    Pages,
}

const VIEW_KEY: &str = "lesson_view";

fn load_view() -> View {
    // BATNA_LESSON_VIEW=pages, for previews and screenshots.
    match std::env::var("BATNA_LESSON_VIEW").as_deref() {
        Ok("pages") => View::Pages,
        Ok("steps") => View::Steps,
        _ => crate::store::shared().setting(VIEW_KEY).unwrap_or_default(),
    }
}

pub static VIEW: GlobalSignal<View> = Signal::global(load_view);

pub fn set_view(v: View) {
    crate::store::save_setting(VIEW_KEY, &v);
    *VIEW.write() = v;
}

/// Settings › Review: Steps or Pages.
#[component]
pub fn ViewSection() -> Element {
    let now = VIEW();
    rsx! {
        section { class: "card",
            h2 { {tx::VIEW_TITLE} }
            p { class: "muted small", {tx::VIEW_BODY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::VIEW_TITLE,
                for (v , label , hint) in [(View::Steps, tx::VIEW_STEPS, tx::VIEW_STEPS_HINT), (View::Pages, tx::VIEW_PAGES, tx::VIEW_PAGES_HINT)] {
                    button {
                        key: "{label}",
                        role: "radio",
                        "aria-checked": now == v,
                        onclick: move |_| set_view(v),
                        {label}
                        span { class: "seg-hint", {hint} }
                    }
                }
            }
        }
    }
}

/// The lesson to do next: the first unfinished one of the earliest lecture
/// in `prefer` that has one left, else the first unfinished in book order.
pub fn next_lesson(prefer: &[String]) -> Option<&'static Lesson> {
    let lib = Library::get();
    let left = |l: &&Lesson| mark(&l.id).is_none();
    prefer
        .iter()
        .filter_map(|lec| lib.lessons.get(lec))
        .find_map(|f| f.lessons.iter().find(left))
        .or_else(|| lib.all_lessons().map(|(_, l)| l).find(left))
}

/// The lesson that teaches an item: one that practises it, else the first
/// lesson of its lecture that covers its topic.
pub fn lesson_for_item(item_id: &str) -> Option<&'static Lesson> {
    let lib = Library::get();
    let lecture = item_id.rsplit_once('.').map(|(l, _)| l)?;
    let file = lib.lessons.get(lecture)?;
    let listed = file.lessons.iter().find(|l| {
        l.practice.iter().any(|p| p == item_id)
            || l.steps.iter().any(|s| matches!(&s.body, batna_core::lesson::StepBody::Try { items } if items.iter().any(|i| i == item_id)))
    });
    listed.or_else(|| {
        let topic = practice::find(item_id)?.1.topic.as_deref()?;
        file.lessons.iter().find(|l| l.topics.iter().any(|t| t == topic))
    })
}

#[component]
pub fn Learn() -> Element {
    let open = OPEN();
    rsx! {
        style { {crate::study::STUDY_CSS.text()} }
        style { {CSS.text()} }
        match open.and_then(|id| Library::get().lesson(&id).map(|(lec, l)| (lec, l.clone()))) {
            Some((lecture, lesson)) => rsx! {
                player::Player { key: "{lesson.id}", lesson, lecture_title: Library::get().title(&lecture.lecture).to_string() }
            },
            None => rsx! { LessonList {} },
        }
    }
}

/// The lecture the list shows: the one picked under "other lectures", else
/// the one with the next lesson. Kept while the app runs.
static SHOWN: GlobalSignal<Option<String>> = Signal::global(|| None);
/// What the search box holds, kept when a lesson opens and closes.
static QUERY: GlobalSignal<String> = Signal::global(String::new);

/// Where an unfinished lesson stands on this device: the step it reopens at
/// (0-based), read from the player's saved position (`player.rs`).
pub fn resume_step(id: &str) -> Option<usize> {
    #[derive(Deserialize)]
    struct At {
        at: usize,
    }
    if mark(id).is_some() {
        return None;
    }
    crate::store::shared().local_setting::<At>(&format!("lessonpos.{id}")).map(|p| p.at).filter(|at| *at > 0)
}

/// A lesson's state in words: finished, part-way, the next one, or new.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Done(Mark),
    Current { at: usize },
    Next,
    New,
}

pub fn state(lesson: &Lesson, next: Option<&str>) -> State {
    if let Some(m) = mark(&lesson.id) {
        State::Done(m)
    } else if let Some(at) = resume_step(&lesson.id) {
        State::Current { at }
    } else if next == Some(lesson.id.as_str()) {
        State::Next
    } else {
        State::New
    }
}

/// Does a lesson match the search? Its title, hook, topics and lecture.
fn matches(q: &str, lecture: &str, lecture_title: &str, l: &Lesson) -> bool {
    let code = crate::content::lecture_code(lecture).to_lowercase();
    [l.title.as_str(), l.hook.as_str(), lecture_title, code.as_str()]
        .into_iter()
        .chain(l.topics.iter().map(String::as_str))
        .any(|t| t.to_lowercase().contains(q))
}

#[component]
fn LessonList() -> Element {
    let lib = Library::get();
    let total: usize = lib.lessons.values().map(|f| f.lessons.len()).sum();
    let done = lib.all_lessons().filter(|(_, l)| mark(&l.id).is_some()).count();
    let next = next_lesson(&[]);
    let next_id = next.map(|l| l.id.clone());
    let next_lecture = next.and_then(|l| l.id.rsplit_once('.')).map(|(lec, _)| lec.to_string());
    let shown = SHOWN().filter(|id| lib.lessons.contains_key(id)).or(next_lecture).or_else(|| {
        lib.index.lectures.iter().find(|e| lib.lessons.contains_key(&e.id)).map(|e| e.id.clone())
    });
    let query = QUERY();
    let q = query.trim().to_lowercase();
    rsx! {
        header { class: "page-head",
            h1 { {tx::HEAD} }
            p { class: "page-sub",
                {tx::HEAD_SUB}
                if total > 0 { " · " span { class: "ls-nowrap", {tx::done_of(done, total)} } }
            }
        }
        if total == 0 {
            p { class: "note", {tx::NONE_YET} }
        } else {
            label { class: "search ls-search",
                {crate::icons::search()}
                input {
                    r#type: "search",
                    placeholder: tx::SEARCH,
                    aria_label: tx::SEARCH_LABEL,
                    value: "{query}",
                    oninput: move |e| *QUERY.write() = e.value(),
                }
                if !query.is_empty() {
                    button {
                        class: "ls-search-clear",
                        aria_label: tx::CLEAR_SEARCH,
                        title: tx::CLEAR_SEARCH,
                        onclick: move |_| QUERY.write().clear(),
                        "×"
                    }
                }
            }
        }
        if !q.is_empty() {
            Results { q: q.clone(), next_id: next_id.clone() }
        } else if let Some(shown) = shown {
            Shown { id: shown.clone(), next_id: next_id.clone() }
            p { class: "note small", {tx::UNVERIFIED_NOTE} }
            if done == total && total > 0 {
                p { class: "ls-verdict good", {tx::ALL_LESSONS_DONE} }
            }
            div { class: "section-head", h2 { {tx::OTHER_LECTURES} } }
            Others { shown: shown.clone() }
        }
    }
}

/// The other lectures' cards: the screen's view switcher picks cards, rows
/// or one line each, and the order.
pub(crate) const LECTURES_VIEW: Collection = Collection {
    key: "lessons.lectures",
    layout: Layout::Cards,
    sorts: &[Sort::Book, Sort::Left, Sort::Name],
    groups: false,
};
/// A lecture's lessons on this screen (the picked lecture, search results).
pub(crate) const LESSONS_VIEW: Collection =
    Collection { key: "lessons.rows", layout: Layout::List, sorts: &[Sort::Book, Sort::Left], groups: false };
/// A lecture's lessons on its page.
pub(crate) const LECTURE_PAGE_VIEW: Collection = Collection {
    key: "lecture.lessons",
    layout: Layout::List,
    sorts: &[Sort::Book, Sort::Left],
    groups: false,
};

/// Every lecture but the shown one. One with lessons moves to the top when
/// picked; one not written yet opens its page (the book's pages and cards).
#[component]
fn Others(shown: String) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let pick = LECTURES_VIEW.pick();
    let mut others: Vec<_> = lib.index.lectures.iter().filter(|e| e.id != shown).collect();
    let left =
        |id: &str| lib.lessons.get(id).is_some_and(|f| f.lessons.iter().any(|l| mark(&l.id).is_none()));
    sort_by(&mut others, pick.sort, |e| left(&e.id), |_| false, |e| e.title.clone());
    rsx! {
        ViewBar { of: LECTURES_VIEW }
        ul { class: "coll ls-others", "data-layout": pick.layout.as_str(),
            for entry in others {
                {
                    let id = entry.id.clone();
                    let code = crate::content::lecture_code(&entry.id);
                    match lib.lessons.get(&entry.id) {
                        Some(file) => {
                            let n = file.lessons.len();
                            let d = file.lessons.iter().filter(|l| mark(&l.id).is_some()).count();
                            rsx! {
                                li { key: "{entry.id}",
                                    button {
                                        class: "coll-btn ls-other",
                                        title: tx::SHOW_LECTURE,
                                        onclick: move |_| {
                                            *SHOWN.write() = Some(id.clone());
                                            document::eval(r#"document.querySelector("main.content")?.scrollTo(0, 0);"#);
                                        },
                                        span { class: "coll-lead",
                                            span { class: "coll-badge", "data-tone": if d == n && n > 0 { "good" } else { "" }, "{code}" }
                                        }
                                        span { class: "coll-main",
                                            bdi { class: "coll-title", dir: "ltr", "{entry.title}" }
                                            span { class: "coll-sub", {tx::lessons_n(n)} " · " {tx::done_of(d, n)} }
                                            span { class: "coll-extra", Meter { done: d, total: n } }
                                        }
                                        span { class: "coll-end coll-when-compact",
                                            crate::parts::Count { done: d, total: n }
                                        }
                                        span { class: "chev", aria_hidden: "true" }
                                    }
                                }
                            }
                        }
                        None => rsx! {
                            li { key: "{entry.id}",
                                button {
                                    class: "coll-btn ls-other ls-soon",
                                    title: tx::OPEN_LECTURE,
                                    onclick: move |_| crate::lecture::open(ui, &id),
                                    span { class: "coll-lead",
                                        span { class: "coll-badge", "data-tone": "quiet", "{code}" }
                                    }
                                    span { class: "coll-main",
                                        bdi { class: "coll-title", dir: "ltr", "{entry.title}" }
                                        span { class: "coll-sub", {tx::OPEN_LECTURE} }
                                    }
                                    span { class: "coll-end", span { class: "chip", {tx::WRITING} } }
                                    span { class: "chev", aria_hidden: "true" }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// A thin progress line, `done` of `total`.
#[component]
fn Meter(done: usize, total: usize) -> Element {
    let pct = (done * 100).checked_div(total).unwrap_or(0);
    rsx! {
        span {
            class: "progress ls-meter",
            role: "progressbar",
            aria_valuemin: "0",
            aria_valuemax: "{total}",
            aria_valuenow: "{done}",
            span { class: "progress-fill", style: "inline-size: {pct}%" }
        }
    }
}

/// The picked lecture: a header with its progress and the way into its
/// next lesson, then every lesson as a row.
#[component]
fn Shown(id: String, next_id: Option<String>) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let Some(file) = lib.lessons.get(&id) else { return rsx! {} };
    let title = lib.title(&id).to_string();
    let code = crate::content::lecture_code(&id);
    let n = file.lessons.len();
    let d = file.lessons.iter().filter(|l| mark(&l.id).is_some()).count();
    let pct = (d * 100).checked_div(n).unwrap_or(0);
    // This lecture's way in: the lesson part-way through, else the first unfinished.
    let go = file
        .lessons
        .iter()
        .find(|l| resume_step(&l.id).is_some())
        .or_else(|| file.lessons.iter().find(|l| mark(&l.id).is_none()));
    let go_id = go.map(|l| l.id.clone());
    let next = next_id.as_deref();
    let pick = LESSONS_VIEW.pick();
    let rows = numbered(&file.lessons, pick.sort);
    rsx! {
        // The whole header opens the lecture's next lesson (its button's
        // click bubbles up); the lecture page has its own link.
        section {
            class: if go_id.is_some() { "card card-flat card-action ls-lec" } else { "card card-flat ls-lec" },
            onclick: move |_| if let Some(g) = &go_id { open(ui, g) },
            div { class: "ls-lec-meta",
                span { {tx::lecture_n(&code)} }
                button {
                    class: "btn-link ls-lec-page",
                    onclick: { let id = id.clone(); move |e: MouseEvent| { e.stop_propagation(); crate::lecture::open(ui, &id) } },
                    {tx::LECTURE_PAGE}
                    span { class: "chev", aria_hidden: "true" }
                }
            }
            h2 { bdi { dir: "ltr", "{title}" } }
            p { class: "small ls-lec-sub", {tx::lessons_n(n)} }
            Meter { done: d, total: n }
            div { class: "ls-lec-caption small",
                span { {tx::done_of(d, n)} }
                bdi { dir: "ltr", "{pct}%" }
            }
            if let Some(l) = go {
                button { class: "btn btn-primary ls-lec-go",
                    span { class: "ls-btn-label",
                        {if resume_step(&l.id).is_some() || d > 0 { tx::CONTINUE } else { tx::START_FIRST }}
                        " · "
                        text::Line { text: l.title.clone() }
                    }
                    {crate::icons::arrow_go()}
                }
            }
        }
        ViewBar { of: LESSONS_VIEW }
        ol { class: "coll ls-rows", "data-layout": pick.layout.as_str(),
            for (n , l) in rows {
                LessonRow { key: "{l.id}", n, lesson: l.clone(), next: next.map(str::to_string) }
            }
        }
    }
}

/// A lecture's lessons with their numbers, in the picked order.
pub(crate) fn numbered(lessons: &[Lesson], sort: Sort) -> Vec<(usize, &Lesson)> {
    let mut rows: Vec<(usize, &Lesson)> = lessons.iter().enumerate().map(|(i, l)| (i + 1, l)).collect();
    sort_by(&mut rows, sort, |(_, l)| mark(&l.id).is_none(), |_| false, |(_, l)| l.title.clone());
    rows
}

/// One lecture's search hits: its id, title, and (number, lesson) pairs.
type Hits = (String, String, Vec<(usize, Lesson)>);

/// Search results, by lecture.
#[component]
fn Results(q: String, next_id: Option<String>) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let groups: Vec<Hits> = lib
        .index
        .lectures
        .iter()
        .filter_map(|e| {
            let file = lib.lessons.get(&e.id)?;
            let hits: Vec<(usize, Lesson)> = file
                .lessons
                .iter()
                .enumerate()
                .filter(|(_, l)| matches(&q, &e.id, &e.title, l))
                .map(|(i, l)| (i + 1, l.clone()))
                .collect();
            (!hits.is_empty()).then(|| (e.id.clone(), e.title.clone(), hits))
        })
        .collect();
    let layout = LESSONS_VIEW.pick().layout;
    rsx! {
        if groups.is_empty() {
            p { class: "note", role: "status", {tx::no_match(&q)} }
        }
        for (id , title , hits) in groups {
            section { key: "{id}", class: "ls-group",
                h2 { class: "ls-sec",
                    button {
                        class: "tap tap-plain ls-group-head",
                        title: tx::OPEN_LECTURE,
                        onclick: { let id = id.clone(); move |_| crate::lecture::open(ui, &id) },
                        span { class: "num", "{crate::content::lecture_code(&id)}" }
                        bdi { dir: "ltr", "{title}" }
                        span { class: "chev", aria_hidden: "true" }
                    }
                }
                ol { class: "coll ls-rows", "data-layout": layout.as_str(),
                    for (n , l) in hits {
                        LessonRow { key: "{l.id}", n, lesson: l, next: next_id.clone() }
                    }
                }
            }
        }
    }
}

/// One lesson as a row: number (or a tick), title and hook, and its state
/// in words with the time it takes.
#[component]
fn LessonRow(n: usize, lesson: Lesson, next: Option<String>) -> Element {
    let ui = use_context::<Ui>();
    let st = state(&lesson, next.as_deref());
    let steps = lesson.steps.len();
    let id = lesson.id.clone();
    let key = match st {
        State::Done(_) => "done",
        State::Current { .. } => "current",
        State::Next => "next",
        State::New => "new",
    };
    let tone = match st {
        State::Done(_) => "good",
        State::Current { .. } | State::Next => "strong",
        State::New => "quiet",
    };
    rsx! {
        li { class: "ls-item", "data-state": key,
            button { class: "coll-btn", onclick: move |_| open(ui, &id),
                span { class: "coll-lead", aria_hidden: "true",
                    span { class: "coll-badge", "data-tone": tone,
                        if matches!(st, State::Done(_)) { {crate::icons::check()} } else { "{n}" }
                    }
                }
                span { class: "coll-main",
                    strong { class: "coll-title", text::Line { text: lesson.title.clone() } }
                    span { class: "coll-sub", text::Line { text: lesson.hook.clone() } }
                }
                span { class: "coll-end",
                    match st {
                        State::Done(m) => rsx! {
                            span { class: "ls-state", {tx::STATE_DONE} }
                            bdi { class: "num-ltr", "{m.right}/{m.asked}" }
                        },
                        State::Current { at } => rsx! {
                            span { class: "ls-state", {tx::STATE_CURRENT} }
                            span { {tx::step_of(at + 1, steps)} }
                        },
                        State::Next => rsx! {
                            span { class: "ls-state", {tx::STATE_NEXT} }
                            span { {count(lesson.minutes, &MINUTE)} }
                        },
                        State::New => rsx! {
                            span { class: "ls-state", {tx::STATE_NEW} }
                            span { {count(lesson.minutes, &MINUTE)} }
                        },
                    }
                }
                span { class: "chev", aria_hidden: "true" }
            }
        }
    }
}

/// One lesson on its lecture's page: number (or a tick), title and hook,
/// time, questions and book pages, and whether it is done.
#[component]
pub fn LessonCard(n: usize, lesson: Lesson) -> Element {
    let ui = use_context::<Ui>();
    let m = mark(&lesson.id);
    let id = lesson.id.clone();
    let questions = lesson.steps.iter().filter(|s| s.body.is_question()).count();
    rsx! {
        li { class: "ls-card", "data-done": if m.is_some() { "yes" } else { "no" },
            button { class: "coll-btn ls-card-btn", onclick: move |_| open(ui, &id),
                span { class: "coll-lead", aria_hidden: "true",
                    span { class: "coll-badge", "data-tone": if m.is_some() { "good" } else { "quiet" }, "{n}" }
                }
                span { class: "coll-main",
                    strong { class: "coll-title", text::Line { text: lesson.title.clone() } }
                    span { class: "coll-sub", text::Line { text: lesson.hook.clone() } }
                    span { class: "coll-extra ls-card-meta small",
                        span { class: "ls-nowrap", {count(lesson.minutes, &MINUTE)} }
                        span { class: "ls-nowrap", {count(questions, &QUESTION)} }
                        span { class: "ls-nowrap",
                            "{tx::PAGES} "
                            bdi { dir: "ltr", "{lesson.pages[0]}–{lesson.pages[1]}" }
                        }
                    }
                }
                span { class: "coll-end",
                    match m {
                        Some(m) => rsx! { span { class: "chip ls-done", "✓ {m.right}/{m.asked}" } },
                        None => rsx! { span { class: "chip", {tx::NEW} } },
                    }
                }
                span { class: "chev", aria_hidden: "true" }
            }
        }
    }
}

/// Today's card: the lesson to go on with (part-way through first, else
/// the next one), one tap away, with how far along it is.
#[component]
pub fn TodayLesson(prefer: Vec<String>) -> Element {
    let ui = use_context::<Ui>();
    let lib = Library::get();
    let part_way = lib.all_lessons().map(|(_, l)| l).find(|l| resume_step(&l.id).is_some());
    let Some(l) = part_way.or_else(|| next_lesson(&prefer)) else {
        return rsx! {};
    };
    let lecture = l.id.rsplit_once('.').map(|(l, _)| l).unwrap_or(&l.id);
    let title = lib.title(lecture).to_string();
    let steps = l.steps.len();
    let at = resume_step(&l.id);
    let done_steps = at.unwrap_or(0);
    rsx! {
        style { {CSS.text()} }
        button { class: "card tap ls-now",
            onclick: { let id = l.id.clone(); move |_| open(ui, &id) },
            span { class: "ls-now-top",
                span { class: "chip", "data-tone": "accent", {if at.is_some() { tx::NOW_KICKER } else { tx::NEXT_KICKER }} }
                span { class: "muted small ls-now-where",
                    match at {
                        Some(at) => rsx! { {tx::step_of(at + 1, steps)} },
                        None => rsx! { {count(l.minutes, &MINUTE)} },
                    }
                    span { class: "chev", aria_hidden: "true" }
                }
            }
            strong { class: "ls-now-title", text::Line { text: l.title.clone() } }
            span { class: "muted small", bdi { dir: "ltr", "{crate::content::lecture_code(lecture)} · {title}" } }
            span { class: "small ls-now-hook", text::Line { text: l.hook.clone() } }
            Meter { done: done_steps, total: steps }
        }
    }
}

/// A small "explain it to me" link under a study item whose lesson exists.
#[component]
pub fn ExplainLink(item_id: String) -> Element {
    let ui = use_context::<Ui>();
    let Some(l) = lesson_for_item(&item_id) else {
        return rsx! {};
    };
    let id = l.id.clone();
    rsx! {
        button { class: "st-link ls-explain", title: tx::EXPLAIN_HINT, onclick: move |_| open(ui, &id),
            "📖 {tx::EXPLAIN} · "
            text::Line { text: l.title.clone() }
        }
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;

    #[test]
    fn search_finds_lessons_by_title_topic_and_lecture() {
        let lib = Library::get();
        let (lec, l) = lib.all_lessons().next().expect("a lesson is written");
        let title = lib.title(lec).to_string();
        let code = crate::content::lecture_code(lec).to_lowercase();
        assert!(matches(&code, lec, &title, l));
        assert!(matches(&title.to_lowercase(), lec, &title, l));
        if let Some(t) = l.topics.first() {
            assert!(matches(&t.to_lowercase(), lec, &title, l));
        }
        assert!(!matches("zzz-no-such-word", lec, &title, l));
    }

    #[test]
    fn lesson_counts_read_naturally() {
        assert_eq!(tx::lessons_n(1), "درس واحد");
        assert_eq!(tx::lessons_n(2), "درسين");
        assert_eq!(tx::lessons_n(7), "7 دروس");
        assert_eq!(tx::lessons_n(12), "12 درس");
    }
}
