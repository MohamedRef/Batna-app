//! The "add a card" form (a sheet over any screen), the list on the Study
//! screen, and the Android share inbox that opens the form with a draft.

use batna_core::own::{self as core_own, OwnCard, Problem};
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::own as tx;
use crate::look::{Css, css};
use crate::store::{self, own::new_id, use_review};

const CSS: Css = css!("src/own/own.css");

/// The card being written or edited; the sheet shows while it's set.
pub static DRAFT: GlobalSignal<Option<Draft>> = Signal::global(|| None);
/// A share that arrived while he was writing a card: it waits, and opens
/// once that sheet closes, so nothing he typed is replaced.
static QUEUED: GlobalSignal<Option<Draft>> = Signal::global(|| None);

#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub card: OwnCard,
    pub kind: DraftKind,
    /// A shared passage was longer than a card's back and its end was cut.
    pub trimmed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftKind {
    New,
    Edit,
    Shared,
}

fn open_new() {
    let card = OwnCard {
        id: new_id(),
        front: String::new(),
        back: String::new(),
        lecture: None,
        page: None,
        source: None,
        created_ms: store::now_ms(),
    };
    *DRAFT.write() = Some(Draft { card, kind: DraftKind::New, trimmed: false });
}

/// Mounted once in the app: picks up text shared from other apps (Android),
/// and shows the form sheet whenever there is a draft.
#[component]
pub fn ShareInbox() -> Element {
    use_future(|| async move {
        if !cfg!(target_os = "android") {
            return;
        }
        // A share brings Batna to the front, so look each time the page
        // becomes visible (and once at start, for a cold start), not on a timer.
        let mut eval = document::eval(
            r#"
            const send = () => { if (!document.hidden) dioxus.send(true); };
            document.addEventListener("visibilitychange", send);
            send();
            await new Promise(() => {});
            "#,
        );
        while eval.recv::<bool>().await.is_ok() {
            let Some(shared) = super::share::take() else { continue };
            let (card, trimmed) = core_own::draft_from_shared(
                &shared.text,
                shared.source().as_deref(),
                new_id(),
                store::now_ms(),
            );
            let draft = Draft { card, kind: DraftKind::Shared, trimmed };
            if DRAFT.peek().is_some() {
                *QUEUED.write() = Some(draft);
            } else {
                *DRAFT.write() = Some(draft);
            }
        }
    });
    // The sheet closed: the share that waited opens now.
    use_effect(|| {
        if DRAFT.read().is_none()
            && let Some(next) = QUEUED.write().take()
        {
            *DRAFT.write() = Some(next);
        }
    });
    rsx! {
        if let Some(draft) = DRAFT() {
            Sheet { key: "{draft.card.id}", draft }
        }
    }
}

#[component]
fn Sheet(draft: Draft) -> Element {
    let review = use_review();
    let mut card = use_signal(|| draft.card.clone());
    let mut problem = use_signal(|| None::<String>);
    let title = match draft.kind {
        DraftKind::New => tx::FORM_NEW,
        DraftKind::Edit => tx::FORM_EDIT,
        DraftKind::Shared => tx::FORM_SHARED,
    };
    let lectures: Vec<(String, String)> =
        Library::get().index.lectures.iter().map(|e| (e.id.clone(), e.title.clone())).collect();
    let close = move || *DRAFT.write() = None;
    let save = move |_| {
        let c = card().tidy();
        match c.check() {
            Err(p) => problem.set(Some(problem_text(p).to_string())),
            Ok(()) => match review.backend.borrow_mut().save_own(&c) {
                Ok(()) => *DRAFT.write() = None,
                Err(e) => problem.set(Some(format!("{} {e}", tx::FAILED))),
            },
        }
    };
    let c = card();
    rsx! {
        style { {CSS.text()} }
        div { class: "own-backdrop", onclick: move |_| close() }
        div {
            class: "own-sheet",
            role: "dialog",
            aria_modal: "true",
            aria_label: title,
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    close();
                }
            },
            h2 { {title} }
            if draft.kind == DraftKind::Shared {
                p { class: "muted small", {tx::SHARED_HINT} }
            }
            if draft.trimmed {
                p { class: "muted small", {tx::SHARED_TRIMMED} }
            }
            if QUEUED.read().is_some() {
                p { class: "small own-waiting", role: "status", {tx::SHARE_WAITING} }
            }
            label { class: "own-field",
                span { {tx::FRONT} }
                textarea {
                    dir: "auto",
                    lang: "en",
                    rows: 3,
                    autofocus: true,
                    value: "{c.front}",
                    oninput: move |e| card.write().front = e.value(),
                }
            }
            label { class: "own-field",
                span { {tx::BACK} }
                textarea {
                    dir: "auto",
                    lang: "en",
                    rows: 7,
                    value: "{c.back}",
                    oninput: move |e| card.write().back = e.value(),
                }
            }
            div { class: "own-row",
                label { class: "own-field",
                    span { {tx::LECTURE} }
                    select {
                        onchange: move |e| {
                            let v = e.value();
                            card.write().lecture = (!v.is_empty()).then_some(v);
                        },
                        option { value: "", selected: c.lecture.is_none(), {tx::NO_LECTURE} }
                        for (id , name) in lectures {
                            option {
                                key: "{id}",
                                value: "{id}",
                                selected: c.lecture.as_deref() == Some(id.as_str()),
                                "{id.rsplit('.').next().unwrap_or_default()} · {name}"
                            }
                        }
                    }
                }
                label { class: "own-field own-page",
                    span { {tx::PAGE} }
                    input {
                        r#type: "number",
                        inputmode: "numeric",
                        min: "1",
                        dir: "ltr",
                        value: c.page.map(|p| p.to_string()).unwrap_or_default(),
                        oninput: move |e| card.write().page = e.value().trim().parse().ok(),
                    }
                }
            }
            label { class: "own-field",
                span { {tx::SOURCE} }
                input {
                    dir: "auto",
                    placeholder: tx::SOURCE_HINT,
                    value: c.source.clone().unwrap_or_default(),
                    oninput: move |e| card.write().source = Some(e.value()),
                }
            }
            if let Some(p) = problem() {
                p { class: "own-error", role: "alert", "{p}" }
            }
            div { class: "own-actions",
                button { class: "btn btn-primary", onclick: save, {tx::SAVE} }
                button { class: "btn btn-secondary", onclick: move |_| close(), {tx::CANCEL} }
            }
        }
    }
}

