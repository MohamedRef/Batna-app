use super::*;

/// #1350's terms with the lessons that teach each in a step and the cards
/// that use it, measured at `ba56f9d` (the issue's "milan criteria" row
/// is "milan" here: two of its lessons never say "criteria"). Content only
/// grows, so each must find at least that many.
const TERMS: &[(&str, usize, usize)] = &[
    ("ferritin", 19, 43),
    ("rifaximin", 7, 19),
    ("tolvaptan", 8, 18),
    ("schistocyte", 11, 26),
    ("lactulose", 5, 13),
    ("cinacalcet", 3, 14),
    ("d-dimer", 6, 11),
    ("milan", 3, 3),
    ("rituximab", 34, 83),
    ("octreotide", 28, 71),
    ("bisphosphonate", 18, 72),
    ("child-pugh", 21, 57),
];

#[test]
fn the_issue_terms_find_every_lesson_and_card_that_teaches_them() {
    for (term, lessons, items) in TERMS {
        let f = find(term).expect("long enough");
        assert!(f.lessons.len() >= *lessons, "{term}: {} lessons, want {lessons}", f.lessons.len());
        assert!(f.items.len() >= *items, "{term}: {} items, want {items}", f.items.len());
    }
}

#[test]
fn each_hit_quotes_its_text_around_the_word() {
    let f = find("Ferritin").unwrap();
    for h in &f.lessons {
        let s = step_snippet(&h.lesson, h.step, "Ferritin").expect(&h.lesson);
        assert!(s.hit.eq_ignore_ascii_case("ferritin"), "{s:?}");
    }
    for id in &f.items {
        let s = item_snippet(id, "Ferritin").expect(id);
        assert!(s.hit.eq_ignore_ascii_case("ferritin"), "{s:?}");
    }
}

#[test]
fn the_course_in_view_comes_first_with_counts_per_course() {
    let f = find("ferritin").unwrap();
    let courses = per_course(f.lessons.iter().map(|h| h.lesson.as_str()), None);
    assert!(courses.len() >= 3, "ferritin is taught in several courses: {courses:?}");
    let last = courses.last().unwrap().0.clone();
    let (lessons, items) = ordered(&f, Some(&last));
    assert!(lessons[0].lesson.starts_with(&format!("{last}.")), "{:?}", lessons[0]);
    let focused = per_course(lessons.iter().map(|h| h.lesson.as_str()), Some(&last));
    assert_eq!(focused[0].0, last);
    assert_eq!(focused.iter().map(|c| c.1).sum::<usize>(), f.lessons.len());
    assert_eq!(items.len(), f.items.len());
}

#[test]
fn a_lesson_reached_from_a_card_opens_where_the_word_is() {
    let f = find("rifaximin").unwrap();
    let h = &f.lessons[0];
    let (_, l) = Library::get().lesson(&h.lesson).unwrap();
    assert_eq!(batna_core::search::find_step(l, &words("rifaximin")), Some(h.step));
    assert_eq!(batna_core::search::find_step(l, &[]), None);
}
