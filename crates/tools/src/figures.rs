//! The book's figures in the lessons (Mohamed, 2026-11-14, #983;
//! docs/LESSONS.md rule 10).
//!
//! Every picture and diagram the book prints goes into the lesson that
//! teaches its page, right after the step that refers to it, and is
//! explained there. Each lecture's figures are listed, from the page images
//! in the private sources, in a tools-only file (never in the content pack):
//! `content/book-figures/<course>/LNN.yaml`.
//!
//! ```yaml
//! lecture: hb.L01
//! figures:
//!   - page: 2
//!     label: "Figure 2"          # the book's own number, or what it is
//!     kind: histology
//!     file: hb/L01-macro-micronodular.webp   # shown as the book's crop
//!   - page: 3
//!     label: "Figure 3"
//!     kind: diagram
//!     drawn: true                # redrawn as SVG; its title names the book's figure
//!   - page: 9
//!     label: "publisher logo"
//!     kind: other
//!     skip: "decoration, nothing to teach"
//! ```
//!
//! A lecture with a file has had its figures inventoried, so `validate`
//! fails when one of them is not shown or skipped with a reason. Lectures
//! without one are counted as not done yet (a warning). A diagram,
//! algorithm or chart skipped because the lesson "restates" it in words is
//! warned about too (#1156): its boxes and arrows are what words lose.
//! `batna-tools figures --summary` prints the progress table kept on #983.

use std::collections::BTreeMap;
use std::path::Path;

use batna_core::content::Index;
use batna_core::lesson::{LessonFile, StepBody};
use serde::Deserialize;

/// What a figure is. Photos and the like are always the book's crop.
pub const KINDS: &[&str] = &[
    "photo",
    "histology",
    "imaging",
    "endoscopy",
    "clinical",
    "ecg",
    "film",
    "diagram",
    "algorithm",
    "chart",
    "table",
    "other",
];

/// Kinds that must be shown as the book's own crop, never redrawn.
pub const CROP_ONLY: &[&str] = &["photo", "histology", "imaging", "endoscopy", "clinical", "ecg", "film"];

/// The words a redrawn figure's title carries («شكل 3 في الكتاب، ص 9»).
pub const DRAWN_MARK: &str = "في الكتاب";

/// Kinds whose picture carries structure (boxes, arrows, axes) that words
/// alone lose, so "the lesson restates it" is a weak reason to skip (#1156).
pub const STRUCTURED: &[&str] = &["diagram", "algorithm", "chart"];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub lecture: String,
    #[serde(default)]
    pub figures: Vec<Figure>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Figure {
    pub page: u32,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub drawn: bool,
    #[serde(default)]
    pub skip: Option<String>,
}

impl Figure {
    pub fn skipped(&self) -> bool {
        self.skip.is_some()
    }
}

/// Every inventory file, by lecture id.
pub fn read_all(root: &Path) -> Result<BTreeMap<String, Inventory>, Vec<String>> {
    let dir = root.join("content/book-figures");
    let mut paths = Vec::new();
    for course in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        for f in std::fs::read_dir(course.path()).into_iter().flatten().flatten() {
            if f.path().extension().is_some_and(|e| e == "yaml") {
                paths.push(f.path());
            }
        }
    }
    paths.sort();
    let mut out = BTreeMap::new();
    let mut problems = Vec::new();
    for p in paths {
        match crate::read_yaml::<Inventory>(&p) {
            Ok(inv) => {
                let expect = inv.lecture.split_once('.').map(|(c, n)| format!("{c}/{n}.yaml"));
                let rel = p.strip_prefix(&dir).map(|r| r.to_string_lossy().replace('\\', "/"));
                if expect.as_deref() != rel.as_ref().ok().map(String::as_str) {
                    problems.push(format!(
                        "{}: lecture {} belongs in content/book-figures/{}",
                        p.display(),
                        inv.lecture,
                        expect.unwrap_or_default()
                    ));
                }
                out.insert(inv.lecture.clone(), inv);
            }
            Err(e) => problems.push(e),
        }
    }
    if problems.is_empty() { Ok(out) } else { Err(problems) }
}

