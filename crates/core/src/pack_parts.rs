//! The content pack in two parts (#1354): the text (everything but the
//! lesson pictures) and the pictures, so a one-word fix doesn't download
//! 16 MB of pictures that did not change.
//!
//! The full pack stays the one the feed signs and devices keep: a device
//! that fetches the parts joins them and encodes the result, and takes it
//! only when those bytes hash to the full pack's signed SHA-256. The parts
//! need no trust of their own, and a mismatch (another encoder, a part
//! that doesn't fit) falls back to downloading the full pack.

use std::collections::BTreeMap;

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec_with_limit;

use super::{LEVEL, MAX_UNPACKED, Pack, PackError};
use crate::lesson::StepBody;

/// The first bytes of a pictures part.
pub const PICTURES_MAGIC: &[u8; 8] = b"BATNAPC1";

/// Every lesson picture's `data:` URL, by file name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pictures(pub BTreeMap<String, String>);

impl Pictures {
    pub fn encode(&self) -> Vec<u8> {
        let json = serde_json::to_vec(&self.0).expect("strings always serialize");
        let mut out = PICTURES_MAGIC.to_vec();
        out.extend(compress_to_vec(&json, LEVEL));
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PackError> {
        let body = bytes.strip_prefix(PICTURES_MAGIC.as_slice()).ok_or(PackError::Magic)?;
        let json = decompress_to_vec_with_limit(body, MAX_UNPACKED)
            .map_err(|e| PackError::Inflate(format!("{e:?}")))?;
        Ok(Pictures(serde_json::from_slice(&json)?))
    }
}

impl Pack {
    fn figures_mut(&mut self) -> impl Iterator<Item = &mut crate::lesson::Picture> {
        self.lessons.iter_mut().flat_map(|f| &mut f.lessons).flat_map(|l| &mut l.steps).filter_map(|s| {
            match &mut s.body {
                StepBody::Figure { image: Some(p), .. } => Some(p),
                _ => None,
            }
        })
    }

    /// The lesson pictures this pack carries.
    pub fn pictures(&self) -> Pictures {
        let mut out = BTreeMap::new();
        for p in self.lessons.iter().flat_map(|f| &f.lessons).flat_map(|l| &l.steps).filter_map(|s| match &s
            .body
        {
            StepBody::Figure { image: Some(p), .. } if !p.data.is_empty() => Some(p),
            _ => None,
        }) {
            out.entry(p.file.clone()).or_insert_with(|| p.data.clone());
        }
        Pictures(out)
    }

    /// The text part and the pictures part.
    pub fn split(&self) -> (Pack, Pictures) {
        let pictures = self.pictures();
        let mut text = self.clone();
        text.figures_mut().for_each(|p| p.data.clear());
        (text, pictures)
    }

    /// The text part with its pictures put back. Fails naming a picture the
    /// pictures part doesn't have.
    pub fn join(mut self, pictures: &Pictures) -> Result<Pack, String> {
        for p in self.figures_mut() {
            match pictures.0.get(&p.file) {
                Some(data) => p.data.clone_from(data),
                None => return Err(format!("the pictures part has no {}", p.file)),
            }
        }
        Ok(self)
    }
}

/// The full pack's bytes from its two parts' bytes: what a device checks
/// against the signed SHA-256 of the full pack.
pub fn join_parts(text: &[u8], pictures: &[u8]) -> Result<Vec<u8>, String> {
    let pictures = Pictures::decode(pictures).map_err(|e| format!("pictures part: {e}"))?;
    let text = Pack::decode(text).map_err(|e| format!("text part: {e}"))?;
    Ok(text.join(&pictures)?.encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lesson::LessonFile;

    fn with_pictures() -> Pack {
        let lessons: LessonFile = serde_yaml_ng::from_str(
            r#"
schema: batna.lesson/v1
lecture: hb.L01
lessons:
  - id: hb.L01.lesson1
    title: t
    hook: h
    minutes: 5
    pages: [1, 2]
    status: unverified
    recap: []
    steps:
      - kind: figure
        image: { file: a.webp, book: true }
        caption: c
        alt: a
      - kind: figure
        image: { file: a.webp, book: true }
        caption: again
        alt: a
      - kind: figure
        image: { file: b.webp, book: true }
        caption: c
        alt: b
"#,
        )
        .unwrap();
        let mut p = Pack::new(super::super::tests::index(), vec![super::super::tests::sample()])
            .with_lessons(vec![lessons]);
        for (k, pic) in p.figures_mut().enumerate() {
            pic.data = format!("data:image/webp;base64,{}", if k == 2 { "Qg==" } else { "QQ==" });
        }
        p
    }

    #[test]
    fn split_and_join_give_back_the_same_bytes() {
        let p = with_pictures();
        let (text, pictures) = p.split();
        assert_eq!(pictures.0.len(), 2, "one entry per file");
        assert!(text.pictures().0.is_empty(), "the text part has no picture data");
        let bytes = join_parts(&text.encode(), &pictures.encode()).unwrap();
        assert_eq!(bytes, p.encode());
        assert_eq!(Pictures::decode(&pictures.encode()).unwrap(), pictures);
    }

    #[test]
    fn a_missing_picture_or_a_wrong_part_is_refused() {
        let (text, mut pictures) = with_pictures().split();
        pictures.0.remove("b.webp");
        assert_eq!(text.clone().join(&pictures).unwrap_err(), "the pictures part has no b.webp");
        assert!(join_parts(&text.encode(), &text.encode()).is_err(), "a pack is not a pictures part");
        assert!(matches!(Pictures::decode(b"BATNAPK1"), Err(PackError::Magic)));
    }
}
