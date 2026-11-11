//! The validator's rules, one broken at a time on a lesson that passes.

use super::*;

const LECTURE: &str = "schema: batna.content/v1\nlecture: hb.L01\ntitle: Liver Cirrhosis\nbook: { chapter: 1, pages: [1, 22] }\ntopics:\n  - { id: definition, title: Definition }\nitems:\n  - { id: hb.L01.001, kind: card, front: Cirrhosis?, back: Fibrosis and nodules., src: { page: 1, quote: fibrosis }, status: unverified }\n";

const FILE: &str = r#"schema: batna.lesson/v1
lecture: hb.L01
lessons:
  - id: hb.L01.lesson1
    title: "يعني إيه cirrhosis؟"
    hook: "حكاية الـ cirrhosis."
    minutes: 3
    pages: [1, 2]
    topics: [definition]
    status: unverified
    steps:
      - { kind: explain, page: 1, quote: fibrosis, text: "الـ **fibrosis** و الـ nodules." }
      - { kind: reveal, prompt: "إيه الاتنين؟", answer: "fibrosis + nodules" }
      - kind: check
        question: "Cirrhosis is?"
        options: [Fibrosis and nodules, Fat, Iron]
        answer: 0
        why: ["صح", "غلط", "غلط"]
    recap: ["fibrosis + nodules"]
    practice: [hb.L01.001]
"#;

fn lecture() -> Lecture {
    serde_yaml_ng::from_str(LECTURE).unwrap()
}

fn file() -> LessonFile {
    serde_yaml_ng::from_str(FILE).unwrap()
}

fn check(
    f: &LessonFile,
    approved: Option<&BTreeSet<String>>,
    keys: Vec<((usize, usize), Vec<String>)>,
) -> Vec<String> {
    let lec = lecture();
    let cx = LessonContext { lecture: &lec, approved, raw_step_keys: keys };
    validate(f, &cx).into_iter().map(|p| format!("{}: {}", p.at, p.message)).collect()
}

fn broken(change: impl FnOnce(&mut LessonFile)) -> Vec<String> {
    let mut f = file();
    change(&mut f);
    check(&f, None, Vec::new())
}

#[test]
fn the_sample_lesson_passes() {
    assert_eq!(check(&file(), None, Vec::new()), Vec::<String>::new());
}

#[test]
fn file_and_lesson_rules() {
    let l = "hb.L01.lesson1";
    assert_eq!(
        broken(|f| f.schema = "batna.lesson/v0".into()),
        ["hb.L01: schema is \"batna.lesson/v0\", expected \"batna.lesson/v1\""]
    );
    assert_eq!(
        broken(|f| f.lecture = "hb.L02".into()),
        [
            "hb.L02: lessons are for hb.L02 but the lecture is hb.L01",
            "hb.L01.lesson1: lesson id must be hb.L02.lessonN",
        ]
    );
    assert_eq!(broken(|f| f.lessons.clear()), ["hb.L01: no lessons"]);
    assert_eq!(broken(|f| f.lessons.push(f.lessons[0].clone())), [format!("{l}: lesson id is used twice")]);
    for bad in ["hb.L01.lesson0", "hb.L01.lesson", "hb.L01.lessonX", "hb.L01.lesson01"] {
        assert_eq!(
            broken(|f| f.lessons[0].id = bad.into()),
            [format!("{bad}: lesson id must be hb.L01.lessonN")]
        );
    }
    assert_eq!(broken(|f| f.lessons[0].hook = " ".into()), [format!("{l}: title and hook are required")]);
    assert_eq!(
        broken(|f| f.lessons[0].minutes = 16),
        [format!("{l}: minutes is 16: keep a lesson between 2 and 15 minutes")]
    );
    assert_eq!(broken(|f| f.lessons[0].minutes = 1).len(), 1);
    assert_eq!(
        broken(|f| f.lessons[0].pages = [20, 23]),
        [
            format!("{l}: pages 20-23 are not inside the lecture (1-22)"),
            format!("{l} step 1: page 1 is outside the lesson's pages 20-23")
        ]
    );
    assert_eq!(broken(|f| f.lessons[0].pages = [2, 1]).len(), 2, "backwards pages, and step 1 falls outside");
    assert_eq!(
        broken(|f| f.lessons[0].topics = vec!["ascites".into()]),
        [format!("{l}: topic `ascites` is not one of the lecture's topics")]
    );
    assert_eq!(
        broken(|f| f.lessons[0].recap.clear()),
        [format!("{l}: recap is empty: end with the take-aways")]
    );
    assert_eq!(
        broken(|f| f.lessons[0].practice.push("hb.L01.099".into())),
        [format!("{l}: practice item hb.L01.099 is not in hb.L01")]
    );
    assert_eq!(
        broken(|f| {
            f.lessons[0].steps.remove(2);
        }),
        [
            format!("{l}: a lesson needs at least 3 steps"),
            format!("{l}: a lesson needs at least one check, order, match, sort or case step"),
        ]
    );
    // Approved only once Mohamed did.
    let mut f = file();
    f.lessons[0].status = Status::Approved;
    assert_eq!(
        check(&f, None, Vec::new()),
        [format!("{l}: status approved, but Mohamed hasn't approved it (content/approved.txt)")]
    );
    let approved = BTreeSet::from([l.to_string()]);
    assert!(check(&f, Some(&approved), Vec::new()).is_empty());
}

