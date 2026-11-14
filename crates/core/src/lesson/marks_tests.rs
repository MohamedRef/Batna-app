//! Marks on a figure's picture (#984): the YAML shape and its checks.

use super::*;

fn marks(yaml: &str) -> Vec<FigureMark> {
    serde_yaml_ng::from_str(yaml).unwrap()
}

fn figure(image: bool, marks: Vec<FigureMark>) -> StepBody {
    StepBody::Figure {
        title: None,
        svg: if image { String::new() } else { r#"<svg viewBox="0 0 1 1"></svg>"#.into() },
        image: image.then(|| Picture { file: "hb/a.webp".into(), book: true, ..Picture::default() }),
        caption: "شكل".into(),
        alt: "A liver biopsy.".into(),
        marks,
    }
}

fn problems(image: bool, m: Vec<FigureMark>) -> Vec<String> {
    step_problems(&figure(image, m), &BTreeSet::new())
}

const THREE: &str = r#"
- { at: [0.62, 0.30], label: "Mallory-Denk body" }
- { at: [0.20, 0.75], label: "Neutrophils", kind: arrow }
- { at: [0.50, 0.50], size: [0.30, 0.20], label: "Portal tract", kind: box }
"#;

#[test]
fn marks_read_from_yaml_with_point_as_the_default_kind() {
    let m = marks(THREE);
    assert_eq!(m.len(), 3);
    assert_eq!(m[0].kind, MarkKind::Point);
    assert_eq!(m[1].kind, MarkKind::Arrow);
    assert_eq!(m[2].size, Some([0.30, 0.20]));
    // Written back without the default kind, and read the same.
    let back = serde_yaml_ng::to_string(&m).unwrap();
    assert!(!back.contains("point"), "{back}");
    assert_eq!(serde_yaml_ng::from_str::<Vec<FigureMark>>(&back).unwrap(), m);
    assert!(
        serde_yaml_ng::from_str::<Vec<FigureMark>>("- { at: [0.1, 0.1], label: x, colour: red }").is_err()
    );
    assert!(
        serde_yaml_ng::from_str::<Vec<FigureMark>>("- { at: [0.1, 0.1], label: x, kind: circle }").is_err()
    );
    assert!(problems(true, m).is_empty());
}

#[test]
fn a_figure_step_takes_marks_and_the_labels_are_shown_text() {
    let yaml = r#"{ kind: figure, page: 3, image: { file: hb/a.webp, book: true }, caption: "c", alt: "a", marks: [{ at: [0.5, 0.5], label: "HSC" }] }"#;
    let step: Step = serde_yaml_ng::from_str(yaml).unwrap();
    assert!(allowed_keys("figure").unwrap().contains(&"marks"));
    let StepBody::Figure { marks, .. } = &step.body else { panic!("figure") };
    assert_eq!(marks[0].label, "HSC");
    // The abbreviation check reads the labels, as the step shows them.
    assert!(step.shown_text().iter().any(|t| t == "HSC"));
    // A figure without marks writes none (older packs read the same).
    let plain = serde_json::to_string(&figure(true, vec![])).unwrap();
    assert!(!plain.contains("marks"), "{plain}");
}

#[test]
fn marks_are_checked() {
    let one = |y: &str| marks(&format!("- {y}"));
    let p = |m: Vec<FigureMark>| problems(true, m);
    assert_eq!(
        problems(false, one("{ at: [0.5, 0.5], label: x }")),
        ["marks go on an image figure; a drawn (svg) figure labels its own parts"]
    );
    let nine: Vec<FigureMark> = (0..9).flat_map(|_| one("{ at: [0.5, 0.5], label: x }")).collect();
    assert_eq!(p(nine), ["9 marks: keep it to 8 or fewer on one picture"]);
    assert_eq!(
        p(one("{ at: [1.2, -0.1], label: x }")),
        ["mark 1: at [1.2, -0.1] must be fractions of the picture, from 0 to 1"]
    );
    assert_eq!(p(one("{ at: [0.5, 0.5], label: \" \" }")), ["mark 1 needs a label"]);
    assert_eq!(
        p(one("{ at: [0.5, 0.5], label: \"**HSC**\" }")),
        ["mark 1: the label is one plain line (no ** or ==)"]
    );
    assert_eq!(
        p(one("{ at: [0.5, 0.5], label: x, kind: box }")),
        ["mark 1: a box needs size: [width, height] (fractions of the picture)"]
    );
    assert_eq!(
        p(one("{ at: [0.5, 0.5], label: x, size: [0.1, 0.1] }")),
        ["mark 1: only a box or a bracket takes a size"]
    );
    assert_eq!(
        p(one("{ at: [0.9, 0.5], label: x, kind: bracket, size: [0.3, 0.1] }")),
        ["mark 1: the bracket runs past the picture's edge"]
    );
    assert_eq!(
        p(one("{ at: [0.5, 0.5], label: x, kind: box, size: [0, 0.1] }")),
        ["mark 1: size [0, 0.1] must be fractions above 0, up to 1"]
    );
    // The edges themselves are on the picture.
    assert!(p(one("{ at: [0, 1], label: x }")).is_empty());
    assert!(p(one("{ at: [0.5, 0.5], label: x, kind: box, size: [1, 1] }")).is_empty());
}

#[test]
fn numbers_sit_where_the_picture_has_room() {
    let at = |x: f64, y: f64| FigureMark { at: [x, y], label: "x".into(), kind: MarkKind::Point, size: None };
    assert_eq!(at(0.3, 0.3).outward(), (-1, 1), "sideways out, up or down in");
    assert_eq!(at(0.7, 0.6).outward(), (1, -1));
    // Near the left or right edge it turns inwards.
    assert_eq!(at(0.05, 0.95).outward(), (1, -1));
    let bracket = |x: f64, y: f64, w: f64, h: f64| FigureMark {
        at: [x, y],
        label: "x".into(),
        kind: MarkKind::Bracket,
        size: Some([w, h]),
    };
    assert!(bracket(0.5, 0.5, 0.1, 0.4).upright());
    assert_eq!(bracket(0.5, 0.5, 0.1, 0.4).outward(), (-1, 0));
    assert_eq!(bracket(0.05, 0.5, 0.1, 0.4).outward(), (1, 0));
    assert!(!bracket(0.5, 0.5, 0.4, 0.1).upright());
    assert_eq!(bracket(0.5, 0.5, 0.4, 0.1).outward(), (0, 1));
    assert_eq!(bracket(0.5, 0.95, 0.4, 0.1).outward(), (0, -1));
}

#[test]
fn marks_start_hidden_before_a_question_about_the_picture() {
    let explain = || Step {
        body: StepBody::Explain { title: None, text: "t".into() },
        page: None,
        quote: None,
        remind: vec![],
    };
    let question = || Step {
        body: StepBody::Check { question: "q".into(), options: vec![], answer: 0, why: vec![] },
        ..explain()
    };
    let fig = || Step { body: figure(true, vec![]), ..explain() };
    let mut lesson: Lesson = serde_yaml_ng::from_str(
        "{ id: hb.L01.lesson1, title: t, hook: h, minutes: 3, pages: [1, 1], steps: [], recap: [r], status: unverified }",
    )
    .unwrap();
    lesson.steps = vec![fig(), question(), fig(), explain(), fig()];
    assert!(!lesson.marks_start_shown(0), "a question follows: look first");
    assert!(lesson.marks_start_shown(2));
    assert!(lesson.marks_start_shown(4), "the last step");
}
