//! «ابدأ من الأول» (#1341): restart progress for one course or the whole
//! curriculum. The rule lives in `batna_core::reset`: a reset is one synced
//! marker, nothing is deleted, and undo removes the marker. This module
//! keeps the markers in a global for the readers (lesson marks and places,
//! papers), runs a reset (refused under a running session or paper, a
//! backup first) and draws the card in Settings › بياناتك and on a course's
//! page on the map.

use batna_core::reset::{Marker, Resets, Scope};
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::reset as tx;
use crate::store::Review;

/// The resets in force, re-read after every reset, undo, sync or restore.
static RESETS: GlobalSignal<Resets> = Signal::global(|| crate::store::shared().resets());

/// Re-reads the markers (a sync or restore changed the store).
pub fn reload() {
    let fresh = crate::store::shared().resets();
    if *RESETS.peek() != fresh {
        *RESETS.write() = fresh;
    }
}

/// Whether something in `id`'s course done at `at_ms` (a finished lesson,
/// a lesson's attempt) still counts after the resets in force.
pub fn counts_at(id: &str, at_ms: i64) -> bool {
    RESETS.read().counts_at(id, at_ms)
}

/// A scope's name as he reads it: «المنهج كله», or the course's short name.
pub fn name(scope: &Scope) -> String {
    match scope {
        Scope::All => tx::ALL.to_string(),
        Scope::Course(c) => Library::get().program.course(c).map_or_else(|| c.clone(), |c| c.short.clone()),
    }
}

/// Why a reset can't run right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocked {
    Session,
    Paper,
}

impl Blocked {
    pub fn line(self) -> &'static str {
        match self {
            Blocked::Session => tx::BLOCKED_SESSION,
            Blocked::Paper => tx::BLOCKED_PAPER,
        }
    }
}

/// A study session or a timed paper is open: the reset waits for it.
pub fn blocked() -> Option<Blocked> {
    if crate::study::running_progress().is_some() {
        Some(Blocked::Session)
    } else if crate::bank::paper_open() {
        Some(Blocked::Paper)
    } else {
        None
    }
}

/// What a reset of a scope would restart, with real numbers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Preview {
    /// Items with a schedule now.
    pub cards: usize,
    /// Lessons finished.
    pub lessons: usize,
    /// Finished papers shown (a whole-curriculum reset can take them too).
    pub papers: usize,
}

pub fn preview(review: &Review, scope: &Scope) -> Preview {
    let cards = review.backend.borrow().seen_in(scope);
    let lessons = Library::get()
        .lessons
        .iter()
        .filter(|(lecture, _)| scope.covers(lecture))
        .flat_map(|(_, f)| f.lessons.iter())
        .filter(|l| crate::learn::mark(&l.id).is_some())
        .count();
    let papers = if *scope == Scope::All { crate::bank::papers_shown() } else { 0 };
    Preview { cards, lessons, papers }
}

/// Why a reset did not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    Blocked(Blocked),
    /// The backup failed: nothing was reset.
    Backup(String),
    Store(String),
}

/// Runs a reset: refused under a running session or paper, then a backup
/// (nothing happens if it fails), then the marker. Returns where the
/// backup went.
pub fn run(review: &Review, scope: &Scope, papers: bool) -> Result<String, Refused> {
    if let Some(b) = blocked() {
        return Err(Refused::Blocked(b));
    }
    let backup = crate::data_export::backup_now(crate::store::shared()).map_err(Refused::Backup)?;
    apply(review, scope, papers).map_err(Refused::Store)?;
    Ok(backup)
}

/// Writes the marker and brings every screen up to date. The catch-up plan
/// running now goes into the marker (undo puts it back); the whole
/// curriculum ends it, a course reset shrinks it to what is still due.
pub fn apply(review: &Review, scope: &Scope, papers: bool) -> Result<Marker, String> {
    let store = crate::store::shared();
    let plan = crate::catchup::stored();
    let marker = store
        .put_reset(scope, |m| {
            m.papers = papers && *scope == Scope::All;
            m.catchup = plan.and_then(|p| serde_json::to_value(p).ok());
        })
        .map_err(|e| e.to_string())?;
    review.backend.borrow_mut().reload_and_notify();
    if let Some(p) = plan {
        let now = crate::store::now_ms();
        let (today, due) = {
            let b = review.backend.borrow();
            (b.study_day(now), b.overview(Library::get(), now).due)
        };
        match scope {
            Scope::All => crate::catchup::replace(None),
            Scope::Course(_) if p.running(today) => crate::catchup::replace(
                (due > 0).then(|| crate::catchup::Plan { backlog: p.backlog.min(due as u32), ..p }),
            ),
            Scope::Course(_) => {}
        }
    }
    Ok(marker)
}

