//! The hover glossary, `content/glossary.yaml`: abbreviations and drug names
//! with a one-line expansion. The app shows an entry when the S Pen or the
//! mouse rests on the word (PARITY.md §6.3); `validate::glossary` checks the
//! file. Entries are content like any other: `status: unverified` until
//! Mohamed approves them.

use serde::{Deserialize, Serialize};

use crate::content::Status;

pub const SCHEMA: &str = "batna.glossary/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Glossary {
    pub schema: String,
    pub entries: Vec<Entry>,
    /// Words in capitals that are not abbreviations (`NOT`, `II`), so the
    /// lesson check ([`Glossary::unknown_in`]) leaves them alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The word as written in the content: `SAAG`, `terlipressin`.
    pub term: String,
    /// Other spellings that show the same entry (`TIPSS`, `UDCA`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub kind: Kind,
    /// The expansion, short enough to read at a glance.
    pub text: String,
    pub status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Abbr,
    Drug,
}

impl Entry {
    /// The id used in `content/approved.txt`: `glossary.SAAG`.
    pub fn id(&self) -> String {
        format!("glossary.{}", self.term)
    }

    /// The term and its aliases.
    pub fn spellings(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.term.as_str()).chain(self.aliases.iter().map(String::as_str))
    }
}

impl Glossary {
    /// Finds the entry for a word under the pointer. Abbreviations match
    /// exactly (`HE` is not `he`); drug names ignore case, so a drug at the
    /// start of a sentence still matches.
    pub fn find(&self, word: &str) -> Option<&Entry> {
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if word.is_empty() {
            return None;
        }
        self.entries
            .iter()
            .filter(|e| e.status != Status::Rejected)
            .find(|e| e.spellings().any(|s| s == word))
            .or_else(|| {
                self.entries
                    .iter()
                    .filter(|e| e.status != Status::Rejected && e.kind == Kind::Drug)
                    .find(|e| e.spellings().any(|s| s.eq_ignore_ascii_case(word)))
            })
    }
}

impl Glossary {
    /// The table the app's hover script looks words up in, as JSON:
    /// `{"exact": {spelling: [text, approved]}, "lower": {drug: [...]}}`.
    /// Same rules as [`Glossary::find`]; rejected entries are left out.
    pub fn peek_table(&self) -> serde_json::Value {
        let mut exact = serde_json::Map::new();
        let mut lower = serde_json::Map::new();
        for e in self.entries.iter().filter(|e| e.status != Status::Rejected) {
            let v = serde_json::json!([e.text, e.status == Status::Approved]);
            for s in e.spellings() {
                exact.entry(s.to_string()).or_insert_with(|| v.clone());
                if e.kind == Kind::Drug {
                    lower.entry(s.to_lowercase()).or_insert_with(|| v.clone());
                }
            }
        }
        serde_json::json!({ "exact": exact, "lower": lower })
    }
}

/// A spelling that reads as an abbreviation: two or more capital letters
/// (`HE`, `NITs`, `HBsAg`, `FIB-4`), unlike `terlipressin` or `Child`.
pub fn looks_like_abbr(word: &str) -> bool {
    word.chars().filter(char::is_ascii_uppercase).count() >= 2
}

/// A letter or digit of a Latin (or Greek) word. Arabic letters don't
/// count, so `وSLE` ("and SLE") still shows `SLE`.
fn word_char(c: char) -> bool {
    c.is_alphanumeric() && !('\u{0600}'..='\u{06FF}').contains(&c)
}

/// Whether `text[start..end]` stands as a whole word: no letter or digit
/// touches it, except a plural `s` after it (`PPIs` shows `PPI`).
fn whole_word(text: &str, start: usize, end: usize) -> Option<usize> {
    let before = text[..start].chars().next_back();
    if before.is_some_and(word_char) {
        return None;
    }
    let mut rest = text[end..].chars();
    match rest.next() {
        Some(c) if word_char(c) => {
            let plural = c == 's' && !rest.next().is_some_and(word_char);
            plural.then_some(end + 1)
        }
        _ => Some(end),
    }
}

