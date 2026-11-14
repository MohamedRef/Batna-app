//! Browser fixtures for the production lesson player: the first step of each
//! kind in book order, in Steps and in Pages, opened at that step from a saved
//! attempt the way the app reopens a lesson (tests/screens-browser.cjs).

use super::*;
use crate::browser_fixture::{Shell, out_dir, out_dir_in, write_page};
use batna_core::lesson::StepBody;
use batna_core::response::{Attempt, Draft, sig};

#[derive(Props, Clone, PartialEq)]
struct Case {
    lesson: String,
    at: usize,
    view: View,
    /// A response already on the step (sort: some cards placed, or checked).
    draft: Option<(Draft, Option<bool>)>,
    /// The step's abbreviations sheet is open.
    #[props(default)]
    abbr: bool,
}

fn root(case: Case) -> Element {
    use_hook(|| {
        let (_, lesson) = Library::get().lesson(&case.lesson).expect("lesson");
        let mut attempt = Attempt { at: case.at, ..Attempt::new(lesson.steps.len(), crate::store::now_ms()) };
        if let Some((draft, answer)) = case.draft.clone() {
            attempt.put(case.at, sig(&lesson.steps[case.at].body), draft);
            attempt.answers[case.at] = answer;
        }
        crate::store::shared().put_place(&player::pos_key(&case.lesson), &attempt).unwrap();
        *OPEN.write() = Some(case.lesson.clone());
        *VIEW.write() = case.view;
        if case.abbr {
            let texts = lesson.steps[case.at].shown_text();
            let course = batna_core::program::course_of(&case.lesson);
            let terms = crate::abbrsheet::ids(&texts, Some(course));
            *crate::abbrsheet::OPEN.write() = Some(terms);
        }
    });
    rsx! {
        Shell { screen: Screen::Learn, focus: true, Learn {} }
    }
}

#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_learn() {
    export_learn(&out_dir("learn"));
}

/// The fixtures build on every test run too (into a temporary folder), so
/// a lesson or player change that breaks one fails here, not only in the
/// browser job: every page is the player at its step, and every kind and
/// mixed-text case is in the manifest.
#[test]
fn screen_fixtures_render_every_step_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = out_dir_in(tmp.path(), "learn");
    export_learn(&dir);
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    let kinds: std::collections::BTreeSet<&str> =
        manifest.iter().filter_map(|m| m["kind"].as_str()).collect();
    assert_eq!(kinds.len(), 13, "{kinds:?}");
    for m in &manifest {
        let file = m["file"].as_str().unwrap();
        let html = std::fs::read_to_string(dir.join(file)).unwrap();
        assert!(html.starts_with("<!doctype html>"), "{file}");
        if m["screen"] == "learn" {
            let (step, steps) = (m["step"].as_u64().unwrap(), m["steps"].as_u64().unwrap());
            assert!(html.contains("class=\"ls-player\""), "{file} is the player");
            assert!(html.contains(&format!(">{step}/{steps}<")), "{file} opens at its step");
            assert!(html.contains(&format!("data-view=\"{}\"", m["view"].as_str().unwrap())), "{file}");
        }
    }
    // The sort and case states are the ones asked for.
    let sort = std::fs::read_to_string(dir.join("lesson-sort-checked-steps.html")).unwrap();
    assert!(sort.contains("ls-chiplet-done") && sort.contains("data-state=\"wrong\""));
    let case = std::fs::read_to_string(dir.join("lesson-case-done-steps.html")).unwrap();
    assert!(case.contains("ls-retry") && case.contains("ls-why-box"));
    let abbr = std::fs::read_to_string(dir.join("lesson-mixed-hb.L01.lesson2-9-abbr-steps.html")).unwrap();
    assert!(
        abbr.len()
            > std::fs::read_to_string(dir.join("lesson-mixed-hb.L01.lesson2-9-steps.html")).unwrap().len()
    );
    let remind = std::fs::read_to_string(dir.join("lesson-mixed-hb.L01.lesson8-8-steps.html")).unwrap();
    assert!(remind.contains("ls-remind") && remind.contains("SHIBS") && remind.contains("ls-remind-from"));
    assert!(dir.join("abbr-dictionary.html").exists() && dir.join("app.css").exists());
}

