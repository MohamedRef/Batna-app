//! Checking a typed answer against the expected one.

use super::*;

/// How close a typed answer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    Right,
    /// The numbers are right but a sign (≥, <) is missing, or a word has a
    /// small typo. Shown with the full answer.
    Close,
    Wrong,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Op(&'static str),
    Word(String),
}

/// Lower-cases, maps Arabic-Indic digits and look-alike symbols to ASCII, and
/// splits into numbers, comparison signs and words. Spacing, dashes and other
/// punctuation are dropped, so "5 - 6", "5-6" and "5 to 6" read the same.
fn tokens(s: &str) -> Vec<Tok> {
    let mut norm = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '٠'..='٩' => norm.push(char::from(b'0' + (c as u32 - '٠' as u32) as u8)),
            '۰'..='۹' => norm.push(char::from(b'0' + (c as u32 - '۰' as u32) as u8)),
            '٫' => norm.push('.'),
            '≥' | '⩾' => norm.push_str(">="),
            '≤' | '⩽' => norm.push_str("<="),
            '×' => norm.push('x'),
            '−' | '–' | '—' => norm.push('-'),
            'µ' | 'μ' => norm.push('u'),
            _ => norm.extend(c.to_lowercase()),
        }
    }
    let chars: Vec<char> = norm.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    // A dash before a number is a minus sign unless it joins a range ("5-6");
    // after "to" or a comparison sign it is a minus again ("-2.6 to -1.4").
    let mut after_to = false;
    while i < chars.len() {
        let c = chars[i];
        let digit_at = |j: usize| {
            chars.get(j).is_some_and(|d| d.is_ascii_digit())
                || (chars.get(j) == Some(&'.') && chars.get(j + 1).is_some_and(|d| d.is_ascii_digit()))
        };
        if c == '-' && digit_at(i + 1) && (after_to || !matches!(out.last(), Some(Tok::Num(_)))) {
            i += 1;
            let start = i;
            while i < chars.len() {
                let d = chars[i];
                let sep_then_digit =
                    (d == '.' || d == ',') && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
                if d.is_ascii_digit() || sep_then_digit {
                    i += 1;
                } else {
                    break;
                }
            }
            let raw: String = chars[start..i].iter().collect();
            out.push(Tok::Num(-parse_number(&raw)));
            after_to = false;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            after_to = false;
            let start = i;
            while i < chars.len() {
                let d = chars[i];
                let sep_then_digit =
                    (d == '.' || d == ',') && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
                if d.is_ascii_digit() || sep_then_digit {
                    i += 1;
                } else {
                    break;
                }
            }
            let raw: String = chars[start..i].iter().collect();
            out.push(Tok::Num(parse_number(&raw)));
        } else if c == '>' || c == '<' {
            let eq = chars.get(i + 1) == Some(&'=');
            out.push(Tok::Op(match (c, eq) {
                ('>', true) => ">=",
                ('>', false) => ">",
                ('<', true) => "<=",
                _ => "<",
            }));
            i += if eq { 2 } else { 1 };
            after_to = true;
        } else if c.is_alphanumeric() {
            let start = i;
            while i < chars.len() && chars[i].is_alphanumeric() && !chars[i].is_ascii_digit() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if word == "to" {
                after_to = true;
            } else {
                after_to = false;
                out.push(Tok::Word(us_spelling(word)));
            }
        } else {
            i += 1;
        }
    }
    out
}

/// `1,000` and `1,000,000` are thousands, `1,5` is one and a half, `2.5` is
/// two and a half, `1,000.5` is a thousand and a half.
fn parse_number(raw: &str) -> f64 {
    let int_part = raw.split('.').next().unwrap_or(raw);
    let thousands = int_part.contains(',') && int_part.split(',').skip(1).all(|g| g.len() == 3);
    let cleaned = if thousands || raw.contains('.') {
        raw.replace(',', "")
    } else {
        raw.replacen(',', ".", 1).replace(',', "")
    };
    cleaned.parse().unwrap_or(f64::NAN)
}

