//! Settings › Your data › Restore: pick a `batna-export` file, see what it
//! holds and what it would change, then merge it (`Store::apply_import`).
//! Nothing on this device is deleted; newer rows win, as in sync.

use std::path::{Path, PathBuf};

use batna_core::review::verify::rfc3339;
use dioxus::prelude::*;
use serde_json::Value;

use crate::i18n::data as tx;
use crate::platform::documents::{self, DocState};
use crate::store::import::{ImportPlan, parse};
use crate::store::{self, use_review};

/// Export files found where Batna writes them, newest name first.
fn found_files() -> Vec<PathBuf> {
    store::export_dir().map(|dir| found_in(&dir)).unwrap_or_default()
}

/// The `batna-export-*.json` files in `dir`, newest name first.
fn found_in(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("batna-export-") && n.ends_with(".json"))
        })
        .collect();
    out.sort();
    out.reverse();
    out
}

/// A real export is a few MB; anything this big is the wrong file.
const MAX_IMPORT_BYTES: u64 = 50 * 1024 * 1024;

/// Reads, parses and plans a backup file off the UI thread.
async fn read_plan(path: PathBuf) -> Result<(Value, ImportPlan), String> {
    crate::platform::time::blocking(move || plan_file(&path, store::shared())).await?
}

/// A backup file read and planned against `store`: too big, unreadable or
/// not an export is an error to show.
fn plan_file(path: &Path, store: &store::Store) -> Result<(Value, ImportPlan), String> {
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > MAX_IMPORT_BYTES {
        return Err(tx::IMPORT_TOO_BIG.to_string());
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = parse(&text).map_err(|e| e.to_string())?;
    let plan = store.plan_import(&doc).map_err(|e| e.to_string())?;
    Ok((doc, plan))
}

/// The step after picking or pasting `text`: its preview, or why it can't be read.
fn plan_text(name: String, text: &str, store: &store::Store) -> Step {
    match parse(text).and_then(|doc| store.plan_import(&doc).map(|plan| (doc, plan))) {
        Ok((doc, plan)) => Step::Preview { name, doc, plan },
        Err(e) => Step::Failed(e.to_string()),
    }
}

#[derive(Clone, PartialEq)]
enum Step {
    Pick,
    Preview { name: String, doc: Value, plan: ImportPlan },
    Done(ImportPlan),
    Failed(String),
}

#[component]
pub fn ImportCard() -> Element {
    rsx! { ImportFlow { start: Step::Pick } }
}

/// The restore card from `start` (always the file picker in the app).
#[component]
fn ImportFlow(start: Step) -> Element {
    let review = use_review();
    let mut step = use_signal(|| start.clone());
    let mut pasted = use_signal(String::new);
    let mut picking = use_signal(|| false);

    let mut load_file = move |name: String, path: PathBuf, remove: bool| {
        picking.set(true);
        spawn(async move {
            let planned = read_plan(path.clone()).await;
            if remove {
                let _ = std::fs::remove_file(&path);
            }
            picking.set(false);
            step.set(match planned {
                Ok((doc, plan)) => Step::Preview { name, doc, plan },
                Err(e) => Step::Failed(e),
            });
        });
    };
    let mut load = move |name: String, text: String| step.set(plan_text(name, &text, store::shared()));

    let body = match step() {
        Step::Pick => {
            let files = found_files();
            let none_found = files.is_empty();
            rsx! {
                if !files.is_empty() {
                    p { class: "small", {tx::IMPORT_FOUND} }
                    ul { class: "import-files",
                        for path in files {
                            li { key: "{path.display()}",
                                button {
                                    class: "btn-link",
                                    onclick: {
                                        let path = path.clone();
                                        move |_| {
                                            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                            load_file(name, path.clone(), false);
                                        }
                                    },
                                    bdi { dir: "ltr", {path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()} }
                                }
                            }
                        }
                    }
                }
                if cfg!(target_os = "android") {
                    // The WebView has no file chooser here: the system picker copies the file in.
                    button {
                        class: "verify-btn plain",
                        disabled: picking(),
                        onclick: move |_| {
                            if documents::open().is_ok() {
                                picking.set(true);
                                spawn(async move {
                                    let done = documents::finished().await;
                                    picking.set(false);
                                    match done {
                                        DocState::Opened { name, path } => load_file(name, PathBuf::from(path), true),
                                        DocState::Failed => step.set(Step::Failed(tx::IMPORT_COPY_FAILED.to_string())),
                                        _ => {}
                                    }
                                });
                            } else {
                                step.set(Step::Failed(tx::IMPORT_PICK_FAILED.to_string()));
                            }
                        },
                        if picking() { {tx::IMPORT_PICKING} } else if none_found { {tx::IMPORT_PICK_FIRST} } else { {tx::IMPORT_PICK} }
                    }
                } else {
                // The native "Choose File" control is English; the shared button
                // carries the Arabic label and the input stays hidden inside it.
                label { class: "btn btn-secondary file-pick",
                    span { if none_found { {tx::IMPORT_PICK_FIRST} } else { {tx::IMPORT_PICK} } }
                    input {
                        r#type: "file",
                        accept: ".json,application/json",
                        onchange: move |e| async move {
                            if let Some(file) = e.files().into_iter().next() {
                                match file.read_string().await {
                                    Ok(text) => load(file.name(), text),
                                    Err(err) => step.set(Step::Failed(err.to_string())),
                                }
                            }
                        },
                    }
                }
                }
                details {
                    summary { class: "small", {tx::IMPORT_PASTE} }
                    textarea {
                        class: "export-text",
                        dir: "ltr",
                        rows: 6,
                        value: "{pasted}",
                        oninput: move |e| pasted.set(e.value()),
                    }
                    button {
                        class: "verify-btn plain",
                        disabled: pasted.read().trim().is_empty(),
                        onclick: move |_| load(tx::IMPORT_PASTED.to_string(), pasted()),
                        {tx::IMPORT_READ}
                    }
                }
            }
        }
        Step::Preview { name, doc, plan } => {
            let changes = plan.changes();
            let review = review.clone();
            rsx! {
                p { class: "small", "{tx::IMPORT_FILE} " bdi { dir: "ltr", "{name}" } }
                p { class: "muted small",
                    "{tx::IMPORT_FROM} " bdi { dir: "ltr", "{plan.device_id}" }
                    " · {tx::IMPORT_AT} " bdi { dir: "ltr", "{rfc3339(plan.exported_at_ms)}" }
                }
                PlanTable { plan: plan.clone() }
                if changes == 0 {
                    p { {tx::IMPORT_NOTHING} }
                    button { class: "verify-btn plain", onclick: move |_| step.set(Step::Pick), {tx::IMPORT_BACK} }
                } else {
                    p { class: "muted small", {tx::IMPORT_SAFE} }
                    div { class: "verify-actions",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                step.set(match store::shared().apply_import(&doc) {
                                    Ok(plan) => {
                                        review.backend.borrow_mut().reload_and_notify();
                                        Step::Done(plan)
                                    }
                                    Err(e) => Step::Failed(e.to_string()),
                                });
                            },
                            {tx::import_apply(changes)}
                        }
                        button { class: "verify-btn plain", onclick: move |_| step.set(Step::Pick), {tx::IMPORT_CANCEL} }
                    }
                }
            }
        }
        Step::Done(plan) => rsx! {
            p { role: "status", {tx::import_done(plan.changes())} }
            if plan.failed() > 0 {
                p { class: "verify-error", role: "alert", {tx::import_partial(plan.failed())} }
            }
            button { class: "verify-btn plain", onclick: move |_| step.set(Step::Pick), {tx::IMPORT_AGAIN} }
        },
        Step::Failed(e) => rsx! {
            p { class: "verify-error", role: "alert", "{tx::IMPORT_FAILED} " bdi { dir: "ltr", "{e}" } }
            button { class: "verify-btn plain", onclick: move |_| step.set(Step::Pick), {tx::IMPORT_BACK} }
        },
    };

    rsx! {
        crate::ui::Card { class: "",
            h2 { {tx::IMPORT_TITLE} }
            {body}
        }
    }
}

