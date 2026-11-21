//! Video links on a lesson: the second in a lecturer's YouTube video where
//! this lesson's subject is explained (Dr. Mahmoud Allam's 24-day revision
//! course, Mohamed 2026-11-21). The lesson header shows a button per link
//! that opens the video at that second.
//!
//! The id check is adapted from steps-app main: crates/core/src/sealed.rs
//! (`Video::is_id`).

use serde::{Deserialize, Serialize};

/// One place in a video that explains the lesson.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Video {
    /// The 11-character YouTube video id (`7Ds6eQ8uA0k`).
    pub youtube: String,
    /// The second where the explanation starts.
    pub at: u32,
    /// What is explained there: a few words of Egyptian Arabic with the
    /// English terms (`تفسير الـ LFTs`).
    pub label: String,
    /// Who explains it (`د. محمود علام`).
    pub by: String,
}

impl Video {
    /// The most links one lesson carries.
    pub const MAX_PER_LESSON: usize = 6;
    /// The longest label, in characters.
    pub const MAX_LABEL: usize = 60;
    /// A start past this is a typo (the course's videos are under 5 hours).
    pub const MAX_AT: u32 = 5 * 3600;

    /// `https://youtu.be/<id>?t=<seconds>`.
    pub fn url(&self) -> String {
        format!("https://youtu.be/{}?t={}", self.youtube, self.at)
    }

    /// The start as a clock: `3:39`, or `1:02:05` past an hour.
    pub fn clock(&self) -> String {
        let (h, m, s) = (self.at / 3600, self.at / 60 % 60, self.at % 60);
        if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
    }

    /// Whether `id` has a YouTube video id's shape: 11 of A–Z, a–z, 0–9, `-`, `_`.
    pub fn is_id(id: &str) -> bool {
        id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    }
}

/// What is wrong with a lesson's video list.
pub fn video_problems(videos: &[Video]) -> Vec<String> {
    let mut out = Vec::new();
    if videos.len() > Video::MAX_PER_LESSON {
        out.push(format!("{} videos: keep at most {}", videos.len(), Video::MAX_PER_LESSON));
    }
    let mut seen = std::collections::BTreeSet::new();
    for v in videos {
        if !Video::is_id(&v.youtube) {
            out.push(format!("video `{}` is not a YouTube id (11 of A-Z a-z 0-9 - _)", v.youtube));
        }
        if v.at > Video::MAX_AT {
            out.push(format!("video {} starts at {} s: past {} s", v.youtube, v.at, Video::MAX_AT));
        }
        let label = v.label.trim();
        if label.is_empty() || v.by.trim().is_empty() {
            out.push(format!("video {} needs a label and who explains it (`by`)", v.youtube));
        }
        if label.chars().count() > Video::MAX_LABEL {
            out.push(format!("video {} label is over {} characters", v.youtube, Video::MAX_LABEL));
        }
        if !seen.insert((v.youtube.as_str(), v.at)) {
            out.push(format!("video {} at {} s is listed twice", v.youtube, v.at));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(youtube: &str, at: u32, label: &str) -> Video {
        Video { youtube: youtube.into(), at, label: label.into(), by: "د. محمود علام".into() }
    }

    #[test]
    fn a_link_opens_the_video_at_its_second_and_shows_a_clock() {
        let x = v("7Ds6eQ8uA0k", 219, "تفسير الـ LFTs");
        assert_eq!(x.url(), "https://youtu.be/7Ds6eQ8uA0k?t=219");
        assert_eq!(x.clock(), "3:39");
        assert_eq!(v("7Ds6eQ8uA0k", 3725, "x").clock(), "1:02:05");
        assert_eq!(v("7Ds6eQ8uA0k", 5, "x").clock(), "0:05");
    }

    #[test]
    fn links_are_checked() {
        assert!(video_problems(&[v("7Ds6eQ8uA0k", 219, "تفسير الـ LFTs")]).is_empty());
        let bad = video_problems(&[
            v("7Ds6eQ8uA0", 1, "x"),
            v("7Ds6eQ8uA0k?", 1, "x"),
            v("7Ds6eQ8uA0k", Video::MAX_AT + 1, "x"),
            v("7Ds6eQ8uA0k", 2, " "),
            v("7Ds6eQ8uA0k", 3, &"ا".repeat(61)),
            v("7Ds6eQ8uA0k", 4, "x"),
            v("7Ds6eQ8uA0k", 5, "x"),
        ]);
        let all = bad.join("\n");
        assert!(all.contains("7 videos: keep at most 6"), "{all}");
        assert_eq!(bad.iter().filter(|m| m.contains("not a YouTube id")).count(), 2, "{all}");
        assert!(all.contains("past"), "{all}");
        assert!(all.contains("needs a label"), "{all}");
        assert!(all.contains("over 60"), "{all}");
        let twice = video_problems(&[v("7Ds6eQ8uA0k", 9, "a"), v("7Ds6eQ8uA0k", 9, "b")]);
        assert_eq!(twice, ["video 7Ds6eQ8uA0k at 9 s is listed twice"]);
    }

    #[test]
    fn a_lesson_without_videos_writes_no_key_and_reads_them_from_yaml() {
        let yaml = "youtube: 7Ds6eQ8uA0k\nat: 219\nlabel: \"تفسير الـ LFTs\"\nby: \"د. محمود علام\"\n";
        let x: Video = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(x, v("7Ds6eQ8uA0k", 219, "تفسير الـ LFTs"));
        assert!(serde_yaml_ng::from_str::<Video>(&format!("{yaml}start: 3\n")).is_err());
    }
}
