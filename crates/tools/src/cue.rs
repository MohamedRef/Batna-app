//! The longest-option cue (#816): how often an MCQ's key is strictly the
//! longest option. Test-wise learners pick the longest option, so a bank where
//! that wins far above chance (20–25%) teaches the wrong habit.
//!
//! Questions whose options are printed in a source (book, slide, past paper,
//! lecture quiz) keep their wording and are left out of the count.
//!
//! ```text
//! batna-tools cue [--all]   share per lecture, highest first (--all lists every lecture)
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use batna_core::content::{Body, Index, Lecture};

/// Tags whose options are someone else's words and stay as printed.
pub const PRINTED: &[&str] = &["book-question", "slide-question", "past-paper", "lecture-quiz"];

/// The bank-wide share may not rise above this (per cent, one decimal).
/// Lower it as the rewrites in #816 land; never raise it.
#[cfg(test)]
pub const CEILING_PERCENT: f64 = 29.0;

/// Lectures with fewer authored MCQs than this are not ranked in the summary.
const MIN_RANKED: usize = 5;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    /// Keys strictly longer than every other option.
    pub longest: usize,
    /// Authored MCQs.
    pub total: usize,
}

impl Count {
    pub fn percent(self) -> f64 {
        if self.total == 0 { 0.0 } else { 100.0 * self.longest as f64 / self.total as f64 }
    }
}

/// Per-lecture counts, keyed by lecture id.
#[derive(Debug, Default)]
pub struct Tally(pub BTreeMap<String, Count>);

/// Is the key strictly the longest option (counted in characters)?
pub fn key_is_longest(options: &[String], answer: usize) -> bool {
    let len = |s: &String| s.chars().count();
    let Some(key) = options.get(answer).map(len) else {
        return false;
    };
    options.iter().enumerate().all(|(i, o)| i == answer || len(o) < key)
}

impl Tally {
    pub fn add(&mut self, lec: &Lecture) {
        let count = self.0.entry(lec.lecture.clone()).or_default();
        for item in &lec.items {
            let Body::Mcq { options, answer, .. } = &item.body else {
                continue;
            };
            if item.tags.iter().any(|t| PRINTED.contains(&t.as_str())) {
                continue;
            }
            count.total += 1;
            if key_is_longest(options, *answer) {
                count.longest += 1;
            }
        }
    }

    pub fn overall(&self) -> Count {
        self.0
            .values()
            .fold(Count::default(), |a, c| Count { longest: a.longest + c.longest, total: a.total + c.total })
    }

    /// Lectures with at least `min` authored MCQs, highest share first.
    pub fn ranked(&self, min: usize) -> Vec<(&str, Count)> {
        let mut v: Vec<_> =
            self.0.iter().filter(|(_, c)| c.total >= min).map(|(l, c)| (l.as_str(), *c)).collect();
        v.sort_by(|a, b| b.1.percent().total_cmp(&a.1.percent()).then(a.0.cmp(b.0)));
        v
    }

    /// A second line for `validate`'s report, or nothing when there are no
    /// authored MCQs.
    pub fn line(&self) -> String {
        if self.overall().total == 0 { String::new() } else { format!("\n{}", self.summary()) }
    }

    /// The bank share and the five highest lectures.
    pub fn summary(&self) -> String {
        let all = self.overall();
        let top: Vec<String> = self
            .ranked(MIN_RANKED)
            .into_iter()
            .take(5)
            .map(|(l, c)| format!("{l} {}/{}", c.longest, c.total))
            .collect();
        format!(
            "longest-option key: {:.1}% of {} authored MCQs (highest: {}; `batna-tools cue` lists all)",
            all.percent(),
            all.total,
            top.join(", ")
        )
    }
}

/// Reads every written lecture under `root/content`.
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
    Ok(tally)
}

pub fn run(root: &Path, args: &[String]) -> Result<String, Vec<String>> {
    let tally = tally(root)?;
    let min = if args.iter().any(|a| a == "--all") { 1 } else { MIN_RANKED };
    let mut out = vec![tally.summary()];
    for (l, c) in tally.ranked(min) {
        out.push(format!("{l}\t{}/{}\t{:.0}%", c.longest, c.total, c.percent()));
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

    #[test]
    fn the_bank_stays_under_the_ceiling() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let all = tally(&root).unwrap().overall();
        assert!(all.total > 0);
        assert!(
            all.percent() <= CEILING_PERCENT,
            "the key is the longest option in {:.1}% of authored MCQs, above the {CEILING_PERCENT}% ceiling (#816): \
             make a distractor as long as the key, or trim the key; `batna-tools cue` lists the lectures",
            all.percent()
        );
    }
}
