//! Each coverage rule on the test tree's lecture (`testkit`): one broken
//! thing, exactly its line.

use super::*;
use crate::testkit::{self, LECTURE, LESSON};

fn parsed(lesson: &str) -> (Lecture, LessonFile) {
    (serde_yaml_ng::from_str(LECTURE).unwrap(), serde_yaml_ng::from_str(lesson).unwrap())
}

fn check(pages: [u32; 2], lesson: &str, skips: &Skips) -> (Vec<String>, Vec<String>) {
    let (lecture, file) = parsed(lesson);
    problems(&lecture_row(pages, &lecture, &file, skips), skips)
}

#[test]
fn a_covered_lecture_has_nothing_to_say() {
    let (e, w) = check([1, 1], LESSON, &Skips::new());
    assert!(e.is_empty() && w.is_empty(), "{e:?} {w:?}");
}

#[test]
fn an_uncited_page_warns_until_the_lecture_is_reviewed_then_fails() {
    let (e, w) = check([1, 2], LESSON, &Skips::new());
    assert!(e.is_empty(), "{e:?}");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].starts_with("hb.L01: no lesson step cites p2 (0 items)"), "{w:?}");
    // Listed in lesson-skips.yaml: reviewed, so the gap is an error...
    let reviewed: Skips = serde_yaml_ng::from_str("hb.L01: {}").unwrap();
    let (e, w) = check([1, 2], LESSON, &reviewed);
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(w.is_empty());
    // ...until the page is skipped with a reason.
    let skipped: Skips = serde_yaml_ng::from_str("hb.L01: { 2: references }").unwrap();
    let (e, w) = check([1, 2], LESSON, &skipped);
    assert!(e.is_empty() && w.is_empty(), "{e:?} {w:?}");
}

#[test]
fn a_skip_needs_a_reason_and_a_page_of_the_lecture() {
    let bad: Skips = serde_yaml_ng::from_str("hb.L01: { 2: '', 40: blank }").unwrap();
    let (e, _) = check([1, 2], LESSON, &bad);
    assert!(e.iter().any(|p| p.contains("skips p2 without a reason")), "{e:?}");
    assert!(e.iter().any(|p| p.contains("skips p40, outside")), "{e:?}");
}

#[test]
fn pages_wider_than_the_steps_fail() {
    let wide = LESSON.replace("pages: [1, 1]", "pages: [1, 2]");
    let (e, _) = check([1, 2], &wide, &serde_yaml_ng::from_str("hb.L01: { 2: blank }").unwrap());
    assert_eq!(e, vec!["hb.L01.lesson1: pages 1–2 but its steps cite 1–1; set pages to what it teaches"]);
}

#[test]
fn a_practice_card_from_a_page_the_lesson_never_cites_warns() {
    let lecture =
        LECTURE.replace("src: { page: 1, quote: \"The hepatic", "src: { page: 2, quote: \"The hepatic");
    let lesson = LESSON.replace("practice: [hb.L01.001]", "practice: [hb.L01.001, hb.L01.002]");
    let lecture: Lecture = serde_yaml_ng::from_str(&lecture).unwrap();
    let file: LessonFile = serde_yaml_ng::from_str(&lesson).unwrap();
    let skips: Skips = serde_yaml_ng::from_str("hb.L01: { 2: shown elsewhere }").unwrap();
    let (e, w) = problems(&lecture_row([1, 2], &lecture, &file, &skips), &skips);
    assert!(e.is_empty(), "{e:?}");
    assert_eq!(w, vec!["hb.L01.lesson1: practice cards from pages no step of the lesson cites: hb.L01.002"]);
}

#[test]
fn a_wide_lesson_warns() {
    let wide = LESSON.replace("topics: [definition]", "topics: [definition, a, b]");
    let (_, w) = check([1, 1], &wide, &Skips::new());
    assert_eq!(
        w,
        vec!["hb.L01.lesson1: wide lesson (1 cards, 1 pages, 3 topics); split it at a topic boundary"]
    );
}

#[test]
fn depth_counts_words_and_explain_steps_without_a_reason() {
    let (lecture, file) = parsed(LESSON);
    let row = lecture_row([1, 1], &lecture, &file, &Skips::new());
    let l = &row.lessons[0];
    assert_eq!((l.explains, l.explains_no_reason), (2, 2));
    assert!(l.words > 20 && row.words_per_page() == l.words, "{l:?}");
    assert!(has_reason("الـ fibrosis بتحصل عشان الخلايا اتشدت"));
    assert!(!has_reason("1. Gender 2. Race 3. Genetic factors"));
}

#[test]
fn validate_fails_on_an_over_wide_lesson_and_reports_the_tree() {
    let dir = testkit::tree();
    let root = dir.path();
    let ok = crate::validate_all(root).unwrap();
    assert!(ok.contains("lesson coverage: 1 warning(s)"), "{ok}");
    testkit::write(root, "content/lessons/hb/L01.yaml", &LESSON.replace("pages: [1, 1]", "pages: [1, 3]"));
    let err = crate::validate_all(root).unwrap_err();
    assert!(err.iter().any(|p| p.starts_with("lessons coverage: hb.L01.lesson1: pages 1–3")), "{err:?}");
    // The course table and the worklist.
    testkit::write(root, "content/lessons/hb/L01.yaml", LESSON);
    let table = run(root, &testkit::strings(&["--summary"])).unwrap();
    assert!(table.contains("| hb | 1 | 22 | 21 | 0 | 0 | 0 | 0 |"), "{table}");
    let tsv = run(root, &testkit::strings(&["--report", "hb"])).unwrap();
    assert_eq!(tsv.lines().count(), 2, "{tsv}");
    assert!(tsv.lines().nth(1).unwrap().starts_with("hb.L01\thb.L01.lesson1\t1-1\t1-1\t1\t1\t3\t"), "{tsv}");
    let warn = run(root, &testkit::strings(&["--warnings", "hb"])).unwrap();
    assert!(warn.starts_with("warning: hb.L01: no lesson step cites p2"), "{warn}");
    testkit::write(root, "content/lesson-skips.yaml", "hb.L99: {}\n");
    assert!(run(root, &testkit::strings(&["--summary"])).is_err());
}