/// Takes a reset back: the marker goes, and with it everything reads as it
/// was; a catch-up plan the reset ended comes back.
pub fn undo(review: &Review, scope: &Scope) -> Result<(), String> {
    let store = crate::store::shared();
    let marker = store.resets().get(scope).cloned();
    store.remove_reset(scope).map_err(|e| e.to_string())?;
    if let Some(plan) = marker.and_then(|m| m.catchup).and_then(|v| serde_json::from_value(v).ok()) {
        crate::catchup::replace(Some(plan));
    }
    review.backend.borrow_mut().reload_and_notify();
    Ok(())
}

/// What the card shows after a tap.
#[derive(Debug, Clone, PartialEq)]
enum Shown {
    Idle,
    Done { scope: Scope, backup: String },
    Failed(Refused),
    Undone,
}

/// Settings › بياناتك: pick a course or the whole curriculum, then confirm.
/// Lists the resets in force, each with its undo.
#[component]
pub fn ResetCard() -> Element {
    let lib = Library::get();
    let mut picked = use_signal(|| None::<Scope>);
    let scopes: Vec<Scope> = lib
        .program
        .courses
        .iter()
        .filter(|c| {
            lib.lessons.keys().any(|l| batna_core::program::course_of(l) == c.id)
                || lib.lectures.keys().any(|l| batna_core::program::course_of(l) == c.id)
        })
        .map(|c| Scope::Course(c.id.clone()))
        .chain([Scope::All])
        .collect();
    rsx! {
        crate::ui::Card { class: "reset-card",
            h2 { {tx::TITLE} }
            p { class: "small", {tx::INTRO} }
            div { role: "radiogroup", aria_label: tx::PICK,
                p { class: "muted small", {tx::PICK} }
                div { class: "chips",
                    for s in scopes {
                        button {
                            key: "{s.id()}",
                            class: "chip toggle",
                            role: "radio",
                            "aria-checked": picked() == Some(s.clone()),
                            onclick: {
                                let s = s.clone();
                                move |_| {
                                    let same = picked() == Some(s.clone());
                                    picked.set(if same { None } else { Some(s.clone()) });
                                }
                            },
                            bdi { {name(&s)} }
                        }
                    }
                }
            }
            if let Some(scope) = picked() {
                Confirm { key: "{scope.id()}", scope, on_cancel: move |_| picked.set(None) }
            }
            InForce {}
        }
    }
}

/// A course's page on the map: one button, then the same confirm.
#[component]
pub fn CourseReset(course: String) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        if open() {
            Confirm { scope: Scope::Course(course.clone()), on_cancel: move |_| open.set(false) }
        } else {
            button { class: "btn btn-ghost reset-open", onclick: move |_| open.set(true), {tx::COURSE_BUTTON} }
        }
        InForceOne { scope: Scope::Course(course) }
    }
}