#[test]
fn step_keys_pages_and_quotes() {
    let l = "hb.L01.lesson1";
    let keys = vec![((0, 0), ["kind", "page", "quote", "text", "titel"].map(String::from).to_vec())];
    assert_eq!(check(&file(), None, keys), [format!("{l} step 1: unknown key `titel` for a explain step")]);
    assert_eq!(
        broken(|f| f.lessons[0].steps[1].quote = Some("x".into())),
        [format!("{l} step 2: a quote needs its page")]
    );
    let figure = |book: bool| Step {
        body: StepBody::Figure {
            title: None,
            svg: String::new(),
            image: Some(Picture { file: "hb/a.webp".into(), book, ..Picture::default() }),
            caption: "شكل".into(),
            alt: "A liver".into(),
        },
        page: None,
        quote: None,
        remind: vec![],
    };
    assert_eq!(
        broken(|f| f.lessons[0].steps.push(figure(true))),
        [format!("{l} step 4: a book figure needs its page")]
    );
    assert_eq!(
        broken(|f| f.lessons[0].steps.push(figure(false))),
        [format!("{l} step 4: an image needs credit, license and source (or book: true for a book figure)")]
    );
}

fn step(yaml: &str) -> Vec<String> {
    let body: StepBody = serde_yaml_ng::from_str(yaml).unwrap();
    step_problems(&body, &BTreeSet::from(["hb.L01.001"]))
}

