//! Reminders under a lesson step. When a step teaches a list that already
//! has a تحشيشة (an enumerate item's `hook`, or a mnemonic step of another
//! lesson), its `remind` list brings it back here, with a link to the lesson
//! that taught it when that is another lesson. Mohamed, 2026-11-08: a topic
//! that comes back should bring its تحشيشة back, or at least remind him.

use batna_core::content::Body;
use batna_core::lesson::{Reminder, StepBody};
use dioxus::prelude::*;

use super::text::{Line, Rich};
use crate::Ui;
use crate::content::Library;
use crate::i18n::learn as tx;

/// One reminder, ready to draw.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shown {
    /// What it is a تحشيشة for: the item's prompt, or the mnemonic's word.
    pub what: String,
    /// The mnemonic's letters, `(key, text)`; empty for an item.
    pub parts: Vec<(String, String)>,
    pub text: String,
    /// The lesson that teaches it, `(id, title)`.
    pub from: Option<(String, String)>,
}

/// What a `remind` entry points at, or `None` when it no longer resolves
/// (the validator stops that at build time).
pub(crate) fn resolve(entry: &str) -> Option<Shown> {
    match Reminder::parse(entry)? {
        Reminder::Item(id) => {
            let (_, item) = super::practice::find(id)?;
            let Body::Enumerate { prompt, hook: Some(h), .. } = &item.body else { return None };
            Some(Shown {
                what: prompt.clone(),
                parts: Vec::new(),
                text: h.clone(),
                from: super::lesson_for_item(id).map(|l| (l.id.clone(), l.title.clone())),
            })
        }
        Reminder::Mnemonic { lesson, word } => {
            let lecture = lesson.rsplit_once('.').map(|(l, _)| l)?;
            let l = Library::get().lessons.get(lecture)?.lessons.iter().find(|l| l.id == lesson)?;
            l.steps.iter().find_map(|s| match &s.body {
                StepBody::Mnemonic { word: w, parts, hook } if w == word => Some(Shown {
                    what: word.to_string(),
                    parts: parts.iter().map(|p| (p.key.clone(), p.text.clone())).collect(),
                    text: hook.clone().unwrap_or_default(),
                    from: Some((l.id.clone(), l.title.clone())),
                }),
                _ => None,
            })
        }
    }
}

/// The reminders of one step, `here` being the open lesson's id.
#[component]
pub fn Reminders(remind: Vec<String>, here: String) -> Element {
    let ui = use_context::<Ui>();
    let shown: Vec<Shown> = remind.iter().filter_map(|r| resolve(r)).collect();
    if shown.is_empty() {
        return rsx! {};
    }
    rsx! {
        aside { class: "ls-remind", aria_label: tx::REMIND,
            for (k , s) in shown.into_iter().enumerate() {
                div { key: "{k}", class: "ls-remind-one",
                    p { class: "ls-remind-label",
                        span { aria_hidden: "true", "😄 " }
                        if s.parts.is_empty() { {tx::REMIND} } else { {tx::REMIND_LIST} }
                    }
                    if s.parts.is_empty() {
                        p { class: "ls-remind-what", dir: "ltr", lang: "en", "{s.what}" }
                    } else {
                        p { class: "ls-remind-word", dir: "ltr", lang: "en", "{s.what}" }
                        ul { class: "ls-remind-parts", dir: "ltr", lang: "en",
                            for (j , (key , text)) in s.parts.iter().enumerate() {
                                li { key: "{j}",
                                    b { "{key}" }
                                    " {text}"
                                }
                            }
                        }
                    }
                    if !s.text.trim().is_empty() {
                        Rich { text: s.text.clone(), class: "ls-remind-text ls-rich" }
                    }
                    if let Some((id, title)) = s.from.filter(|(id, _)| *id != here) {
                        button {
                            class: "st-link ls-remind-from",
                            title: tx::REMIND_FROM_HINT,
                            onclick: move |_| super::open(ui, &id),
                            "📖 {tx::REMIND_FROM}:\u{a0}"
                            Line { text: title }
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
    fn the_aclf_step_brings_back_its_hooks_and_shibs_from_lesson_two() {
        let item = resolve("hb.L01.033").expect("ACLF precipitants");
        assert!(item.text.contains("ACLF"), "{item:?}");
        assert!(item.parts.is_empty());
        assert_eq!(item.from.as_ref().map(|f| f.0.as_str()), Some("hb.L01.lesson3"));

        let shibs = resolve("hb.L01.lesson2:SHIBS").expect("SHIBS");
        assert_eq!(shibs.what, "SHIBS");
        assert_eq!(shibs.parts.len(), 5);
        assert!(shibs.text.contains("شيبس"));
        assert_eq!(shibs.from.as_ref().map(|f| f.0.as_str()), Some("hb.L01.lesson2"));

        assert_eq!(resolve("hb.L01.999"), None);
        assert_eq!(resolve("hb.L01.lesson2:NOPE"), None);
    }

    #[test]
    fn every_reminder_in_the_library_resolves() {
        let lib = Library::get();
        let mut n = 0;
        for (_, l) in lib.all_lessons() {
            for s in &l.steps {
                for r in &s.remind {
                    assert!(resolve(r).is_some(), "{}: {r}", l.id);
                    n += 1;
                }
            }
        }
        assert!(n > 0, "reminders are written");
    }
}
