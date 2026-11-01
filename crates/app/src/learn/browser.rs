//! Browser fixtures for the production lesson player: the first step of each
//! kind in book order, in Steps and in Pages, opened at that step from a saved
//! attempt the way the app reopens a lesson (tests/screens-browser.cjs).

use super::*;
use crate::browser_fixture::{Shell, out_dir, write_page};
use batna_core::lesson::StepBody;
use batna_core::response::{Attempt, Draft, sig};

#[derive(Props, Clone, PartialEq)]
struct Case {
    lesson: String,
    at: usize,
    view: View,
    /// A response already on the step (sort: some cards placed, or checked).
    draft: Option<(Draft, Option<bool>)>,
}

fn root(case: Case) -> Element {
    use_hook(|| {
        let (_, lesson) = Library::get().lesson(&case.lesson).expect("lesson");
        let mut attempt = Attempt { at: case.at, ..Attempt::new(lesson.steps.len(), crate::store::now_ms()) };
        if let Some((draft, answer)) = case.draft.clone() {
            attempt.put(case.at, sig(&lesson.steps[case.at].body), draft);
            attempt.answers[case.at] = answer;
        }
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
                let case = Case { lesson: lesson.id.clone(), at, view, draft: None };
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
    sort_states(&dir, &mut manifest);
    assert_eq!(kinds.len(), 13, "every lesson step kind has a fixture: {kinds:?}");
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

/// The sort step part-way through and after Check, on the step with the
/// longest bin and card names (portal hypertension, #-reported clipping).
fn sort_states(dir: &std::path::Path, manifest: &mut Vec<serde_json::Value>) {
    let longest = |b: &StepBody| match b {
        StepBody::Sort { bins, cards, .. } => {
            bins.iter().chain(cards.iter().map(|c| &c.text)).map(|t| t.len()).max().unwrap_or(0)
        }
        _ => 0,
    };
    let (lesson, at, step) = Library::get()
        .all_lessons()
        .flat_map(|(_, l)| l.steps.iter().enumerate().map(move |(at, s)| (l, at, s)))
        .filter(|(_, _, s)| s.body.kind() == "sort")
        .max_by_key(|(_, _, s)| longest(&s.body))
        .expect("a sort step");
    let StepBody::Sort { bins, cards, .. } = &step.body else { unreachable!() };
    // Half the cards placed; then all placed with the first one wrong.
    let half: Vec<_> = cards.iter().enumerate().map(|(i, c)| (i % 2 == 0).then_some(c.bin)).collect();
    let mut done: Vec<_> = cards.iter().map(|c| Some(c.bin)).collect();
    done[0] = Some((cards[0].bin + 1) % bins.len());
    let states = [
        ("placed", Draft::Sort { put: half, checked: false }, None),
        ("checked", Draft::Sort { put: done, checked: true }, Some(false)),
    ];
    for (name, draft, answer) in states {
        let case = Case { lesson: lesson.id.clone(), at, view: View::Steps, draft: Some((draft, answer)) };
        let mut dom = VirtualDom::new_with_props(root, case);
        dom.rebuild_in_place();
        let file = format!("lesson-sort-{name}-steps.html");
        write_page(dir, &file, &dom);
        manifest.push(serde_json::json!({
            "file": file, "screen": "learn", "kind": "sort", "view": "steps",
            "lesson": lesson.id, "step": at + 1, "steps": lesson.steps.len(),
            "question": answer.is_none(),
        }));
    }
}