/// Figure steps of one lecture: (page, image file if any, title).
fn figure_steps(file: &LessonFile) -> Vec<(u32, Option<String>, String)> {
    file.lessons
        .iter()
        .flat_map(|l| &l.steps)
        .filter_map(|s| match &s.body {
            StepBody::Figure { title, image, .. } => {
                Some((s.page?, image.as_ref().map(|i| i.file.clone()), title.clone().unwrap_or_default()))
            }
            _ => None,
        })
        .collect()
}

/// Book crops a lecture's steps show: (page, file).
fn book_crops(file: &LessonFile) -> Vec<(u32, String)> {
    file.lessons
        .iter()
        .flat_map(|l| &l.steps)
        .filter_map(|s| match &s.body {
            StepBody::Figure { image: Some(i), .. } if i.book => Some((s.page?, i.file.clone())),
            _ => None,
        })
        .collect()
}

/// Warnings (never failures) for one lecture's inventory: a diagram,
/// algorithm or chart skipped because the lesson restates it in words
/// (#1156). Show it as the crop or a redrawn SVG instead, or say in the
/// reason that it is plain text in boxes.
pub fn warnings(inv: &Inventory) -> Vec<String> {
    inv.figures
        .iter()
        .filter(|f| STRUCTURED.contains(&f.kind.as_str()))
        .filter(|f| f.skip.as_deref().is_some_and(|why| why.to_lowercase().contains("restates")))
        .map(|f| {
            format!(
                "{} p{} \"{}\": {} skipped as restated in words; show it (crop or redrawn SVG)",
                inv.lecture, f.page, f.label, f.kind
            )
        })
        .collect()
}

/// Problems with one lecture's inventory.
pub fn problems(inv: &Inventory, pages: [u32; 2], file: &LessonFile) -> Vec<String> {
    let id = &inv.lecture;
    let mut out = Vec::new();
    let steps = figure_steps(file);
    let mut drawn_on: BTreeMap<u32, usize> = BTreeMap::new();
    for f in &inv.figures {
        let at = format!("{id} p{} \"{}\"", f.page, f.label);
        if !(pages[0]..=pages[1]).contains(&f.page) {
            out.push(format!("{at}: page outside the lecture's pages {}–{}", pages[0], pages[1]));
        }
        if f.label.trim().is_empty() {
            out.push(format!("{id} p{}: a figure needs a label", f.page));
        }
        if !KINDS.contains(&f.kind.as_str()) {
            out.push(format!("{at}: kind must be one of {}", KINDS.join(", ")));
        }
        let ways = usize::from(f.file.is_some()) + usize::from(f.drawn) + usize::from(f.skip.is_some());
        if ways != 1 {
            out.push(format!("{at}: give exactly one of file, drawn: true or skip"));
            continue;
        }
        if let Some(why) = &f.skip
            && why.trim().is_empty()
        {
            out.push(format!("{at}: skip needs a reason"));
        }
        if let Some(name) = &f.file
            && !steps.iter().any(|(p, img, _)| *p == f.page && img.as_deref() == Some(name.as_str()))
        {
            out.push(format!("{at}: no figure step on p{} shows {name}", f.page));
        }
        if f.drawn {
            if CROP_ONLY.contains(&f.kind.as_str()) {
                out.push(format!("{at}: a {} is shown as the book's crop, not redrawn", f.kind));
            }
            *drawn_on.entry(f.page).or_default() += 1;
        }
    }
    for (page, n) in drawn_on {
        let have =
            steps.iter().filter(|(p, img, t)| *p == page && img.is_none() && t.contains(DRAWN_MARK)).count();
        if have < n {
            out.push(format!(
                "{id} p{page}: {n} figure(s) listed as drawn but {have} SVG step(s) on that page say «{DRAWN_MARK}» in the title"
            ));
        }
    }
    for (page, crop) in book_crops(file) {
        if !inv.figures.iter().any(|f| f.page == page && f.file.as_deref() == Some(crop.as_str())) {
            out.push(format!("{id} p{page}: the lessons show {crop}; list it in content/book-figures"));
        }
    }
    out
}

