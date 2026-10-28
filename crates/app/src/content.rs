//! The study content built into the app: build.rs packs it
//! (`batna_core::pack`) and this unpacks it once, on first use.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use batna_core::content::{Index, Lecture};
use batna_core::pack::Pack;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

pub struct Library {
    pub index: Index,
    /// Written lectures, by id (`hb.L01`).
    pub lectures: BTreeMap<String, Lecture>,
}

impl Library {
    pub fn get() -> &'static Library {
        static LIB: OnceLock<Library> = OnceLock::new();
        LIB.get_or_init(|| {
            // A newer pack from the update feed wins (checked and decoded
            // there); else the embedded one, which build.rs wrote from types
            // it just parsed, so it can't fail.
            let Pack { index, lectures } = crate::update::installed_pack()
                .unwrap_or_else(|| Pack::decode(embedded::PACK).expect("embedded content pack"));
            let lectures = lectures.into_iter().map(|l| (l.lecture.clone(), l)).collect();
            Library { index, lectures }
        })
    }
}

impl Library {
    /// A lecture's title as the index spells it, so every screen agrees.
    pub fn title<'a>(&'a self, id: &'a str) -> &'a str {
        let indexed = self.index.lectures.iter().find(|e| e.id == id).map(|e| e.title.as_str());
        indexed.or_else(|| self.lectures.get(id).map(|l| l.title.as_str())).unwrap_or(id)
    }
}

/// `hb.L1` or `hb.L01` → `L01`: the one way every screen names a lecture.
pub fn lecture_code(id: &str) -> String {
    let n = id.rsplit('.').next().unwrap_or(id).trim_start_matches('L').trim_start_matches('0');
    format!("L{n:0>2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_codes_and_titles_are_one_spelling() {
        assert_eq!(lecture_code("hb.L01"), "L01");
        assert_eq!(lecture_code("hb.L1"), "L01");
        assert_eq!(lecture_code("hb.L20"), "L20");
        let lib = Library::get();
        let first = &lib.index.lectures[0];
        assert_eq!(lib.title(&first.id), first.title);
    }

    #[test]
    fn every_embedded_file_parses() {
        let lib = Library::get();
        assert_eq!(lib.index.lectures.len(), 20);
        let written = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/hb"))
            .unwrap()
            .flatten()
            .filter(|f| f.path().extension().is_some_and(|e| e == "yaml"))
            .count();
        assert_eq!(lib.lectures.len(), written, "every lecture file made it into the pack");
    }
}
