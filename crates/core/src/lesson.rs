//! Lessons, `batna.lesson/v1`: the teaching layer on top of the study items.
//!
//! One YAML file per lecture (`content/lessons/hb/L01.yaml`) holds a few
//! short lessons. A lesson is a list of steps: explanations in Egyptian
//! Arabic (medical terms stay English), drawn SVG figures, "why?" reveals,
//! mnemonics, asides (a tip, an exam trap, a joke), and small interactive
//! checks between them (choose, order, match, sort, a mini case, a table
//! with hidden cells). A `try` step and the lesson's `practice` list point at
//! the lecture's existing items, so a lesson hands over to spaced review.
//!
//! The rules for writing lessons are in `docs/LESSONS.md`; [`validate`]
//! enforces them.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::content::{CaseStep, Lecture, Status};
use crate::validate::{MAX_QUOTE_WORDS, Problem};

pub const SCHEMA: &str = "batna.lesson/v1";

/// The lessons of one lecture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonFile {
    pub schema: String,
    /// `hb.L01`, the lecture these lessons teach.
    pub lecture: String,
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    /// `hb.L01.lesson1`. Never reused or renumbered.
    pub id: String,
    /// Egyptian Arabic.
    pub title: String,
    /// One line that makes him want to open it.
    pub hook: String,
    /// About how long it takes, in minutes.
    pub minutes: u32,
    /// Printed book pages it covers.
    pub pages: [u32; 2],
    /// Lecture topic ids it covers (for Today and the planner).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<String>,
    /// "After this lesson you can…", in Arabic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<String>,
    pub steps: Vec<Step>,
    /// The take-aways shown at the end.
    pub recap: Vec<String>,
    /// Item ids of this lecture to practise afterwards (spaced review).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub practice: Vec<String>,
    pub status: Status,
}

/// One screen of a lesson.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub body: StepBody,
    /// The printed book page this step teaches from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// At most 25 words from that page, for checking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StepBody {
    /// Prose in [`spans`] markup: paragraphs, `- ` bullets, `**bold**`.
    Explain {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        text: String,
    },
    /// A diagram drawn as inline SVG with theme colours only.
    Figure {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        svg: String,
        caption: String,
        /// What the figure shows, for a screen reader.
        alt: String,
    },
    /// A question to think about first; the answer opens on a tap.
    Reveal { prompt: String, answer: String },
    /// Letters to remember a list by. Each letter opens its line on a tap.
    Mnemonic {
        /// The word or phrase the letters make (`"VAMPIRE"`).
        word: String,
        parts: Vec<MnemonicPart>,
        /// Why it sticks, in Arabic (the story, the picture, the joke).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hook: Option<String>,
    },
    /// A side note in a coloured box.
    Aside { tone: Tone, text: String },
    /// Where a current guideline differs from the book. The book stays the answer.
    Update {
        text: String,
        #[serde(rename = "ref")]
        reference: String,
    },
    /// Single best answer, feedback on every option.
    Check { question: String, options: Vec<String>, answer: usize, why: Vec<String> },
    /// Put the steps in order (written in the correct order).
    Order {
        prompt: String,
        steps: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        why: Option<String>,
    },
    /// Match each left side to its right side (written matched).
    Match {
        prompt: String,
        pairs: Vec<[String; 2]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        why: Option<String>,
    },
    /// Drop each card into its bin.
    Sort {
        prompt: String,
        bins: Vec<String>,
        cards: Vec<SortCard>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        why: Option<String>,
    },
    /// A mini case: each step reveals data and asks one decision.
    Case { stem: String, steps: Vec<CaseStep> },
    /// A table; `hide` cells (`[row, column]`) open on a tap.
    Table {
        title: String,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        hide: Vec<[usize; 2]>,
    },
    /// Practise these items of the lecture right here, graded into review.
    Try { items: Vec<String> },
}

