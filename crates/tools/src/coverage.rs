//! Lesson coverage against the book (#833): do a lecture's lessons cover
//! every page, does each lesson's `pages` match the pages its steps cite,
//! does `practice` come from pages the lesson teaches, and how deep is the
//! explanation (docs/LESSONS.md rules 5, 8, 9).
//!
//! `validate` fails on the hard rules and counts the rest as warnings;
//! `batna-tools lessons --summary` prints the course table for the CI
//! summary and `batna-tools lessons --report <course>` one TSV row per
//! lesson, the writer's worklist.
//!
//! Pages with nothing to teach (blank, references, a figure already shown,
//! review questions) are listed with a reason in `content/lesson-skips.yaml`
//! (a tools-only file, never in the pack):
//!
//! ```yaml
//! hem.L03: { 44: "figure already shown on p43" }
//! hem.L01: {}
//! ```
//!
//! A lecture listed there has had its coverage reviewed, so from then on a
//! page that no step cites and that is not skipped is an error, not a
//! warning.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use batna_core::content::{Index, Lecture};
use batna_core::lesson::{Lesson, LessonFile, StepBody};

/// Lessons over these limits get a warning (Mohamed, 2026-11-13, #828).
pub const MAX_CARDS: usize = 30;
pub const MAX_PAGES: u32 = 8;
pub const MAX_TOPICS: usize = 2;

/// Words that give a reason or a mechanism (#827).
const REASON: &[&str] = &[
    "ليه",
    "عشان",
    "علشان",
    "لأن",
    "لان",
    "السبب",
    "بسبب",
    "يعني",
    "فـ",
    "النتيجة",
    "تخيّل",
    "تخيل",
    "إزاي",
    "ازاي",
];

/// `content/lesson-skips.yaml`: lecture → page → why it has no step.
pub type Skips = BTreeMap<String, BTreeMap<u32, String>>;

pub fn read_skips(root: &Path) -> Result<Skips, String> {
    let path = root.join("content/lesson-skips.yaml");
    if !path.exists() {
        return Ok(Skips::new());
    }
    crate::read_yaml(&path)
}

/// One lesson's figures.
#[derive(Debug, Clone, PartialEq)]
pub struct LessonRow {
    pub id: String,
    pub pages: [u32; 2],
    /// First and last page its steps cite.
    pub cited: Option<[u32; 2]>,
    pub topics: usize,
    pub cards: usize,
    pub steps: usize,
    pub words: usize,
    /// `practice` ids whose `src.page` no step of this lesson cites.
    pub practice_uncited: Vec<String>,
    pub explains: usize,
    pub explains_no_reason: usize,
}

/// One lecture's figures.
#[derive(Debug, Clone, PartialEq)]
pub struct LectureRow {
    pub id: String,
    pub pages: [u32; 2],
    /// Pages no step cites and `lesson-skips.yaml` does not skip, with the
    /// number of items from each.
    pub uncited: Vec<(u32, usize)>,
    pub skipped: usize,
    /// Coverage reviewed: listed in `lesson-skips.yaml`.
    pub strict: bool,
    pub lessons: Vec<LessonRow>,
}

impl LectureRow {
    pub fn book_pages(&self) -> u32 {
        self.pages[1] + 1 - self.pages[0]
    }
    pub fn words(&self) -> usize {
        self.lessons.iter().map(|l| l.words).sum()
    }
    /// Lesson words per page of the lecture.
    pub fn words_per_page(&self) -> usize {
        self.words() / self.book_pages().max(1) as usize
    }
    pub fn explains(&self) -> (usize, usize) {
        self.lessons.iter().fold((0, 0), |(a, b), l| (a + l.explains_no_reason, b + l.explains))
    }
}

fn words(s: &str) -> usize {
    s.split_whitespace().count()
}

/// The words a learner reads in a step: its shown text without the SVG.
fn step_words(step: &batna_core::lesson::Step) -> usize {
    match &step.body {
        StepBody::Figure { title, caption, .. } => title.as_deref().map_or(0, words) + words(caption),
        _ => step.shown_text().iter().map(|t| words(t)).sum(),
    }
}

fn has_reason(text: &str) -> bool {
    REASON.iter().any(|w| text.contains(w))
}

