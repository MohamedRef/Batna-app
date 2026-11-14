//! Checks a lecture against the content rules (SPLIT.md §4–5). Pure: the
//! tools crate reads the files and passes the parsed lecture plus the raw
//! keys of each item.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::content::{self, Body, Item, Lecture, Status};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The item id, or the lecture id for file-level problems.
    pub at: String,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.at, self.message)
    }
}

/// What the validator needs besides the lecture itself.
#[derive(Debug, Default)]
pub struct Context<'a> {
    /// The keys written on each item, by item id (from the raw YAML).
    pub raw_keys: BTreeMap<String, Vec<String>>,
    /// Item ids Mohamed approved (`content/approved.txt`).
    pub approved: Option<&'a BTreeSet<String>>,
}

pub fn lecture(lec: &Lecture, cx: &Context<'_>) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut bad = |at: &str, message: String| out.push(Problem { at: at.to_string(), message });
    let lid = lec.lecture.as_str();

    if lec.schema != content::SCHEMA {
        bad(lid, format!("schema is {:?}, expected {:?}", lec.schema, content::SCHEMA));
    }
    if !is_lecture_id(lid) {
        bad(lid, "lecture id must look like hb.L01".into());
    }
    let [first, last] = lec.book.pages;
    if first == 0 || first > last {
        bad(lid, format!("book pages {first}-{last} are not a range"));
    }
    let topics: BTreeSet<&str> = lec.topics.iter().map(|t| t.id.as_str()).collect();
    if topics.len() != lec.topics.len() {
        bad(lid, "topic ids repeat".into());
    }

    let mut seen = BTreeSet::new();
    for item in &lec.items {
        let at = item.id.as_str();
        if !seen.insert(at) {
            bad(at, "id is used twice".into());
        }
        match at.strip_prefix(lid).and_then(|r| r.strip_prefix('.')) {
            Some(n) if n.len() == 3 && n.bytes().all(|b| b.is_ascii_digit()) => {}
            _ => bad(at, format!("id must be {lid}.NNN (three digits)")),
        }
        if let Some(keys) = cx.raw_keys.get(at) {
            let own = content::allowed_keys(item.body.kind()).unwrap_or(&[]);
            for k in keys {
                if !own.contains(&k.as_str()) && !content::SHARED_KEYS.contains(&k.as_str()) {
                    bad(at, format!("unknown key `{k}` for kind {}", item.body.kind()));
                }
            }
        }
        match &item.topic {
            Some(t) if !topics.is_empty() && !topics.contains(t.as_str()) => {
                bad(at, format!("topic `{t}` is not in the lecture's topics"));
            }
            None if !topics.is_empty() => bad(at, "topic is missing".into()),
            _ => {}
        }
        if item.src.page < first || item.src.page > last {
            bad(at, format!("src.page {} is outside the lecture (pages {first}-{last})", item.src.page));
        }
        if item.status == Status::Approved && !cx.approved.is_some_and(|a| a.contains(at)) {
            bad(
                at,
                "status approved: only Mohamed approves (add the id to content/approved.txt from his export)"
                    .into(),
            );
        }
        if let Some(u) = &item.update
            && (u.text.trim().is_empty() || u.reference.trim().is_empty())
        {
            bad(at, "update needs both text and ref".into());
        }
        for message in body_problems(&item.body) {
            bad(at, message);
        }
        for message in item_picture_problems(item) {
            bad(at, message);
        }
    }
    out
}

/// An item's own picture (#1154): a book figure on a card, mcq or case, with
/// sound marks. Whether the file is one of the lecture's lesson figures is
/// checked with the lessons ([`crate::lesson::validate`]).
pub fn item_picture_problems(item: &Item) -> Vec<String> {
    let mut out = Vec::new();
    let Some(p) = &item.image else {
        if !item.marks.is_empty() {
            out.push("marks need an image".into());
        }
        return out;
    };
    if !content::PICTURE_KINDS.contains(&item.body.kind()) {
        out.push(format!("only a card, mcq or case carries an image, not a {}", item.body.kind()));
    }
    if !p.book {
        out.push("an item's image is a book figure (book: true), the same file as a lesson figure".into());
    }
    out.extend(crate::lesson::picture_problems(p));
    out.extend(crate::lesson::mark_problems(&item.marks, true));
    out
}