#[component]
fn PlanTable(plan: ImportPlan) -> Element {
    rsx! {
        table { class: "plan",
            thead {
                tr {
                    th { {tx::COL_WHAT} }
                    th { {tx::COL_IN_FILE} }
                    th { {tx::COL_NEW} }
                    th { {tx::COL_NEWER} }
                }
            }
            tbody {
                for t in plan.tables.iter() {
                    tr { key: "{t.name}",
                        td { {table_label(&t.name)} }
                        td { "{t.rows}" }
                        td { "{t.new}" }
                        td { "{t.newer}" }
                    }
                }
            }
        }
    }
}

fn table_label(name: &str) -> String {
    match name {
        "review_events" => tx::T_REVIEWS.to_string(),
        "item_status" => tx::T_CHECKS.to_string(),
        "settings" => tx::T_SETTINGS.to_string(),
        "own_cards" => tx::T_OWN.to_string(),
        other => other.replace('_', " "),
    }
}

#[cfg(test)]
mod tests {
    use batna_core::review::{Grade, Origin, ReviewEvent};

    use super::*;

    /// A phone's backup with two answers, as the export writes it.
    fn backup() -> String {
        let phone = store::Store::open_in_memory().unwrap();
        for (item, at_ms) in [("hb.L01.001", 10), ("hb.L01.002", 20)] {
            let e = ReviewEvent {
                item_id: item.into(),
                grade: Grade::Good,
                origin: Origin::Review,
                at_ms,
                took_ms: 3000,
            };
            phone.append_review(e).unwrap();
        }
        serde_json::to_string(&phone.export("0.1.0", 2).unwrap().document).unwrap()
    }