/// Unit words and their spellings. Typed units are optional, but a different
/// unit is a different answer ("7 weeks" for "7 days").
const UNITS: &[(&str, &[&str])] = &[
    ("sec", &["s", "sec", "secs", "second", "seconds"]),
    ("min", &["min", "mins", "minute", "minutes"]),
    ("hour", &["h", "hr", "hrs", "hour", "hours"]),
    ("day", &["d", "day", "days"]),
    ("week", &["wk", "wks", "week", "weeks"]),
    ("month", &["mo", "mos", "month", "months"]),
    ("year", &["y", "yr", "yrs", "year", "years"]),
    ("kg", &["kg"]),
    ("g", &["g", "gm", "gram", "grams"]),
    ("mg", &["mg"]),
    ("ug", &["ug", "mcg"]),
    ("ng", &["ng"]),
    ("l", &["l", "liter", "litre", "liters", "litres"]),
    ("dl", &["dl"]),
    ("ml", &["ml"]),
    ("mmol", &["mmol"]),
    ("umol", &["umol"]),
    ("meq", &["meq"]),
    ("iu", &["iu"]),
    ("mmhg", &["mmhg"]),
    ("cmh", &["cmh"]),
    ("cm", &["cm"]),
    ("mm", &["mm"]),
    ("copies", &["copies", "copy"]),
];

fn unit_of(word: &str) -> Option<&'static str> {
    UNITS.iter().find(|(_, spellings)| spellings.contains(&word)).map(|(unit, _)| *unit)
}

/// British spellings mapped to American ones, so both read the same
/// (haemolysis = hemolysis, oesophagus = esophagus, anaemia = anemia).
const SPELLINGS: &[(&str, &str)] = &[
    ("haem", "hem"),
    ("aem", "em"),
    ("oesoph", "esoph"),
    ("oedem", "edem"),
    ("oestr", "estr"),
    ("paed", "ped"),
    ("foet", "fet"),
    ("coeli", "celi"),
    ("isation", "ization"),
    ("ise", "ize"),
    ("our", "or"),
];

fn us_spelling(word: String) -> String {
    let mut w = word;
    for (uk, us) in SPELLINGS {
        // "our" and "ise" only as endings (tumour, localise), not "four", "wise".
        if matches!(*uk, "our" | "ise") {
            if w.len() >= uk.len() + 3 && w.ends_with(uk) {
                w.truncate(w.len() - uk.len());
                w.push_str(us);
            }
        } else if w.contains(uk) {
            w = w.replace(uk, us);
        }
    }
    w
}

/// Opposite prefixes: one edit apart, opposite meanings. A typed word that
/// swaps one for another is wrong, not a typo. Each group lists prefixes that
/// stand for each other on the same stem; nothing else counts as a swap.
const CONTRASTS: &[&[&str]] = &[
    &["hypo", "hyper", "normo", "eu"],
    &["intra", "extra", "inter"],
    &["pre", "post"],
    &["ante", "retro"],
    &["brady", "tachy"],
    &["micro", "macro"],
    &["endo", "exo"],
    &["ab", "ad"],
    &["af", "ef"],
    &["in", "ex"],
    &["sub", "supra"],
    &["mono", "poly", "oligo"],
];

/// Whether two words are the same stem behind opposite prefixes
/// (hypokalaemia vs hyperkalaemia, abduct vs adduct). The stems must be
/// identical, so a typo or an unrelated word never counts as a swap.
fn contrasting(a: &str, b: &str) -> bool {
    CONTRASTS.iter().any(|group| {
        group.iter().any(|pa| {
            group.iter().any(|pb| {
                pa != pb
                    && a.strip_prefix(pa)
                        .zip(b.strip_prefix(pb))
                        .is_some_and(|(sa, sb)| sa.len() >= 3 && sa == sb)
            })
        })
    })
}

