//! The longest-option cue (#816, #1349): how often a question's key is
//! strictly the longest option. Test-wise learners pick the longest option, so
//! content where that wins far above chance (20–25%) teaches the wrong habit
//! and lets a lesson's "right first time" score be passed by shape.
//!
//! Four kinds of question have options, and each is counted on its own:
//! authored MCQs, the steps of case items (`content/<course>/`), lesson checks
//! and the steps of lesson mini-cases (`content/lessons/`). Each kind has a
//! ceiling that the rewrites lower and nothing raises.
//!
//! Questions whose options are printed in a source (book, slide, past paper,
//! lecture quiz) keep their wording and are left out of the count.
//!
//! ```text
//! batna-tools cue [--all]                  share per kind and lecture, highest first (--all lists every lecture)
//! batna-tools cue --kind K [--all]         one kind: mcq, case, check or lesson-case
//! batna-tools cue --list LECTURE [--kind K] every cued question in a lecture, key and options' lengths
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use batna_core::content::{Body, CaseStep, Index, Lecture};
use batna_core::lesson::{LessonFile, StepBody};

/// Tags whose options are someone else's words and stay as printed.
pub const PRINTED: &[&str] = &["book-question", "slide-question", "past-paper", "lecture-quiz"];

/// A kind of question with options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// Authored MCQ items.
    Mcq,
    /// A step of a case item.
    Case,
    /// A lesson `check` step.
    Check,
    /// A step of a lesson `case` step.
    LessonCase,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Mcq, Kind::Case, Kind::Check, Kind::LessonCase];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Mcq => "mcq",
            Kind::Case => "case",
            Kind::Check => "check",
            Kind::LessonCase => "lesson-case",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Kind::Mcq => "authored MCQs",
            Kind::Case => "case steps",
            Kind::Check => "lesson checks",
            Kind::LessonCase => "lesson mini-case steps",
        }
    }

    fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == s)
    }

    /// The share of this kind may not rise above this (per cent, one
    /// decimal). Lower it as the rewrites in #816 and #1349 land; never
    /// raise it.
    #[cfg(test)]
    pub fn ceiling(self) -> f64 {
        match self {
            Kind::Mcq => 25.0,
            Kind::Case => 53.0,
            Kind::Check => 47.5,
            Kind::LessonCase => 54.5,
        }
    }
}

/// Lectures with fewer counted questions than this are not ranked in the summary.
const MIN_RANKED: usize = 5;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    /// Keys strictly longer than every other option.
    pub longest: usize,
    /// Counted questions.
    pub total: usize,
}

impl Count {
    pub fn percent(self) -> f64 {
        if self.total == 0 { 0.0 } else { 100.0 * self.longest as f64 / self.total as f64 }
    }
}

/// One counted question whose key is the longest option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cued {
    pub kind: Kind,
    /// `hb.L03.091 step 1`, `dm.L13.lesson6 step 9`, `… step 9 part 2`.
    pub at: String,
    /// The key's length and the mean length of the wrong options, in characters.
    pub key: usize,
    pub others: usize,
    pub question: String,
    pub answer: String,
}

/// Per-kind, per-lecture counts, and the cued questions themselves.
#[derive(Debug, Default)]
pub struct Tally {
    pub counts: BTreeMap<(Kind, String), Count>,
    pub cued: Vec<(String, Cued)>,
}

fn len(s: &str) -> usize {
    s.chars().count()
}

/// Is the key strictly the longest option (counted in characters)?
pub fn key_is_longest(options: &[String], answer: usize) -> bool {
    let Some(key) = options.get(answer).map(|s| len(s)) else {
        return false;
    };
    options.iter().enumerate().all(|(i, o)| i == answer || len(o) < key)
}

impl Tally {
    /// Counts one question; options shorter than two are not a question.
    fn one(
        &mut self,
        kind: Kind,
        lecture: &str,
        at: String,
        question: &str,
        options: &[String],
        answer: usize,
    ) {
        if options.len() < 2 {
            return;
        }
        let count = self.counts.entry((kind, lecture.to_string())).or_default();
        count.total += 1;
        if key_is_longest(options, answer) {
            count.longest += 1;
            let wrong: Vec<usize> =
                options.iter().enumerate().filter(|(i, _)| *i != answer).map(|(_, o)| len(o)).collect();
            self.cued.push((
                lecture.to_string(),
                Cued {
                    kind,
                    at,
                    key: len(&options[answer]),
                    others: wrong.iter().sum::<usize>() / wrong.len().max(1),
                    question: question.to_string(),
                    answer: options[answer].clone(),
                },
            ));
        }
    }

