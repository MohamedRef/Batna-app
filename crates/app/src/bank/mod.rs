//! The question bank (SPLIT.md chunk 4): practise MCQs by lecture, topic or
//! tag, in tutor mode (the explanation after every question) or timed mode
//! (the review at the end), and sit timed mock exams scored per lecture and
//! topic. Picking, timing and scoring live in `batna_core::exam`.

mod player;
mod question;
mod result;
mod setup;

use std::time::{SystemTime, UNIX_EPOCH};

use batna_core::content::Body;
use batna_core::content::{Item, Lecture};
use batna_core::exam::{self, Answer, History, Mode, Outcome, Question};
use batna_core::review::{Grade, Origin, ReviewEvent, StudyStoreHandle};
use dioxus::prelude::*;

use crate::content::Library;

const BANK_CSS: &str = include_str!("bank.css");

/// A paper in progress.
#[derive(Clone, PartialEq)]
pub struct Paper {
    pub mode: Mode,
    pub questions: Vec<Question>,
    pub answers: Vec<Answer>,
    pub at: usize,
    pub started_ms: i64,
    /// When the current question came on screen, for `Answer::took_ms`.
    pub shown_ms: i64,
}

impl Paper {
    fn new(mode: Mode, questions: Vec<Question>) -> Self {
        let now = now_ms();
        Self {
            mode,
            answers: vec![Answer::default(); questions.len()],
            questions,
            at: 0,
            started_ms: now,
            shown_ms: now,
        }
    }

    /// Add the time on the current question before moving off it.
    fn leave(&mut self, now: i64) {
        let spent = u32::try_from(now.saturating_sub(self.shown_ms).max(0)).unwrap_or(u32::MAX);
        if let Some(a) = self.answers.get_mut(self.at) {
            a.took_ms = a.took_ms.saturating_add(spent);
        }
        self.shown_ms = now;
    }

    pub fn overdue(&self, now: i64) -> bool {
        self.mode.timed() && exam::remaining_ms(self.questions.len(), self.started_ms, now) == 0
    }
}

#[derive(Clone, PartialEq)]
pub enum Phase {
    Setup,
    Running(Paper),
    Done { paper: Paper, outcome: Outcome },
}

// Globals, so a paper survives switching tabs mid-exam.
pub static PHASE: GlobalSignal<Phase> = Signal::global(|| Phase::Setup);
pub static HISTORY: GlobalSignal<History> = Signal::global(History::default);
static HISTORY_LOADED: GlobalSignal<bool> = Signal::global(|| false);

/// Finished timed papers and mocks, as a synced setting in the review store.
const PAPERS_KEY: &str = "bank.papers";

