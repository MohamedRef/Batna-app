//! macOS-style quick navigation for screens, content, calculators, Settings and question practice.
//! The on-device Library is the only search index; no network or sync
//! mutation is needed to jump directly into existing content.

use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::palette as tx;
use crate::settings::{Section, SettingsNav};
use crate::{Screen, Ui};

use super::{PALETTE_OPEN, open_palette};

const MAX_RESULTS: usize = 12;

/// Closing search returns focus to the opener if that control still exists.
/// Selecting a search result does not restore focus to the old screen.
fn dismiss() {
    open_palette(false);
    document::eval(
        "requestAnimationFrame(() => { const el = window.__batnaPaletteOpener; if (el?.isConnected && el.getClientRects().length) el.focus({preventScroll:true}); });",
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    Screen(Screen),
    Settings,
    SettingsSection(Section),
    Lecture(String),
    Lesson(String),
    Calculator(&'static str),
    BankLecture(String),
    /// A printed page of a course's book, in this lecture (`crate::gopage`).
    Page(u32, String),
}

#[derive(Clone, Debug)]
struct Hit {
    id: String,
    name: String,
    subtitle: String,
    destination: Destination,
}

/// Every typed word must appear. Both sides go through the calculator
/// search's Arabic folding, so hamza forms, ة/ه, ى/ي, tashkeel and Arabic
/// digits don't hide a lecture or a lesson.
fn matches_query(query: &str, words: &str) -> bool {
    let haystack = crate::calc::fold(words);
    crate::calc::fold(query).split_whitespace().all(|word| haystack.contains(word))
}

/// Deliberate "questions <lecture>" requests open that lecture's existing
/// filtered practice setup, not an arbitrary question in a running paper.
fn bank_query(query: &str) -> Option<&str> {
    ["questions ", "question ", "bank ", "mcq ", "أسئلة ", "اسئلة ", "سؤال "]
        .into_iter()
        .find_map(|prefix| query.strip_prefix(prefix))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// The initial palette shows navigable sections, not a wall of hundreds of
/// lessons. Typed queries search the real lecture and lesson index.
fn find(query: &str, lib: &Library) -> Vec<Hit> {
    let q = query.trim().to_lowercase();
    let mut hits = Vec::new();
    for screen in Screen::NAV {
        let title = screen.label().to_string();
        let ascii = format!("{screen:?}");
        if q.is_empty() || matches_query(&q, &format!("{title} {ascii}")) {
            hits.push(Hit {
                id: format!("screen-{ascii}"),
                name: title,
                subtitle: tx::KIND_SCREEN.into(),
                destination: Destination::Screen(screen),
            });
        }
    }
    if q.is_empty() || matches_query(&q, "الإعدادات settings preferences") {
        hits.push(Hit {
            id: "screen-settings".into(),
            name: Screen::Settings.label().into(),
            subtitle: tx::KIND_SETTINGS.into(),
            destination: Destination::Settings,
        });
    }
    if q.is_empty() {
        return hits;
    }
    for lecture in &lib.index.lectures {
        let title = lib.title(&lecture.id);
        let context = lib.lecture_tag(&lecture.id);
        if matches_query(&q, &format!("{} {title} {context}", lecture.id)) {
            hits.push(Hit {
                id: format!("lecture-{}", lecture.id),
                name: title.into(),
                subtitle: tx::lecture(&context),
                destination: Destination::Lecture(lecture.id.clone()),
            });
        }
    }
    for (lecture_id, lesson) in lib.all_lessons() {
        let context = lib.crumb(lecture_id);
        if matches_query(&q, &format!("{} {} {context}", lesson.id, lesson.title)) {
            hits.push(Hit {
                id: format!("lesson-{}", lesson.id),
                name: lesson.title.clone(),
                subtitle: tx::lesson(&context),
                destination: Destination::Lesson(lesson.id.clone()),
            });
        }
    }
    // Settings sections and calculators use their existing searchable
    // metadata, rather than inventing another list of destinations.
    for section in Section::ALL {
        if section.applies() && section.matches(&q) {
            hits.push(Hit {
                id: format!("settings-{}", section.id()),
                name: section.title().into(),
                subtitle: tx::KIND_SETTINGS_SECTION.into(),
                destination: Destination::SettingsSection(section),
            });
        }
    }
    for spec in crate::calc::search_for_palette(&q) {
        hits.push(Hit {
            id: format!("calc-{}", spec.id),
            name: spec.name.into(),
            subtitle: tx::calculator(crate::calc::area_label(spec.area)),
            destination: Destination::Calculator(spec.id),
        });
    }
    // "30" or "ص 30": that page in every course book that has it.
    if let Some(page) = crate::gopage::page_query(&q) {
        for (course, lecture) in crate::gopage::books_with(lib, page) {
            hits.push(Hit {
                id: format!("page-{course}-{page}"),
                name: tx::page(&crate::i18n::cite::page(page), lib.course_short(&course)),
                subtitle: lib.crumb(&lecture),
                destination: Destination::Page(page, lecture),
            });
        }
    }
    if let Some(needle) = bank_query(&q) {
        for lecture in &lib.index.lectures {
            let title = lib.title(&lecture.id);
            let context = lib.lecture_tag(&lecture.id);
            if !matches_query(needle, &format!("{} {title} {context}", lecture.id)) {
                continue;
            }
            let has_questions = lib.lectures.get(&lecture.id).is_some_and(|l| {
                l.items.iter().any(|item| matches!(&item.body, batna_core::content::Body::Mcq { .. }))
            });
            if has_questions {
                hits.push(Hit {
                    id: format!("bank-{}", lecture.id),
                    name: tx::questions(title),
                    subtitle: tx::practice(&context),
                    destination: Destination::BankLecture(lecture.id.clone()),
                });
            }
        }
    }
    // Exact ids and titles first, then prefix matches, then general hits.
    // Stable sorting keeps the original library order for equal relevance.
    hits.sort_by_key(|hit| {
        let id = hit.id.to_lowercase();
        let name = hit.name.to_lowercase();
        if id == q || name == q {
            0
        } else if name.starts_with(&q) {
            1
        } else if id.contains(&q) {
            2
        } else {
            3
        }
    });
    hits.truncate(MAX_RESULTS);
    hits
}

/// Follow the same navigation code paths as tapping the existing screen,
/// lecture and lesson controls; no task or review is silently discarded.
fn activate(target: Destination, mut ui: Ui, settings: SettingsNav) {
    open_palette(false);
    match target {
        Destination::Screen(screen) => {
            settings.close();
            ui.screen.set(screen);
        }
        Destination::Settings => settings.show(None),
        Destination::SettingsSection(section) => settings.show(Some(section)),
        Destination::Calculator(id) => {
            settings.close();
            crate::calc::request_open(id);
            crate::case::open_calculators(ui);
        }
        Destination::BankLecture(id) => {
            settings.close();
            crate::bank::practise(ui, &id, None);
        }
        Destination::Lecture(id) => {
            settings.close();
            crate::lecture::open(ui, &id);
        }
        Destination::Lesson(id) => {
            settings.close();
            crate::learn::open(ui, &id);
        }
        Destination::Page(page, lecture) => {
            settings.close();
            crate::gopage::go(page, &lecture);
        }
    }
}

/// Touch trigger on the compact top bar. Search logic and destinations stay
/// in the same palette used by Cmd/Ctrl+K, with no duplicate index or route.
#[component]
pub fn MobileSearchButton() -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "icon-button topbar-search",
            "aria-label": tx::OPEN,
            title: tx::OPEN,
            onclick: move |_| open_palette(true),
            {crate::icons::search()}
        }
    }
}