impl Glossary {
    /// The abbreviations written in `texts`, each once, in the order they
    /// first appear: what a lesson step's "اختصارات" chip lists. A longer
    /// spelling wins over the shorter ones inside it (`HRS-AKI` over `HRS`).
    /// Drug names count only when written like an abbreviation (`UDCA`).
    pub fn abbreviations_in<'a, S: AsRef<str>>(&'a self, texts: &[S]) -> Vec<&'a Entry> {
        let mut found: Vec<(usize, usize, &Entry)> = Vec::new();
        for (t, text) in texts.iter().enumerate() {
            let mut spans = self.matches(text.as_ref());
            spans.sort_by_key(|(start, _, _)| *start);
            // Text `t` comes after every earlier text.
            found.extend(spans.into_iter().map(|(start, _, e)| (t, start, e)));
        }
        found.sort_by_key(|(t, start, _)| (*t, *start));
        let mut out: Vec<&Entry> = Vec::new();
        for (_, _, e) in found {
            if !out.iter().any(|o| std::ptr::eq(*o, e)) {
                out.push(e);
            }
        }
        out
    }

    /// Where the glossary's abbreviations stand in `text`: (start, end,
    /// entry), with no two overlapping (the longest wins).
    fn matches<'a>(&'a self, text: &str) -> Vec<(usize, usize, &'a Entry)> {
        let mut all: Vec<(usize, usize, &Entry)> = Vec::new();
        for e in self.entries.iter().filter(|e| e.status != Status::Rejected) {
            for s in e.spellings().filter(|s| e.kind == Kind::Abbr || looks_like_abbr(s)) {
                for (start, _) in text.match_indices(s) {
                    if let Some(end) = whole_word(text, start, start + s.len()) {
                        all.push((start, end, e));
                    }
                }
            }
        }
        all.sort_by_key(|(start, end, _)| std::cmp::Reverse(end - start));
        let mut kept: Vec<(usize, usize, &Entry)> = Vec::new();
        for m in all {
            if !kept.iter().any(|k| m.0 < k.1 && k.0 < m.1) {
                kept.push(m);
            }
        }
        kept
    }

    /// Words in `text` that look like abbreviations (two or more capitals)
    /// but have no glossary entry and aren't listed as `plain`. The lesson
    /// check reports them, so every abbreviation a step shows can be
    /// explained.
    pub fn unknown_in(&self, text: &str) -> Vec<String> {
        let kept = self.matches(text);
        let mut out = Vec::new();
        let mut start = None;
        for (i, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            match (word_char(c), start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    let word = &text[s..i];
                    let covered = kept.iter().any(|k| k.0 <= s && i <= k.1);
                    let plain = self.plain.iter().any(|p| p == word);
                    if looks_like_abbr(word) && !covered && !plain && !out.iter().any(|o| o == word) {
                        out.push(word.to_string());
                    }
                    start = None;
                }
                _ => {}
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g() -> Glossary {
        serde_yaml_ng::from_str(
            "schema: batna.glossary/v1
entries:
  - { term: HE, kind: abbr, text: Hepatic encephalopathy, status: unverified }
  - { term: UDCA, aliases: [UDC], kind: drug, text: Ursodeoxycholic acid, status: unverified }
  - { term: terlipressin, kind: drug, text: Vasopressin analogue, status: unverified }
  - { term: XYZ, kind: abbr, text: Wrong, status: rejected }",
        )
        .unwrap()
    }

    #[test]
    fn abbreviations_match_exactly() {
        let g = g();
        assert_eq!(g.find("HE").map(|e| e.text.as_str()), Some("Hepatic encephalopathy"));
        assert_eq!(g.find("(HE),").map(|e| e.term.as_str()), Some("HE"));
        assert!(g.find("he").is_none());
    }

    #[test]
    fn drugs_ignore_case_and_aliases_work() {
        let g = g();
        assert_eq!(g.find("Terlipressin").map(|e| e.term.as_str()), Some("terlipressin"));
        assert_eq!(g.find("UDC").map(|e| e.term.as_str()), Some("UDCA"));
        assert_eq!(g.find("udca").map(|e| e.term.as_str()), Some("UDCA"));
    }

    #[test]
    fn rejected_entries_never_show() {
        assert!(g().find("XYZ").is_none());
    }

    #[test]
    fn peek_table_matches_find() {
        let t = g().peek_table();
        assert_eq!(t["exact"]["HE"][0], "Hepatic encephalopathy");
        assert_eq!(t["exact"]["HE"][1], false);
        assert!(t["lower"].get("he").is_none());
        assert_eq!(t["lower"]["udc"][0], "Ursodeoxycholic acid");
        assert!(t["exact"].get("XYZ").is_none());
    }

    fn lesson_glossary() -> Glossary {
        serde_yaml_ng::from_str(
            "schema: batna.glossary/v1
entries:
  - { term: HRS, kind: abbr, text: Hepatorenal syndrome, status: unverified }
  - { term: AKI, kind: abbr, text: Acute kidney injury, status: unverified }
  - { term: HRS-AKI, kind: abbr, text: HRS type AKI, status: unverified }
  - { term: PPI, kind: abbr, text: Proton pump inhibitor, status: unverified }
  - { term: SLE, kind: abbr, text: Lupus, status: unverified }
  - { term: UDCA, kind: drug, text: Ursodeoxycholic acid, status: unverified }
  - { term: terlipressin, kind: drug, text: Vasopressin analogue, status: unverified }
plain: [NOT]",
        )
        .unwrap()
    }

    fn terms<'a>(found: &[&'a Entry]) -> Vec<&'a str> {
        found.iter().map(|e| e.term.as_str()).collect()
    }

    #[test]
    fn abbreviations_in_a_step_come_once_in_order_longest_first() {
        let g = lesson_glossary();
        let found = g.abbreviations_in(&["Give terlipressin for HRS-AKI, then UDCA.", "AKI and PPIs, وSLE."]);
        // HRS-AKI swallows HRS and AKI in the first text; AKI shows from the second.
        assert_eq!(terms(&found), ["HRS-AKI", "UDCA", "AKI", "PPI", "SLE"]);
        // Drug names written in lower case are not abbreviations.
        assert!(!terms(&found).contains(&"terlipressin"));
        // Inside a longer word, nothing matches.
        assert!(g.abbreviations_in(&["HRSX PPIX"]).is_empty());
    }

    #[test]
    fn unknown_capitals_are_reported_but_plain_words_are_not() {
        let g = lesson_glossary();
        assert_eq!(g.unknown_in("Do NOT stop the PPI in ACLF or HRS-AKI; FIB-4 is low."), ["ACLF", "FIB"]);
        assert!(g.unknown_in("Child-Pugh and mmHg are fine, وSLE too.").is_empty());
    }
}
