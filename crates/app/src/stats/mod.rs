//! The progress screen (nav "تقدّمي"): stats per lecture and topic, the
//! weakest topics, the mistakes log, and the exam planner's tab.
//! The numbers come from `batna_core::stats`; this module only draws them.

pub mod charts;
mod demo;
mod mistakes;

use std::collections::BTreeMap;

use batna_core::content::{Lecture, Status};
use batna_core::review::ReviewEvent;
use batna_core::stats::{self, Clock, Counts, LectureStats, Ratio, WeakTopic};
use dioxus::prelude::*;

use crate::content::Library;
use crate::i18n::stats as tx;
use crate::plan::Planner;

/// Egypt's standard time. Summer time moves the 4 am rollover to 3 am, which
/// doesn't change which day a study session belongs to in practice.
const UTC_OFFSET_MIN: i32 = 120;
const ROLLOVER_HOUR: u32 = 4;
const STATS_CSS: &str = include_str!("stats.css");

/// What the stats read: the content, the review log and Mohamed's marks.
/// Loaded once per visit to the screen.
#[derive(PartialEq)]
pub struct StudyData {
    pub lectures: Vec<Lecture>,
    pub events: Vec<ReviewEvent>,
    pub marks: BTreeMap<String, Status>,
    pub clock: Clock,
    /// Demo numbers (BATNA_STATS_DEMO=1), labelled on screen.
    pub demo: bool,
}

impl StudyData {
    /// Today's data. Until the review store lands (chunk 3) the log is empty,
    /// so the screen shows the calm "nothing yet" state.
    pub fn load() -> StudyData {
        let lib = Library::get();
        // Book order, from the index.
        let lectures: Vec<Lecture> =
            lib.index.lectures.iter().filter_map(|e| lib.lectures.get(&e.id).cloned()).collect();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or_default();
        let clock = Clock { now_ms, utc_offset_min: UTC_OFFSET_MIN, rollover_hour: ROLLOVER_HOUR };
        if std::env::var("BATNA_STATS_DEMO").is_ok_and(|v| v == "1") {
            let events = demo::events(&lectures, now_ms);
            return StudyData { lectures, events, marks: demo::marks(), clock, demo: true };
        }
        StudyData { lectures, events: Vec::new(), marks: BTreeMap::new(), clock, demo: false }
    }