fn body_problems(body: &Body) -> Vec<String> {
    let mut p = Vec::new();
    let empty = |s: &str| s.trim().is_empty();
    match body {
        Body::Card { front, back } => {
            if empty(front) || empty(back) {
                p.push("card needs front and back".into());
            }
        }
        Body::Cloze { text, .. } => match content::cloze_blanks(text) {
            None => p.push("cloze braces are malformed; use {{c1::answer}}".into()),
            Some(b) if b.is_empty() => p.push("cloze has no {{cN::…}} blank".into()),
            Some(b) => {
                let nums: BTreeSet<u32> = b.iter().map(|(n, _)| *n).collect();
                if nums.len() > 3 {
                    p.push("cloze has more than 3 blanks (c1–c3)".into());
                }
            }
        },
        Body::Mcq { stem, options, answer, why } => {
            if empty(stem) {
                p.push("mcq stem is empty".into());
            }
            if !(4..=5).contains(&options.len()) {
                p.push(format!("mcq needs 4 or 5 options, has {}", options.len()));
            }
            if *answer >= options.len() {
                p.push(format!("mcq answer {answer} is out of range"));
            }
            if why.len() != options.len() {
                p.push("mcq needs one `why` line per option".into());
            }
            let distinct: BTreeSet<&str> = options.iter().map(|o| o.trim()).collect();
            if distinct.len() != options.len() {
                p.push("mcq options repeat".into());
            }
        }
        Body::Enumerate { prompt, points, pass, hook } => {
            if empty(prompt) || points.len() < 2 {
                p.push("enumerate needs a prompt and at least 2 points".into());
            }
            if *pass == 0 || *pass > points.len() {
                p.push(format!("enumerate pass {pass} must be 1..={}", points.len()));
            }
            if hook.as_deref().is_some_and(empty) {
                p.push("enumerate hook is empty".into());
            }
        }
        Body::Essay { prompt, outline } => {
            if empty(prompt) || outline.is_empty() {
                p.push("essay needs a prompt and an outline".into());
            }
        }
        Body::Case { stem, steps } => {
            if empty(stem) || steps.is_empty() {
                p.push("case needs a stem and at least one step".into());
            }
            for (i, s) in steps.iter().enumerate() {
                match (s.options.len(), s.answer) {
                    (0, None) => {}
                    (0, Some(_)) => p.push(format!("case step {} has an answer but no options", i + 1)),
                    (_, None) => p.push(format!("case step {} has options but no answer", i + 1)),
                    (n, Some(a)) if a >= n => p.push(format!("case step {} answer is out of range", i + 1)),
                    _ => {}
                }
                if empty(&s.why) {
                    p.push(format!("case step {} needs `why`", i + 1));
                }
            }
        }
        Body::Table { columns, rows, blanks, .. } => {
            if columns.len() < 2 || rows.is_empty() {
                p.push("table needs at least 2 columns and 1 row".into());
            }
            for (i, r) in rows.iter().enumerate() {
                if r.len() != columns.len() {
                    p.push(format!("table row {} has {} cells, expected {}", i + 1, r.len(), columns.len()));
                }
            }
            if blanks.is_empty() {
                p.push("table has no blanks".into());
            }
            for [r, c] in blanks {
                if *r >= rows.len() || *c >= columns.len() || *c == 0 {
                    p.push(format!("table blank [{r}, {c}] is outside the grid (column 0 is the label)"));
                }
            }
        }
        Body::Order { steps, .. } => {
            if steps.len() < 3 {
                p.push("order needs at least 3 steps".into());
            }
        }
        Body::Calc { calculator, inputs, expect, .. } => {
            if inputs.is_empty() {
                p.push("calc needs inputs".into());
            }
            match crate::calc::spec(calculator) {
                None => p.push(format!("unknown calculator `{calculator}`")),
                Some(spec) => {
                    // Compute it: the inputs must be complete and `expect` must agree.
                    let inputs = crate::calc::Inputs::for_spec(spec, inputs);
                    for key in crate::calc::unknown_inputs(spec, &inputs) {
                        p.push(format!("calc input `{key}` is not used by `{calculator}`"));
                    }
                    match spec.run(&inputs) {
                        Err(e) => p.push(format!("calc inputs: {e}")),
                        Ok(o) if !crate::calc::expect_matches(&o, expect) => p.push(format!(
                            "calc expect {expect:?} disagrees with the calculator: {} ({}, band `{}`)",
                            o.display, o.label, o.band
                        )),
                        Ok(_) => {}
                    }
                }
            }
        }
    }
    p
}