/// Checks a typed cloze answer against the expected one.
///
/// With numbers: the numbers must match in order; units and spacing don't
/// matter; a missing comparison sign is Close, a different one is Wrong.
/// Without numbers: the words must match, ignoring case and punctuation; a
/// small typo is Close.
pub fn check_typed(expected: &str, typed: &str) -> Match {
    let whole = check_one(expected, typed);
    if whole != Match::Wrong {
        return whole;
    }
    // A list answer ("vitamin A and retinoids", "ALT or AST"): every listed
    // part typed, in any order, is Right; some of an "and" list is Close; any
    // one of an "or" list is as good as that part.
    let (exp_parts, either) = list_parts(expected);
    if exp_parts.len() < 2 || exp_parts.iter().any(|p| p.chars().any(|c| c.is_ascii_digit())) {
        return whole;
    }
    let (got_parts, _) = list_parts(typed);
    let mut covered = vec![Match::Wrong; exp_parts.len()];
    for g in &got_parts {
        let best =
            exp_parts.iter().enumerate().map(|(i, e)| (i, check_one(e, g))).min_by_key(|(_, m)| match m {
                Match::Right => 0,
                Match::Close => 1,
                Match::Wrong => 2,
            });
        match best {
            Some((i, m)) if m != Match::Wrong => {
                if covered[i] != Match::Right {
                    covered[i] = m;
                }
            }
            // Something typed that is not on the list.
            _ => return Match::Wrong,
        }
    }
    let hits: Vec<Match> = covered.into_iter().filter(|m| *m != Match::Wrong).collect();
    if hits.is_empty() {
        Match::Wrong
    } else if (either || hits.len() == exp_parts.len()) && hits.iter().all(|m| *m == Match::Right) {
        Match::Right
    } else {
        Match::Close
    }
}

/// Splits "a, b and c" / "a or b" / "a/b" into its parts, and says whether
/// they are alternatives ("or", "/").
fn list_parts(s: &str) -> (Vec<String>, bool) {
    let lower = s.to_lowercase();
    let either = lower.contains(" or ") || lower.contains('/');
    let parts = lower
        .replace(" and ", ",")
        .replace(" or ", ",")
        .replace(['/', ';', '&'], ",")
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    (parts, either)
}

