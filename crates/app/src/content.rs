//! The study content built into the app: build.rs packs it
//! (`batna_core::pack`) and this unpacks it on first use. A newer pack from
//! the update feed replaces it, at start or live once he is between
//! sessions ([`Library::reload`]), with no app install.

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use batna_core::content::{Index, Lecture};
use batna_core::lesson::{Lesson, LessonFile};
use batna_core::pack::Pack;
use batna_core::program::Program;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

pub struct Library {
    pub index: Index,
    /// Written lectures, by id (`hb.L01`).
    pub lectures: BTreeMap<String, Lecture>,
    /// Lesson files, by lecture id (`hb.L01`), in book order.
    pub lessons: BTreeMap<String, LessonFile>,
    /// The MSc's courses (`content/program.yaml`).
    pub program: Program,
}

/// The library in use. Each one is leaked: screens hold `&'static` items,
/// and it is replaced at most a few times a session.
static LIB: RwLock<Option<&'static Library>> = RwLock::new(None);
/// Bumped each time the library is replaced, so caches built from it know.
static GENERATION: AtomicU64 = AtomicU64::new(0);

impl Library {
    pub fn get() -> &'static Library {
        if let Some(lib) = *LIB.read().unwrap_or_else(|e| e.into_inner()) {
            return lib;
        }
        let mut lib = LIB.write().unwrap_or_else(|e| e.into_inner());
        match *lib {
            Some(l) => l,
            None => {
                let l: &'static Library = Box::leak(Box::new(Library::load()));
                *lib = Some(l);
                l
            }
        }
    }

    /// Opens the newest pack again (one the update feed just delivered).
    /// Screens built from the old one must be shown again afterwards.
    pub fn reload() {
        let fresh: &'static Library = Box::leak(Box::new(Library::load()));
        *LIB.write().unwrap_or_else(|e| e.into_inner()) = Some(fresh);
        GENERATION.fetch_add(1, Ordering::SeqCst);
    }

    /// Changes each time the library is replaced.
    pub fn generation() -> u64 {
        GENERATION.load(Ordering::SeqCst)
    }

    fn load() -> Library {
        // A newer pack from the update feed wins (checked and decoded
        // there); else the embedded one, which build.rs wrote from types
        // it just parsed, so it can't fail.
        let pack = crate::update::installed_pack()
            .unwrap_or_else(|| Pack::decode(embedded::PACK).expect("embedded content pack"));
        let program = pack.program_or_fallback();
        let Pack { index, lectures, lessons, look, .. } = pack;
        crate::look::set(look);
        let lectures = lectures.into_iter().map(|l| (l.lecture.clone(), l)).collect();
        let lessons = lessons.into_iter().map(|l| (l.lecture.clone(), l)).collect();
        Library { index, lectures, lessons, program }
    }

    /// Every lesson in book order, with its lecture id.
    pub fn all_lessons(&self) -> impl Iterator<Item = (&str, &Lesson)> {
        self.index
            .lectures
            .iter()
            .filter_map(|e| self.lessons.get(&e.id))
            .flat_map(|f| f.lessons.iter().map(move |l| (f.lecture.as_str(), l)))
    }

    pub fn lesson(&self, id: &str) -> Option<(&Lecture, &Lesson)> {
        let lecture = id.rsplit_once('.').map(|(l, _)| l)?;
        let lesson = self.lessons.get(lecture)?.lessons.iter().find(|l| l.id == id)?;
        Some((self.lectures.get(lecture)?, lesson))
    }
}

impl Library {
    /// A lecture's title as the index spells it, so every screen agrees.
    pub fn title<'a>(&'a self, id: &'a str) -> &'a str {
        let indexed = self.index.lectures.iter().find(|e| e.id == id).map(|e| e.title.as_str());
        indexed.or_else(|| self.lectures.get(id).map(|l| l.title.as_str())).unwrap_or(id)
    }
}

/// Bumped when the library is replaced while Batna is open; the shell
/// draws its screens again under this key (`update::view::use_content_switch`).
pub static REV: dioxus::prelude::GlobalSignal<u64> = dioxus::prelude::Signal::global(|| 0);

/// Something built from the library once, and again after it is replaced.
pub struct Derived<T: 'static> {
    cell: RwLock<Option<(u64, &'static T)>>,
}

impl<T: Send + Sync + 'static> Derived<T> {
    pub const fn new() -> Self {
        Self { cell: RwLock::new(None) }
    }

    pub fn get(&self, build: impl FnOnce(&'static Library) -> T) -> &'static T {
        let generation = Library::generation();
        if let Some((g, value)) = *self.cell.read().unwrap_or_else(|e| e.into_inner())
            && g == generation
        {
            return value;
        }
        let value: &'static T = Box::leak(Box::new(build(Library::get())));
        *self.cell.write().unwrap_or_else(|e| e.into_inner()) = Some((generation, value));
        value
    }
}

/// `hb.L1` or `hb.L01` → `L01`, `hem.L03` → `HEM L03`: the one way every
/// screen names a lecture. Hepatology, the first course, keeps the bare
/// number it always had; every other course says which it is.
pub fn lecture_code(id: &str) -> String {
    let n = id.rsplit('.').next().unwrap_or(id).trim_start_matches('L').trim_start_matches('0');
    match batna_core::program::course_of(id) {
        "hb" => format!("L{n:0>2}"),
        c if c != id => format!("{} L{n:0>2}", c.to_uppercase()),
        _ => format!("L{n:0>2}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_codes_and_titles_are_one_spelling() {
        assert_eq!(lecture_code("hb.L01"), "L01");
        assert_eq!(lecture_code("hb.L1"), "L01");
        assert_eq!(lecture_code("hb.L20"), "L20");
        assert_eq!(lecture_code("hem.L3"), "HEM L03");
        assert_eq!(lecture_code("L05"), "L05");
        let lib = Library::get();
        let first = &lib.index.lectures[0];
        assert_eq!(lib.title(&first.id), first.title);
    }

    #[test]
    fn every_embedded_file_parses() {
        let lib = Library::get();
        assert!(lib.index.lectures.iter().filter(|l| l.id.starts_with("hb.")).count() == 20);
        let content = concat!(env!("CARGO_MANIFEST_DIR"), "/../../content");
        let written: usize = lib
            .program
            .courses
            .iter()
            .filter_map(|c| std::fs::read_dir(format!("{content}/{}", c.id)).ok())
            .flat_map(|dir| dir.flatten())
            .filter(|f| f.path().extension().is_some_and(|e| e == "yaml"))
            .count();
        assert_eq!(lib.lectures.len(), written, "every lecture file made it into the pack");
    }
}