    pub fn stats(&self) -> Vec<LectureStats> {
        stats::lecture_stats(&self.lectures, &self.events, &self.marks, self.clock)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tab {
    Progress,
    Mistakes,
    Plan,
}

impl Tab {
    fn from_env() -> Option<Tab> {
        // BATNA_STATS_TAB=plan opens that tab (previews and screenshots).
        match std::env::var("BATNA_STATS_TAB").ok()?.as_str() {
            "mistakes" => Some(Tab::Mistakes),
            "plan" => Some(Tab::Plan),
            _ => None,
        }
    }
}

#[component]
pub fn Stats() -> Element {
    let mut tab = use_signal(|| Tab::from_env().unwrap_or(Tab::Progress));
    let data = use_hook(|| std::rc::Rc::new(StudyData::load()));
    let current = *tab.read();
    let tabs =
        [(Tab::Progress, tx::TAB_PROGRESS), (Tab::Mistakes, tx::TAB_MISTAKES), (Tab::Plan, tx::TAB_PLAN)];
    rsx! {
        style { {STATS_CSS} }
        div { class: "tabs", role: "tablist", aria_label: tx::TABS_LABEL,
            for (t, label) in tabs {
                button {
                    key: "{t:?}",
                    class: "tab",
                    role: "tab",
                    "aria-selected": if current == t { "true" } else { "false" },
                    onclick: move |_| tab.set(t),
                    {label}
                }
            }
        }
        if data.demo {
            p { class: "note", {tx::DEMO} }
        }
        match current {
            Tab::Progress => rsx! { Progress { data: data.clone() } },
            Tab::Mistakes => rsx! { mistakes::Mistakes { data: data.clone() } },
            Tab::Plan => rsx! { Planner { data: data.clone() } },
        }
    }
}

#[component]
fn Progress(data: std::rc::Rc<StudyData>) -> Element {
    let rows = data.stats();
    let mut all = Counts::default();
    for l in &rows {
        all.total += l.counts.total;
        all.seen += l.counts.seen;
        all.seen_twice += l.counts.seen_twice;
        all.approved += l.counts.approved;
    }
    let weak = stats::weakest_topics(&rows, 3);
    let weak_ids = stats::topics_session(&data.lectures, &weak, &data.events, &data.marks);
    rsx! {
        section { class: "card",
            h1 { {tx::SUMMARY} }
            div { class: "summary",
                Big { n: all.seen, of: all.total, label: tx::SEEN }
                Big { n: all.seen_twice, of: all.total, label: tx::SEEN_TWICE }
                Big { n: all.approved, of: all.total, label: tx::APPROVED }
            }
            if all.seen == 0 {
                p { class: "muted", {tx::NOTHING_YET} }
            }
        }
        WeakCard { weak, ids: weak_ids }
        section { class: "stat-lectures",
            for l in rows.iter() {
                LectureCard { key: "{l.id}", l: l.clone() }
            }
        }
        p { class: "muted small", {tx::RETENTION_HINT} }
        if rows.len() < Library::get().index.lectures.len() {
            p { class: "muted small", {tx::NOT_WRITTEN} }
        }
    }
}

#[component]
fn Big(n: u32, of: u32, label: &'static str) -> Element {
    rsx! {
        div { class: "big",
            span { class: "big-n", "{n}" }
            span { class: "muted small", "{tx::OF} {of}" }
            span { class: "big-label", {label} }
        }
    }
}

#[component]
fn WeakCard(weak: Vec<WeakTopic>, ids: Vec<String>) -> Element {
    rsx! {
        section { class: "card",
            h2 { {tx::WEAK_TITLE} }
            if weak.is_empty() {
                p { class: "muted", {tx::WEAK_EMPTY} }
            } else {
                ol { class: "weak-list",
                    for w in weak.iter() {
                        li { key: "{w.lecture}-{w.title}",
                            span { class: "weak-title", bdi { dir: "ltr", "{w.title}" } }
                            span { class: "muted small",
                                "{tx::LECTURE} {lecture_number(&w.lecture)} · {tx::COL_RETENTION} "
                                bdi { {pct(w.retention)} }
                                " · "
                                bdi { "MCQ" }
                                " "
                                bdi { {pct(w.mcq)} }
                            }
                        }
                    }
                }
                SessionButton { label: tx::WEAK_START, ids }
            }
        }
    }
}

/// Starts a session from a list of item ids. The study session player
/// (chunk 2) hasn't landed yet, so for now it says the items are ready.
#[component]
pub fn SessionButton(label: &'static str, ids: Vec<String>) -> Element {
    let mut pressed = use_signal(|| false);
    rsx! {
        div { class: "session-start",
            button { class: "primary", disabled: ids.is_empty(), onclick: move |_| pressed.set(true), {label} }
            span { class: "muted small", "{ids.len()} {tx::SESSION_COUNT}" }
        }
        if *pressed.read() {
            p { class: "note", {tx::SESSION_SOON} }
        }
    }
}

#[component]
fn LectureCard(l: LectureStats) -> Element {
    let c = &l.counts;
    let seen = Ratio { right: c.seen, total: c.total };
    let approved = Ratio { right: c.approved, total: c.total };
    rsx! {
        details { class: "stat-lecture",
            summary {
                span { class: "num", "{lecture_number(&l.id)}" }
                div { class: "lecture-main",
                    span { class: "lecture-title", bdi { dir: "ltr", "{l.title}" } }
                    div { class: "bars",
                        Bar { label: tx::COL_SEEN, r: seen }
                        Bar { label: tx::COL_RETENTION, r: c.retention }
                        Bar { label: tx::COL_MCQ, r: c.mcq }
                        Bar { label: tx::COL_APPROVED, r: approved }
                    }
                }
            }
            div { class: "topic-table", role: "table", aria_label: tx::TOPICS,
                div { class: "topic-row head", role: "row",
                    span { role: "columnheader", {tx::TOPICS} }
                    span { role: "columnheader", {tx::COL_SEEN} }
                    span { role: "columnheader", {tx::COL_RETENTION} }
                    span { role: "columnheader", {tx::COL_MCQ} }
                }
                for t in l.topics.iter() {
                    div { class: "topic-row", role: "row", key: "{t.title}",
                        span { role: "cell",
                            if t.id.is_some() { bdi { dir: "ltr", "{t.title}" } } else { {tx::UNTAGGED} }
                        }
                        span { role: "cell", bdi { "{t.counts.seen}/{t.counts.total}" } }
                        span { role: "cell", bdi { {pct(t.counts.retention)} } }
                        span { role: "cell", bdi { {pct(t.counts.mcq)} } }
                    }
                }
            }
        }
    }
}

/// One labelled bar: the share as a filled track plus the number.
#[component]
fn Bar(label: &'static str, r: Ratio) -> Element {
    let width = r.share().map_or(0.0, |s| s * 100.0);
    rsx! {
        div { class: "bar", "data-empty": if r.total == 0 { "yes" } else { "no" },
            span { class: "bar-label small", {label} }
            span { class: "bar-track", span { class: "bar-fill", style: "inline-size: {width:.0}%" } }
            bdi { class: "bar-value small", {pct(r)} }
        }
    }
}

pub fn pct(r: Ratio) -> String {
    r.percent().map_or_else(|| tx::NO_DATA.to_string(), |p| format!("{p}%"))
}

/// `hb.L01` → `1`.
pub fn lecture_number(id: &str) -> String {
    id.rsplit('.').next().unwrap_or(id).trim_start_matches('L').trim_start_matches('0').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_numbers() {
        assert_eq!(lecture_number("hb.L01"), "1");
        assert_eq!(lecture_number("hb.L20"), "20");
        assert_eq!(pct(Ratio { right: 1, total: 4 }), "25%");
        assert_eq!(pct(Ratio::default()), tx::NO_DATA);
    }
}
