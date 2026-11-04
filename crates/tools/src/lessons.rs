//! Lesson files: `content/lessons/<chapter>/LNN.yaml` (`batna.lesson/v1`).
//! `batna-tools validate` checks them against their lecture
//! ([`batna_core::lesson::validate`]); `packs` puts them in the content pack.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use batna_core::content::Lecture;
use batna_core::glossary::Glossary;
use batna_core::lesson::{self, LessonContext, LessonFile};

/// Every lesson file, sorted by path: `(path, parsed, raw text)`.
pub fn load(root: &Path) -> Result<Vec<(PathBuf, LessonFile, String)>, Vec<String>> {
    let dir = root.join("content/lessons");
    let mut paths = Vec::new();
    for chapter in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        for f in std::fs::read_dir(chapter.path()).into_iter().flatten().flatten() {
            if f.path().extension().is_some_and(|e| e == "yaml") {
                paths.push(f.path());
            }
        }
    }
    paths.sort();
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for p in paths {
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                problems.push(format!("{}: {e}", p.display()));
                continue;
            }
        };
        match serde_yaml_ng::from_str::<LessonFile>(&text) {
            Ok(f) => out.push((p, f, text)),
            Err(e) => problems.push(format!("{}: {e}", crate::rel(root, &p))),
        }
    }
    if problems.is_empty() { Ok(out) } else { Err(problems) }
}

