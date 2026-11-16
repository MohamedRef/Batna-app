//! Full-text search over lesson steps and items (#1350): "where did we
//! cover X?". Titles alone miss most of what a lesson teaches, so the text
//! every step shows ([`Step::shown_text`]) and every item asks and answers
//! ([`item_text`]) is folded once into an [`Index`] and scanned per query.
//!
//! Folding evens out case and Arabic spelling (hamza forms, ة/ه, ى/ي,
//! tashkeel, Arabic digits), the same as the calculator search. Every
//! typed word must appear: in one step for a lesson (else anywhere in the
//! lesson), anywhere in an item.

use crate::content::{Body, Item, OutlineEntry, Status};
use crate::lesson::{Lesson, Step, StepBody};

/// One character, folded: `None` drops it (tashkeel, tatweel).
fn fold_char(c: char) -> Option<char> {
    match c {
        '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{0640}' => None,
        'أ' | 'إ' | 'آ' | 'ٱ' => Some('ا'),
        'ة' => Some('ه'),
        'ى' | 'ئ' => Some('ي'),
        'ؤ' => Some('و'),
        '٠'..='٩' => char::from_digit(c as u32 - '٠' as u32, 10),
        // "Child-Pugh", "Child–Pugh" and "Child Pugh" are one name.
        '-' | '‐' | '‑' | '‒' | '–' | '—' | '−' | '/' => Some(' '),
        _ => Some(c),
    }
}

/// Lower case, with Arabic spelling evened out and dashes read as spaces.
pub fn fold(s: &str) -> String {
    s.chars().filter_map(fold_char).flat_map(char::to_lowercase).collect()
}

/// [`fold`], plus for each byte of the folded text the byte range of the
/// original character it came from.
fn fold_mapped(s: &str) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(s.len());
    let mut map = Vec::with_capacity(s.len());
    for (at, c) in s.char_indices() {
        let Some(f) = fold_char(c) else { continue };
        for l in f.to_lowercase() {
            let before = out.len();
            out.push(l);
            map.extend(std::iter::repeat_n((at, at + c.len_utf8()), out.len() - before));
        }
    }
    (out, map)
}

/// The query's words, folded, each once, in the order typed.
pub fn words(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in fold(query).split_whitespace() {
        if !out.iter().any(|o| o == w) {
            out.push(w.to_string());
        }
    }
    out
}

/// A cloze's text as read once answered: `{{c1::answer::hint}}` → `answer`.
pub fn cloze_plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find("{{") {
        out.push_str(&rest[..open]);
        let inner = &rest[open + 2..];
        let Some(close) = inner.find("}}") else {
            out.push_str(&rest[open..]);
            return out;
        };
        let blank = &inner[..close];
        // `c1::answer` or `c1::answer::hint`: the answer is the middle part.
        let answer = blank.split("::").nth(1).unwrap_or(blank);
        out.push_str(answer);
        rest = &inner[close + 2..];
    }
    out.push_str(rest);
    out
}

/// Every piece of text an item shows: question, answer, options and their
/// feedback, points, outline, table cells, and its guideline update. Not
/// ids, sources or calculator inputs.
pub fn item_text(item: &Item) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |s: &str| {
        if !s.trim().is_empty() {
            out.push(s.to_string());
        }
    };
    match &item.body {
        Body::Card { front, back } => {
            add(front);
            add(back);
        }
        Body::Cloze { text, extra } => {
            add(&cloze_plain(text));
            extra.iter().for_each(|e| add(e));
        }
        Body::Mcq { stem, options, why, .. } => {
            add(stem);
            options.iter().chain(why).for_each(|o| add(o));
        }
        Body::Enumerate { prompt, points, hook, .. } => {
            add(prompt);
            points.iter().for_each(|p| add(p));
            hook.iter().for_each(|h| add(h));
        }
        Body::Essay { prompt, outline } => {
            add(prompt);
            for entry in outline {
                match entry {
                    OutlineEntry::Heading(h) => add(h),
                    OutlineEntry::Section(map) => {
                        for (head, lines) in map {
                            add(head);
                            lines.iter().for_each(|l| add(l));
                        }
                    }
                }
            }
        }
        Body::Case { stem, steps } => {
            add(stem);
            for s in steps {
                s.reveal.iter().for_each(|r| add(r));
                add(&s.question);
                s.options.iter().for_each(|o| add(o));
                add(&s.why);
            }
        }
        Body::Table { title, columns, rows, .. } => {
            add(title);
            columns.iter().chain(rows.iter().flatten()).for_each(|c| add(c));
        }
        Body::Order { prompt, steps } => {
            add(prompt);
            steps.iter().for_each(|s| add(s));
        }
        Body::Calc { stem, question, expect, interpret, .. } => {
            add(stem);
            question.iter().for_each(|q| add(q));
            add(expect);
            add(interpret);
        }
    }
    if let Some(u) = &item.update {
        add(&u.text);
        add(&u.reference);
    }
    out
}

/// The text a step shows, as searched and quoted in results: what
/// [`Step::shown_text`] lists, plus a picture's description (its `alt`),
/// which names what the picture shows.
pub fn step_text(step: &Step) -> Vec<String> {
    let mut out = step.shown_text();
    if let StepBody::Figure { alt, .. } = &step.body
        && !alt.trim().is_empty()
    {
        out.push(alt.clone());
    }
    out
}

/// A lesson's own lines: title, hook, goals and recap. A match only here
/// opens the lesson at its first step.
pub fn lesson_text(lesson: &Lesson) -> Vec<String> {
    [&lesson.title, &lesson.hook].into_iter().chain(&lesson.goals).chain(&lesson.recap).cloned().collect()
}