/// One course's progress for the #983 table.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CourseRow {
    pub course: String,
    pub lectures: usize,
    pub done: usize,
    pub placed: usize,
    pub skipped: usize,
}

/// What [`scan`] finds across the repo.
#[derive(Debug, Default)]
pub struct Scan {
    pub errors: Vec<String>,
    pub rows: Vec<CourseRow>,
    /// Lectures not inventoried yet.
    pub pending: usize,
    /// [`warnings`] of every inventory.
    pub warnings: Vec<String>,
}

pub fn scan(root: &Path) -> Result<Scan, Vec<String>> {
    let content = root.join("content");
    let index: Index = crate::read_yaml(&content.join("index.yaml")).map_err(|e| vec![e])?;
    let inventories = read_all(root)?;
    let files: BTreeMap<String, LessonFile> =
        crate::lessons::load(root)?.into_iter().map(|(_, f, _)| (f.lecture.clone(), f)).collect();
    let mut errors = Vec::new();
    let mut rows: Vec<CourseRow> = Vec::new();
    let mut pending = 0;
    let mut warned = Vec::new();
    for e in &index.lectures {
        let Some(file) = files.get(&e.id) else { continue };
        let course = batna_core::program::course_of(&e.id).to_owned();
        if rows.last().is_none_or(|r| r.course != course) {
            rows.push(CourseRow { course: course.clone(), ..CourseRow::default() });
        }
        let row = rows.last_mut().expect("pushed above");
        row.lectures += 1;
        match inventories.get(&e.id) {
            Some(inv) => {
                row.done += 1;
                row.skipped += inv.figures.iter().filter(|f| f.skipped()).count();
                row.placed += inv.figures.iter().filter(|f| !f.skipped()).count();
                errors.extend(problems(inv, e.pages, file).into_iter().map(|p| format!("book figures: {p}")));
                warned.extend(warnings(inv));
            }
            None => pending += 1,
        }
    }
    for k in inventories.keys() {
        if !files.contains_key(k) {
            errors.push(format!("book figures: content/book-figures lists {k}, which has no lessons"));
        }
    }
    Ok(Scan { errors, rows, pending, warnings: warned })
}

/// For `validate`: the errors, how many lectures are not inventoried, and
/// how many figures [`warnings`] flags.
pub fn check(root: &Path) -> (Vec<String>, usize, usize) {
    match scan(root) {
        Ok(s) => (s.errors, s.pending, s.warnings.len()),
        Err(e) => (e, 0, 0),
    }
}

/// The progress table (Markdown) kept on #983.
pub fn summary(rows: &[CourseRow]) -> String {
    let mut out =
        String::from("| Course | Lectures done | Figures placed | Figures skipped |\n|---|--:|--:|--:|\n");
    let (mut l, mut d, mut p, mut s) = (0, 0, 0, 0);
    for r in rows {
        out.push_str(&format!(
            "| {} | {} of {} | {} | {} |\n",
            r.course, r.done, r.lectures, r.placed, r.skipped
        ));
        (l, d, p, s) = (l + r.lectures, d + r.done, p + r.placed, s + r.skipped);
    }
    out.push_str(&format!("| **All** | **{d} of {l}** | **{p}** | **{s}** |\n"));
    out
}

