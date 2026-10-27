//! The study content built into the app (see build.rs).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use batna_core::content::{Index, Lecture};

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
            // CI validates every file, so a parse failure here is a bug.
            let index: Index = serde_yaml_ng::from_str(embedded::INDEX).expect("content/index.yaml");
            let lectures = embedded::LECTURES
                .iter()
                .filter_map(|text| serde_yaml_ng::from_str::<Lecture>(text).ok())
                .map(|l| (l.lecture.clone(), l))
                .collect();
            Library { index, lectures }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_file_parses() {
        let lib = Library::get();
        assert_eq!(lib.index.lectures.len(), 20);
        assert_eq!(lib.lectures.len(), embedded::LECTURES.len());
    }
}