impl StepBody {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Explain { .. } => "explain",
            Self::Figure { .. } => "figure",
            Self::Reveal { .. } => "reveal",
            Self::Mnemonic { .. } => "mnemonic",
            Self::Aside { .. } => "aside",
            Self::Update { .. } => "update",
            Self::Check { .. } => "check",
            Self::Order { .. } => "order",
            Self::Match { .. } => "match",
            Self::Sort { .. } => "sort",
            Self::Case { .. } => "case",
            Self::Table { .. } => "table",
            Self::Try { .. } => "try",
        }
    }

    /// Steps that ask something and count towards the lesson score.
    pub fn is_question(&self) -> bool {
        matches!(
            self,
            Self::Check { .. }
                | Self::Order { .. }
                | Self::Match { .. }
                | Self::Sort { .. }
                | Self::Case { .. }
        )
    }
}

/// Every key a step of each kind may have, shared keys included. Serde
/// can't reject unknown keys on a flattened enum, so [`validate`] checks.
pub fn allowed_keys(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "explain" => &["title", "text"],
        "figure" => &["title", "svg", "caption", "alt"],
        "reveal" => &["prompt", "answer"],
        "mnemonic" => &["word", "parts", "hook"],
        "aside" => &["tone", "text"],
        "update" => &["text", "ref"],
        "check" => &["question", "options", "answer", "why"],
        "order" => &["prompt", "steps", "why"],
        "match" => &["prompt", "pairs", "why"],
        "sort" => &["prompt", "bins", "cards", "why"],
        "case" => &["stem", "steps"],
        "table" => &["title", "columns", "rows", "hide"],
        "try" => &["items"],
        _ => return None,
    })
}

pub const SHARED_STEP_KEYS: &[&str] = &["kind", "page", "quote"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MnemonicPart {
    /// The letter (or syllable) as it appears in the word.
    pub key: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SortCard {
    pub text: String,
    /// Index into `bins`.
    pub bin: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// A practical pearl.
    Tip,
    /// "The examiner loves this": how it's asked.
    Exam,
    /// A classic mix-up.
    Trap,
    /// An everyday picture that explains the mechanism.
    Analogy,
    /// A light Egyptian joke that makes it stick (tahshish).
    Joke,
}

// ---------- the text markup ----------

/// A run of lesson text with one look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    /// Latin script (English terms, numbers with units): isolated left-to-right.
    pub latin: bool,
}

/// A block of lesson text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Para(Vec<Span>),
    Bullets(Vec<Vec<Span>>),
}

/// Splits lesson text into paragraphs (blank lines) and bullet lists
/// (lines starting `- `), with `**bold**` and Latin runs marked.
pub fn blocks(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    for chunk in text.split("\n\n").map(str::trim).filter(|c| !c.is_empty()) {
        let lines: Vec<&str> = chunk.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if lines.iter().all(|l| l.starts_with("- ")) {
            out.push(Block::Bullets(lines.iter().map(|l| spans(&l[2..])).collect()));
        } else {
            let mut para = Vec::new();
            let mut list = Vec::new();
            for l in lines {
                if let Some(b) = l.strip_prefix("- ") {
                    list.push(spans(b));
                } else {
                    if !list.is_empty() {
                        out.push(Block::Para(std::mem::take(&mut para)));
                        out.push(Block::Bullets(std::mem::take(&mut list)));
                    }
                    if !para.is_empty() {
                        para.push(Span { text: " ".into(), bold: false, latin: false });
                    }
                    para.extend(spans(l));
                }
            }
            if !para.is_empty() {
                out.push(Block::Para(para));
            }
            if !list.is_empty() {
                out.push(Block::Bullets(list));
            }
        }
    }
    out.retain(|b| !matches!(b, Block::Para(p) if p.is_empty()));
    out
}

