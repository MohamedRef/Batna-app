use super::*;

fn item(yaml: &str) -> Item {
    serde_yaml_ng::from_str(yaml).unwrap()
}

fn lesson() -> Lesson {
    serde_yaml_ng::from_str(
        r#"
id: hb.L01.lesson1
title: "الكبد بيتليف إزاي؟"
hook: "hook"
minutes: 5
pages: [1, 2]
status: unverified
recap: []
steps:
  - kind: explain
    page: 1
    text: "Cirrhosis is **fibrosis** with nodules."
    quote: "Serum ferritin is only in the quote."
  - kind: explain
    text: |
      - Iron overload: ==serum== **Ferritin** above 1000.
      - الإستسقاء بيحصل.
  - kind: check
    question: "Which drug treats hepatic encephalopathy?"
    options: ["Lactulose", "Propranolol"]
    answer: 0
    why: ["Right", "Wrong"]
"#,
    )
    .unwrap()
}

fn items() -> Vec<Item> {
    vec![
        item(
            "id: hb.L01.001\nkind: card\nfront: Raised ferritin?\nback: Haemochromatosis.\nsrc: { page: 1 }\nstatus: unverified",
        ),
        item(
            "id: hb.L01.002\nkind: cloze\ntext: First line for HE is {{c1::lactulose::a sugar}}.\nsrc: { page: 2 }\nstatus: unverified",
        ),
        item(
            "id: hb.L01.003\nkind: card\nfront: Ferritin, rejected\nback: x\nsrc: { page: 2 }\nstatus: rejected",
        ),
    ]
}

fn index() -> Index {
    let l = lesson();
    Index::build([&l], items().iter())
}

#[test]
fn a_word_inside_a_step_finds_the_lesson_at_that_step() {
    let found = index().find(&words("ferritin"));
    assert_eq!(found.lessons, vec![LessonHit { lesson: "hb.L01.lesson1".into(), step: 1, steps: 1 }]);
    // The rejected card is not searched; the quote is not either (step 0).
    assert_eq!(found.items, vec!["hb.L01.001".to_string()]);
}

#[test]
fn every_word_must_appear_and_case_and_spelling_fold() {
    let ix = index();
    assert_eq!(ix.find(&words("LACTULOSE encephalopathy")).lessons[0].step, 2);
    assert!(ix.find(&words("lactulose ferritin")).items.is_empty());
    // Spread over two steps: still the lesson, opened where most words are.
    let spread = ix.find(&words("nodules ferritin"));
    assert_eq!(spread.lessons, vec![LessonHit { lesson: "hb.L01.lesson1".into(), step: 0, steps: 0 }]);
    assert_eq!(ix.find(&words("الاستسقاء")).lessons.len(), 1);
    assert!(ix.find(&words("warfarin")).is_empty());
    assert!(ix.find(&[]).is_empty());
}

#[test]
fn cloze_answers_are_searched_without_their_markup() {
    assert_eq!(
        cloze_plain("is {{c1::lactulose::a sugar}} and {{c2::rifaximin}}."),
        "is lactulose and rifaximin."
    );
    assert_eq!(cloze_plain("open {{c1::x"), "open {{c1::x");
    assert_eq!(index().find(&words("lactulose")).items, vec!["hb.L01.002".to_string()]);
    assert!(index().find(&words("c1")).items.is_empty());
}

#[test]
fn snippet_quotes_the_original_spelling_around_the_match() {
    let l = lesson();
    let s = snippet(&step_text(&l.steps[1]), &words("ferritin"), 10, 8).unwrap();
    assert_eq!(s.hit, "Ferritin");
    assert!(s.before.starts_with('…') && s.before.ends_with("serum "), "{s:?}");
    assert_eq!(s.after, " above 1…");
    // Arabic folds but the quote keeps the hamza as written.
    let ar = snippet(&step_text(&l.steps[1]), &words("الاستسقاء"), 3, 3).unwrap();
    assert_eq!(ar.hit, "الإستسقاء");
    assert!(snippet(&step_text(&l.steps[0]), &words("ferritin"), 5, 5).is_none());
}

#[test]
fn words_are_folded_and_deduplicated() {
    assert_eq!(
        words("  Ferritin ferritin  D-dimer "),
        vec!["ferritin".to_string(), "d".to_string(), "dimer".to_string()]
    );
    assert_eq!(fold("Child–Pugh"), fold("child pugh"));
    assert_eq!(fold("كُلَى ٤٨"), "كلي 48");
}
