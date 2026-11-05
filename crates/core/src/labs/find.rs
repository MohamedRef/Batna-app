//! Finding lab values in a text.

use super::*;

/// A lab value found in a text. Offsets are byte offsets into the text.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// Where the test's name starts.
    pub name: usize,
    /// The number (and its unit): the part to mark.
    pub start: usize,
    pub end: usize,
    pub lab: &'static Lab,
    pub value: f64,
    /// Low or high when the text says so: an exact value outside the
    /// range, or a cut-off that is all low ("< 135") or all high (">300").
    pub place: Option<Place>,
    /// The number had no unit, so the chip names it.
    pub bare: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Cmp {
    Eq,
    Below,
    Above,
}

/// Comparators and linking words allowed between a name and its number.
/// Longest first where one starts another.
const LINKS: &[(&str, Option<Cmp>)] = &[
    ("<=", Some(Cmp::Below)),
    (">=", Some(Cmp::Above)),
    ("<", Some(Cmp::Below)),
    (">", Some(Cmp::Above)),
    ("≤", Some(Cmp::Below)),
    ("≥", Some(Cmp::Above)),
    ("=", None),
    (":", None),
    ("~", None),
    ("≈", None),
    ("(", None),
    ("of", None),
    ("is", None),
    ("was", None),
    ("level", None),
    ("levels", None),
    ("value", None),
    ("count", None),
    ("concentration", None),
    ("about", None),
    ("around", None),
    ("approximately", None),
    ("below", Some(Cmp::Below)),
    ("under", Some(Cmp::Below)),
    ("less than", Some(Cmp::Below)),
    ("lower than", Some(Cmp::Below)),
    ("above", Some(Cmp::Above)),
    ("over", Some(Cmp::Above)),
    ("more than", Some(Cmp::Above)),
    ("greater than", Some(Cmp::Above)),
    ("higher than", Some(Cmp::Above)),
    ("at least", Some(Cmp::Above)),
    ("exceeding", Some(Cmp::Above)),
    ("أقل من", Some(Cmp::Below)),
    ("اقل من", Some(Cmp::Below)),
    ("تحت", Some(Cmp::Below)),
    ("أكتر من", Some(Cmp::Above)),
    ("اكتر من", Some(Cmp::Above)),
    ("أكثر من", Some(Cmp::Above)),
    ("أعلى من", Some(Cmp::Above)),
    ("فوق", Some(Cmp::Above)),
    ("حوالي", None),
    ("=<", Some(Cmp::Below)),
    ("=>", Some(Cmp::Above)),
];

/// Words just before the name that make the number something else: another
/// fluid, a fraction of the total, or a change.
const NOT_SERUM: &[&str] = &[
    "urine",
    "urinary",
    "ascitic",
    "fluid",
    "pleural",
    "csf",
    "stool",
    "faecal",
    "fecal",
    "sweat",
    "direct",
    "indirect",
    "conjugated",
    "unconjugated",
    "ionized",
    "ionised",
    "free",
    "delta",
    "δ",
    "change",
    "rise",
    "increase",
    "increases",
    "increased",
    "decrease",
    "decreased",
    "drop",
    "fall",
    "reduction",
    "by",
    "raise",
    "raising",
    "lower",
    "lowering",
    "correct",
    "correction",
    "gradient",
    "spot",
    "24-h",
    "24-hour",
    "زيادة",
    "ارتفاع",
    "نزول",
    "انخفاض",
    "بمقدار",
    "تصحيح",
];

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || ('\u{0600}'..='\u{06FF}').contains(&c)
}

/// The text at `at` starts with `s`, by exact case or ignoring it.
fn starts(text: &str, at: usize, s: &str, case: bool) -> bool {
    let t = &text[at..];
    if case {
        t.starts_with(s)
    } else {
        t.len() >= s.len() && t.is_char_boundary(s.len()) && t[..s.len()].eq_ignore_ascii_case(s)
    }
}