    #[test]
    fn only_batna_exports_are_offered_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "batna-export-2026-01-01.json",
            "batna-export-2026-03-01.json",
            "notes.json",
            "batna-export-x.txt",
        ] {
            std::fs::write(dir.path().join(name), "{}").unwrap();
        }
        let names: Vec<String> = found_in(dir.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["batna-export-2026-03-01.json", "batna-export-2026-01-01.json"]);
        assert!(found_in(&dir.path().join("missing")).is_empty());
    }

    #[test]
    fn a_file_is_read_planned_or_refused_with_a_reason() {
        let tablet = store::Store::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("good.json");
        std::fs::write(&good, backup()).unwrap();
        let (_, plan) = plan_file(&good, &tablet).unwrap();
        assert_eq!(plan.table("review_events").unwrap().new, 2);
        assert_eq!(plan.changes(), 2);

        assert!(plan_file(&dir.path().join("absent.json"), &tablet).is_err());
        let junk = dir.path().join("junk.json");
        std::fs::write(&junk, "not json").unwrap();
        assert!(plan_file(&junk, &tablet).is_err());
        // Far too big to be a backup: refused before reading.
        let big = dir.path().join("big.json");
        std::fs::File::create(&big).unwrap().set_len(MAX_IMPORT_BYTES + 1).unwrap();
        assert_eq!(plan_file(&big, &tablet).unwrap_err(), tx::IMPORT_TOO_BIG);
    }

    #[test]
    fn pasted_text_previews_or_fails() {
        let tablet = store::Store::open_in_memory().unwrap();
        match plan_text("pasted".into(), &backup(), &tablet) {
            Step::Preview { name, plan, .. } => {
                assert_eq!(name, "pasted");
                assert_eq!(plan.changes(), 2);
            }
            _ => panic!("expected a preview"),
        }
        assert!(matches!(plan_text("x".into(), "{", &tablet), Step::Failed(_)));
    }

    fn render(start: Step) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct P {
            start: Step,
        }
        fn app(p: P) -> Element {
            crate::store::use_provide_review();
            rsx! { ImportFlow { start: p.start.clone() } }
        }
        crate::store::use_memory_store();
        let mut dom = VirtualDom::new_with_props(app, P { start });
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn each_step_of_the_restore_card_says_what_happens() {
        let page = render(Step::Pick);
        assert!(page.contains(tx::IMPORT_TITLE) && page.contains(tx::IMPORT_PASTE), "{page}");
        assert!(page.contains("type=\"file\"") && page.contains(tx::IMPORT_READ), "{page}");

        let tablet = store::Store::open_in_memory().unwrap();
        let Step::Preview { doc, plan, .. } = plan_text("batna-export-1.json".into(), &backup(), &tablet)
        else {
            panic!("a preview")
        };
        let page = render(Step::Preview {
            name: "batna-export-1.json".into(),
            doc: doc.clone(),
            plan: plan.clone(),
        });
        for text in
            [tx::IMPORT_FILE, tx::IMPORT_FROM, tx::T_REVIEWS, tx::COL_NEW, tx::IMPORT_SAFE, tx::IMPORT_CANCEL]
        {
            assert!(page.contains(text), "missing {text}: {page}");
        }
        assert!(page.contains("batna-export-1.json") && page.contains(&tx::import_apply(2)), "{page}");
        assert!(page.contains(&plan.device_id));

        // Nothing new in the file: nothing to merge.
        let mut same = plan.clone();
        for t in &mut same.tables {
            t.new = 0;
            t.newer = 0;
        }
        let page = render(Step::Preview { name: "x".into(), doc, plan: same.clone() });
        assert!(page.contains(tx::IMPORT_NOTHING) && page.contains(tx::IMPORT_BACK), "{page}");

        let mut partial = plan.clone();
        partial.tables[0].failed = 3;
        let page = render(Step::Done(partial));
        assert!(page.contains(&tx::import_done(2)) && page.contains(&tx::import_partial(3)), "{page}");
        assert!(page.contains(tx::IMPORT_AGAIN));
        assert!(!render(Step::Done(plan)).contains("role=\"alert\""), "all merged: no warning");

        let page = render(Step::Failed("bad file".into()));
        assert!(page.contains(tx::IMPORT_FAILED) && page.contains("bad file"), "{page}");
    }

    #[test]
    fn table_names_read_as_words() {
        assert_eq!(table_label("review_events"), tx::T_REVIEWS);
        assert_eq!(table_label("item_status"), tx::T_CHECKS);
        assert_eq!(table_label("settings"), tx::T_SETTINGS);
        assert_eq!(table_label("own_cards"), tx::T_OWN);
        assert_eq!(table_label("bank_sessions"), "bank sessions");
    }

    #[test]
    fn pasting_a_backup_previews_it_and_cancel_goes_back() {
        use crate::test_dom::TestDom;
        fn app() -> Element {
            crate::store::use_provide_review();
            rsx! { ImportFlow { start: Step::Pick } }
        }
        crate::store::use_memory_store();
        let mut t = TestDom::of(app);
        assert!(t.html().contains("disabled=true"), "nothing pasted, nothing to read");
        t.input("class=\"export-text\"", "{ not json");
        t.click(tx::IMPORT_READ);
        assert!(t.html().contains(tx::IMPORT_FAILED), "{}", t.html());
        t.click(tx::IMPORT_BACK);
        assert!(t.html().contains(tx::IMPORT_PASTE));

        t.input("class=\"export-text\"", &backup());
        t.click(tx::IMPORT_READ);
        let page = t.html();
        assert!(page.contains(tx::IMPORT_PASTED) && page.contains(&tx::import_apply(2)), "{page}");
        t.click(tx::IMPORT_CANCEL);
        assert!(t.html().contains(tx::IMPORT_PASTE) && !t.html().contains(tx::IMPORT_SAFE));
    }

    #[test]
    fn after_a_merge_or_a_failure_the_card_starts_over() {
        use crate::test_dom::TestDom;
        #[derive(Props, Clone, PartialEq)]
        struct P {
            start: Step,
        }
        fn app(p: P) -> Element {
            crate::store::use_provide_review();
            rsx! { ImportFlow { start: p.start.clone() } }
        }
        crate::store::use_memory_store();
        let tablet = store::Store::open_in_memory().unwrap();
        let Step::Preview { doc, plan, .. } = plan_text("b.json".into(), &backup(), &tablet) else {
            panic!()
        };
        let mut nothing = plan.clone();
        for t in &mut nothing.tables {
            t.new = 0;
            t.newer = 0;
        }
        for (start, button) in [
            (Step::Done(plan), tx::IMPORT_AGAIN),
            (Step::Failed("x".into()), tx::IMPORT_BACK),
            (Step::Preview { name: "b.json".into(), doc, plan: nothing }, tx::IMPORT_BACK),
        ] {
            let mut t = TestDom::new(VirtualDom::new_with_props(app, P { start }));
            t.click(button);
            assert!(t.html().contains(tx::IMPORT_PASTE), "{button}: {}", t.html());
        }
    }
}