/// One line of lesson text as spans: `**bold**`, and runs of Latin script
/// apart from Arabic so the UI can isolate them (`<bdi dir="ltr">`).
pub fn spans(line: &str) -> Vec<Span> {
    // Script runs come from the whole line, so bold can't split an English
    // phrase ("**Genome:** single-stranded") into runs that flip.
    let mut plain = String::new();
    let mut bold_at: Vec<bool> = Vec::new();
    for (i, part) in line.split("**").enumerate() {
        plain.push_str(part);
        bold_at.extend(part.chars().map(|_| i % 2 == 1));
    }
    let mut out: Vec<Span> = Vec::new();
    let mut at = 0;
    for (text, latin) in script_runs(&plain) {
        for c in text.chars() {
            let bold = bold_at[at];
            at += 1;
            match out.last_mut() {
                Some(last) if last.bold == bold && last.latin == latin => last.text.push(c),
                _ => out.push(Span { text: c.to_string(), bold, latin }),
            }
        }
    }
    out
}

/// Runs of Latin and non-Latin text. Spaces, digits and punctuation between
/// two Latin words stay in the Latin run ("Child-Pugh class C"), so a term
/// is isolated whole; elsewhere they go with the Arabic around them.
fn script_runs(text: &str) -> Vec<(String, bool)> {
    #[derive(Clone, Copy, PartialEq)]
    enum C {
        Latin,
        Arabic,
        Neutral,
    }
    let class = |c: char| {
        if c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(&c) || "µβαγκ".contains(c) {
            C::Latin
        } else if ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{0750}'..='\u{077F}').contains(&c) {
            C::Arabic
        } else {
            C::Neutral
        }
    };
    let chars: Vec<char> = text.chars().collect();
    let strong: Vec<C> = chars.iter().map(|&c| class(c)).collect();
    let mut resolved = strong.clone();
    // Each run of neutral chars (spaces, digits, signs) between two strong
    // chars: between two Latin words it is Latin ("Child-Pugh class C").
    // Right after a Latin word, the numbers, units and closing brackets that
    // continue it are Latin ("Zone 3", "PaO2 <80", "(MELD)"), but not the
    // sentence's own punctuation. Right before one, an opening bracket or
    // quote is Latin ("(morphology").
    let mut i = 0;
    while i < chars.len() {
        if strong[i] != C::Neutral {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && strong[i] == C::Neutral {
            i += 1;
        }
        let before = start.checked_sub(1).map(|k| strong[k]);
        let after = strong.get(i).copied();
        let run = start..i;
        for k in run.clone() {
            resolved[k] = C::Arabic;
        }
        if before == Some(C::Latin) && after == Some(C::Latin) {
            for k in run {
                resolved[k] = C::Latin;
            }
            continue;
        }
        if before == Some(C::Latin) {
            let continues = |c: char| c.is_ascii_digit() || c == ' ' || "%()[]+-−–≥≤<>/.×±°'\"·,".contains(c);
            let mut end = start;
            while end < i && continues(chars[end]) {
                end += 1;
            }
            // Brackets stay only in pairs: an opening one must close inside
            // the run ("Low risk (≤ 2%)"), a closing one must close a bracket
            // the Latin term opened ("(morphology)"). The others are Arabic's
            // ("(مع albumin 20%)").
            let mut term = start;
            while term > 0 && resolved[term - 1] == C::Latin {
                term -= 1;
            }
            let mut opened = chars[term..start].iter().fold(0usize, |d, &c| match c {
                '(' | '[' => d + 1,
                ')' | ']' => d.saturating_sub(1),
                _ => d,
            });
            let mut open: Vec<usize> = Vec::new();
            for (k, &c) in chars.iter().enumerate().take(end).skip(start) {
                match c {
                    '(' | '[' => open.push(k),
                    ')' | ']' if open.pop().is_none() => {
                        if opened == 0 {
                            end = k;
                            break;
                        }
                        opened -= 1;
                    }
                    _ => {}
                }
            }
            if let Some(&first) = open.first() {
                end = first;
            }
            // Don't take the sentence's punctuation or trailing spaces.
            while end > start && " .,-–·".contains(chars[end - 1]) {
                end -= 1;
            }
            for r in &mut resolved[start..end] {
                *r = C::Latin;
            }
        }
        if after == Some(C::Latin) {
            // A number that starts the term is part of it too ("20 cm",
            // "1.5 ml/kg/h", "3–4 L").
            let mut from = i;
            while from > start
                && (chars[from - 1].is_ascii_digit() || " .,–-×/~<>≥≤±%([\"'«".contains(chars[from - 1]))
            {
                from -= 1;
            }
            while from < i && !(chars[from].is_ascii_digit() || "<>≥≤±~([\"'«".contains(chars[from])) {
                from += 1;
            }
            for r in &mut resolved[from..i] {
                *r = C::Latin;
            }
        }
    }
    // Ranges and ratios ("5–6", "1/3") read right to left in an Arabic
    // line ("6–5"), so they are isolated too.
    let mut k = 0;
    while k < chars.len() {
        if !chars[k].is_ascii_digit() || resolved[k] == C::Latin {
            k += 1;
            continue;
        }
        let start = k;
        let mut end = k;
        let mut joined = false;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
            if end + 1 < chars.len() && "–-/:×".contains(chars[end]) && chars[end + 1].is_ascii_digit() {
                joined = true;
                end += 1;
            } else if end + 1 < chars.len() && chars[end] == '.' && chars[end + 1].is_ascii_digit() {
                end += 1;
            }
        }
        if joined {
            for r in &mut resolved[start..end] {
                *r = C::Latin;
            }
        }
        k = end;
    }
    let mut out: Vec<(String, bool)> = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        let latin = resolved[i] == C::Latin;
        match out.last_mut() {
            Some((s, l)) if *l == latin => s.push(c),
            _ => out.push((c.to_string(), latin)),
        }
    }
    out
}