/// The few words before `at`, lower-case, nearest first.
fn words_before(text: &str, at: usize, k: usize) -> Vec<String> {
    text[..at]
        .split(|c: char| c.is_whitespace() || ",;()[]".contains(c))
        .filter(|w| !w.is_empty())
        .rev()
        .take(k)
        .map(|w| w.trim_matches(|c: char| c == '.' || c == ':').to_lowercase())
        .collect()
}

/// A comma at `i` that groups thousands: three digits after it, then no
/// fourth ("50,000", not "5,1234" or a list "5, 6").
fn thousands(b: &[u8], i: usize) -> bool {
    b.len() >= i + 4
        && b[i + 1..i + 4].iter().all(u8::is_ascii_digit)
        && b.get(i + 4).is_none_or(|d| !d.is_ascii_digit())
}

/// A number at `at`: digits, thousands commas, a decimal point. Returns the
/// value and where it ends.
fn number(text: &str, at: usize) -> Option<(f64, usize)> {
    let b = text.as_bytes();
    let mut i = at;
    let mut digits = String::new();
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_digit() {
            digits.push(c as char);
            i += 1;
        } else if c == b',' && thousands(b, i) && !digits.is_empty() && !digits.contains('.') {
            i += 1;
        } else if c == b'.'
            && !digits.is_empty()
            && !digits.contains('.')
            && b.get(i + 1).is_some_and(u8::is_ascii_digit)
        {
            digits.push('.');
            i += 1;
        } else {
            break;
        }
    }
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok().map(|v| (v, i))
}