/// Always mounted in the app shell; the dialog mounts when either a
/// keyboard shortcut or the phone search button opens the palette.
#[component]
pub fn Palette() -> Element {
    let ui = use_context::<Ui>();
    let settings = use_context::<SettingsNav>();
    let mut query = use_signal(String::new);
    let _ = crate::content::REV.read();
    if !*PALETTE_OPEN.read() {
        return rsx! {};
    }
    let results = find(query.read().as_str(), Library::get());
    rsx! {
        div {
            class: "sheet-backdrop command-backdrop",
            onclick: move |_| dismiss(),
        }
        section {
            class: "command-palette",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "command-palette-title",
            onmounted: move |_| {
                document::eval("window.__batnaPaletteOpener = document.activeElement; document.querySelector('.palette-search')?.focus();");
            },
            div { class: "command-palette-head",
                h2 { id: "command-palette-title", {tx::TITLE} }
                button {
                    r#type: "button",
                    class: "icon-button",
                    "aria-label": tx::CLOSE,
                    "data-back": "true",
                    "aria-keyshortcuts": "Escape",
                    onclick: move |_| dismiss(),
                    "×"
                }
            }
            div { class: "command-search-row",
                input {
                    class: "command-search palette-search",
                    r#type: "search",
                    "aria-label": tx::FIELD,
                    placeholder: tx::PLACEHOLDER,
                    autocomplete: "off",
                    autofocus: true,
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                }
                kbd { "⌘K" }
            }
            nav { class: "command-results", aria_label: tx::RESULTS,
                if results.is_empty() {
                    p { class: "command-empty", role: "status", {tx::EMPTY} }
                }
                for hit in results {
                    button {
                        key: "{hit.id}",
                        r#type: "button",
                        class: "palette-result",
                        onclick: {
                            let destination = hit.destination.clone();
                            move |_| activate(destination.clone(), ui, settings)
                        },
                        span { class: "palette-result-name", "{hit.name}" }
                        span { class: "palette-result-subtitle", "{hit.subtitle}" }
                    }
                }
            }
            p { class: "command-palette-hint", {tx::HINT} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_search_button_opens_shared_palette() {
        fn app() -> Element {
            use_hook(|| open_palette(false));
            rsx! { MobileSearchButton {} }
        }
        let mut dom = crate::tap_dom::TestDom::of(app);
        let label = format!("aria-label=\"{}\"", tx::OPEN);
        assert!(dom.html().contains(&label));
        dom.click(&label);
        dom.with(|| {
            assert!(*PALETTE_OPEN.peek(), "touch opens the same Cmd/Ctrl+K dialog");
            open_palette(false);
        });
    }

    #[test]
    fn empty_query_shows_existing_sections_and_settings_only() {
        let lib = Library::get();
        let hits = find("", lib);
        assert_eq!(hits.len(), Screen::NAV.len() + 1);
        assert!(hits.iter().any(|h| h.destination == Destination::Settings));
        assert!(hits.iter().all(|h| matches!(h.destination, Destination::Screen(_) | Destination::Settings)));
    }

    #[test]
    fn a_page_number_offers_that_page_in_each_course_book() {
        let lib = Library::get();
        let hits = find("ص 1", lib);
        let pages: Vec<&Destination> =
            hits.iter().map(|h| &h.destination).filter(|d| matches!(d, Destination::Page(..))).collect();
        assert!(pages.contains(&&Destination::Page(1, "hb.L01".into())), "{pages:?}");
        // Gastroenterology's book starts again at page 1 too.
        assert!(
            pages.iter().any(|d| matches!(d, Destination::Page(1, l) if l.starts_with("git."))),
            "{pages:?}"
        );
        // Slides are no book page.
        assert!(pages.iter().all(|d| !matches!(d, Destination::Page(_, l) if l.starts_with("res."))));
        assert!(find("hepatology", lib).iter().all(|h| !matches!(h.destination, Destination::Page(..))));
    }

    #[test]
    fn real_lecture_and_lesson_ids_are_searchable() {
        let lib = Library::get();
        let lecture = lib.index.lectures.first().expect("library has lectures");
        let lecture_hits = find(&lecture.id, lib);
        assert!(lecture_hits.iter().any(|h| h.destination == Destination::Lecture(lecture.id.clone())));
        let lesson = lib.all_lessons().next().expect("library has lessons").1;
        let lesson_hits = find(&lesson.id, lib);
        assert!(lesson_hits.iter().any(|h| h.destination == Destination::Lesson(lesson.id.clone())));
        assert!(lesson_hits.len() <= MAX_RESULTS);
    }

    #[test]
    fn calculator_and_settings_results_open_real_destinations() {
        let lib = Library::get();
        assert!(
            find("child pugh", lib).iter().any(|h| h.destination == Destination::Calculator("child_pugh"))
        );
        if Section::Mac.applies() {
            assert!(
                find("keyboard shortcuts", lib)
                    .iter()
                    .any(|h| h.destination == Destination::SettingsSection(Section::Mac))
            );
        }
        assert!(
            find("update download", lib)
                .iter()
                .any(|h| h.destination == Destination::SettingsSection(Section::Updates))
        );
    }

    #[test]
    fn explicit_question_search_only_targets_lectures_with_mcqs() {
        let lib = Library::get();
        let lecture = lib
            .index
            .lectures
            .iter()
            .find(|entry| {
                lib.lectures.get(&entry.id).is_some_and(|l| {
                    l.items.iter().any(|item| matches!(&item.body, batna_core::content::Body::Mcq { .. }))
                })
            })
            .expect("a lecture has MCQs");
        let hits = find(&format!("questions {}", lecture.id), lib);
        assert!(hits.iter().any(|h| h.destination == Destination::BankLecture(lecture.id.clone())));
        assert!(bank_query("bank").is_none());
        assert!(bank_query("questions ").is_none());
    }

    #[test]
    fn matches_all_words_in_any_case() {
        assert!(matches_query("Heart Failure", "Heart failure notes"));
        assert!(!matches_query("heart renal", "Heart failure notes"));
    }

    #[test]
    fn arabic_spelling_variants_still_match() {
        assert!(matches_query("الاستسقاء", "الإستسقاء والكبد"));
        assert!(matches_query("كلي", "أمراض الكُلى"));
        assert!(matches_query("محاضره ٣", "محاضرة 3"));
        assert!(!matches_query("قلب", "أمراض الكُلى"));
    }
}
