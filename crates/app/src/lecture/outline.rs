//! A lecture laid out as the book is (#831): one row per page with the
//! book section, the lesson steps on it and how many cards it has. Each row
//! opens the page's sheet, which leads on to the step that explains it.
//! It also shows him which pages no lesson explains yet.

use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::count::{ITEM, count};
use crate::i18n::lecture as tx;
use crate::pagesheet::explain::{Explain, explains};

/// One page of the lecture.
#[derive(Clone, Debug, PartialEq)]
pub struct PageRow {
    pub page: u32,
    /// The book section most of the page's cards belong to.
    pub section: Option<String>,
    pub steps: Vec<Explain>,
    pub cards: usize,
}

/// Every page of lecture `id`, in book order.
pub fn rows(lib: &Library, id: &str) -> Vec<PageRow> {
    let Some(entry) = lib.index.lectures.iter().find(|e| e.id == id) else { return Vec::new() };
    let lecture = lib.lectures.get(id);
    let scope = crate::pagesheet::scope(lib, id);
    let prefix = format!("{id}.");
    (entry.pages[0]..=entry.pages[1])
        .map(|page| {
            let items: Vec<_> =
                lecture.map(|l| l.items.iter().filter(|i| i.src.page == page).collect()).unwrap_or_default();
            let mut tally: Vec<(&str, usize)> = Vec::new();
            for t in items.iter().filter_map(|i| i.topic.as_deref()) {
                match tally.iter_mut().find(|(x, _)| *x == t) {
                    Some((_, n)) => *n += 1,
                    None => tally.push((t, 1)),
                }
            }
            // Most cards wins; the first one in book order breaks a tie.
            let top = tally.iter().fold(None::<(&str, usize)>, |best, &(t, n)| match best {
                Some((_, b)) if b >= n => best,
                _ => Some((t, n)),
            });
            let section = top.and_then(|(t, _)| {
                lecture.and_then(|l| l.topics.iter().find(|x| x.id == t)).map(|x| x.title.clone())
            });
            let steps =
                explains(lib, page, &scope).into_iter().filter(|e| e.lesson.starts_with(&prefix)).collect();
            PageRow { page, section, steps, cards: items.len() }
        })
        .collect()
}

/// The collapsible «بترتيب الكتاب» section of a lecture's page.
#[component]
pub fn BookOrder(id: String) -> Element {
    let lib = Library::get();
    let rows = rows(lib, &id);
    if rows.is_empty() {
        return rsx! {};
    }
    let slides = lib.program.slides(&id);
    rsx! {
        details { class: "lec-section lec-outline",
            summary { class: "lec-outline-head",
                h2 { {tx::BOOK_ORDER} }
                span { class: "muted small", {tx::BOOK_ORDER_HINT} }
            }
            ul { class: "tap-list lec-outline-rows",
                for r in rows {
                    li { key: "{r.page}",
                        button {
                            class: "tap tap-row lec-outline-row",
                            "data-explained": if r.steps.is_empty() { "no" } else { "yes" },
                            onclick: {
                                let id = id.clone();
                                let page = r.page;
                                move |_| crate::pagesheet::open(page, &id, None, false, None)
                            },
                            span { class: "lec-outline-page num-ltr", aria_label: if slides { tx::SLIDES } else { tx::PAGES },
                                bdi { "{r.page}" }
                            }
                            span { class: "tap-row-main",
                                if let Some(s) = &r.section {
                                    bdi { dir: "ltr", lang: "en", class: "lecture-title", "{s}" }
                                }
                                span { class: "muted small",
                                    if r.steps.is_empty() {
                                        {tx::NO_STEP}
                                    } else {
                                        {r.steps.iter().map(|e| format!("{} · {}", e.title, crate::i18n::learn::step_n(e.step + 1))).collect::<Vec<_>>().join("، ")}
                                    }
                                    if r.cards > 0 { " · " {count(r.cards, &ITEM)} }
                                }
                            }
                            span { class: "chev", aria_hidden: "true" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_of_the_lecture_has_a_row_in_book_order() {
        let lib = Library::get();
        let id = "hem.L09";
        let e = lib.index.lectures.iter().find(|e| e.id == id).unwrap();
        let r = rows(lib, id);
        assert_eq!(r.len() as u32, e.pages[1] - e.pages[0] + 1);
        assert!(r.windows(2).all(|w| w[0].page + 1 == w[1].page));
        // The first page names its section and the lesson that teaches it.
        assert!(r[0].section.is_some(), "{:?}", r[0]);
        assert!(r[0].steps.iter().all(|s| s.lesson.starts_with("hem.L09.")));
        assert!(r.iter().any(|p| !p.steps.is_empty()));
        assert_eq!(
            r.iter().map(|p| p.cards).sum::<usize>(),
            lib.lectures[id].items.iter().filter(|i| (e.pages[0]..=e.pages[1]).contains(&i.src.page)).count()
        );
    }
}
