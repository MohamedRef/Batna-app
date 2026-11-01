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
    // Mixed Arabic and English (2026-11-01): "label: value" bullets, "الـ"
    // before English words, full stops after English, and arrows between
    // terms, plus a figure whose label once ran off its edge.
    for (lesson, at) in [("hb.L01.lesson2", 5), ("hb.L01.lesson2", 8)] {
        let (_, l) = Library::get().lesson(lesson).expect("mixed-text lesson");
        let step = &l.steps[at];
        let case = Case { lesson: lesson.into(), at, view: View::Steps };
        let mut dom = VirtualDom::new_with_props(root, case);
        dom.rebuild_in_place();
        let file = format!("lesson-mixed-{lesson}-{}-steps.html", at + 1);
        write_page(&dir, &file, &dom);
        manifest.push(serde_json::json!({
            "file": file, "screen": "learn", "kind": step.body.kind(), "view": "steps",
            "lesson": lesson, "step": at + 1, "steps": l.steps.len(),
            "question": step.body.is_question(),
        }));
    }
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

/// Every drawn figure in every lesson, at the lesson's own figure styles, for
/// the browser check that no label runs past its picture's edge
/// (tests/screens-browser.cjs `figureChecks`).
#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_figures() {
    let dir = out_dir("figures");
    let mut body = String::new();
    let mut count = 0;
    for (_, lesson) in Library::get().all_lessons() {
        for (at, step) in lesson.steps.iter().enumerate() {
            if let batna_core::lesson::StepBody::Figure { svg, image: None, .. } = &step.body {
                count += 1;
                body.push_str(&format!(
                    "<div class=\"ls-fig-svg\" dir=\"ltr\" data-at=\"{} step {}\">{svg}</div>\n",
                    lesson.id,
                    at + 1
                ));
            }
        }
    }
    assert!(count > 100, "the lessons have their figures: {count}");
    let page = format!(
        "<!doctype html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"utf-8\">\
         <link rel=\"stylesheet\" href=\"app.css\">\
         <link id=\"prefs\" rel=\"stylesheet\" href=\"prefs-system-normal.css\">\
         <style>{}</style></head><body><div class=\"app\" lang=\"ar\" dir=\"rtl\">{body}</div></body></html>",
        include_str!("learn.css")
    );
    std::fs::write(dir.join("figures.html"), page).unwrap();
}