// ---------- checks ----------

/// What the validator needs to know about the lecture a lesson file teaches.
pub struct LessonContext<'a> {
    pub lecture: &'a Lecture,
    /// Ids Mohamed approved (`content/approved.txt`).
    pub approved: Option<&'a BTreeSet<String>>,
    /// The keys written on each step, by `(lesson index, step index)`.
    pub raw_step_keys: Vec<((usize, usize), Vec<String>)>,
}

/// Figures can't be bigger than this (they are drawn, not pasted).
pub const MAX_SVG_BYTES: usize = 24 * 1024;

/// Checks a lesson file against the rules in `docs/LESSONS.md`.
pub fn validate(file: &LessonFile, cx: &LessonContext<'_>) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut bad = |at: &str, message: String| out.push(Problem { at: at.to_string(), message });
    let lec = cx.lecture;
    let lid = file.lecture.as_str();
    if file.schema != SCHEMA {
        bad(lid, format!("schema is {:?}, expected {SCHEMA:?}", file.schema));
    }
    if file.lecture != lec.lecture {
        bad(lid, format!("lessons are for {lid} but the lecture is {}", lec.lecture));
    }
    if file.lessons.is_empty() {
        bad(lid, "no lessons".into());
    }
    let items: BTreeSet<&str> = lec.items.iter().map(|i| i.id.as_str()).collect();
    let topics: BTreeSet<&str> = lec.topics.iter().map(|t| t.id.as_str()).collect();
    let [first, last] = lec.book.pages;
    let mut ids = BTreeSet::new();
    for (li, lesson) in file.lessons.iter().enumerate() {
        let at = lesson.id.as_str();
        if !ids.insert(at) {
            bad(at, "lesson id is used twice".into());
        }
        match at.strip_prefix(lid).and_then(|r| r.strip_prefix(".lesson")) {
            Some(n) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && !n.starts_with('0') => {}
            _ => bad(at, format!("lesson id must be {lid}.lessonN")),
        }
        if lesson.title.trim().is_empty() || lesson.hook.trim().is_empty() {
            bad(at, "title and hook are required".into());
        }
        if !(2..=15).contains(&lesson.minutes) {
            bad(at, format!("minutes is {}: keep a lesson between 2 and 15 minutes", lesson.minutes));
        }
        let [a, b] = lesson.pages;
        if a > b || a < first || b > last {
            bad(at, format!("pages {a}-{b} are not inside the lecture ({first}-{last})"));
        }
        for t in &lesson.topics {
            if !topics.contains(t.as_str()) {
                bad(at, format!("topic `{t}` is not one of the lecture's topics"));
            }
        }
        if lesson.recap.is_empty() {
            bad(at, "recap is empty: end with the take-aways".into());
        }
        if lesson.steps.len() < 3 {
            bad(at, "a lesson needs at least 3 steps".into());
        }
        if !lesson.steps.iter().any(|s| s.body.is_question()) {
            bad(at, "a lesson needs at least one check, order, match, sort or case step".into());
        }
        match lesson.status {
            Status::Approved if !cx.approved.is_some_and(|a| a.contains(at)) => {
                bad(at, "status approved, but Mohamed hasn't approved it (content/approved.txt)".into());
            }
            _ => {}
        }
        for id in &lesson.practice {
            if !items.contains(id.as_str()) {
                bad(at, format!("practice item {id} is not in {lid}"));
            }
        }
        for (si, step) in lesson.steps.iter().enumerate() {
            let sat = format!("{at} step {}", si + 1);
            if let Some((_, keys)) = cx.raw_step_keys.iter().find(|(k, _)| *k == (li, si)) {
                let own = allowed_keys(step.body.kind()).unwrap_or(&[]);
                for k in keys {
                    if !own.contains(&k.as_str()) && !SHARED_STEP_KEYS.contains(&k.as_str()) {
                        bad(&sat, format!("unknown key `{k}` for a {} step", step.body.kind()));
                    }
                }
            }
            if let Some(p) = step.page
                && (p < a || p > b)
            {
                bad(&sat, format!("page {p} is outside the lesson's pages {a}-{b}"));
            }
            if let Some(q) = &step.quote {
                if q.split_whitespace().count() > MAX_QUOTE_WORDS {
                    bad(
                        &sat,
                        format!("quote is longer than {MAX_QUOTE_WORDS} words (book text stays out of git)"),
                    );
                }
                if step.page.is_none() {
                    bad(&sat, "a quote needs its page".into());
                }
            }
            for m in step_problems(&step.body, &items) {
                bad(&sat, m);
            }
        }
    }
    out
}

