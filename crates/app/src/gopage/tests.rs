//! Adapted from Steps-app main: crates/app/src/gopage/mod.rs (tests).
use super::*;

#[test]
fn page_text_reads_numbers_ranges_and_arabic_digits() {
    assert_eq!(parse("30"), Some(30));
    assert_eq!(parse(" 30–35 "), Some(30));
    assert_eq!(parse("ص ٣٠"), Some(30));
    assert_eq!(parse("p. 42"), Some(42));
    assert_eq!(parse("0"), None);
    assert_eq!(parse("abc"), None);
    assert_eq!(parse(""), None);
    assert_eq!(page_query("ص 30"), Some(30));
    assert_eq!(page_query("page 7"), Some(7));
    assert_eq!(page_query("٣٠"), Some(30));
    assert_eq!(page_query("L02"), None);
    assert_eq!(page_query("hb 30"), None);
}

#[test]
fn printed_pages_find_their_lecture_in_the_chosen_book() {
    let lib = Library::get();
    let first =
        |course: &str| lib.index.lectures.iter().find(|e| e.id.starts_with(&format!("{course}."))).unwrap();
    let hb = first("hb");
    assert_eq!(
        target(lib, "hb", &hb.pages[0].to_string()),
        Target::Page { page: hb.pages[0], lecture: hb.id.clone() }
    );
    // The same number in Gastroenterology's book is its own lecture.
    let git = first("git");
    assert_eq!(
        target(lib, "git", &git.pages[0].to_string()),
        Target::Page { page: git.pages[0], lecture: git.id.clone() }
    );
    assert_eq!(target(lib, "hb", "nope"), Target::Invalid);
    // Past the book's last lecture: the nearest page that has one.
    let last = lib.index.lectures.iter().rfind(|e| e.id.starts_with("hb.")).unwrap();
    assert_eq!(
        target(lib, "hb", "9999"),
        Target::Nearest { asked: 9999, page: last.pages[1], lecture: last.id.clone() }
    );
    // Slides count in their own lecture only.
    assert_eq!(target(lib, "res.L02", "3"), Target::Page { page: 3, lecture: "res.L02".into() });
    assert!(matches!(target(lib, "res.L02", "999"), Target::Nearest { lecture, .. } if lecture == "res.L02"));
}

#[test]
fn a_page_is_found_in_every_book_that_has_it() {
    let lib = Library::get();
    let books = books_with(lib, 1);
    assert!(books.contains(&("hb".to_string(), "hb.L01".to_string())), "{books:?}");
    assert!(books.iter().any(|(c, _)| c == "git"), "{books:?}");
    assert!(books.iter().all(|(c, _)| c != "res"), "slides are no book page");
}

#[test]
fn a_slide_course_opens_on_its_first_lecture() {
    let lib = Library::get();
    assert_eq!(book_of_course(lib, "hb"), "hb");
    assert_eq!(book_of_course(lib, "res"), "res.L01");
    assert!(known(lib, "res.L02") && known(lib, "hb") && !known(lib, "cvs") && !known(lib, "res.L99"));
    let list = courses(lib);
    assert!(list.iter().any(|(id, _, slides)| id == "res" && *slides));
    assert!(list.iter().all(|(id, ..)| id != "cvs"), "only courses with lectures");
}

#[component]
fn Open(text: String, book: String) -> Element {
    use_hook(|| {
        *OPEN.write() = true;
        *TEXT.write() = text.clone();
        *BOOK.write() = Some(book.clone());
    });
    rsx! { GoPage {} }
}

fn render(book: &str, text: &str) -> String {
    crate::store::use_thread_store();
    let mut dom = VirtualDom::new_with_props(Open, OpenProps { text: text.into(), book: book.into() });
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

#[test]
fn the_box_names_the_lecture_or_offers_the_nearest_page() {
    let lib = Library::get();
    let html = render("hb", "1");
    assert!(html.contains(tx::TITLE) && html.contains(tx::GO));
    assert!(html.contains(&lib.crumb("hb.L01")), "{html}");
    let html = render("hb", "9999");
    assert!(html.contains(&tx::outside(9999, false)));
    assert!(render("hb", "x").contains(tx::INVALID));
    // A slide course asks for the lecture too.
    let html = render("res.L02", "3");
    assert!(html.contains(tx::SLIDE_TITLE) && html.contains("<select"), "{html}");
}