fn export_learn(dir: &std::path::Path) {
    crate::store::use_memory_store();
    let dir = dir.to_path_buf();
    let mut kinds = std::collections::BTreeSet::new();
    let mut manifest = Vec::new();
    for (lesson, at) in first_of_kinds() {
        let step = &lesson.steps[at];
        {
            let kind = step.body.kind();
            kinds.insert(kind);
            for view in [View::Steps, View::Pages] {
                let case = Case { lesson: lesson.id.clone(), at, view, draft: None, abbr: false };
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
    case_states(&dir, &mut manifest);
    assert_eq!(kinds.len(), 13, "every lesson step kind has a fixture: {kinds:?}");
    // Mixed Arabic and English (2026-11-01): "label: value" bullets, "الـ"
    // before English words, full stops after English, and arrows between
    // terms, plus a figure whose label once ran off its edge.
    // The last one has the step's abbreviations sheet open (2026-11-01).
    for (lesson, at, abbr) in MIXED {
        let (_, l) = Library::get().lesson(lesson).expect("mixed-text lesson");
        let step = &l.steps[at];
        let case = Case { lesson: lesson.into(), at, view: View::Steps, draft: None, abbr };
        let mut dom = VirtualDom::new_with_props(root, case);
        dom.rebuild_in_place();
        let open = if abbr { "-abbr" } else { "" };
        let file = format!("lesson-mixed-{lesson}-{}{open}-steps.html", at + 1);
        write_page(&dir, &file, &dom);
        manifest.push(serde_json::json!({
            "file": file, "screen": "learn", "kind": step.body.kind(), "view": "steps",
            "lesson": lesson, "step": at + 1, "steps": l.steps.len(),
            "question": step.body.is_question(),
        }));
    }
    dictionary(&dir, &mut manifest);
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

/// The abbreviations dictionary (abbrdict/mod.rs) at the hepatology course:
/// a page, not a lesson, so the browser job checks only the shared layout
/// rules for it.
fn dictionary(dir: &std::path::Path, manifest: &mut Vec<serde_json::Value>) {
    fn page() -> Element {
        use_hook(|| *crate::abbrdict::COURSE.write() = Some("hb".into()));
        rsx! {
            Shell { screen: Screen::Learn, focus: false, crate::abbrdict::Dictionary {} }
        }
    }
    let mut dom = VirtualDom::new(page);
    dom.rebuild_in_place();
    write_page(dir, "abbr-dictionary.html", &dom);
    manifest.push(serde_json::json!({ "file": "abbr-dictionary.html", "screen": "abbr-dictionary", "phase": "dictionary" }));
}

type Pick = (&'static batna_core::lesson::Lesson, usize, &'static batna_core::lesson::Step);

/// The sort step with the longest bin and card names.
fn sort_pick() -> Pick {
    let longest = |b: &StepBody| match b {
        StepBody::Sort { bins, cards, .. } => {
            bins.iter().chain(cards.iter().map(|c| &c.text)).map(|t| t.len()).max().unwrap_or(0)
        }
        _ => 0,
    };
    Library::get()
        .all_lessons()
        .flat_map(|(_, l)| l.steps.iter().enumerate().map(move |(at, s)| (l, at, s)))
        .filter(|(_, _, s)| s.body.kind() == "sort")
        .max_by_key(|(_, _, s)| longest(&s.body))
        .expect("a sort step")
}

/// The first case with a later scenario.
fn case_pick() -> Pick {
    Library::get()
        .all_lessons()
        .flat_map(|(_, l)| l.steps.iter().enumerate().map(move |(at, s)| (l, at, s)))
        .find(|(_, _, s)| match &s.body {
            StepBody::Case { steps, .. } => {
                steps.len() > 1 && steps[1].reveal.is_some() && !steps[0].options.is_empty()
            }
            _ => false,
        })
        .expect("a case with a later scenario")
}

/// The mixed Arabic and English steps (lesson, step, abbreviations open).
/// The last one is the ACLF step with its reminders (2026-11-08): two
/// تحشيشات of this lesson and SHIBS from lesson 2, linked back.
const MIXED: [(&str, usize, bool); 6] = [
    ("hb.L01.lesson2", 5, false),
    ("hb.L01.lesson2", 7, false),
    ("hb.L01.lesson2", 8, false),
    ("hb.L01.lesson2", 8, true),
    ("hb.L01.lesson8", 0, false),
    ("hb.L01.lesson8", 7, false),
];

/// The first lesson with each step kind, in book order.
fn first_of_kinds() -> Vec<(&'static batna_core::lesson::Lesson, usize)> {
    let mut kinds = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for (_, lesson) in Library::get().all_lessons() {
        for (at, step) in lesson.steps.iter().enumerate() {
            if kinds.insert(step.body.kind()) {
                out.push((lesson, at));
            }
        }
    }
    out
}

/// Every lesson these fixtures open (each saves its position in the shared
/// test store), so the player tests leave them alone.
pub(super) fn fixture_lessons() -> std::collections::HashSet<String> {
    first_of_kinds()
        .into_iter()
        .map(|(l, _)| l.id.clone())
        .chain(MIXED.iter().map(|(l, _, _)| l.to_string()))
        .chain([sort_pick().0.id.clone(), case_pick().0.id.clone()])
        .collect()
}

/// The sort step part-way through and after Check, on the step with the
/// longest bin and card names (portal hypertension, #-reported clipping).
fn sort_states(dir: &std::path::Path, manifest: &mut Vec<serde_json::Value>) {
    let (lesson, at, step) = sort_pick();
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
        let case = Case {
            lesson: lesson.id.clone(),
            at,
            view: View::Steps,
            draft: Some((draft, answer)),
            abbr: false,
        };
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

/// A mini case after part 1 (checked wrong, part 2 open with its scenario)
/// and once every part is checked: each part's ↻, the scenario box and the
/// explanation card (Mohamed, 2026-11-01).
fn case_states(dir: &std::path::Path, manifest: &mut Vec<serde_json::Value>) {
    let (lesson, at, step) = case_pick();
    let StepBody::Case { steps, .. } = &step.body else { unreachable!() };
    let n = steps.len();
    let wrong = |i: usize| steps[i].answer.map(|a| (a + 1) % steps[i].options.len().max(1));
    let mut part1 = vec![None; n];
    part1[0] = wrong(0);
    let all: Vec<_> =
        (0..n).map(|i| if steps[i].options.is_empty() { Some(0) } else { steps[i].answer }).collect();
    let mut first = all.clone();
    first[0] = wrong(0);
    let states = [
        ("part1", Draft::Case { picks: part1.clone(), at: 1, pick: None, first: part1 }, None),
        ("done", Draft::Case { picks: all, at: n - 1, pick: None, first }, Some(false)),
    ];
    for (name, draft, answer) in states {
        let case = Case {
            lesson: lesson.id.clone(),
            at,
            view: View::Steps,
            draft: Some((draft, answer)),
            abbr: false,
        };
        let mut dom = VirtualDom::new_with_props(root, case);
        dom.rebuild_in_place();
        let file = format!("lesson-case-{name}-steps.html");
        write_page(dir, &file, &dom);
        manifest.push(serde_json::json!({
            "file": file, "screen": "learn", "kind": "case", "view": "steps",
            "lesson": lesson.id, "step": at + 1, "steps": lesson.steps.len(),
            "question": answer.is_none(),
        }));
    }
}

/// Every drawn figure in every lesson, at the lesson's own figure styles, for
/// the browser check that no label runs past its picture's edge
/// (tests/screens-browser.cjs `figureChecks`).
#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_figures() {
    export_figures(&out_dir("figures"));
}

#[test]
fn every_figure_is_in_the_figures_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = out_dir_in(tmp.path(), "figures");
    let count = export_figures(&dir);
    let page = std::fs::read_to_string(dir.join("figures.html")).unwrap();
    assert_eq!(page.matches("class=\"ls-fig-svg\"").count(), count);
    assert!(page.contains("data-at=\"hb.L01."), "figures name their lesson and step");
}

/// Writes `figures.html`; returns how many figures it holds.
fn export_figures(dir: &std::path::Path) -> usize {
    let dir = dir.to_path_buf();
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
    count
}

/// A book picture with one mark of each kind (#984; no lesson carries marks
/// yet): marks shown, marks hidden before a question, and open in the
/// full-screen viewer, for the browser check that each mark lands on its
/// spot at every width, in both directions, and when zoomed
/// (tests/marks-browser.cjs).
#[test]
#[ignore = "exports production HTML for the screens browser CI job"]
fn export_screen_fixtures_marks() {
    export_marks(&out_dir("marks"));
}

#[test]
fn the_marks_fixtures_draw_every_mark() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = out_dir_in(tmp.path(), "marks");
    export_marks(&dir);
    let n = super::steps::marks_fixture::marks().len();
    let on = std::fs::read_to_string(dir.join("figure-marks-on.html")).unwrap();
    assert_eq!(on.matches("class=\"ls-mk\"").count(), n);
    assert!(on.contains("aria-pressed=\"true\"") && on.contains("ls-marks-list"));
    let off = std::fs::read_to_string(dir.join("figure-marks-off.html")).unwrap();
    assert!(!off.contains("class=\"ls-mk\"") && off.contains("aria-pressed=\"false\""));
    let zoomed = std::fs::read_to_string(dir.join("figure-marks-zoomed.html")).unwrap();
    assert!(zoomed.contains("ls-viewer") && zoomed.matches("class=\"ls-mk\"").count() == 2 * n);
    let manifest: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.len(), 3);
    assert_eq!(manifest[0]["marks"].as_array().unwrap().len(), n);
}

#[derive(Props, Clone, PartialEq)]
struct MarksState {
    image: batna_core::lesson::Picture,
    shown: bool,
    zoomed: bool,
}

fn marks_page(s: MarksState) -> Element {
    use super::steps::marks_fixture::MarkedFigure;
    rsx! {
        Shell { screen: Screen::Learn, focus: true,
            style { {CSS.text()} }
            div { class: "ls-player", "data-view": "steps",
                h1 { class: "ls-sr-only", "Marks" }
                MarkedFigure { image: s.image, shown: s.shown, zoomed: s.zoomed }
            }
        }
    }
}

fn export_marks(dir: &std::path::Path) {
    use super::steps::marks_fixture::marks;
    // The first book picture in the lessons, with its data embedded.
    let image = Library::get()
        .all_lessons()
        .flat_map(|(_, l)| l.steps.iter())
        .find_map(|s| match &s.body {
            StepBody::Figure { image: Some(p), .. } if p.book && !p.data.is_empty() => Some(p.clone()),
            _ => None,
        })
        .expect("a book picture in the lessons");
    let at: Vec<_> = marks()
        .iter()
        .map(|m| serde_json::json!({ "at": m.at, "kind": m.kind.as_str(), "size": m.size }))
        .collect();
    let mut manifest = Vec::new();
    for (name, shown, zoomed) in [("on", true, false), ("off", false, false), ("zoomed", true, true)] {
        let mut dom =
            VirtualDom::new_with_props(marks_page, MarksState { image: image.clone(), shown, zoomed });
        dom.rebuild_in_place();
        let file = format!("figure-marks-{name}.html");
        write_page(dir, &file, &dom);
        manifest.push(serde_json::json!({ "file": file, "shown": shown, "zoomed": zoomed, "marks": at }));
    }
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}