fn lesson_row(l: &Lesson, lecture: &Lecture) -> LessonRow {
    let cited: BTreeSet<u32> = l.steps.iter().filter_map(|s| s.page).collect();
    let page_of: BTreeMap<&str, u32> = lecture.items.iter().map(|i| (i.id.as_str(), i.src.page)).collect();
    let practice_uncited = l
        .practice
        .iter()
        .filter(|id| page_of.get(id.as_str()).is_some_and(|p| !cited.contains(p)))
        .cloned()
        .collect();
    let explains: Vec<&str> = l
        .steps
        .iter()
        .filter_map(|s| match &s.body {
            StepBody::Explain { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    LessonRow {
        id: l.id.clone(),
        pages: l.pages,
        cited: cited.first().zip(cited.last()).map(|(a, b)| [*a, *b]),
        topics: l.topics.len(),
        cards: l.practice.len(),
        steps: l.steps.len(),
        words: l.steps.iter().map(step_words).sum(),
        practice_uncited,
        explains: explains.len(),
        explains_no_reason: explains.iter().filter(|t| !has_reason(t)).count(),
    }
}

/// The figures for one lecture and its lesson file.
pub fn lecture_row(pages: [u32; 2], lecture: &Lecture, file: &LessonFile, skips: &Skips) -> LectureRow {
    let id = file.lecture.clone();
    let cited: BTreeSet<u32> =
        file.lessons.iter().flat_map(|l| l.steps.iter().filter_map(|s| s.page)).collect();
    let skip = skips.get(&id);
    let uncited = (pages[0]..=pages[1])
        .filter(|p| !cited.contains(p) && !skip.is_some_and(|s| s.contains_key(p)))
        .map(|p| (p, lecture.items.iter().filter(|i| i.src.page == p).count()))
        .collect();
    LectureRow {
        id,
        pages,
        uncited,
        skipped: skip.map_or(0, |s| s.len()),
        strict: skip.is_some(),
        lessons: file.lessons.iter().map(|l| lesson_row(l, lecture)).collect(),
    }
}

/// Errors and warnings for one lecture.
pub fn problems(row: &LectureRow, skips: &Skips) -> (Vec<String>, Vec<String>) {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let id = &row.id;
    if let Some(s) = skips.get(id) {
        for (p, why) in s {
            if !(row.pages[0]..=row.pages[1]).contains(p) {
                errors.push(format!("{id}: lesson-skips.yaml skips p{p}, outside the lecture's pages"));
            }
            if why.trim().is_empty() {
                errors.push(format!("{id}: lesson-skips.yaml skips p{p} without a reason"));
            }
        }
    }
    if !row.uncited.is_empty() {
        let list =
            row.uncited.iter().map(|(p, n)| format!("p{p} ({n} items)")).collect::<Vec<_>>().join(", ");
        let line = format!(
            "{id}: no lesson step cites {list}; teach it or skip it with a reason in content/lesson-skips.yaml"
        );
        if row.strict { errors.push(line) } else { warnings.push(line) }
    }
    for w in row.lessons.windows(2) {
        if w[1].pages[0] < w[0].pages[0] {
            warnings.push(format!(
                "{}: starts on p{} before {} (p{})",
                w[1].id, w[1].pages[0], w[0].id, w[0].pages[0]
            ));
        }
    }
    for l in &row.lessons {
        if let Some([a, b]) = l.cited
            && (l.pages[0] < a || l.pages[1] > b)
        {
            errors.push(format!(
                "{}: pages {}–{} but its steps cite {a}–{b}; set pages to what it teaches",
                l.id, l.pages[0], l.pages[1]
            ));
        }
        if !l.practice_uncited.is_empty() {
            warnings.push(format!(
                "{}: practice cards from pages no step of the lesson cites: {}",
                l.id,
                l.practice_uncited.join(", ")
            ));
        }
        let span = l.pages[1] + 1 - l.pages[0];
        if l.cards > MAX_CARDS || span >= MAX_PAGES || l.topics > MAX_TOPICS {
            warnings.push(format!(
                "{}: wide lesson ({} cards, {span} pages, {} topics); split it at a topic boundary",
                l.id, l.cards, l.topics
            ));
        }
    }
    (errors, warnings)
}

/// Every lecture with a lesson file, in index order.
pub fn rows(root: &Path) -> Result<(Vec<LectureRow>, Skips), Vec<String>> {
    let content = root.join("content");
    let index: Index = crate::read_yaml(&content.join("index.yaml")).map_err(|e| vec![e])?;
    let skips = read_skips(root).map_err(|e| vec![e])?;
    let files: BTreeMap<String, LessonFile> =
        crate::lessons::load(root)?.into_iter().map(|(_, f, _)| (f.lecture.clone(), f)).collect();
    let mut out = Vec::new();
    for e in &index.lectures {
        let Some(file) = files.get(&e.id) else { continue };
        let Ok(lecture) = crate::read_yaml::<Lecture>(&crate::lecture_path(&content, &e.id)) else {
            continue;
        };
        out.push(lecture_row(e.pages, &lecture, file, &skips));
    }
    let unknown: Vec<String> = skips
        .keys()
        .filter(|k| !out.iter().any(|r| &r.id == *k))
        .map(|k| format!("content/lesson-skips.yaml: {k} is not a lecture with lessons"))
        .collect();
    if !unknown.is_empty() {
        return Err(unknown);
    }
    Ok((out, skips))
}

/// For `validate`: the errors, and a one-line count of the warnings.
pub fn check(root: &Path) -> (Vec<String>, usize) {
    match rows(root) {
        Err(e) => (e, 0),
        Ok((rows, skips)) => {
            let mut errors = Vec::new();
            let mut warnings = 0;
            for r in &rows {
                let (e, w) = problems(r, &skips);
                errors.extend(e.into_iter().map(|p| format!("lessons coverage: {p}")));
                warnings += w.len();
            }
            (errors, warnings)
        }
    }
}

/// `batna-tools lessons --summary | --report <course> | --warnings [<course>]`.
pub fn run(root: &Path, args: &[String]) -> Result<String, Vec<String>> {
    let (rows, skips) = rows(root)?;
    let course_of = |id: &str| batna_core::program::course_of(id).to_string();
    match args.first().map(String::as_str) {
        Some("--summary") => Ok(summary(&rows)),
        Some("--report") => {
            let c = args.get(1).ok_or_else(|| vec!["lessons --report <course>".to_string()])?;
            Ok(report(rows.iter().filter(|r| &course_of(&r.id) == c)))
        }
        Some("--warnings") => {
            let c = args.get(1);
            let mut out = Vec::new();
            for r in rows.iter().filter(|r| c.is_none_or(|c| &course_of(&r.id) == c)) {
                let (e, w) = problems(r, &skips);
                out.extend(e.into_iter().map(|p| format!("error: {p}")));
                out.extend(w.into_iter().map(|p| format!("warning: {p}")));
            }
            Ok(out.join("\n"))
        }
        _ => Err(vec![
            "usage: batna-tools lessons --summary | --report <course> | --warnings [<course>]".into(),
        ]),
    }
}

/// The course table for the CI job summary (Markdown).
pub fn summary(rows: &[LectureRow]) -> String {
    let mut by: BTreeMap<String, Vec<&LectureRow>> = BTreeMap::new();
    for r in rows {
        by.entry(batna_core::program::course_of(&r.id).to_string()).or_default().push(r);
    }
    let mut out = String::from(
        "### Lesson coverage (#833)\n\n| Course | Lessons | Book pages | Pages no step cites | Skipped | Over-wide `pages` | Wide lessons | Practice cards off the lesson's pages | Words per page | Explain steps with no reason |\n|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|\n",
    );
    for (c, rs) in by {
        let lessons: Vec<&LessonRow> = rs.iter().flat_map(|r| r.lessons.iter()).collect();
        let pages: u32 = rs.iter().map(|r| r.book_pages()).sum();
        let uncited: usize = rs.iter().map(|r| r.uncited.len()).sum();
        let skipped: usize = rs.iter().map(|r| r.skipped).sum();
        let over =
            lessons.iter().filter(|l| l.cited.is_some_and(|[a, b]| l.pages[0] < a || l.pages[1] > b)).count();
        let wide = lessons
            .iter()
            .filter(|l| {
                l.cards > MAX_CARDS || l.pages[1] + 1 - l.pages[0] >= MAX_PAGES || l.topics > MAX_TOPICS
            })
            .count();
        let off: usize = lessons.iter().map(|l| l.practice_uncited.len()).sum();
        let words: usize = rs.iter().map(|r| r.words()).sum();
        let (nr, ex) = rs.iter().fold((0, 0), |(a, b), r| {
            let (x, y) = r.explains();
            (a + x, b + y)
        });
        out.push_str(&format!(
            "| {c} | {} | {pages} | {uncited} | {skipped} | {over} | {wide} | {off} | {} | {}% |\n",
            lessons.len(),
            words / pages.max(1) as usize,
            (nr * 100).checked_div(ex).unwrap_or(0)
        ));
    }
    out
}

/// One TSV row per lesson, then one per lecture.
pub fn report<'a>(rows: impl Iterator<Item = &'a LectureRow>) -> String {
    let mut out = String::from(
        "lecture\tlesson\tpages\tcited\ttopics\tcards\tsteps\twords\tpractice_off_pages\texplain_no_reason\tlecture_words_per_page\tlecture_uncited_pages\n",
    );
    for r in rows {
        let uncited = r.uncited.iter().map(|(p, _)| p.to_string()).collect::<Vec<_>>().join(",");
        for l in &r.lessons {
            let cited = l.cited.map_or("-".to_string(), |[a, b]| format!("{a}-{b}"));
            out.push_str(&format!(
                "{}\t{}\t{}-{}\t{cited}\t{}\t{}\t{}\t{}\t{}\t{}/{}\t{}\t{uncited}\n",
                r.id,
                l.id,
                l.pages[0],
                l.pages[1],
                l.topics,
                l.cards,
                l.steps,
                l.words,
                l.practice_uncited.len(),
                l.explains_no_reason,
                l.explains,
                r.words_per_page(),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests;
