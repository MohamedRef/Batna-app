//! Browser fixtures for the production lesson player: the first step of each
//! kind in book order, in Steps and in Pages, opened at that step from a saved
//! attempt the way the app reopens a lesson (tests/screens-browser.cjs).

use super::*;
use crate::browser_fixture::{Shell, out_dir, write_page};
use batna_core::response::Attempt;

#[derive(Props, Clone, PartialEq)]
struct Case {
    lesson: String,
    at: usize,
    view: View,
}

fn root(case: Case) -> Element {
    use_hook(|| {
        let (_, lesson) = Library::get().lesson(&case.lesson).expect("lesson");
        let attempt = Attempt { at: case.at, ..Attempt::new(lesson.steps.len(), crate::store::now_ms()) };
        crate::store::shared().put_local_setting(&player::pos_key(&case.lesson), &attempt).unwrap();
        *OPEN.write() = Some(case.lesson.clone());
        *VIEW.write() = case.view;
    });
    rsx! {
        Shell { screen: Screen::Learn, focus: true, Learn {} }
    }
}

#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_learn() {
    crate::store::use_memory_store();
    let dir = out_dir("learn");
    let mut kinds = std::collections::BTreeSet::new();
    let mut manifest = Vec::new();
    for (_, lesson) in Library::get().all_lessons() {
        for (at, step) in lesson.steps.iter().enumerate() {
            let kind = step.body.kind();
            if !kinds.insert(kind) {
                continue;
            }
            for view in [View::Steps, View::Pages] {
                let case = Case { lesson: lesson.id.clone(), at, view };
                let mut dom = VirtualDom::new_with_props(root, case);
                dom.rebuild_in_place();
                let view = if view == View::Pages { "pages" } else { "steps" };
                let file = format!("lesson-{kind}-{view}.html");
                write_page(&dir, &file, &dom);
                manifest.push(serde_json::json!({
                    "file": file, "screen": "learn", "kind": kind, "view": view,
                    "lesson": lesson.id, "step": at + 1, "steps": lesson.steps.len(),
                    "question": step.body.is_question(),
                }));
            }
        }
    }
    assert_eq!(kinds.len(), 13, "every lesson step kind has a fixture: {kinds:?}");
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}