/// Every word a lesson step shows that looks like an abbreviation but has
/// no entry in `content/glossary.yaml`, so the step's "اختصارات" chip can
/// explain all of them. A capitalised word that isn't one goes in the
/// glossary's `plain:` list.
pub fn unknown_abbreviations(file: &LessonFile, g: &Glossary) -> Vec<String> {
    let g = g.finder_in(Some(batna_core::program::course_of(&file.lecture)));
    let mut out = Vec::new();
    for l in &file.lessons {
        for (k, step) in l.steps.iter().enumerate() {
            let mut unknown: Vec<String> = Vec::new();
            for t in step.shown_text() {
                for w in g.unknown_in(&t) {
                    if !unknown.contains(&w) {
                        unknown.push(w);
                    }
                }
            }
            if !unknown.is_empty() {
                out.push(format!(
                    "{} step {}: {} not in content/glossary.yaml (add an entry, or list it under plain: if it isn't an abbreviation)",
                    l.id,
                    k + 1,
                    unknown.iter().map(|w| format!("`{w}`")).collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }
    out
}

/// Checks every lesson file. Returns `(files, lessons)` and pushes problems.
pub fn validate_all(
    root: &Path,
    approved: &BTreeSet<String>,
    glossary: Option<&Glossary>,
    problems: &mut Vec<String>,
) -> (usize, usize) {
    let files = match load(root) {
        Ok(f) => f,
        Err(p) => {
            problems.extend(p);
            return (0, 0);
        }
    };
    let content = root.join("content");
    let mut lessons = 0;
    for (path, file, text) in &files {
        let at = crate::rel(root, path);
        // content/lessons/hb/L01.yaml teaches hb.L01.
        let chapter = path.parent().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().to_string());
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string());
        let named = format!("{}.{}", chapter.unwrap_or_default(), stem.unwrap_or_default());
        if file.lecture != named {
            problems.push(format!("{at}: lecture is {}, the file name says {named}", file.lecture));
        }
        let lecture_path = crate::lecture_path(&content, &file.lecture);
        let lecture: Lecture = match crate::read_yaml(&lecture_path) {
            Ok(l) => l,
            Err(_) => {
                problems.push(format!("{at}: its lecture {} has no content file yet", file.lecture));
                continue;
            }
        };
        let cx =
            LessonContext { lecture: &lecture, approved: Some(approved), raw_step_keys: raw_step_keys(text) };
        problems.extend(crate::content::split_options(text).into_iter().map(|p| format!("{at}: {p}")));
        for p in lesson::validate(file, &cx) {
            problems.push(format!("{at}: {p}"));
        }
        if let Some(g) = glossary {
            problems.extend(unknown_abbreviations(file, g).into_iter().map(|p| format!("{at}: {p}")));
        }
        lessons += file.lessons.len();
    }
    let mut all: Vec<LessonFile> = files.into_iter().map(|(_, f, _)| f).collect();
    problems.extend(
        lesson::embed_images(&mut all, read_image(root)).into_iter().map(|p| format!("lessons: {p}")),
    );
    (all.len(), lessons)
}

/// Reads `content/lessons/img/<path>`, for [`lesson::embed_images`].
pub fn read_image(root: &Path) -> impl FnMut(&str) -> Option<Vec<u8>> + '_ {
    move |path| std::fs::read(root.join("content/lessons/img").join(path)).ok()
}

/// The keys written on each step, by `(lesson, step)` index.
fn raw_step_keys(text: &str) -> Vec<((usize, usize), Vec<String>)> {
    let mut out = Vec::new();
    let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text) else {
        return out;
    };
    let seq = |v: &serde_yaml_ng::Value, k: &str| {
        v.get(k).and_then(|s| s.as_sequence()).cloned().unwrap_or_default()
    };
    for (li, lesson) in seq(&doc, "lessons").iter().enumerate() {
        for (si, step) in seq(lesson, "steps").iter().enumerate() {
            let keys = step
                .as_mapping()
                .map(|m| m.keys().filter_map(|k| k.as_str().map(String::from)).collect())
                .unwrap_or_default();
            out.push(((li, si), keys));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_lessons_are_valid() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let approved = BTreeSet::new();
        let mut problems = Vec::new();
        validate_all(&root, &approved, None, &mut problems);
        assert!(problems.is_empty(), "{problems:#?}");
    }

    use crate::testkit::{self, write};

    fn problems(root: &Path, glossary: Option<&Glossary>) -> (usize, usize, Vec<String>) {
        let mut p = Vec::new();
        let (files, lessons) = validate_all(root, &BTreeSet::new(), glossary, &mut p);
        (files, lessons, p)
    }

    #[test]
    fn a_lesson_file_must_match_its_lecture() {
        let tree = testkit::tree();
        let root = tree.path();
        assert_eq!(problems(root, None), (1, 1, vec![]));

        write(
            root,
            "content/lessons/hb/L02.yaml",
            &testkit::LESSON.replace("hb.L01.lesson1", "hb.L02.lesson1"),
        );
        let (_, _, p) = problems(root, None);
        assert!(
            p.contains(
                &"content/lessons/hb/L02.yaml: lecture is hb.L01, the file name says hb.L02".to_owned()
            ),
            "{p:#?}"
        );
        write(
            root,
            "content/lessons/hb/L02.yaml",
            &testkit::LESSON
                .replace("lecture: hb.L01", "lecture: hb.L02")
                .replace("hb.L01.lesson1", "hb.L02.lesson1"),
        );
        let (_, _, p) = problems(root, None);
        assert_eq!(p, ["content/lessons/hb/L02.yaml: its lecture hb.L02 has no content file yet"]);
        std::fs::remove_file(root.join("content/lessons/hb/L02.yaml")).unwrap();

        // A file that doesn't parse, or can't be read, stops the check.
        write(root, "content/lessons/hb/L03.yaml", "lessons: [");
        std::fs::create_dir_all(root.join("content/lessons/hb/L04.yaml")).unwrap();
        let (files, lessons, p) = problems(root, None);
        assert_eq!((files, lessons, p.len()), (0, 0, 2), "{p:#?}");
        assert!(p[0].starts_with("content/lessons/hb/L03.yaml: ") && p[1].contains("L04.yaml"), "{p:#?}");
    }

    #[test]
    fn every_abbreviation_a_step_shows_needs_a_glossary_entry() {
        let tree = testkit::tree();
        let root = tree.path();
        let glossary = |yaml: &str| -> Glossary {
            serde_yaml_ng::from_str(&format!("schema: batna.glossary/v1\nentries:\n{yaml}")).unwrap()
        };
        let lesson = testkit::LESSON.replace("هي الـ **major", "(HSC، ECM) هي الـ **major");
        write(root, "content/lessons/hb/L01.yaml", &lesson);
        let one =
            glossary("  - { term: HSC, kind: abbr, text: Hepatic stellate cell, status: unverified }\n");
        let (_, _, p) = problems(root, Some(&one));
        assert_eq!(
            p,
            [
                "content/lessons/hb/L01.yaml: hb.L01.lesson1 step 2: `ECM` not in content/glossary.yaml (add an entry, or list it under plain: if it isn't an abbreviation)"
            ]
        );
        let both = glossary(
            "  - { term: HSC, kind: abbr, text: Hepatic stellate cell, status: unverified }\n  - { term: ECM, kind: abbr, text: Extracellular matrix, status: unverified }\n",
        );
        assert_eq!(problems(root, Some(&both)).2, Vec::<String>::new());
    }

    #[test]
    fn typos_in_step_keys_are_caught() {
        let keys = raw_step_keys("lessons:\n  - steps:\n      - { kind: check, anwser: 1 }\n");
        assert_eq!(keys, vec![((0, 0), vec!["kind".to_string(), "anwser".to_string()])]);
        assert!(raw_step_keys("lessons: [").is_empty());
    }
}
