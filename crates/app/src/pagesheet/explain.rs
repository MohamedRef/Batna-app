//! Which lesson steps explain a book page (#831): the page sheet lists them
//! so a page leads to its explanation, and the lecture's book-order outline
//! uses the same list.

use crate::content::Library;

use super::{Scope, in_scope};

/// One lesson step that cites a page.
#[derive(Clone, Debug, PartialEq)]
pub struct Explain {
    pub lesson: String,
    pub title: String,
    /// 0-based step index.
    pub step: usize,
}

/// Every lesson step citing `page` in `scope`'s book, in book order
/// (lecture, then lesson, then step).
pub fn explains(lib: &Library, page: u32, scope: &Scope) -> Vec<Explain> {
    lib.all_lessons()
        .filter(|(lec, _)| in_scope(lib, scope, lec))
        .flat_map(|(_, l)| {
            l.steps.iter().enumerate().filter(|(_, s)| s.page == Some(page)).map(|(k, _)| Explain {
                lesson: l.id.clone(),
                title: l.title.clone(),
                step: k,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cited_page_lists_its_steps_and_an_uncited_one_none() {
        let lib = Library::get();
        let (lec, l) = lib.all_lessons().find(|(lec, _)| lec.starts_with("hb.")).unwrap();
        let (k, page) = l.steps.iter().enumerate().find_map(|(k, s)| s.page.map(|p| (k, p))).unwrap();
        let scope = super::super::scope(lib, lec);
        let found = explains(lib, page, &scope);
        assert!(found.iter().any(|e| e.lesson == l.id && e.step == k), "{found:?}");
        assert!(found.iter().all(|e| e.lesson.starts_with("hb.")), "stays in the hepatology book");
        assert!(explains(lib, 99_999, &scope).is_empty());
    }
}