/// The folded text of some pieces, one piece per line.
fn folded(pieces: &[String]) -> String {
    let mut out = String::new();
    for p in pieces {
        out.push_str(&fold(p));
        out.push('\n');
    }
    out
}

struct LessonDoc {
    id: String,
    /// Each step's folded text.
    steps: Vec<String>,
    /// [`lesson_text`], folded.
    own: String,
}

impl LessonDoc {
    fn of(l: &Lesson) -> LessonDoc {
        LessonDoc {
            id: l.id.clone(),
            steps: l.steps.iter().map(|s| folded(&step_text(s))).collect(),
            own: folded(&lesson_text(l)),
        }
    }
}

struct ItemDoc {
    id: String,
    text: String,
}

/// The folded text of every lesson step and every item, built once.
#[derive(Default)]
pub struct Index {
    lessons: Vec<LessonDoc>,
    items: Vec<ItemDoc>,
}

/// A lesson that teaches the words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonHit {
    pub lesson: String,
    /// The step to open: the first that holds every word, else the first
    /// that holds the most of them.
    pub step: usize,
    /// How many steps hold every word (0 when they are spread out).
    pub steps: usize,
}

/// What a query found, each list in the order the index was built (book order).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub lessons: Vec<LessonHit>,
    /// Item ids.
    pub items: Vec<String>,
}

impl Found {
    pub fn is_empty(&self) -> bool {
        self.lessons.is_empty() && self.items.is_empty()
    }
}

impl Index {
    /// Lessons and items in the order results should list them. Rejected
    /// items are left out.
    pub fn build<'a>(
        lessons: impl IntoIterator<Item = &'a Lesson>,
        items: impl IntoIterator<Item = &'a Item>,
    ) -> Index {
        Index {
            lessons: lessons.into_iter().map(LessonDoc::of).collect(),
            items: items
                .into_iter()
                .filter(|i| i.status != Status::Rejected)
                .map(|i| ItemDoc { id: i.id.clone(), text: folded(&item_text(i)) })
                .collect(),
        }
    }

    /// Lessons and items holding every one of `words` (from [`words`]).
    pub fn find(&self, words: &[String]) -> Found {
        if words.is_empty() {
            return Found::default();
        }
        let lessons = self.lessons.iter().filter_map(|l| lesson_hit(l, words)).collect();
        let items = self
            .items
            .iter()
            .filter(|i| words.iter().all(|w| i.text.contains(w.as_str())))
            .map(|i| i.id.clone())
            .collect();
        Found { lessons, items }
    }
}

fn lesson_hit(l: &LessonDoc, words: &[String]) -> Option<LessonHit> {
    let held = |s: &str| words.iter().filter(|w| s.contains(w.as_str())).count();
    let counts: Vec<usize> = l.steps.iter().map(|s| held(s)).collect();
    let whole: Vec<usize> = (0..counts.len()).filter(|&k| counts[k] == words.len()).collect();
    if let Some(&first) = whole.first() {
        return Some(LessonHit { lesson: l.id.clone(), step: first, steps: whole.len() });
    }
    // Spread over the lesson (its own lines too): every word somewhere in it.
    let all =
        words.iter().all(|w| l.own.contains(w.as_str()) || l.steps.iter().any(|s| s.contains(w.as_str())));
    if !all {
        return None;
    }
    let best = counts.iter().copied().max().unwrap_or(0);
    let step = if best == 0 { 0 } else { counts.iter().position(|&c| c == best).unwrap_or(0) };
    Some(LessonHit { lesson: l.id.clone(), step, steps: 0 })
}

/// The step of `lesson` to open for `words`, the way [`Index::find`]
/// picks it (for a lesson reached another way, like an item's).
pub fn find_step(lesson: &Lesson, words: &[String]) -> Option<usize> {
    if words.is_empty() {
        return None;
    }
    lesson_hit(&LessonDoc::of(lesson), words).map(|h| h.step)
}

/// A short piece of text around the first word found, in the original
/// spelling: what comes before, the matched words, what comes after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub before: String,
    pub hit: String,
    pub after: String,
}

/// The text as read, for a one-line quote: no `**bold**` or `==key==`
/// marks, bullets or line breaks.
fn plain(s: &str) -> String {
    let joined: Vec<String> = s
        .lines()
        .map(|l| l.trim())
        .map(|l| l.strip_prefix("- ").unwrap_or(l))
        .filter(|l| !l.is_empty())
        .map(|l| crate::lesson::spans(l).into_iter().map(|s| s.text).collect())
        .collect();
    joined.join(" · ")
}

/// The first piece that holds one of `words` (the first word first), cut
/// to about `before` characters before the match and `after` after it.
pub fn snippet(pieces: &[String], words: &[String], before: usize, after: usize) -> Option<Snippet> {
    for w in words {
        for piece in pieces {
            let text = plain(piece);
            let (f, map) = fold_mapped(&text);
            let Some(at) = f.find(w.as_str()) else { continue };
            let start = map[at].0;
            let end = map[at + w.len() - 1].1;
            let head = &text[..start];
            let tail = &text[end..];
            let n_head = head.chars().count();
            let cut_head: String = head.chars().skip(n_head.saturating_sub(before)).collect();
            let mut cut_tail: String = tail.chars().take(after).collect();
            if tail.chars().count() > after {
                cut_tail.push('…');
            }
            let lead = if n_head > before { format!("…{}", cut_head.trim_start()) } else { cut_head };
            return Some(Snippet { before: lead, hit: text[start..end].to_string(), after: cut_tail });
        }
    }
    None
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