/// A unit spelling folded for comparing: no spaces, lower case, one micro
/// sign, superscripts as digits, `x`/`*` as `×`.
fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != '^')
        .map(|c| match c {
            'μ' => 'µ',
            '⁰' => '0',
            '¹' => '1',
            '²' => '2',
            '³' => '3',
            '⁹' => '9',
            'x' | 'X' | '*' => '×',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// The unit right after a number, if `units` holds it: the byte end of the
/// longest spelling that matches.
fn unit_at(text: &str, at: usize, units: &[&str]) -> Option<usize> {
    let rest = &text[at..];
    let mut best: Option<usize> = None;
    for u in units {
        let want = fold(u);
        // Walk the text, folding as we go, until the folded text is as long.
        let mut got = String::new();
        let mut end = None;
        for (i, c) in rest.char_indices() {
            if got.len() >= want.len() {
                end = Some(i);
                break;
            }
            // A space may sit between the number and its unit, or inside one
            // ("mm Hg", "× 10⁹/L"), but not a line break.
            if c == '\n' {
                break;
            }
            got.push_str(&fold(&c.to_string()));
            if !want.starts_with(&got) {
                break;
            }
            end = Some(i + c.len_utf8());
        }
        let Some(e) = end else { continue };
        if got != want {
            continue;
        }
        // The unit ends there: "mg/dL" but not "mg/dLx", "s" but not "sec…".
        if rest[e..].chars().next().is_some_and(|c| c.is_alphanumeric()) {
            continue;
        }
        if best.is_none_or(|b| e > b) {
            best = Some(e);
        }
    }
    best.map(|e| at + e)
}

fn skip_spaces(text: &str, mut at: usize) -> usize {
    while let Some(c) = text[at..].chars().next() {
        if c == ' ' || c == '\u{a0}' || c == '\u{2009}' || c == '\u{200A}' {
            at += c.len_utf8();
        } else {
            break;
        }
    }
    at
}

/// The name of a test at `at` (with its case where it is an abbreviation):
/// the labs it may be and where the name ends.
fn name_at(text: &str, at: usize) -> Option<(usize, Vec<&'static Lab>)> {
    let mut best: Option<(usize, Vec<&'static Lab>)> = None;
    for lab in LABS {
        let spellings = lab.abbr.iter().map(|s| (s, true)).chain(lab.words.iter().map(|s| (s, false)));
        for (s, case) in spellings {
            if !starts(text, at, s, case) {
                continue;
            }
            let end = at + s.len();
            // The name ends there: "Na" but not "NaCl", "Hb" but not "HbA1c".
            let next = text[end..].chars().next();
            if next.is_some_and(|c| {
                c.is_alphanumeric()
                    || (c == '-'
                        && s.ends_with(char::is_alphabetic)
                        && text[end + 1..].starts_with(char::is_alphabetic))
            }) {
                continue;
            }
            match &mut best {
                Some((e, labs)) if *e == end => {
                    if !labs.iter().any(|l| std::ptr::eq(*l, lab)) {
                        labs.push(lab);
                    }
                }
                Some((e, _)) if *e > end => {}
                _ => best = Some((end, vec![lab])),
            }
        }
    }
    best
}

/// Lab values in a text, in order.
pub fn find(text: &str) -> Vec<Hit> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut prev: Option<char> = None;
    while at < text.len() {
        let c = text[at..].chars().next().unwrap_or(' ');
        let starts_word = !prev.is_some_and(|p| is_word(p) || p == '.' || p == '-' || p == '/');
        prev = Some(c);
        if starts_word
            && c.is_alphabetic()
            && let Some(hit) = hit_at(text, at)
        {
            at = hit.end;
            prev = text[..at].chars().next_back();
            out.push(hit);
            continue;
        }
        at += c.len_utf8();
    }
    out
}

fn hit_at(text: &str, at: usize) -> Option<Hit> {
    let (mut i, labs) = name_at(text, at)?;
    let before = words_before(text, at, 3);
    if before.iter().any(|w| NOT_SERUM.contains(&w.as_str())) {
        return None;
    }
    // A closing bracket or a written sign after the name: "(Na+)", "K⁺".
    while let Some(c) = text[i..].chars().next() {
        if ")⁺⁻".contains(c) {
            i += c.len_utf8();
        } else {
            break;
        }
    }
    // Up to three links: "of <", "level is about".
    let mut cmp = Cmp::Eq;
    for _ in 0..4 {
        i = skip_spaces(text, i);
        if text[i..].starts_with(|c: char| c.is_ascii_digit()) {
            break;
        }
        let link = LINKS.iter().filter(|(w, _)| {
            let case = !w.is_ascii() || !w.chars().all(|c| c.is_ascii_alphabetic() || c == ' ');
            starts(text, i, w, case)
                && (!w.ends_with(|c: char| c.is_alphabetic()) || !text[i + w.len()..].starts_with(is_word))
        });
        let (w, c) = link.max_by_key(|(w, _)| w.len())?;
        if let Some(c) = c {
            cmp = *c;
        }
        i += w.len();
    }
    i = skip_spaces(text, i);
    let start = i;
    let (value, mut end) = number(text, i)?;
    // A range: "7–8", "7-8", "7 to 8".
    let mut high_end = value;
    {
        let j = skip_spaces(text, end);
        let dash = ["–", "-", "—", "to "].iter().find(|d| text[j..].starts_with(**d));
        if let Some(d) = dash {
            let k = skip_spaces(text, j + d.len());
            if let Some((v2, e2)) = number(text, k)
                && v2 > value
            {
                high_end = v2;
                end = e2;
            }
        }
    }
    let u = skip_spaces(text, end);
    let tail = &text[u..];
    // A multiple ("3× ULN", "2 times"), a fold or a percentage of something.
    let multiple = tail.starts_with(['×', 'x', 'X', '*'])
        && !fold(&tail.chars().take(8).collect::<String>()).starts_with("×10");
    if multiple || tail.starts_with("times") || tail.starts_with("-fold") || tail.starts_with("fold") {
        return None;
    }
    let mut found: Option<(&'static Lab, usize, bool)> = None;
    for lab in &labs {
        if let Some(e) = unit_at(text, end, lab.units) {
            found = Some((lab, e, false));
            break;
        }
    }
    if found.is_none() {
        // No known unit: a bare number, if nothing unit-like follows.
        let unitish = tail.starts_with(|c: char| c.is_ascii_alphabetic() || "%/×µμ".contains(c))
            && !tail.starts_with(|c: char| c.is_ascii_alphabetic())
            || tail.split(|c: char| !c.is_ascii_alphanumeric() && c != '/').next().is_some_and(|w| {
                w.contains('/')
                    || ["mg", "g", "mmol", "mEq", "U", "IU", "mL", "ml", "kg", "mm", "fL"].contains(&w)
            });
        if !unitish {
            found = labs
                .iter()
                .find(|l| l.bare.is_some_and(|(lo, hi)| value >= lo && high_end <= hi))
                .map(|l| (*l, end, !l.units.is_empty()));
        }
    }
    let (lab, end, bare) = found?;
    let (low, high) = lab.normal.bounds(None);
    let place = match cmp {
        Cmp::Eq => {
            let a = lab.normal.place(value, None);
            let b = lab.normal.place(high_end, None);
            (a == b && a != Place::Normal).then_some(a)
        }
        Cmp::Below => (value <= low).then_some(Place::Low),
        Cmp::Above => (value >= high && high.is_finite()).then_some(Place::High),
    };
    Some(Hit { name: at, start, end, lab, value, place, bare })
}

/// A number as the chip shows it: no ".0", thousands with commas.
fn show(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() >= 10_000.0 {
        let s = format!("{}", v as i64);
        let mut out = String::new();
        for (k, c) in s.chars().enumerate() {
            if k > 0 && (s.len() - k) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        out
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn span(low: f64, high: f64, below: bool) -> String {
    match (low.is_finite(), high.is_finite()) {
        (true, true) => format!("{}–{}", show(low), show(high)),
        (false, true) if below => format!("<{}", show(high)),
        (false, true) => format!("≤{}", show(high)),
        (true, false) => format!("≥{}", show(low)),
        _ => String::new(),
    }
}

impl Lab {
    /// The unit as shown ("mmol/L"); empty for pH and INR.
    pub fn unit(&self) -> &'static str {
        self.units.first().copied().unwrap_or("")
    }

    /// The range in a few characters: "135–145", "♂ 13.5–17.5 · ♀ 12–16",
    /// "<150", "≥90".
    pub fn range(&self) -> String {
        let n = &self.normal;
        let men = span(n.low, n.high, n.below);
        match n.female {
            Some((fl, fh)) => format!("♂ {men} · ♀ {}", span(fl, fh, n.below)),
            None => men,
        }
    }

    /// The whole line for a tap or hover: the range, its unit, what it is
    /// for, and the source.
    pub fn peek(&self) -> String {
        let unit = if self.unit().is_empty() { String::new() } else { format!(" {}", self.unit()) };
        let note = if self.note.is_empty() { String::new() } else { format!(" ({})", self.note) };
        format!("Adult normal{note}: {}{unit}. Source: {}. Labs differ a little.", self.range(), self.source)
    }
}

/// A text cut into plain pieces and lab values, for drawing. Each test's
/// chip is shown once per text: `chip` is false on its later values.
#[derive(Debug, Clone)]
pub enum Piece<'a> {
    Text(&'a str),
    Value { text: &'a str, hit: Hit, chip: bool },
}

pub fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let mut at = 0;
    for hit in find(text) {
        if hit.start > at {
            out.push(Piece::Text(&text[at..hit.start]));
        }
        let test = base(hit.lab.id);
        let chip = !seen.contains(&test);
        seen.push(test);
        out.push(Piece::Value { text: &text[hit.start..hit.end], hit, chip });
        at = hit.end;
    }
    if at < text.len() {
        out.push(Piece::Text(&text[at..]));
    }
    out
}

/// The lab with this id.
pub fn get(id: &str) -> Option<&'static Lab> {
    LABS.iter().find(|l| l.id == id)
}

/// The test an id is for, whatever the unit: `hb_si` → `hb`.
pub fn base(id: &str) -> &str {
    id.strip_suffix("_si").or_else(|| id.strip_suffix("_e9")).unwrap_or(id)
}