/// `batna-tools figures [--summary]`: the table, then each warning.
pub fn run(root: &Path, _args: &[String]) -> Result<String, Vec<String>> {
    let s = scan(root)?;
    if !s.errors.is_empty() {
        return Err(s.errors);
    }
    let mut out = summary(&s.rows);
    if !s.warnings.is_empty() {
        out.push_str(&format!("\n{} warning(s):\n", s.warnings.len()));
        for w in &s.warnings {
            out.push_str(&format!("- {w}\n"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lesson_file(yaml: &str) -> LessonFile {
        serde_yaml_ng::from_str(yaml).unwrap()
    }

    const LESSONS: &str = r#"
schema: batna.lesson/v1
lecture: hb.L01
lessons:
  - id: hb.L01.lesson1
    title: "t"
    hook: "h"
    minutes: 5
    pages: [2, 3]
    topics: [a]
    goals: ["g"]
    recap: ["r"]
    status: unverified
    steps:
      - kind: figure
        page: 2
        title: "t"
        image: { file: hb/L01-a.webp, book: true }
        caption: "c"
        alt: "a"
      - kind: figure
        page: 3
        title: "شكل 3 في الكتاب، ص 3"
        svg: "<svg viewBox='0 0 10 10'></svg>"
        caption: "c"
        alt: "a"
"#;

    fn inv(yaml: &str) -> Inventory {
        serde_yaml_ng::from_str(yaml).unwrap()
    }

    #[test]
    fn a_complete_inventory_passes() {
        let i = inv(r#"
lecture: hb.L01
figures:
  - { page: 2, label: "Figure 2", kind: histology, file: hb/L01-a.webp }
  - { page: 3, label: "Figure 3", kind: diagram, drawn: true }
  - { page: 4, label: "logo", kind: other, skip: "decoration" }
"#);
        assert_eq!(problems(&i, [1, 4], &lesson_file(LESSONS)), Vec::<String>::new());
    }

    #[test]
    fn missing_and_wrong_entries_are_reported() {
        let i = inv(r#"
lecture: hb.L01
figures:
  - { page: 3, label: "Figure 3", kind: histology, drawn: true }
  - { page: 3, label: "Figure 4", kind: diagram, drawn: true }
  - { page: 4, label: "x", kind: blob, skip: " " }
  - { page: 9, label: "y", kind: photo, file: hb/none.webp }
  - { page: 2, label: "z", kind: photo }
"#);
        let p = problems(&i, [1, 4], &lesson_file(LESSONS)).join("\n");
        assert!(p.contains("a histology is shown as the book's crop"), "{p}");
        assert!(p.contains("2 figure(s) listed as drawn but 1"), "{p}");
        assert!(p.contains("kind must be one of"), "{p}");
        assert!(p.contains("skip needs a reason"), "{p}");
        assert!(p.contains("page outside"), "{p}");
        assert!(p.contains("no figure step on p9 shows hb/none.webp"), "{p}");
        assert!(p.contains("exactly one of"), "{p}");
        assert!(p.contains("the lessons show hb/L01-a.webp; list it"), "{p}");
    }

    #[test]
    fn a_structured_figure_skipped_as_restated_warns() {
        let i = inv(r#"
lecture: hb.L01
figures:
  - { page: 2, label: "Figure 2", kind: algorithm, skip: "Restates step text on p2" }
  - { page: 3, label: "Figure 3", kind: chart, skip: "the lesson restates it" }
  - { page: 3, label: "Figure 4", kind: diagram, skip: "plain text in boxes, no arrows or structure" }
  - { page: 4, label: "slide", kind: other, skip: "restates step text" }
  - { page: 4, label: "Figure 5", kind: diagram, drawn: true }
"#);
        let w = warnings(&i);
        assert_eq!(w.len(), 2, "{w:#?}");
        assert!(w[0].starts_with("hb.L01 p2 \"Figure 2\": algorithm skipped as restated"), "{}", w[0]);
        assert!(w[1].contains("p3 \"Figure 3\": chart skipped"), "{}", w[1]);
        // A warning never fails the inventory.
        assert!(problems(&i, [1, 4], &lesson_file(LESSONS)).iter().all(|p| !p.contains("restated")));
    }

    #[test]
    fn summary_totals_the_courses() {
        let rows = vec![
            CourseRow { course: "hb".into(), lectures: 20, done: 2, placed: 9, skipped: 1 },
            CourseRow { course: "hem".into(), lectures: 12, done: 1, placed: 3, skipped: 0 },
        ];
        let s = summary(&rows);
        assert!(s.contains("| hb | 2 of 20 | 9 | 1 |"));
        assert!(s.contains("| **All** | **3 of 32** | **12** | **1** |"));
    }

    #[test]
    fn the_repo_inventories_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let s = scan(&root).unwrap();
        assert_eq!(s.errors, Vec::<String>::new());
        assert!(s.rows.iter().any(|r| r.course == "hb"));
    }
}
