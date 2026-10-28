//! A calm word at the end of the day on Today, and a weekly look back:
//! what he learned, how much stuck, the topics to revisit and the days to
//! the exam. Facts only: no streaks, no guilt, nothing about missed days.

use batna_core::stats::{self, Ratio};
use dioxus::prelude::*;

use crate::i18n::recap as tx;
use crate::stats::StudyData;
use crate::store::{now_ms, use_review};

/// What today's line is built from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DayFacts {
    pub answered: usize,
    pub learned: u32,
    pub due_left: usize,
    pub due_tomorrow: u32,
}

/// Today's line, once he has studied; nothing before that.
pub fn day_line(f: DayFacts) -> Option<String> {
    if f.answered == 0 {
        return None;
    }
    let mut line = format!("{} {} {}", tx::TODAY_1, f.answered, tx::TODAY_ANSWERS);
    if f.learned > 0 {
        line += &format!("، {} {} {}", tx::AND, f.learned, tx::TODAY_NEW);
    }
    line += ". ";
    line += &if f.due_left == 0 {
        format!("{} {} {}", tx::DONE_REST, f.due_tomorrow, tx::REVIEWS)
    } else {
        format!("{} {} {}", tx::LEFT_1, f.due_left, tx::LEFT_2)
    };
    Some(line)
}

/// The weekly look back.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Week {
    /// New items per day, oldest first, today last.
    pub learned: Vec<u32>,
    pub retention: Ratio,
    pub weak: Vec<String>,
    pub days_to_exam: Option<i32>,
}

impl Week {
    pub fn learned_total(&self) -> u32 {
        self.learned.iter().sum()
    }

    pub fn study_days(&self) -> usize {
        self.learned.iter().filter(|&&n| n > 0).count()
    }

    /// Shown once there is something to look back on.
    pub fn worth_showing(&self) -> bool {
        self.learned_total() > 0 || self.retention.total > 0
    }
}

/// The line and, when there is a week behind him, the weekly card.
#[component]
pub fn TodayRecap(due_left: usize, due_tomorrow: u32) -> Element {
    let ui = use_context::<crate::Ui>();
    let review = use_review();
    let now = now_ms();
    let (answered, learned) = {
        let b = review.backend.borrow();
        (b.answers_today(now), b.learned_per_day(7, now))
    };
    let line = day_line(DayFacts {
        answered,
        learned: learned.last().copied().unwrap_or(0),
        due_left,
        due_tomorrow,
    });
    let week = use_memo(move || {
        let _ = (review.revision)();
        let data = StudyData::load(&*review.backend.borrow());
        let rows = data.stats();
        let mut retention = Ratio::default();
        for l in &rows {
            retention.right += l.counts.retention.right;
            retention.total += l.counts.retention.total;
        }
        // The study day, like the planner, so "days to the exam" agree (#36).
        let today = crate::store::zone().study_day(now_ms());
        Week {
            learned: review.backend.borrow().learned_per_day(7, now_ms()),
            retention,
            weak: stats::weakest_topics(&rows, 3).into_iter().map(|w| w.title).collect(),
            days_to_exam: crate::plan::SETTINGS.read().exam.map(|e| today.until(e)).filter(|d| *d >= 0),
        }
    });
    let week = week.read().clone();
    rsx! {
        // Both open the progress screen, where the numbers come from.
        if let Some(line) = line {
            button {
                class: "card tap recap-line",
                title: tx::OPEN_PROGRESS,
                onclick: move |_| crate::stats::open(ui, crate::stats::Tab::Progress),
                {line}
            }
        }
        if week.worth_showing() {
            button {
                class: "card tap recap-week",
                title: tx::OPEN_PROGRESS,
                onclick: move |_| crate::stats::open(ui, crate::stats::Tab::Progress),
                h2 { {tx::WEEK_TITLE} span { class: "chev", aria_hidden: "true" } }
                ul {
                    li { "{tx::WEEK_LEARNED_1} {week.learned_total()} {tx::WEEK_LEARNED_2} {week.study_days()} {tx::WEEK_DAYS}" }
                    if let Some(p) = week.retention.percent() {
                        li { "{tx::WEEK_RETENTION} {p}%" }
                    }
                    if !week.weak.is_empty() {
                        li { "{tx::WEEK_WEAK} " bdi { {week.weak.join("، ")} } }
                    }
                    if let Some(d) = week.days_to_exam {
                        li { "{tx::WEEK_EXAM_1} {d} {tx::WEEK_EXAM_2}" }
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
    fn nothing_is_said_before_he_studies() {
        assert_eq!(day_line(DayFacts { due_left: 40, ..DayFacts::default() }), None);
    }

    #[test]
    fn the_line_says_what_he_did_and_what_is_next() {
        let done = day_line(DayFacts { answered: 30, learned: 8, due_left: 0, due_tomorrow: 25 }).unwrap();
        assert!(done.contains("30") && done.contains('8') && done.contains("25"), "{done}");
        assert!(done.contains(tx::DONE_REST));
        let more = day_line(DayFacts { answered: 12, learned: 0, due_left: 9, due_tomorrow: 3 }).unwrap();
        assert!(more.contains("12") && more.contains('9') && !more.contains(tx::TODAY_NEW), "{more}");
    }

    #[test]
    fn the_week_card_waits_for_something_to_show() {
        assert!(!Week::default().worth_showing());
        let w = Week { learned: vec![0, 3, 0, 0, 5, 0, 2], ..Week::default() };
        assert_eq!((w.learned_total(), w.study_days()), (10, 3));
        assert!(w.worth_showing());
    }
}