fn step_problems(body: &StepBody, items: &BTreeSet<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let empty = |s: &str| s.trim().is_empty();
    match body {
        StepBody::Explain { text, .. } | StepBody::Aside { text, .. } => {
            if empty(text) {
                out.push("text is empty".into());
            }
            if text.matches("**").count() % 2 == 1 {
                out.push("`**` is not closed".into());
            }
        }
        StepBody::Figure { svg, caption, alt, .. } => {
            out.extend(svg_problems(svg));
            if empty(caption) || empty(alt) {
                out.push("a figure needs a caption and an alt text".into());
            }
        }
        StepBody::Reveal { prompt, answer } => {
            if empty(prompt) || empty(answer) {
                out.push("prompt and answer are required".into());
            }
        }
        StepBody::Mnemonic { word, parts, .. } => {
            if parts.len() < 2 {
                out.push("a mnemonic needs at least 2 parts".into());
            }
            // The keys spell the word, in order (ignoring spaces and case).
            let norm = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
            let spelled: String = parts.iter().map(|p| norm(&p.key)).collect();
            if spelled != norm(word) {
                out.push(format!("the keys spell {spelled:?}, not the word {word:?}"));
            }
        }
        StepBody::Update { text, reference } => {
            if empty(text) || empty(reference) {
                out.push("an update needs text and ref (guideline and year)".into());
            } else if !reference.chars().any(|c| c.is_ascii_digit()) {
                out.push("ref must name the guideline's year".into());
            }
        }
        StepBody::Check { options, answer, why, .. } => {
            if !(3..=5).contains(&options.len()) {
                out.push(format!("a check has {} options; use 3 to 5", options.len()));
            }
            if *answer >= options.len() {
                out.push("answer is not one of the options".into());
            }
            if why.len() != options.len() {
                out.push("why needs one line per option".into());
            }
            if options.iter().collect::<BTreeSet<_>>().len() != options.len() {
                out.push("two options are the same".into());
            }
        }
        StepBody::Order { steps, .. } => {
            if !(3..=8).contains(&steps.len()) {
                out.push("order needs 3 to 8 steps".into());
            }
        }
        StepBody::Match { pairs, .. } => {
            if !(3..=6).contains(&pairs.len()) {
                out.push("match needs 3 to 6 pairs".into());
            }
            let rights: BTreeSet<&String> = pairs.iter().map(|p| &p[1]).collect();
            if rights.len() != pairs.len() {
                out.push("two pairs have the same right side".into());
            }
        }
        StepBody::Sort { bins, cards, .. } => {
            if !(2..=4).contains(&bins.len()) {
                out.push("sort needs 2 to 4 bins".into());
            }
            if !(3..=10).contains(&cards.len()) {
                out.push("sort needs 3 to 10 cards".into());
            }
            if cards.iter().any(|c| c.bin >= bins.len()) {
                out.push("a card's bin is out of range".into());
            }
            if (0..bins.len()).any(|b| !cards.iter().any(|c| c.bin == b)) {
                out.push("every bin needs a card".into());
            }
        }
        StepBody::Case { steps, .. } => {
            if steps.is_empty() {
                out.push("a case needs steps".into());
            }
            for (i, s) in steps.iter().enumerate() {
                match s.answer {
                    Some(a) if a >= s.options.len() => {
                        out.push(format!("case step {}: answer out of range", i + 1))
                    }
                    None if !s.options.is_empty() => {
                        out.push(format!("case step {}: options without an answer", i + 1))
                    }
                    _ => {}
                }
            }
        }
        StepBody::Table { columns, rows, hide, .. } => {
            if columns.len() < 2 || rows.is_empty() {
                out.push("a table needs 2+ columns and a row".into());
            }
            if rows.iter().any(|r| r.len() != columns.len()) {
                out.push("every row needs one cell per column".into());
            }
            if hide.iter().any(|[r, c]| *r >= rows.len() || *c >= columns.len()) {
                out.push("a hidden cell is outside the table".into());
            }
        }
        StepBody::Try { items: ids } => {
            if ids.is_empty() || ids.len() > 5 {
                out.push("try 1 to 5 items".into());
            }
            for id in ids {
                if !items.contains(id.as_str()) {
                    out.push(format!("item {id} is not in this lecture"));
                }
            }
        }
    }
    out
}