#[test]
fn each_kind_of_step_has_its_own_rules() {
    let ok = [
        "{ kind: explain, text: نص }",
        "{ kind: aside, tone: trap, text: فخ }",
        "{ kind: reveal, prompt: سؤال, answer: جواب }",
        "{ kind: mnemonic, word: \"A B\", parts: [{ key: a, text: x }, { key: B, text: y }] }",
        "{ kind: update, text: جديد, ref: AASLD 2023 }",
        "{ kind: order, prompt: رتب, steps: [a, b, c] }",
        "{ kind: match, prompt: وصّل, pairs: [[a, x], [b, y], [c, z]] }",
        "{ kind: sort, prompt: صنّف, bins: [x, y], cards: [{ text: a, bin: 0 }, { text: b, bin: 1 }, { text: c, bin: 1 }] }",
        "{ kind: case, stem: عيان, steps: [{ question: q, options: [a, b], answer: 1, why: w }, { reveal: r, question: q2, why: w2 }] }",
        "{ kind: table, title: t, columns: [a, b], rows: [[p, q]], hide: [[0, 1]] }",
        "{ kind: try, items: [hb.L01.001] }",
    ];
    for y in ok {
        assert_eq!(step(y), Vec::<String>::new(), "{y}");
    }
    let cases: &[(&str, &[&str])] = &[
        ("{ kind: explain, text: \" \" }", &["text is empty"]),
        ("{ kind: aside, tone: tip, text: \"\" }", &["text is empty"]),
        ("{ kind: reveal, prompt: \"\", answer: x }", &["prompt and answer are required"]),
        (
            "{ kind: mnemonic, word: AB, parts: [{ key: A, text: x }] }",
            &["a mnemonic needs at least 2 parts", "the keys spell \"a\", not the word \"AB\""],
        ),
        ("{ kind: update, text: x, ref: \"\" }", &["an update needs text and ref (guideline and year)"]),
        ("{ kind: update, text: x, ref: AASLD }", &["ref must name the guideline's year"]),
        (
            "{ kind: check, question: q, options: [a, a], answer: 2, why: [x] }",
            &[
                "a check has 2 options; use 3 to 5",
                "answer is not one of the options",
                "why needs one line per option",
                "two options are the same",
            ],
        ),
        ("{ kind: order, prompt: p, steps: [a, b] }", &["order needs 3 to 8 steps"]),
        (
            "{ kind: match, prompt: p, pairs: [[a, x], [b, x]] }",
            &["match needs 3 to 6 pairs", "two pairs have the same right side"],
        ),
        (
            "{ kind: sort, prompt: p, bins: [x], cards: [{ text: a, bin: 3 }] }",
            &[
                "sort needs 2 to 4 bins",
                "sort needs 3 to 10 cards",
                "a card's bin is out of range",
                "every bin needs a card",
            ],
        ),
        ("{ kind: case, stem: s, steps: [] }", &["a case needs steps"]),
        (
            "{ kind: case, stem: s, steps: [{ question: q, options: [a], answer: 1, why: w }, { question: q, options: [a], why: w }] }",
            &["case step 1: answer out of range", "case step 2: options without an answer"],
        ),
        (
            "{ kind: table, title: t, columns: [a], rows: [], hide: [[0, 0]] }",
            &["a table needs 2+ columns and a row", "a hidden cell is outside the table"],
        ),
        ("{ kind: table, title: t, columns: [a, b], rows: [[p]] }", &["every row needs one cell per column"]),
        ("{ kind: try, items: [] }", &["try 1 to 5 items"]),
        ("{ kind: try, items: [hb.L01.009] }", &["item hb.L01.009 is not in this lecture"]),
        (
            "{ kind: figure, svg: \"<svg viewBox='0 0 1 1'></svg>\", image: { file: a.webp, book: true }, caption: c, alt: \"\" }",
            &["a figure has either svg or image, not both", "a figure needs a caption and an alt text"],
        ),
        (
            "{ kind: explain, text: \"==a== ==b==\\n==c== ==d==\" }",
            &["4 key terms (`==`): keep the marker for the 3 that matter most"],
        ),
        (
            "{ kind: explain, text: \"**open\\n===x===\\n==y\" }",
            &[
                "text: `**` is not closed on its line",
                "text: `===` (write `==term==` with no extra =)",
                "text: `==` is not closed on its line",
            ],
        ),
    ];
    for (y, want) in cases {
        assert_eq!(step(y), *want, "{y}");
    }
}