/// `hb.L01`: a lowercase chapter slug, `.L`, two digits.
pub fn is_lecture_id(id: &str) -> bool {
    let Some((chapter, num)) = id.split_once(".L") else {
        return false;
    };
    !chapter.is_empty()
        && chapter.bytes().all(|b| b.is_ascii_lowercase())
        && num.len() == 2
        && num.bytes().all(|b| b.is_ascii_digit())
}

/// Checks `content/calc-quotes.yaml`: the schema, a calculator the app has,
/// a listed lecture whose pages hold the page, a quote, and one quote per
/// calculator and page. That the words are on the page is `batna-tools
/// refs`' job, like an item's quote.
pub fn calc_quotes(q: &content::CalcQuotes, index: &content::Index) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut bad = |at: String, message: String| out.push(Problem { at, message });
    if q.schema != content::CALC_QUOTES_SCHEMA {
        bad(
            "calc-quotes".into(),
            format!("schema is {:?}, expected {:?}", q.schema, content::CALC_QUOTES_SCHEMA),
        );
    }
    let mut seen = BTreeSet::new();
    for c in &q.quotes {
        let at = content::calc_quote_ref(&c.calculator, c.page);
        if crate::calc::spec(&c.calculator).is_none() {
            bad(at.clone(), format!("no calculator {:?}", c.calculator));
        }
        match index.lectures.iter().find(|e| e.id == c.lecture) {
            None => bad(at.clone(), format!("lecture {} is not in content/index.yaml", c.lecture)),
            Some(e) if !(e.pages[0]..=e.pages[1]).contains(&c.page) => bad(
                at.clone(),
                format!("p{} is outside {} (pp. {}–{})", c.page, c.lecture, e.pages[0], e.pages[1]),
            ),
            Some(_) => {}
        }
        if c.quote.trim().is_empty() {
            bad(at.clone(), "has no quote".into());
        }
        if !seen.insert((c.calculator.clone(), c.page)) {
            bad(at, "is listed twice".into());
        }
    }
    out
}

