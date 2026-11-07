//! macOS-style quick navigation for sections, lectures, and lessons.
//! The on-device Library is the only search index; no network or sync
//! mutation is needed to jump directly into existing content.

use dioxus::prelude::*;

use crate::content::Library;
use crate::settings::SettingsNav;
use crate::{Screen, Ui};

use super::{PALETTE_OPEN, open_palette};

const MAX_RESULTS: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    Screen(Screen),
    Settings,
    Lecture(String),
    Lesson(String),
}

#[derive(Clone, Debug)]
struct Hit {
    id: String,
    name: String,
    subtitle: String,
    destination: Destination,
}

fn matches_query(query: &str, words: &str) -> bool {
    let haystack = words.to_lowercase();
    query.split_whitespace().all(|word| haystack.contains(word))
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
                subtitle: "قسم".into(),
                destination: Destination::Screen(screen),
            });
        }
    }
    if q.is_empty() || matches_query(&q, "الإعدادات settings preferences") {
        hits.push(Hit {
            id: "screen-settings".into(),
            name: Screen::Settings.label().into(),
            subtitle: "إعدادات التطبيق".into(),
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
                subtitle: format!("محاضرة · {context}"),
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
                subtitle: format!("درس · {context}"),
                destination: Destination::Lesson(lesson.id.clone()),
            });
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
        Destination::Lecture(id) => {
            settings.close();
            crate::lecture::open(ui, &id);
        }
        Destination::Lesson(id) => {
            settings.close();
            crate::learn::open(ui, &id);
        }
    }
}

/// Always mounted in the app shell; the dialog itself mounts only when
/// `Cmd+K` / `Ctrl+K` is pressed. The focus starts in the search box.
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
            onclick: move |_| open_palette(false),
        }
        section {
            class: "command-palette",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "command-palette-title",
            onmounted: move |_| {
                document::eval("document.querySelector('.palette-search')?.focus();");
            },
            div { class: "command-palette-head",
                h2 { id: "command-palette-title", "الانتقال السريع" }
                button {
                    r#type: "button",
                    class: "icon-button",
                    "aria-label": "إغلاق البحث السريع",
                    "data-back": "true",
                    "aria-keyshortcuts": "Escape",
                    onclick: move |_| open_palette(false),
                    "×"
                }
            }
            div { class: "command-search-row",
                input {
                    class: "command-search palette-search",
                    r#type: "search",
                    "aria-label": "ابحث عن قسم أو محاضرة أو درس",
                    placeholder: "ابحث عن قسم، محاضرة أو درس…",
                    autocomplete: "off",
                    autofocus: true,
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                }
                kbd { "⌘K" }
            }
            nav { class: "command-results", aria_label: "نتائج البحث السريع",
                if results.is_empty() {
                    p { class: "command-empty", role: "status", "مفيش نتائج مطابقة." }
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
            p { class: "command-palette-hint", "↑ ↓ للتنقل · Enter للفتح · Esc للإغلاق" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_shows_existing_sections_and_settings_only() {
        let lib = Library::get();
        let hits = find("", lib);
        assert_eq!(hits.len(), Screen::NAV.len() + 1);
        assert!(hits.iter().any(|h| h.destination == Destination::Settings));
        assert!(hits.iter().all(|h| matches!(h.destination, Destination::Screen(_) | Destination::Settings)));
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
    fn matches_all_words_in_any_case() {
        assert!(matches_query("Heart Failure", "Heart failure notes"));
        assert!(!matches_query("heart renal", "Heart failure notes"));
    }
}