/// What restarts and what stays, with the real numbers, and the deliberate
/// confirm: a second tap for one course, the typed word for everything.
#[component]
fn Confirm(scope: Scope, on_cancel: EventHandler) -> Element {
    let review = crate::store::use_review();
    let _ = RESETS();
    let p = preview(&review, &scope);
    let mut armed = use_signal(|| false);
    let mut typed = use_signal(String::new);
    let mut papers = use_signal(|| false);
    let mut shown = use_signal(|| Shown::Idle);
    let all = scope == Scope::All;
    let n = name(&scope);
    let ready = !all || typed().trim() == tx::TYPE_WORD;
    let go = {
        let scope = scope.clone();
        let review = review.clone();
        move |_| {
            shown.set(match run(&review, &scope, papers()) {
                Ok(backup) => Shown::Done { scope: scope.clone(), backup },
                Err(e) => Shown::Failed(e),
            });
            armed.set(false);
            typed.set(String::new());
        }
    };
    let blocked_now = blocked();
    rsx! {
        div { class: "reset-confirm", role: "region", aria_label: tx::TITLE,
            match shown() {
                Shown::Done { scope, backup } => rsx! {
                    Done { scope, backup, on_undone: move |_| shown.set(Shown::Undone) }
                },
                Shown::Undone => rsx! {
                    p { class: "small", role: "status", {tx::UNDONE} }
                },
                other => rsx! {
                    p { class: "small", strong { {tx::RESTARTS} } " " {tx::restarts_line(p.cards, p.lessons)} " " bdi { {tx::in_scope(&n)} } }
                    p { class: "small", strong { {tx::STAYS} } " " {tx::STAYS_LINE} }
                    if all {
                        p { class: "muted small", {tx::STAYS_OWN} }
                        if p.papers > 0 {
                            label { class: "reset-papers",
                                input { r#type: "checkbox", checked: papers(), onchange: move |e| papers.set(e.checked()) }
                                span { {tx::papers_too(p.papers)} }
                            }
                        }
                    }
                    p { class: "muted small", {tx::BACKUP_FIRST} " " {tx::UNDO_HINT} }
                    if let Shown::Failed(e) = other {
                        p { class: "small", role: "alert",
                            match e {
                                Refused::Blocked(b) => rsx! { {b.line()} },
                                Refused::Backup(d) => rsx! { {tx::BACKUP_FAILED} " " bdi { dir: "ltr", "{d}" } },
                                Refused::Store(d) => rsx! { {tx::FAILED} " " bdi { dir: "ltr", "{d}" } },
                            }
                        }
                    }
                    if let Some(b) = blocked_now {
                        p { class: "small", role: "status", {b.line()} }
                    } else if all {
                        label { class: "reset-type",
                            span { class: "small", {tx::TYPE_TO_CONFIRM} }
                            input {
                                r#type: "text",
                                value: "{typed}",
                                aria_label: tx::TYPE_TO_CONFIRM,
                                oninput: move |e| typed.set(e.value()),
                            }
                        }
                        div { class: "reset-actions",
                            button { class: "btn btn-primary", disabled: !ready, onclick: go, {tx::GO} }
                            button { class: "btn btn-ghost", onclick: move |_| on_cancel.call(()), {tx::CANCEL} }
                        }
                    } else if armed() {
                        div { class: "reset-actions",
                            button { class: "btn btn-primary", onclick: go, {tx::confirm(&n)} }
                            button { class: "btn btn-ghost", onclick: move |_| on_cancel.call(()), {tx::CANCEL} }
                        }
                    } else {
                        div { class: "reset-actions",
                            button { class: "btn btn-primary", onclick: move |_| armed.set(true), {tx::GO} }
                            button { class: "btn btn-ghost", onclick: move |_| on_cancel.call(()), {tx::CANCEL} }
                        }
                    }
                },
            }
        }
    }
}

/// Right after a reset: what happened, where the backup is, and undo.
#[component]
fn Done(scope: Scope, backup: String, on_undone: EventHandler) -> Element {
    let review = crate::store::use_review();
    let mut failed = use_signal(|| None::<String>);
    let n = name(&scope);
    rsx! {
        div { role: "status",
            p { class: "small", bdi { {tx::done(&n)} } }
            p { class: "muted small", {tx::BACKUP_AT} " " bdi { dir: "ltr", "{backup}" } }
            button {
                class: "btn btn-ghost reset-undo",
                onclick: move |_| match undo(&review, &scope) {
                    Ok(()) => on_undone.call(()),
                    Err(e) => failed.set(Some(e)),
                },
                {tx::UNDO}
            }
            if let Some(e) = failed() {
                p { class: "small", role: "alert", {tx::FAILED} " " bdi { dir: "ltr", "{e}" } }
            }
        }
    }
}

/// Every reset in force, each with its undo, for as long as it stands.
#[component]
fn InForce() -> Element {
    let resets = RESETS();
    if resets.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "reset-in-force",
            h3 { class: "small", {tx::IN_FORCE} }
            for (scope , _) in resets.scopes() {
                InForceOne { key: "{scope.id()}", scope }
            }
        }
    }
}

#[component]
fn InForceOne(scope: Scope) -> Element {
    let review = crate::store::use_review();
    let mut failed = use_signal(|| None::<String>);
    if RESETS().get(&scope).is_none() {
        return rsx! {};
    }
    let n = name(&scope);
    rsx! {
        button {
            class: "btn btn-ghost reset-undo",
            onclick: {
                let scope = scope.clone();
                move |_| {
                    if let Err(e) = undo(&review, &scope) {
                        failed.set(Some(e));
                    }
                }
            },
            bdi { {tx::undo_scope(&n)} }
        }
        if let Some(e) = failed() {
            p { class: "small", role: "alert", {tx::FAILED} " " bdi { dir: "ltr", "{e}" } }
        }
    }
}

#[cfg(test)]
mod tests;