fn problem_text(p: Problem) -> &'static str {
    match p {
        Problem::EmptyFront => tx::E_FRONT,
        Problem::EmptyBack => tx::E_BACK,
        Problem::FrontTooLong => tx::E_FRONT_LONG,
        Problem::BackTooLong => tx::E_BACK_LONG,
        Problem::SourceTooLong => tx::E_SOURCE_LONG,
    }
}

/// His cards on the Study screen: add, edit, delete, study just these.
#[component]
pub fn OwnCardsCard(on_study: EventHandler<()>) -> Element {
    let review = use_review();
    // Re-read whenever the review store changes (a save, an import, a sync).
    let _ = review.revision.read();
    let cards = super::cards();
    let mut confirm = use_signal(|| None::<String>);
    rsx! {
        style { {CSS.text()} }
        section { class: "card own",
            div { class: "own-head",
                h2 { {tx::TITLE} }
                if !cards.is_empty() {
                    span { class: "chip", "{cards.len()} {tx::COUNT}" }
                }
            }
            p { class: "muted small", {tx::INTRO} }
            if cfg!(target_os = "android") {
                p { class: "muted small", {tx::SHARE_HINT} }
            }
            div { class: "own-actions",
                button { class: "btn btn-primary", onclick: move |_| open_new(), {tx::NEW} }
                if !cards.is_empty() {
                    button { class: "btn btn-secondary", onclick: move |_| on_study.call(()), {tx::STUDY_THEM} }
                }
            }
            if cards.is_empty() {
                p { class: "muted small", {tx::NONE_YET} }
            }
            ul { class: "own-list",
                for c in cards {
                    li { key: "{c.id}",
                        div { class: "own-text",
                            bdi { lang: "en", dir: "auto", class: "own-front", "{c.front}" }
                            span { class: "muted small",
                                if let Some(l) = &c.lecture {
                                    bdi { dir: "ltr", "{l.rsplit('.').next().unwrap_or_default()}" }
                                    " "
                                }
                                if let Some(s) = &c.source {
                                    "{tx::FROM} "
                                    bdi { "{s}" }
                                }
                            }
                        }
                        if confirm().as_deref() == Some(c.id.as_str()) {
                            div { class: "own-actions",
                                span { class: "small", {tx::DELETE_SURE} }
                                button {
                                    class: "btn btn-danger",
                                    onclick: {
                                        let id = c.id.clone();
                                        let review = review.clone();
                                        move |_| {
                                            if let Err(e) = review.backend.borrow_mut().delete_own(&id) {
                                                crate::store::write_failed(&format!("deleting own card {id}"), &e);
                                            }
                                            confirm.set(None);
                                        }
                                    },
                                    {tx::DELETE_YES}
                                }
                                button { class: "btn btn-secondary", onclick: move |_| confirm.set(None), {tx::CANCEL} }
                            }
                        } else {
                            div { class: "own-actions",
                                button {
                                    class: "btn btn-secondary",
                                    onclick: {
                                        let c = c.clone();
                                        move |_| *DRAFT.write() = Some(Draft { card: c.clone(), kind: DraftKind::Edit, trimmed: false })
                                    },
                                    {tx::EDIT}
                                }
                                button {
                                    class: "btn btn-secondary",
                                    onclick: {
                                        let id = c.id.clone();
                                        move |_| confirm.set(Some(id.clone()))
                                    },
                                    {tx::DELETE}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