    fn case_steps(&mut self, kind: Kind, lecture: &str, at: &str, steps: &[CaseStep]) {
        for (k, s) in steps.iter().enumerate() {
            if let Some(a) = s.answer {
                self.one(kind, lecture, format!("{at} step {}", k + 1), &s.question, &s.options, a);
            }
        }
    }

    /// A lecture's items: MCQs and case steps.
    pub fn add(&mut self, lec: &Lecture) {
        // A lecture with items is listed even when none is cued.
        self.counts.entry((Kind::Mcq, lec.lecture.clone())).or_default();
        for item in &lec.items {
            if item.tags.iter().any(|t| PRINTED.contains(&t.as_str())) {
                continue;
            }
            match &item.body {
                Body::Mcq { stem, options, answer, .. } => {
                    self.one(Kind::Mcq, &lec.lecture, item.id.clone(), stem, options, *answer)
                }
                Body::Case { steps, .. } => self.case_steps(Kind::Case, &lec.lecture, &item.id, steps),
                _ => {}
            }
        }
    }

    /// A lesson file's checks and mini-case steps.
    pub fn add_lessons(&mut self, file: &LessonFile) {
        for l in &file.lessons {
            for (k, step) in l.steps.iter().enumerate() {
                let at = format!("{} step {}", l.id, k + 1);
                match &step.body {
                    StepBody::Check { question, options, answer, .. } => {
                        self.one(Kind::Check, &file.lecture, at, question, options, *answer)
                    }
                    StepBody::Case { steps, .. } => {
                        self.case_steps(Kind::LessonCase, &file.lecture, &at, steps)
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn of(&self, kind: Kind) -> impl Iterator<Item = (&str, Count)> {
        self.counts.iter().filter(move |((k, _), _)| *k == kind).map(|((_, l), c)| (l.as_str(), *c))
    }

    pub fn overall(&self, kind: Kind) -> Count {
        self.of(kind).fold(Count::default(), |a, (_, c)| Count {
            longest: a.longest + c.longest,
            total: a.total + c.total,
        })
    }

    /// Lectures with at least `min` counted questions of a kind, highest share first.
    pub fn ranked(&self, kind: Kind, min: usize) -> Vec<(&str, Count)> {
        let mut v: Vec<_> = self.of(kind).filter(|(_, c)| c.total >= min).collect();
        v.sort_by(|a, b| b.1.percent().total_cmp(&a.1.percent()).then(a.0.cmp(b.0)));
        v
    }

    /// Lines for `validate`'s report, or nothing when nothing is counted.
    pub fn line(&self) -> String {
        Kind::ALL
            .into_iter()
            .filter(|k| self.overall(*k).total > 0)
            .map(|k| format!("\n{}", self.summary(k)))
            .collect()
    }

    /// A kind's share and its five highest lectures.
    pub fn summary(&self, kind: Kind) -> String {
        let all = self.overall(kind);
        let top: Vec<String> = self
            .ranked(kind, MIN_RANKED)
            .into_iter()
            .take(5)
            .map(|(l, c)| format!("{l} {}/{}", c.longest, c.total))
            .collect();
        let highest = if top.is_empty() { String::new() } else { format!("highest: {}; ", top.join(", ")) };
        format!(
            "longest-option key: {:.1}% of {} {} ({highest}`batna-tools cue --kind {}` lists all)",
            all.percent(),
            all.total,
            kind.label(),
            kind.name()
        )
    }
}

/// Reads every written lecture under `root/content` and every lesson file.
pub fn tally(root: &Path) -> Result<Tally, Vec<String>> {
    let content = root.join("content");
    let text = std::fs::read_to_string(content.join("index.yaml")).map_err(|e| vec![e.to_string()])?;
    let index: Index = serde_yaml_ng::from_str(&text).map_err(|e| vec![e.to_string()])?;
    let mut tally = Tally::default();
    for entry in &index.lectures {
        let (chapter, num) = entry.id.split_once('.').unwrap_or((&entry.id, ""));
        let path = content.join(chapter).join(format!("{num}.yaml"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue; // not written yet
        };
        let lec: Lecture =
            serde_yaml_ng::from_str(&text).map_err(|e| vec![format!("{}: {e}", path.display())])?;
        tally.add(&lec);
    }
    for (_, file, _) in crate::lessons::load(root)? {
        tally.add_lessons(&file);
    }
    Ok(tally)
}

pub fn run(root: &Path, args: &[String]) -> Result<String, Vec<String>> {
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
    let kinds: Vec<Kind> = match value("--kind") {
        Some(k) => vec![
            Kind::parse(k)
                .ok_or_else(|| vec![format!("unknown kind {k}: mcq, case, check or lesson-case")])?,
        ],
        None => Kind::ALL.to_vec(),
    };
    let tally = tally(root)?;
    let mut out = Vec::new();
    if let Some(lecture) = value("--list") {
        for (_, c) in tally.cued.iter().filter(|(l, c)| l == lecture && kinds.contains(&c.kind)) {
            out.push(format!(
                "{}\t{}\tkey {} vs {}\t{}\t→ {}",
                c.kind.name(),
                c.at,
                c.key,
                c.others,
                c.question.replace('\n', " "),
                c.answer
            ));
        }
        if out.is_empty() {
            out.push(format!("{lecture}: no cued question"));
        }
        return Ok(out.join("\n"));
    }
    let min = if args.iter().any(|a| a == "--all") { 1 } else { MIN_RANKED };
    for kind in kinds {
        out.push(tally.summary(kind));
        for (l, c) in tally.ranked(kind, min) {
            out.push(format!("{}\t{l}\t{}/{}\t{:.0}%", kind.name(), c.longest, c.total, c.percent()));
        }
    }
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn only_a_strictly_longest_key_counts() {
        assert!(key_is_longest(&s(&["a long key", "b", "c"]), 0));
        assert!(!key_is_longest(&s(&["same", "four", "c"]), 0), "a tie is no cue");
        assert!(!key_is_longest(&s(&["k", "longer one"]), 0));
        assert!(!key_is_longest(&s(&["k"]), 3), "an answer past the options is not counted");
        // Characters, not bytes: "≥" is one character.
        assert!(!key_is_longest(&s(&["≥5", "<5"]), 0));
    }

    const LECTURE: &str = r#"
schema: batna.content/v1
lecture: zz.L01
title: T
book: { chapter: 1, pages: [1, 2] }
items:
  - { id: zz.L01.001, kind: mcq, stem: Q, options: [a long key, b, c], answer: 0, why: [x, y, z], src: { page: 1 }, status: unverified }
  - { id: zz.L01.002, kind: mcq, stem: Q, options: [a long key, b], answer: 0, why: [x, y], src: { page: 1 }, status: unverified, tags: [past-paper] }
  - id: zz.L01.003
    kind: case
    stem: S
    steps:
      - { question: Q1, options: [the longest key, b], answer: 0, why: w }
      - { question: Q2, options: [k, longer], answer: 0, why: w }
      - { question: Q3, why: open question }
    src: { page: 1 }
    status: unverified
"#;

    const LESSONS: &str = r#"
schema: batna.lesson/v1
lecture: zz.L01
lessons:
  - id: zz.L01.lesson1
    title: T
    hook: H
    minutes: 1
    pages: [1, 2]
    steps:
      - { kind: check, question: Q, options: [k, other], answer: 0, why: [x, y] }
      - kind: case
        stem: S
        steps:
          - { question: Q, options: [the key is long, b], answer: 0, why: w }
    recap: [r]
    status: unverified
"#;

    #[test]
    fn every_kind_is_counted_on_its_own() {
        let lec: Lecture = serde_yaml_ng::from_str(LECTURE).expect("lecture");
        let lessons: LessonFile = serde_yaml_ng::from_str(LESSONS).expect("lessons");
        let mut t = Tally::default();
        t.add(&lec);
        t.add_lessons(&lessons);
        assert_eq!(t.overall(Kind::Mcq), Count { longest: 1, total: 1 }, "the printed MCQ is left out");
        assert_eq!(
            t.overall(Kind::Case),
            Count { longest: 1, total: 2 },
            "a step without options is skipped"
        );
        assert_eq!(t.overall(Kind::Check), Count { longest: 0, total: 1 });
        assert_eq!(t.overall(Kind::LessonCase), Count { longest: 1, total: 1 });
        let at: Vec<&str> = t.cued.iter().map(|(_, c)| c.at.as_str()).collect();
        assert_eq!(at, ["zz.L01.001", "zz.L01.003 step 1", "zz.L01.lesson1 step 2 step 1"]);
    }

    #[test]
    fn every_kind_stays_under_its_ceiling() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let t = tally(&root).unwrap();
        for kind in Kind::ALL {
            let all = t.overall(kind);
            assert!(all.total > 0, "no {} counted", kind.label());
            assert!(
                all.percent() <= kind.ceiling(),
                "the key is the longest option in {:.1}% of {}, above the {}% ceiling (#816, #1349): \
                 give the wrong options the key's shape and length (or trim the key); \
                 `batna-tools cue --kind {}` lists the lectures",
                all.percent(),
                kind.label(),
                kind.ceiling(),
                kind.name()
            );
        }
    }
}