#[component]
pub fn Bank() -> Element {
    use_hook(load_history);
    let phase = match &*PHASE.read() {
        Phase::Setup => 0,
        Phase::Running(_) => 1,
        Phase::Done { .. } => 2,
    };
    rsx! {
        style { {BANK_CSS} }
        style { {crate::verify::CSS} }
        div { class: "bank",
            match phase {
                0 => rsx! { setup::Setup {} },
                1 => rsx! { player::Player {} },
                _ => rsx! { result::Result {} },
            }
        }
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// The lecture and item behind a question id.
/// With Mohamed's ✎ edit applied.
pub fn lookup(id: &str) -> Option<(&'static Lecture, Item)> {
    let lecture_id = id.rsplit_once('.').map(|(l, _)| l)?;
    let lecture = Library::get().lectures.get(lecture_id)?;
    let item = lecture.items.iter().find(|i| i.id == id)?;
    let item = match try_consume_context::<StudyStoreHandle>() {
        Some(store) => store.0.borrow().effective(item),
        None => item.clone(),
    };
    Some((lecture, item))
}

/// The MCQs of every written lecture as Mohamed should see them: his ✎
/// edits applied, his ✗ items left out.
pub fn study_lectures() -> Vec<Lecture> {
    let store = try_consume_context::<StudyStoreHandle>();
    let store = store.as_ref().map(|s| s.0.borrow());
    Library::get()
        .lectures
        .values()
        .map(|l| Lecture {
            items: l
                .items
                .iter()
                .filter(|i| matches!(i.body, Body::Mcq { .. }))
                .filter(|i| !store.as_ref().is_some_and(|s| s.is_hidden(&i.id)))
                .map(|i| store.as_ref().map_or_else(|| i.clone(), |s| s.effective(i)))
                .collect(),
            ..l.clone()
        })
        .collect()
}

/// Send an answer to the review store, where FSRS counts it as evidence.
fn record(event: ReviewEvent) {
    if let Some(store) = try_consume_context::<StudyStoreHandle>() {
        store.0.borrow_mut().record(event);
    }
}

/// Fill the history once per launch: last results from the logged bank and
/// mock answers, finished papers from the store's settings.
fn load_history() {
    if *HISTORY_LOADED.peek() {
        return;
    }
    *HISTORY_LOADED.write() = true;
    let mut history = History::default();
    if let Some(store) = try_consume_context::<StudyStoreHandle>() {
        for e in store.0.borrow().log() {
            if matches!(e.origin, Origin::Bank | Origin::Mock) {
                history.last.insert(e.item_id, e.grade != Grade::Again);
            }
        }
    }
    history.papers = crate::store::shared().setting(PAPERS_KEY).unwrap_or_default();
    *HISTORY.write() = history;
}

/// A lecture's topic title, falling back to the slug.
pub fn topic_title(lecture: &str, topic: &str) -> String {
    Library::get()
        .lectures
        .get(lecture)
        .and_then(|l| l.topics.iter().find(|t| t.id == topic))
        .map_or_else(|| topic.to_string(), |t| t.title.clone())
}

pub fn lecture_title(lecture: &str) -> String {
    Library::get().lectures.get(lecture).map_or_else(|| lecture.to_string(), |l| l.title.clone())
}

/// `hb.L09` → `9`.
pub fn lecture_number(lecture: &str) -> &str {
    lecture.rsplit('.').next().unwrap_or(lecture).trim_start_matches('L').trim_start_matches('0')
}

pub fn start(mode: Mode, questions: Vec<Question>) {
    if !questions.is_empty() {
        *PHASE.write() = Phase::Running(Paper::new(mode, questions));
    }
}

/// Pick an option. Tutor mode locks the first pick and sends it to review
/// right away; timed papers let him change his mind until he submits.
pub fn choose(option: usize) {
    let now = now_ms();
    let mut sent = None;
    {
        let mut phase = PHASE.write();
        let Phase::Running(p) = &mut *phase else { return };
        if p.overdue(now) {
            drop(phase);
            finish();
            return;
        }
        let at = p.at;
        let mode = p.mode;
        let a = &mut p.answers[at];
        match mode {
            Mode::Tutor if a.chosen.is_some() => return,
            Mode::Tutor => {
                a.chosen = Some(option);
                a.took_ms = u32::try_from(now.saturating_sub(p.shown_ms).max(0)).unwrap_or(u32::MAX);
                sent = Some((p.questions[at].clone(), a.clone()));
            }
            _ => a.chosen = Some(option),
        }
    }
    if let Some((q, a)) = sent {
        if let Some(e) = exam::review_event(&q, &a, Mode::Tutor, now) {
            record(e);
        }
        HISTORY.write().record(std::slice::from_ref(&q), std::slice::from_ref(&a));
    }
}

/// Move to question `to` (clamped).
pub fn go(to: usize) {
    let now = now_ms();
    let overdue = {
        let mut phase = PHASE.write();
        let Phase::Running(p) = &mut *phase else { return };
        p.leave(now);
        p.at = to.min(p.questions.len().saturating_sub(1));
        p.overdue(now)
    };
    if overdue {
        finish();
    }
}

pub fn toggle_flag() {
    if let Phase::Running(p) = &mut *PHASE.write() {
        let at = p.at;
        p.answers[at].flagged ^= true;
    }
}

/// End the paper: score it, and for timed papers send every answer to
/// review and keep the result in the history.
pub fn finish() {
    let now = now_ms();
    let paper = {
        let mut phase = PHASE.write();
        let Phase::Running(p) = &mut *phase else { return };
        p.leave(now);
        p.clone()
    };
    let took = u64::try_from(now.saturating_sub(paper.started_ms)).unwrap_or(0);
    let outcome = exam::score(&paper.questions, &paper.answers, paper.mode, now, took);
    if paper.mode.timed() {
        for (q, a) in paper.questions.iter().zip(&paper.answers) {
            if let Some(e) = exam::review_event(q, a, paper.mode, now) {
                record(e);
            }
        }
        let mut history = HISTORY.write();
        history.record(&paper.questions, &paper.answers);
        history.papers.push(outcome.clone());
        crate::store::save_setting(PAPERS_KEY, &history.papers);
    }
    *PHASE.write() = Phase::Done { paper, outcome };
}

pub fn back_to_setup() {
    *PHASE.write() = Phase::Setup;
}

pub fn minutes(ms: u64) -> u64 {
    ms.div_ceil(60_000)
}

/// Unix ms → `2026-10-27`. Pass local wall-clock ms (`store::zone().local_ms`).
pub fn date(ms: i64) -> String {
    // Howard Hinnant's civil_from_days.
    let z = ms.div_euclid(86_400_000) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_minutes() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(1_793_059_200_000), "2026-10-27");
        assert_eq!(minutes(48 * 60_000), 48);
        assert_eq!(minutes(1), 1);
        assert_eq!(lecture_number("hb.L09"), "9");
        assert_eq!(lecture_number("hb.L20"), "20");
    }

    #[test]
    fn paper_time_adds_up_per_question() {
        let q =
            Question { id: "x".into(), lecture: "hb.L01".into(), topic: None, answer: 0, order: vec![0, 1] };
        let mut p = Paper::new(Mode::Timed, vec![q.clone(), q]);
        p.shown_ms = 1_000;
        p.leave(4_000);
        p.at = 1;
        p.leave(5_000);
        p.at = 0;
        p.leave(6_000);
        assert_eq!(p.answers[0].took_ms, 4_000);
        assert_eq!(p.answers[1].took_ms, 1_000);
    }
}