/// Checks `content/glossary.yaml` (the hover peeks, PARITY.md §6.3): the
/// schema, one entry per spelling, short plain expansions, and no `approved`
/// that Mohamed didn't give (`glossary.TERM` in `content/approved.txt`).
pub fn glossary(g: &crate::glossary::Glossary, approved: Option<&BTreeSet<String>>) -> Vec<Problem> {
    const MAX_WORDS: usize = 25;
    let mut out = Vec::new();
    let mut bad = |at: &str, message: String| out.push(Problem { at: at.to_string(), message });
    if g.schema != crate::glossary::SCHEMA {
        bad("glossary", format!("schema is {:?}, expected {:?}", g.schema, crate::glossary::SCHEMA));
    }
    // Abbreviations match exactly and drug names ignore case
    // (`Glossary::find`), so `HBIG` and `HBIg` may both be abbreviations,
    // but a drug name must not collide with anything in any case.
    // A course's own sense (`course: neph`) may share a spelling with the
    // general entry and with other courses' senses, never with another sense
    // in the same course.
    let (mut exact, mut lower, mut drugs) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut scoped = BTreeSet::new();
    for e in &g.entries {
        let at = e.id();
        let drug = e.kind == crate::glossary::Kind::Drug;
        if let Some(c) = &e.course
            && (c.is_empty() || !c.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
        {
            bad(&at, format!("course `{c}` must be a course id like `neph`"));
        }
        for s in e.spellings() {
            if s.trim().is_empty() || s.chars().any(char::is_whitespace) {
                bad(&at, format!("`{s}` must be one word (the pointer picks one word)"));
            }
            if let Some(c) = &e.course {
                if !scoped.insert((c.clone(), s.to_string())) {
                    bad(&at, format!("`{s}` is listed twice for course {c}"));
                }
                continue;
            }
            let low = s.to_lowercase();
            let clash =
                !exact.insert(s.to_string()) || drugs.contains(&low) || (drug && lower.contains(&low));
            if clash {
                bad(&at, format!("`{s}` is listed twice"));
            }
            lower.insert(low.clone());
            if drug {
                drugs.insert(low);
            }
        }
        let words = e.text.split_whitespace().count();
        if words == 0 {
            bad(&at, "text is empty".into());
        } else if words > MAX_WORDS {
            bad(&at, format!("text is {words} words; keep it to {MAX_WORDS} or fewer"));
        }
        if e.status == Status::Approved && !approved.is_some_and(|a| a.contains(&at)) {
            bad(&at, "status approved: only Mohamed approves (add the id to content/approved.txt)".into());
        }
    }
    // `plain:` words are capitals that aren't abbreviations; one that has an
    // entry would hide nothing and confuse the next reader.
    let mut plain = BTreeSet::new();
    for p in &g.plain {
        if exact.contains(p.as_str()) {
            bad("glossary.plain", format!("`{p}` has an entry, so it isn't plain"));
        }
        if !plain.insert(p.as_str()) {
            bad("glossary.plain", format!("`{p}` is listed twice"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Lecture;

    fn lec(yaml: &str) -> Lecture {
        serde_yaml_ng::from_str(yaml).unwrap()
    }

    const HEAD: &str = "schema: batna.content/v1\nlecture: hb.L01\ntitle: Liver Cirrhosis\nbook: { chapter: 1, pages: [1, 22] }\n";

    #[test]
    fn the_sample_fixture_is_clean() {
        let raw = include_str!("../../../fixtures/content/sample.yaml");
        let l: Lecture = serde_yaml_ng::from_str(raw).unwrap();
        let problems = lecture(&l, &Context::default());
        assert!(problems.is_empty(), "{problems:#?}");
        let kinds: BTreeSet<_> = l.items.iter().map(|i| i.body.kind()).collect();
        assert_eq!(kinds.len(), 9, "the fixture should show every kind");
    }

    #[test]
    fn a_picture_goes_on_a_card_mcq_or_case_as_a_book_figure() {
        let l = lec(&format!(
            "{HEAD}items:
  - {{ id: hb.L01.001, kind: card, front: Sign?, back: b, image: {{ file: hb/a.webp, book: true }}, marks: [{{ at: [0.5, 0.5], label: here }}], src: {{ page: 2 }}, status: unverified }}
  - {{ id: hb.L01.002, kind: cloze, text: '{{{{c1::x}}}}', image: {{ file: hb/a.webp, book: true }}, src: {{ page: 2 }}, status: unverified }}
  - {{ id: hb.L01.003, kind: card, front: a, back: b, image: {{ file: hb/a.gif }}, src: {{ page: 2 }}, status: unverified }}
  - {{ id: hb.L01.004, kind: card, front: a, back: b, marks: [{{ at: [0.5, 0.5], label: x }}], src: {{ page: 2 }}, status: unverified }}
  - {{ id: hb.L01.005, kind: mcq, stem: s, options: [a, b, c, d], answer: 0, why: [w, x, y, z], image: {{ file: hb/a.webp, book: true }}, marks: [{{ at: [1.5, 0.5], label: x }}], src: {{ page: 2 }}, status: unverified }}
"
        ));
        let msgs: Vec<String> = lecture(&l, &Context::default()).iter().map(|p| p.to_string()).collect();
        let at = |id: &str| msgs.iter().filter(|m| m.starts_with(id)).count();
        assert_eq!(at("hb.L01.001"), 0, "{msgs:#?}");
        assert!(msgs.iter().any(|m| m.starts_with("hb.L01.002") && m.contains("not a cloze")), "{msgs:#?}");
        assert!(msgs.iter().any(|m| m.starts_with("hb.L01.003") && m.contains("book: true")), "{msgs:#?}");
        assert!(msgs.iter().any(|m| m.starts_with("hb.L01.003") && m.contains(".webp, .jpg or .png")));
        assert!(msgs.iter().any(|m| m.starts_with("hb.L01.004") && m.contains("marks need an image")));
        assert!(msgs.iter().any(|m| m.starts_with("hb.L01.005") && m.contains("fractions of the picture")));
        // It reads back as written.
        let back: Lecture = serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&l).unwrap()).unwrap();
        assert_eq!(back, l);
    }

    #[test]
    fn catches_the_common_mistakes() {
        let l = lec(&format!(
            "{HEAD}items:
  - {{ id: hb.L01.001, kind: card, front: a, back: b, src: {{ page: 30 }}, status: unverified }}
  - {{ id: hb.L01.001, kind: mcq, stem: s, options: [a, b, c], answer: 3, why: [x], src: {{ page: 2 }}, status: approved }}
  - {{ id: hb.L1.2, kind: cloze, text: 'no blank', src: {{ page: 2 }}, status: unverified }}
"
        ));
        let msgs: Vec<String> = lecture(&l, &Context::default()).iter().map(|p| p.to_string()).collect();
        let has = |s: &str| msgs.iter().any(|m| m.contains(s));
        assert!(has("outside the lecture"), "{msgs:#?}");
        assert!(has("used twice"));
        assert!(has("4 or 5 options"));
        assert!(has("out of range"));
        assert!(has("one `why` line"));
        assert!(has("only Mohamed approves"));
        assert!(has("three digits"));
        assert!(has("no {{cN"));
    }

    #[test]
    fn enumerate_hooks_read_and_must_say_something() {
        let l = lec(&format!(
            "{HEAD}items:
  - {{ id: hb.L01.001, kind: enumerate, prompt: p, points: [a, b], pass: 2, hook: 'حكاية', src: {{ page: 3 }}, status: unverified }}
  - {{ id: hb.L01.002, kind: enumerate, prompt: p, points: [a, b], pass: 2, hook: ' ', src: {{ page: 3 }}, status: unverified }}
"
        ));
        assert!(matches!(&l.items[0].body, Body::Enumerate { hook: Some(h), .. } if h == "حكاية"));
        let msgs: Vec<String> = lecture(&l, &Context::default()).iter().map(|p| p.to_string()).collect();
        assert_eq!(msgs.len(), 1, "{msgs:#?}");
        assert!(msgs[0].contains("hook is empty"));
    }

    #[test]
    fn approved_is_fine_when_mohamed_approved_it() {
        let l = lec(&format!(
            "{HEAD}items:\n  - {{ id: hb.L01.001, kind: card, front: a, back: b, src: {{ page: 3 }}, status: approved }}\n"
        ));
        let ok: BTreeSet<String> = ["hb.L01.001".to_string()].into();
        let cx = Context { approved: Some(&ok), ..Context::default() };
        assert!(lecture(&l, &cx).is_empty());
    }

    #[test]
    fn unknown_keys_are_flagged_and_long_quotes_are_not() {
        let l = lec(&format!(
            "{HEAD}items:\n  - {{ id: hb.L01.001, kind: card, front: a, back: b, src: {{ page: 3, quote: '{}' }}, status: unverified }}\n",
            "word ".repeat(120)
        ));
        let cx = Context {
            raw_keys: [("hb.L01.001".to_string(), vec!["front".into(), "back".into(), "answer".into()])]
                .into(),
            ..Context::default()
        };
        let msgs: Vec<String> = lecture(&l, &cx).iter().map(|p| p.to_string()).collect();
        assert!(msgs.iter().any(|m| m.contains("unknown key `answer`")), "{msgs:#?}");
        // No length cap on quotes (Mohamed, 2026-10-29): a whole table is fine.
        assert!(!msgs.iter().any(|m| m.contains("quote")), "{msgs:#?}");
    }

    #[test]
    fn calc_items_are_computed() {
        let item = |inputs: &str, expect: &str| {
            lec(&format!(
                "{HEAD}items:\n  - id: hb.L01.001\n    kind: calc\n    calculator: child_pugh\n    stem: s\n    \
                 inputs: {{ {inputs} }}\n    expect: {expect}\n    interpret: i\n    src: {{ page: 14 }}\n    status: unverified\n"
            ))
        };
        let full =
            "encephalopathy: none, ascites: mild, bilirubin_mg_dl: 2.5, albumin_g_dl: 3.0, pt_prolonged_s: 3";
        assert!(lecture(&item(full, "Child-Pugh B (8 points)"), &Context::default()).is_empty());
        let p = lecture(&item(full, "Child-Pugh C (10 points)"), &Context::default());
        assert!(p[0].message.contains("disagrees"), "{p:?}");
        let p = lecture(&item("encephalopathy: none, ascites: mild, bili: 2.5", "x"), &Context::default());
        assert!(p.iter().any(|p| p.message.contains("`bili` is not used")), "{p:?}");
        assert!(p.iter().any(|p| p.message.contains("missing input")), "{p:?}");
    }

    /// The messages for one item written after `HEAD` (topics: none).
    fn item_problems(item: &str) -> Vec<String> {
        let l = lec(&format!(
            "{HEAD}items:\n  - {{ id: hb.L01.001, src: {{ page: 3 }}, status: unverified, {item} }}\n"
        ));
        lecture(&l, &Context::default()).into_iter().map(|p| p.message).collect()
    }

    #[test]
    fn every_kind_has_its_own_rules() {
        let cases: &[(&str, &[&str])] = &[
            ("kind: card, front: ' ', back: b", &["card needs front and back"]),
            ("kind: cloze, text: '{{c1::a'", &["cloze braces are malformed; use {{c1::answer}}"]),
            (
                "kind: cloze, text: '{{c1::a}} {{c2::b}} {{c3::c}} {{c4::d}}'",
                &["cloze has more than 3 blanks (c1–c3)"],
            ),
            (
                "kind: mcq, stem: ' ', options: [a, b, c, ' a'], answer: 0, why: [w, x, y, z]",
                &["mcq stem is empty", "mcq options repeat"],
            ),
            (
                "kind: enumerate, prompt: p, points: [a], pass: 0",
                &["enumerate needs a prompt and at least 2 points", "enumerate pass 0 must be 1..=1"],
            ),
            ("kind: essay, prompt: p, outline: []", &["essay needs a prompt and an outline"]),
            ("kind: case, stem: s, steps: []", &["case needs a stem and at least one step"]),
            (
                "kind: case, stem: s, steps: [{ question: q, answer: 0, why: w }, { question: q, options: [a], why: w }, { question: q, options: [a], answer: 1, why: ' ' }]",
                &[
                    "case step 1 has an answer but no options",
                    "case step 2 has options but no answer",
                    "case step 3 answer is out of range",
                    "case step 3 needs `why`",
                ],
            ),
            (
                "kind: table, title: t, columns: [a], rows: [[x, y]], blanks: [[0, 0], [5, 1]]",
                &[
                    "table needs at least 2 columns and 1 row",
                    "table row 1 has 2 cells, expected 1",
                    "table blank [0, 0] is outside the grid (column 0 is the label)",
                    "table blank [5, 1] is outside the grid (column 0 is the label)",
                ],
            ),
            ("kind: table, title: t, columns: [a, b], rows: [[x, y]], blanks: []", &["table has no blanks"]),
            ("kind: order, prompt: p, steps: [a, b]", &["order needs at least 3 steps"]),
            (
                "kind: calc, calculator: no_such, stem: s, inputs: {}, expect: x, interpret: i",
                &["calc needs inputs", "unknown calculator `no_such`"],
            ),
            (
                "kind: card, front: f, back: b, update: { text: ' ', ref: AASLD 2023 }",
                &["update needs both text and ref"],
            ),
        ];
        for (item, want) in cases {
            assert_eq!(item_problems(item), *want, "{item}");
        }
    }

    #[test]
    fn lecture_level_rules() {
        let l = lec(
            "schema: batna.content/v0\nlecture: HB.1\ntitle: t\nbook: { chapter: 1, pages: [5, 2] }\ntopics:\n  - { id: a, title: A }\n  - { id: a, title: A again }\nitems:\n  - { id: HB.1.001, kind: card, front: f, back: b, topic: z, src: { page: 3 }, status: unverified }\n  - { id: HB.1.002, kind: card, front: f, back: b, src: { page: 3 }, status: unverified }\n",
        );
        let msgs: Vec<String> = lecture(&l, &Context::default()).iter().map(|p| p.to_string()).collect();
        assert_eq!(
            msgs,
            [
                "HB.1: schema is \"batna.content/v0\", expected \"batna.content/v1\"",
                "HB.1: lecture id must look like hb.L01",
                "HB.1: book pages 5-2 are not a range",
                "HB.1: topic ids repeat",
                "HB.1.001: topic `z` is not in the lecture's topics",
                "HB.1.001: src.page 3 is outside the lecture (pages 5-2)",
                "HB.1.002: topic is missing",
                "HB.1.002: src.page 3 is outside the lecture (pages 5-2)",
            ]
        );
    }

    #[test]
    fn glossary_text_schema_and_plain_words() {
        let g: crate::glossary::Glossary = serde_yaml_ng::from_str(&format!(
            "schema: batna.glossary/v0
entries:
  - {{ term: HE, kind: abbr, text: ' ', status: unverified }}
  - {{ term: SBP, kind: abbr, text: '{}', status: unverified }}
  - {{ term: Lasix, kind: drug, text: Furosemide, status: unverified }}
  - {{ term: LASIX, kind: abbr, text: Clash with the drug, status: unverified }}
plain: [HE, Child, Child]",
            "word ".repeat(26)
        ))
        .unwrap();
        let msgs: Vec<String> = glossary(&g, None).iter().map(|p| p.message.clone()).collect();
        assert_eq!(
            msgs,
            [
                "schema is \"batna.glossary/v0\", expected \"batna.glossary/v1\"",
                "text is empty",
                "text is 26 words; keep it to 25 or fewer",
                "`LASIX` is listed twice",
                "`HE` has an entry, so it isn't plain",
                "`Child` is listed twice",
            ]
        );
    }

    #[test]
    fn lecture_ids() {
        assert!(is_lecture_id("hb.L01"));
        assert!(is_lecture_id("gi.L11"));
        assert!(!is_lecture_id("hb.L1"));
        assert!(!is_lecture_id("HB.L01"));
        assert!(!is_lecture_id("hbL01"));
    }

    #[test]
    fn glossary_rules() {
        let g: crate::glossary::Glossary = serde_yaml_ng::from_str(
            "schema: batna.glossary/v1
entries:
  - { term: HE, kind: abbr, text: Hepatic encephalopathy, status: unverified }
  - { term: he, kind: drug, text: Duplicate spelling, status: unverified }
  - { term: two words, kind: drug, text: x, status: approved }",
        )
        .unwrap();
        let msgs: Vec<String> = glossary(&g, None).iter().map(|p| p.to_string()).collect();
        assert!(msgs.iter().any(|m| m.contains("listed twice")), "{msgs:#?}");
        assert!(msgs.iter().any(|m| m.contains("must be one word")));
        assert!(msgs.iter().any(|m| m.contains("only Mohamed approves")));
        let ok: BTreeSet<String> = ["glossary.two words".to_string()].into();
        assert_eq!(glossary(&g, Some(&ok)).len(), 2);
    }
}
