//! Browser fixtures for the production study screen: setup, a running session
//! on the first item of each kind the player shows (fresh, on an
//! empty review log), a card reopened by Undo,
//! and the summary (tests/screens-browser.cjs).

use super::*;
use crate::Screen;
use crate::browser_fixture::{Shell, out_dir, write_page};
use batna_core::content::Body;
use batna_core::review::StudyStoreHandle;

#[derive(Props, Clone, PartialEq)]
struct Case {
    phase: &'static str,
    item: Option<String>,
    reopened: bool,
}

fn root(case: Case) -> Element {
    // The app's own review backend, over the in-memory shared store.
    crate::store::use_provide_review();
    let store = use_context::<StudyStoreHandle>();
    use_hook(|| {
        let ids: Vec<String> = case.item.iter().cloned().collect();
        *SESSION.write() = Session::new(plan_items(&store, &ids));
        *PHASE.write() = match case.phase {
            "running" => Phase::Running,
            "summary" => Phase::Summary,
            _ => Phase::Setup,
        };
        if case.reopened {
            *UNDO_NOTE.write() = Some(tx::UNDONE);
        }
    });
    rsx! {
        Shell { screen: Screen::Study, focus: case.phase == "running", Study {} }
    }
}

/// Every state the browser job checks, by name.
fn cases() -> Vec<(String, Case)> {
    let mut cases = vec![("setup".to_string(), Case { phase: "setup", item: None, reopened: false })];
    for kind in item::PLAYABLE {
        let first = ready_lectures().into_iter().flat_map(|l| &l.items).find(|i| i.body.kind() == *kind);
        let Some(first) = first else { panic!("no {kind} item to show") };
        cases.push((
            format!("item-{kind}"),
            Case { phase: "running", item: Some(first.id.clone()), reopened: false },
        ));
        if *kind == "card" {
            let case = Case { phase: "running", item: Some(first.id.clone()), reopened: true };
            cases.push(("item-card-reopened".to_string(), case));
        }
    }
    // An enumerate list with its points open and its تحشيشة under them.
    let hooked = ready_lectures()
        .into_iter()
        .flat_map(|l| &l.items)
        .find(|i| matches!(&i.body, Body::Enumerate { hook: Some(_), .. }))
        .expect("an enumerate item with a hook");
    let open = Case { phase: "running", item: Some(hooked.id.clone()), reopened: false };
    cases.push(("item-enumerate-open".to_string(), open));
    let first = cases[1].1.item.clone();
    cases.push(("summary".to_string(), Case { phase: "summary", item: first, reopened: false }));
    cases
}

/// One state, rendered.
fn render(state: &str, case: Case) -> VirtualDom {
    let opened = state == "item-enumerate-open";
    enumerate::OPEN_FOR_FIXTURE.set(opened);
    let mut dom = VirtualDom::new_with_props(root, case);
    dom.rebuild_in_place();
    dom
}

#[test]
fn every_fixture_state_shows_what_its_name_says() {
    crate::store::use_memory_store();
    let all = cases();
    assert_eq!(all.len(), item::PLAYABLE.len() + 4, "setup, kinds, reopened, open list, summary");
    for (state, case) in all {
        let reopened = case.reopened;
        let kind = case.item.as_deref().and_then(find).map(|(_, i)| i.body.kind());
        let html = dioxus::ssr::render(&render(&state, case));
        match state.as_str() {
            "setup" => assert!(html.contains(tx::LECTURES), "{state}"),
            "summary" => assert!(html.contains("st-summary"), "{state}"),
            _ => {
                assert!(html.contains(&format!("data-kind=\"{}\"", kind.unwrap())), "{state}");
                assert!(html.contains("data-focus=\"true\""), "a running session is in focus mode");
            }
        }
        if reopened {
            assert!(html.contains(tx::UNDONE) && html.contains("st-answer"), "{state}: answer shown");
        }
        if state == "item-enumerate-open" {
            assert!(html.contains("st-hook") && html.contains(tx::ENUM_HOOK), "the تحشيشة shows");
        }
    }
    enumerate::OPEN_FOR_FIXTURE.set(false);
}

#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_study() {
    crate::store::use_memory_store();
    let dir = out_dir("study");
    let mut manifest = Vec::new();
    for (state, case) in cases() {
        let (phase, item) = (case.phase, case.item.clone());
        let dom = render(&state, case);
        let file = format!("study-{state}.html");
        write_page(&dir, &file, &dom);
        let kind = item.as_deref().and_then(find).map(|(_, i)| i.body.kind());
        manifest.push(serde_json::json!({
            "file": file, "screen": "study", "state": state, "phase": phase, "item": item, "kind": kind,
        }));
    }
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}
