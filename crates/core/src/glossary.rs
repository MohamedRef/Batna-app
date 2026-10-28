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
}