/// A figure must be a plain, self-contained drawing in theme colours: no
/// scripts, event handlers, links, embedded images or fixed colours (so it
/// reads in Day, Night and true black alike).
pub fn svg_problems(svg: &str) -> Vec<String> {
    let mut out = Vec::new();
    let s = svg.trim();
    if !s.starts_with("<svg") || !s.ends_with("</svg>") {
        out.push("svg must be one <svg>…</svg> element".into());
    }
    if !s.contains("viewBox") {
        out.push("svg needs a viewBox (it scales to the screen)".into());
    }
    if s.len() > MAX_SVG_BYTES {
        out.push(format!("svg is {} KB; keep figures under {} KB", s.len() / 1024, MAX_SVG_BYTES / 1024));
    }
    let lower = s.to_ascii_lowercase();
    for banned in
        ["<script", "<image", "<foreignobject", "<iframe", "javascript:", "href=\"http", "@import", "<style"]
    {
        if lower.contains(banned) {
            out.push(format!("svg may not contain `{banned}`"));
        }
    }
    // on…= event handler attributes.
    for (i, w) in lower.match_indices(" on") {
        let rest = &lower[i + w.len()..];
        let name: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        if !name.is_empty() && rest[name.len()..].trim_start().starts_with('=') {
            out.push(format!("svg may not have event handlers (`on{name}`)"));
            break;
        }
    }
    for attr in ["fill=\"", "stroke=\"", "stop-color=\"", "color=\""] {
        for (i, _) in s.match_indices(attr) {
            // `stroke="` also matches inside `-stroke="`… only whole attributes.
            if i > 0 && !s.as_bytes()[i - 1].is_ascii_whitespace() {
                continue;
            }
            let value: String = s[i + attr.len()..].chars().take_while(|c| *c != '"').collect();
            let ok = value.starts_with("var(--")
                || value.starts_with("url(#")
                || matches!(value.as_str(), "none" | "currentColor" | "transparent");
            if !ok {
                out.push(format!("svg colour `{attr}{value}\"`: use var(--token), currentColor or none"));
            }
        }
    }
    for fixed in ["rgb(", "rgba(", "hsl(", "#"] {
        if fixed == "#" {
            // `#` is fine in url(#id) and href="#id"; a hex colour isn't.
            let hex = s.match_indices('#').any(|(i, _)| {
                let tail: String = s[i + 1..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                let before = &s[..i];
                matches!(tail.len(), 3 | 6 | 8) && !before.ends_with("url(") && !before.ends_with("href=\"")
            });
            if hex {
                out.push("svg has a hex colour: use var(--token) or a class".into());
            }
        } else if lower.contains(fixed) {
            out.push(format!("svg has a fixed colour `{fixed}…)`: use var(--token)"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn latin_runs(line: &str) -> Vec<String> {
        spans(line).into_iter().filter(|s| s.latin).map(|s| s.text).collect()
    }

    #[test]
    fn terms_are_isolated_whole() {
        assert_eq!(latin_runs("الـ Child-Pugh class C معناها"), ["Child-Pugh class C"]);
        assert_eq!(latin_runs("الـ HSCs بتتنشط"), ["HSCs"]);
        assert_eq!(latin_runs("لما PaO2 <80 mmHg يبقى"), ["PaO2 <80 mmHg"]);
        assert_eq!(latin_runs("كله عربي"), Vec::<String>::new());
        assert_eq!(latin_runs("Zone 3"), ["Zone 3"]);
        assert_eq!(latin_runs("الشكل (morphology): كده"), ["(morphology)"]);
        assert_eq!(latin_runs("الـ nodules. وبعدين"), ["nodules"]);
        assert_eq!(latin_runs("الـ MELD ≥ 21 هنا"), ["MELD ≥ 21"]);
        assert_eq!(latin_runs("لازم يتحققوا (all 5 must be fulfilled). تمام"), ["(all 5 must be fulfilled)"]);
        assert_eq!(latin_runs("Low risk (≤ 2%)"), ["Low risk (≤ 2%)"]);
        assert_eq!(latin_runs("Low risk (≤ 2%) يعني"), ["Low risk (≤ 2%)"]);
        assert_eq!(latin_runs("الـ MELD (يعني"), ["MELD"]);
        assert_eq!(latin_runs("طوله حوالي 20 cm تقريبا"), ["20 cm"]);
        assert_eq!(latin_runs("20 cm"), ["20 cm"]);
        assert_eq!(latin_runs("خليه 1.5 ml/kg/h كده"), ["1.5 ml/kg/h"]);
        assert_eq!(latin_runs("حوالي 3–4 L في اليوم"), ["3–4 L"]);
        assert_eq!(latin_runs("في 24 h"), ["24 h"]);
        assert_eq!(latin_runs("من 5–6 نقط"), ["5–6"]);
        assert_eq!(latin_runs("الـ OR 7.4 هنا"), ["OR 7.4"]);
        assert_eq!(latin_runs("الـ CA 19-9 مش"), ["CA 19-9"]);
        assert_eq!(latin_runs("الـ ECOG 0–1 بس"), ["ECOG 0–1"]);
        assert_eq!(latin_runs("أكتر من (>80 g/day) كده"), ["(>80 g/day)"]);
        assert_eq!(latin_runs("لو (PMN ≥250) يبقى"), ["(PMN ≥250)"]);
        assert_eq!(latin_runs("الـ BCLC 2022"), ["BCLC 2022"]);
        assert_eq!(latin_runs("لو ≥250 cells يبقى"), ["≥250 cells"]);
        assert_eq!(latin_runs("(مع albumin 20%) كده"), ["albumin 20%"]);
        assert_eq!(latin_runs("حوالي 1/3 العيانين و 40 واحد"), ["1/3"]);
    }

    #[test]
    fn bold_does_not_split_terms() {
        let s = spans("**Genome:** single-stranded **positive-sense RNA**. تمام");
        let latin: Vec<_> = s.iter().filter(|s| s.latin).map(|s| (s.text.as_str(), s.bold)).collect();
        assert_eq!(latin, [("Genome:", true), (" single-stranded ", false), ("positive-sense RNA", true)]);
        let s = spans("حوالي **58 million** عايشين");
        assert_eq!(s.iter().filter(|s| s.latin).map(|s| s.text.as_str()).collect::<Vec<_>>(), ["58 million"]);
    }

    #[test]
    fn bold_and_blocks() {
        let s = spans("ده **مهم جدا** يا دكتور");
        assert_eq!(s.iter().filter(|s| s.bold).count(), 1);
        assert_eq!(s.iter().find(|s| s.bold).unwrap().text, "مهم جدا");
        let b = blocks("سطر أول\nكمّلة\n\n- نقطة A\n- نقطة B\n\nآخر");
        assert_eq!(b.len(), 3);
        assert!(matches!(&b[1], Block::Bullets(l) if l.len() == 2));
        // A list right under a lead-in line, with no blank line between.
        let b = blocks("الأسباب:\n- Viral\n- Alcohol");
        assert!(matches!(&b[..], [Block::Para(_), Block::Bullets(l)] if l.len() == 2));
    }

    #[test]
    fn svg_rules() {
        let good = r#"<svg viewBox="0 0 10 10"><rect class="fg-box" fill="var(--surface)" stroke="currentColor" width="5" height="5"/><path marker-end="url(#a)" d="M0 0"/></svg>"#;
        assert!(svg_problems(good).is_empty(), "{:?}", svg_problems(good));
        assert!(!svg_problems(r##"<svg viewBox="0 0 1 1"><rect fill="#ff0000"/></svg>"##).is_empty());
        assert!(!svg_problems(r#"<svg viewBox="0 0 1 1"><rect fill="red"/></svg>"#).is_empty());
        assert!(!svg_problems(r#"<svg viewBox="0 0 1 1" onload="x()"></svg>"#).is_empty());
        assert!(!svg_problems(r#"<svg viewBox="0 0 1 1"><script>1</script></svg>"#).is_empty());
        assert!(!svg_problems(r#"<svg><g/></svg>"#).is_empty());
    }

    #[test]
    fn steps_round_trip_through_yaml() {
        let yaml = r#"
kind: check
question: Which zone first?
options: [Zone 1, Zone 2, Zone 3]
answer: 0
why: [yes, no, no]
page: 3
"#;
        let step: Step = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(step.body.is_question());
        assert_eq!(step.page, Some(3));
        let back: Step = serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&step).unwrap()).unwrap();
        assert_eq!(step, back);
    }
}