#[test]
fn pictures_need_an_open_licence_and_a_safe_path() {
    let pic = |file: &str, licence: Option<&str>, source: Option<&str>, book: bool| Picture {
        file: file.into(),
        credit: Some("Wikimedia, J. Doe".into()),
        license: licence.map(Into::into),
        source: source.map(Into::into),
        book,
        data: String::new(),
    };
    let https = Some("https://commons.wikimedia.org/x");
    for l in ["CC BY-SA 4.0", "CC0", "Public domain", "PD-US", "cc-by-nc 3.0"] {
        assert_eq!(picture_problems(&pic("hb/a.webp", Some(l), https, false)), Vec::<String>::new(), "{l}");
    }
    assert_eq!(
        picture_problems(&pic("hb/a.webp", Some("CC BY-ND 4.0"), https, false)),
        ["license \"CC BY-ND 4.0\": use public domain, CC0 or a CC BY licence without ND (no-derivatives)"]
    );
    assert_eq!(picture_problems(&pic("hb/a.webp", Some("All rights reserved"), https, false)).len(), 1);
    assert_eq!(
        picture_problems(&pic("hb/a.webp", Some("CC0"), Some("http://x.org"), false)),
        ["source must be the https:// page the image came from"]
    );
    assert_eq!(
        picture_problems(&pic("../a.gif", Some("CC0"), https, false)),
        [
            "image file \"../a.gif\" must be .webp, .jpg or .png",
            "image file \"../a.gif\" must be a path under content/lessons/img/"
        ]
    );
    assert_eq!(
        picture_problems(&pic("hb/a.png", Some("CC0"), None, true)),
        ["a book figure takes no credit, license or source (the step's page cites it)"]
    );
    let mut filled = pic("hb/a.JPG", Some("CC0"), https, false);
    filled.data = "data:image/jpeg;base64,AA==".into();
    assert_eq!(picture_problems(&filled), ["image.data is filled in by the pack build; don't write it"]);
}

#[test]
fn embedding_reads_each_picture_once_and_flags_big_or_missing_files() {
    let mut f = file();
    let figure = |file: &str| Step {
        body: StepBody::Figure {
            title: None,
            svg: String::new(),
            image: Some(Picture { file: file.into(), book: true, ..Picture::default() }),
            caption: "c".into(),
            alt: "a".into(),
        },
        page: Some(1),
        quote: None,
        remind: vec![],
    };
    f.lessons[0].steps.extend([figure("hb/small.png"), figure("hb/big.webp"), figure("hb/gone.webp")]);
    let mut files = vec![f];
    let mut asked = Vec::new();
    let problems = embed_images(&mut files, |p| {
        asked.push(p.to_string());
        match p {
            "hb/small.png" => Some(vec![1, 2, 3]),
            "hb/big.webp" => Some(vec![0; MAX_IMAGE_BYTES + 1]),
            _ => None,
        }
    });
    assert_eq!(asked, ["hb/small.png", "hb/big.webp", "hb/gone.webp"]);
    assert_eq!(
        problems,
        [
            "hb.L01.lesson1 step 5: hb/big.webp is 150 KB; keep images under 150 KB (WebP, about 800 px wide)",
            "hb.L01.lesson1 step 6: content/lessons/img/hb/gone.webp is missing",
        ]
    );
    let data = |k: usize| match &files[0].lessons[0].steps[k].body {
        StepBody::Figure { image: Some(p), .. } => p.data.clone(),
        _ => unreachable!(),
    };
    assert_eq!(data(3), "data:image/png;base64,AQID");
    assert!(data(4).starts_with("data:image/webp;base64,AAAA"));
    assert_eq!(data(5), "", "a missing file stays empty");
}

#[test]
fn remind_lists_parse_stay_unique_and_skip_question_steps() {
    let explain = |remind: Vec<&str>| Step {
        body: StepBody::Explain { title: None, text: "ACLF".into() },
        page: None,
        quote: None,
        remind: remind.into_iter().map(String::from).collect(),
    };
    assert!(remind_problems(&explain(vec!["hb.L01.033", "hb.L01.lesson2:SHIBS"])).is_empty());
    let twice = remind_problems(&explain(vec!["hb.L01.033", "hb.L01.033"]));
    assert_eq!(twice, ["remind `hb.L01.033` is listed twice"]);
    assert_eq!(
        remind_problems(&explain(vec!["SHIBS"])),
        ["remind `SHIBS` is neither an item id nor `<lesson id>:<WORD>`"]
    );
    let mut question = explain(vec!["hb.L01.033"]);
    question.body = StepBody::Order { prompt: "p".into(), steps: vec!["a".into(), "b".into()], why: None };
    assert_eq!(
        remind_problems(&question),
        ["remind goes on an explain, figure, aside, mnemonic or open table step, not a order step"]
    );
}