fn check_one(expected: &str, typed: &str) -> Match {
    let exp = tokens(expected);
    let got = tokens(typed);
    if got.is_empty() {
        return Match::Wrong;
    }
    let nums = |t: &[Tok]| -> Vec<f64> {
        t.iter().filter_map(|x| if let Tok::Num(n) = x { Some(*n) } else { None }).collect()
    };
    let ops = |t: &[Tok]| -> Vec<&'static str> {
        t.iter().filter_map(|x| if let Tok::Op(o) = x { Some(*o) } else { None }).collect()
    };
    let words = |t: &[Tok]| -> Vec<String> {
        t.iter().filter_map(|x| if let Tok::Word(w) = x { Some(w.clone()) } else { None }).collect()
    };

    let exp_nums = nums(&exp);
    if !exp_nums.is_empty() {
        let got_nums = nums(&got);
        let same = got_nums.len() == exp_nums.len()
            && got_nums.iter().zip(&exp_nums).all(|(a, b)| (a - b).abs() <= 1e-9 * b.abs().max(1.0));
        if !same {
            return Match::Wrong;
        }
        let units = |t: &[Tok]| -> Vec<&'static str> {
            t.iter().filter_map(|x| if let Tok::Word(w) = x { unit_of(w) } else { None }).collect()
        };
        let got_units = units(&got);
        if !got_units.is_empty() && got_units != units(&exp) {
            return Match::Wrong;
        }
        let (exp_ops, got_ops) = (ops(&exp), ops(&got));
        return if got_ops == exp_ops {
            Match::Right
        } else if got_ops.is_empty() {
            Match::Close
        } else {
            Match::Wrong
        };
    }

    let (exp_words, got_words) = (words(&exp), words(&got));
    if exp_words == got_words {
        return Match::Right;
    }
    let (a, b) = (exp_words.concat(), got_words.concat());
    if a == b {
        return Match::Right;
    }
    let swapped = if exp_words.len() == got_words.len() {
        exp_words.iter().zip(&got_words).any(|(e, g)| contrasting(e, g))
    } else {
        contrasting(&a, &b)
    };
    if swapped {
        return Match::Wrong;
    }
    let len = a.chars().count();
    let allowed = if len >= 9 {
        2
    } else if len >= 5 {
        1
    } else {
        0
    };
    if allowed > 0 && edit_distance(&a, &b) <= allowed { Match::Close } else { Match::Wrong }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = (prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The grade a cloze attempt suggests: all right is Good, near misses Hard,
/// anything wrong or revealed Again. The learner can still pick another.
pub fn suggest_cloze(results: &[Match]) -> Grade {
    if results.contains(&Match::Wrong) {
        Grade::Again
    } else if results.contains(&Match::Close) {
        Grade::Hard
    } else {
        Grade::Good
    }
}

const STOP: &[&str] =
    &["the", "and", "of", "in", "with", "for", "a", "an", "or", "on", "by", "to", "from", "its"];

fn key_words(s: &str) -> Vec<String> {
    tokens(s)
        .into_iter()
        .filter_map(|t| match t {
            Tok::Word(w) if w.chars().count() >= 2 && !STOP.contains(&w.as_str()) => Some(w),
            Tok::Num(n) => Some(n.to_string()),
            _ => None,
        })
        .collect()
}

/// Words that name a kind of thing rather than the thing ("syndrome",
/// "hypertension") or qualify it ("recurrent"). A point is only recalled when
/// its most specific word is.
const GENERIC: &[&str] = &[
    "syndrome",
    "disease",
    "disorder",
    "failure",
    "injury",
    "hypertension",
    "infection",
    "infections",
    "deficiency",
    "cancer",
    "carcinoma",
    "tumor",
    "tumour",
    "test",
    "level",
    "levels",
    "cell",
    "cells",
    "type",
    "liver",
    "hepatic",
    "acute",
    "chronic",
    "primary",
    "secondary",
    "recurrent",
    "refractory",
    "severe",
    "mild",
    "early",
    "late",
    "high",
    "low",
    "increased",
    "decreased",
    "raised",
    "reduced",
    "elevated",
    "risk",
    "factor",
    "factors",
];

/// A typed word covers a wanted one when they match, or share a stem that
/// is most of the wanted word ("bleed" for "bleeding", not "hepa" for
/// "hepatorenal").
fn word_hit(want: &str, have: &[String]) -> bool {
    let want_len = want.chars().count();
    have.iter().any(|h| {
        h == want || {
            let common = want.chars().zip(h.chars()).take_while(|(a, b)| a == b).count();
            common >= 4 && common * 5 >= want_len * 3
        }
    })
}

/// Which points a free-typed recall already covers, to pre-tick them. A point
/// counts when at least half of its key words appear somewhere in what was
/// typed, its most specific word among them ("cardiomyopathy" covers
/// "Cirrhotic cardiomyopathy"; "pulmonary hypertension" doesn't cover
/// "Portal hypertension").
pub fn recalled(points: &[String], typed: &str) -> Vec<bool> {
    let have = key_words(typed);
    points
        .iter()
        .map(|p| {
            let want = key_words(p);
            if want.is_empty() || have.is_empty() {
                return false;
            }
            let hits = want.iter().filter(|w| word_hit(w, &have)).count();
            let key =
                want.iter().filter(|w| !GENERIC.contains(&w.as_str())).max_by_key(|w| w.chars().count());
            hits * 2 >= want.len() && key.is_none_or(|k| word_hit(k, &have))
        })
        .collect()
}

/// The headings of an essay outline with their bullet points.
pub fn outline_sections(outline: &[OutlineEntry]) -> Vec<(String, Vec<String>)> {
    outline
        .iter()
        .flat_map(|e| match e {
            OutlineEntry::Heading(h) => vec![(h.clone(), Vec::new())],
            OutlineEntry::Section(map) => map.iter().map(|(h, b)| (h.clone(), b.clone())).collect(),
        })
        .collect()
}
