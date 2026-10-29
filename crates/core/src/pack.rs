//! The offline content pack: `content/index.yaml` plus every written lecture,
//! as one DEFLATE-compressed JSON blob. `crates/app/build.rs` builds it from
//! the YAML and embeds it with `include_bytes!`; `batna-tools packs build`
//! writes the same bytes to `dist/packs/` and `--check`s them in CI.
//!
//! JSON rather than YAML so the app parses it fast on the phone, and rather
//! than a binary format because items use `#[serde(flatten)]` and untagged
//! enums, which only self-describing formats support.

use std::collections::BTreeMap;

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec_with_limit;
use serde::{Deserialize, Serialize};

use crate::content::{Index, Lecture};
use crate::lesson::LessonFile;
use crate::program::Program;

/// The first bytes of every pack. Bump the digit when the layout changes.
pub const MAGIC: &[u8; 8] = b"BATNAPK1";

/// The content types this build reads. Bump it whenever an item, lesson,
/// step or picture gains a field or a variant that an older app would drop
/// or misread: an older app then refuses the newer pack (and keeps its
/// built-in content) instead of showing it with pieces silently missing.
/// Packs from before the field read as 0.
pub const SCHEMA: u32 = 1;

/// Highest miniz level: slower to pack (only when content changes), not to unpack.
const LEVEL: u8 = 10;

/// An unpacked pack can't be bigger than this (the whole chapter is a few MB).
const MAX_UNPACKED: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    /// [`SCHEMA`] of the build that wrote it.
    #[serde(default)]
    pub schema: u32,
    pub index: Index,
    /// Written lectures, in index order.
    pub lectures: Vec<Lecture>,
    /// Lessons (`batna.lesson/v1`), in lecture order. Older packs have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lessons: Vec<LessonFile>,
    /// The app's stylesheets, by path under `crates/app` (`src/lecture/lecture.css`),
    /// so a change to the look reaches devices without a new app build. Only
    /// the build the feed names in `ui_build` uses them (`update::UpdateManifest`);
    /// the pack built into the app has none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub look: BTreeMap<String, String>,
    /// `content/program.yaml`: the MSc's courses. Older packs have none
    /// (`Program::fallback`); an older app ignores it and shows every lecture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<Program>,
}

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error("not a Batna content pack (bad header)")]
    Magic,
    #[error("the pack is corrupt: {0}")]
    Inflate(String),
    #[error("the pack's JSON doesn't match the content types: {0}")]
    Json(#[from] serde_json::Error),
    #[error("the pack needs content types {0}, this build reads up to {SCHEMA}")]
    TooNew(u32),
}

impl Pack {
    /// Orders the lectures as the index lists them; lectures it doesn't list go last.
    pub fn new(index: Index, mut lectures: Vec<Lecture>) -> Self {
        let pos = |id: &str| index.lectures.iter().position(|e| e.id == id).unwrap_or(usize::MAX);
        lectures.sort_by(|a, b| pos(&a.lecture).cmp(&pos(&b.lecture)).then(a.lecture.cmp(&b.lecture)));
        Pack { schema: SCHEMA, index, lectures, lessons: Vec::new(), look: BTreeMap::new(), program: None }
    }

    /// Adds the lesson files, in the lectures' order.
    pub fn with_lessons(mut self, mut lessons: Vec<LessonFile>) -> Self {
        let pos = |id: &str| self.index.lectures.iter().position(|e| e.id == id).unwrap_or(usize::MAX);
        lessons.sort_by(|a, b| pos(&a.lecture).cmp(&pos(&b.lecture)).then(a.lecture.cmp(&b.lecture)));
        self.lessons = lessons;
        self
    }

    /// Adds the app's stylesheets (path under `crates/app` → CSS).
    pub fn with_program(mut self, program: Program) -> Self {
        self.program = Some(program);
        self
    }

    /// The pack's program, or one course per lecture prefix for an older pack.
    pub fn program_or_fallback(&self) -> Program {
        self.program.clone().unwrap_or_else(|| Program::fallback(&self.index))
    }

    pub fn with_look(mut self, look: BTreeMap<String, String>) -> Self {
        self.look = look;
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let json = serde_json::to_vec(self).expect("content types always serialize");
        let mut out = MAGIC.to_vec();
        out.extend(compress_to_vec(&json, LEVEL));
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PackError> {
        let body = bytes.strip_prefix(MAGIC.as_slice()).ok_or(PackError::Magic)?;
        let json = decompress_to_vec_with_limit(body, MAX_UNPACKED)
            .map_err(|e| PackError::Inflate(format!("{e:?}")))?;
        let pack: Pack = serde_json::from_slice(&json)?;
        if pack.schema > SCHEMA {
            return Err(PackError::TooNew(pack.schema));
        }
        Ok(pack)
    }

    pub fn item_count(&self) -> usize {
        self.lectures.iter().map(|l| l.items.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Lecture {
        let text = include_str!("../../../fixtures/content/sample.yaml");
        serde_yaml_ng::from_str(text).unwrap()
    }

    fn index() -> Index {
        serde_yaml_ng::from_str(
            "schema: batna.content/v1\nlectures:\n  - { id: hb.L01, title: Liver Cirrhosis, pages: [1, 22], chunk: 10 }\n  - { id: hb.L02, title: Two, pages: [23, 36], chunk: 10 }\n",
        )
        .unwrap()
    }

    #[test]
    fn a_pack_for_newer_content_types_is_refused() {
        let mut pack = Pack::new(index(), vec![sample()]);
        assert_eq!(pack.schema, SCHEMA);
        pack.schema = SCHEMA + 1;
        assert!(matches!(Pack::decode(&pack.encode()), Err(PackError::TooNew(n)) if n == SCHEMA + 1));
        // A pack from before the field still opens.
        let json =
            serde_json::to_string(&Pack::new(index(), vec![sample()])).unwrap().replace("\"schema\":1,", "");
        let mut old = MAGIC.to_vec();
        old.extend(compress_to_vec(json.as_bytes(), LEVEL));
        assert_eq!(Pack::decode(&old).unwrap().schema, 0);
    }

    #[test]
    fn a_pack_round_trips_and_keeps_index_order() {
        let mut second = sample();
        second.lecture = "hb.L02".into();
        let pack = Pack::new(index(), vec![second, sample()]);
        assert_eq!(pack.lectures[0].lecture, "hb.L01");
        let bytes = pack.encode();
        assert!(bytes.starts_with(MAGIC));
        let back = Pack::decode(&bytes).unwrap();
        assert_eq!(back, pack);
        assert_eq!(back.item_count(), 18);
    }

    #[test]
    fn the_program_travels_and_old_packs_fall_back() {
        let pack = Pack::new(index(), vec![sample()]);
        assert_eq!(pack.program_or_fallback().courses[0].id, "hb");
        let program: Program =
            serde_yaml_ng::from_str(include_str!("../../../content/program.yaml")).unwrap();
        let pack = pack.with_program(program.clone());
        let back = Pack::decode(&pack.encode()).unwrap();
        assert_eq!(back.program.as_ref(), Some(&program));
        assert_eq!(back.program_or_fallback().courses.len(), 20);
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(matches!(Pack::decode(b"PK\x03\x04"), Err(PackError::Magic)));
        let mut bad = MAGIC.to_vec();
        bad.extend([1, 2, 3]);
        assert!(matches!(Pack::decode(&bad), Err(PackError::Inflate(_))));
    }
}
