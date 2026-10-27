//! The exam planner tab: Mohamed sets the exam day and his study days, and
//! `batna_core::plan` spreads what's left over the days before it.

use std::rc::Rc;

use batna_core::plan::{self, Day, DayPlan, Focus, Plan, PlanInput, PlanSettings, PlanStatus};
use batna_core::stats;
use dioxus::prelude::*;

use crate::i18n::plan as tx;
use crate::stats::{StudyData, charts, lecture_number};

/// The planner settings. In memory until the store (chunk 3) can keep them.
/// BATNA_PLAN_EXAM=2027-01-20 presets the exam day (previews and screenshots).
static SETTINGS: GlobalSignal<PlanSettings> = Signal::global(|| PlanSettings {
    exam: std::env::var("BATNA_PLAN_EXAM").ok().and_then(|v| Day::parse(&v)),
    ..PlanSettings::default()
});

#[component]
pub fn Planner(data: Rc<StudyData>) -> Element {
    let mut editing = use_signal(|| false);
    let settings = SETTINGS.read().clone();
    if settings.exam.is_none() || *editing.read() {
        return rsx! {
            Setup { today: data.clock.today(), settings, on_done: move |_| editing.set(false) }
        };
    }
    let rows = data.stats();
    let progress = stats::progress(&rows);
    // A rough forecast of reviews already due, until FSRS's own (chunk 3).
    let today = data.clock.today();
    let span = settings.exam.map_or(0, |e| today.until(e).max(0) as usize);
    let forecast = stats::rough_forecast(&data.lectures, &data.events, &data.marks, data.clock, span);
    let p = plan::plan(&settings, PlanInput { today, lectures: &progress, review_forecast: &forecast });
    rsx! {
        StatusCard { p: p.clone(), exam: settings.exam }
        TodayCard { p: p.clone() }
        LoadCard { p: p.clone(), cap: settings.review_cap }
        PaceCard { p: p.clone(), past: past_pace(&data) }
        ProgressCard { p: p.clone() }
        WeekCard { p: p.clone() }
        div { class: "plan-actions",
            button { class: "secondary", onclick: move |_| editing.set(true), {tx::EDIT} }
        }
        p { class: "muted small", {tx::NOT_SAVED} }
    }
}

#[component]
fn Setup(today: Day, settings: PlanSettings, on_done: EventHandler<()>) -> Element {
    let mut date = use_signal(|| settings.exam.map(Day::iso).unwrap_or_default());
    let mut weekdays = use_signal(|| settings.weekdays);
    let mut cap = use_signal(|| settings.review_cap);
    let mut error = use_signal(|| None::<&'static str>);
    let had_exam = settings.exam.is_some();
    let min = today.plus(1).iso();
    let save = move |_| {
        let Some(exam) = Day::parse(&date.read()).filter(|d| *d > today) else {
            error.set(Some(tx::PICK_DAY));
            return;
        };
        if !weekdays.read().contains(&true) {
            error.set(Some(tx::PICK_WEEKDAY));
            return;
        }
        let mut s = SETTINGS.write();
        s.exam = Some(exam);
        s.weekdays = *weekdays.read();
        s.review_cap = (*cap.read()).clamp(30, 1000);
        on_done.call(());
    };
    rsx! {
        section { class: "card plan-setup",
            h1 { {tx::SETUP_TITLE} }
            p { class: "muted", {tx::SETUP_BODY} }
            label { class: "field",
                span { {tx::EXAM_DAY} }
                input {
                    r#type: "date",
                    min: "{min}",
                    value: "{date}",
                    oninput: move |e| {
                        date.set(e.value());
                        error.set(None);
                    },
                }
            }
            fieldset { class: "field weekdays",
                legend { {tx::DAYS_LABEL} }
                div { class: "weekday-chips",
                    for (w, name) in tx::WEEKDAYS.iter().enumerate() {
                        button {
                            key: "{w}",
                            class: "weekday",
                            "aria-pressed": if weekdays.read()[w] { "true" } else { "false" },
                            onclick: move |_| {
                                weekdays.write()[w] ^= true;
                                error.set(None);
                            },
                            {*name}
                        }
                    }
                }
            }
            label { class: "field",
                span { {tx::CAP_LABEL} }
                input {
                    r#type: "number",
                    inputmode: "numeric",
                    min: "30",
                    max: "1000",
                    step: "10",
                    value: "{cap}",
                    oninput: move |e| {
                        if let Ok(n) = e.value().parse() {
                            cap.set(n);
                        }
                    },
                }
            }
            if let Some(msg) = *error.read() {
                p { class: "error", role: "alert", {msg} }
            }
            div { class: "plan-actions",
                button { class: "primary", onclick: save, {tx::SAVE} }
                if had_exam {
                    button { class: "secondary", onclick: move |_| on_done.call(()), {tx::CANCEL} }
                }
            }
        }
    }
}

#[component]
fn StatusCard(p: Plan, exam: Option<Day>) -> Element {
    let message = match p.status {
        PlanStatus::OnTrack => tx::ON_TRACK.to_string(),
        PlanStatus::Snug => tx::SNUG.to_string(),
        PlanStatus::Tight { left_over, extra_days: Some(n) } => {
            format!("{} {left_over}. {} {n}.", tx::TIGHT, tx::TIGHT_ADD)
        }
        PlanStatus::Tight { left_over, extra_days: None } => {
            format!("{} {left_over}. {}", tx::TIGHT, tx::TIGHT_CAP)
        }
        PlanStatus::FinalWindow => tx::FINAL_WINDOW.to_string(),
        PlanStatus::ExamDone => tx::EXAM_DONE.to_string(),
        PlanStatus::NoExamDate | PlanStatus::NoStudyDays => String::new(),
    };
    let tight = matches!(p.status, PlanStatus::Tight { .. });
    rsx! {
        section { class: "card plan-status",
            if let Some(exam) = exam {
                h1 { "{tx::EXAM_ON}: {long_date(exam)}" }
            }
            if p.status != PlanStatus::ExamDone {
                p { class: "muted", "{p.study_days_left} {tx::DAYS_LEFT}" }
            }
            if let Some(from) = p.final_from.filter(|_| p.status != PlanStatus::ExamDone) {
                p { class: "muted small", "{tx::FINAL_FROM} {long_date(from)}" }
            }
            p { class: if tight { "note" } else { "" }, {message} }
        }
    }
}

#[component]
fn TodayCard(p: Plan) -> Element {
    let Some(today) = p.today() else { return rsx! {} };
    if today.focus == Focus::Rest {
        let next = p.next_study_day();
        return rsx! {
            section { class: "card plan-today",
                h2 { {tx::TODAY} }
                p { {tx::REST_TODAY} }
                if let Some(next) = next {
                    DayLine { d: next.clone() }
                }
            }
        };
    }
    rsx! {
        section { class: "card plan-today",
            h2 { "{tx::TODAY}: {focus_label(today.focus)}" }
            div { class: "summary",
                if today.new > 0 {
                    div { class: "big",
                        span { class: "big-n", "{today.new}" }
                        span { class: "big-label", {tx::NEW_ITEMS} }
                    }
                }
                div { class: "big",
                    span { class: "big-n", "{today.reviews}" }
                    span { class: "big-label", {tx::REVIEWS} }
                }
            }
            DayDetail { d: today.clone() }
        }
    }
}

/// Items seen by the end of each past day, as (offset from today, count),
/// over the last [`PACE_PAST_DAYS`] days. Empty with no history.
fn past_pace(data: &StudyData) -> Vec<(i32, u32)> {
    let today = data.clock.today();
    let by_day = stats::first_seen_by_day(&data.lectures, &data.events, &data.marks, data.clock);
    let Some(first) = by_day.keys().next().copied() else { return Vec::new() };
    let from = first.max(today.plus(-PACE_PAST_DAYS));
    let mut seen: u32 = by_day.range(..from).map(|(_, n)| n).sum();
    (today.until(from)..=0)
        .map(|off| {
            seen += by_day.get(&today.plus(off)).copied().unwrap_or(0);
            (off, seen)
        })
        .collect()
}

const PACE_PAST_DAYS: i32 = 30;
const LOAD_DAYS: usize = 14;

#[component]
fn LoadCard(p: Plan, cap: u32) -> Element {
    let days: Vec<(Day, charts::Load)> = p
        .days
        .iter()
        .take(LOAD_DAYS)
        .map(|d| (d.day, charts::Load { reviews: d.reviews, new: d.new, rest: d.focus == Focus::Rest }))
        .collect();
    if days.is_empty() {
        return rsx! {};
    }
    let loads: Vec<charts::Load> = days.iter().map(|(_, l)| *l).collect();
    let svg = charts::forecast(&loads, cap, tx::LOAD_LABEL, |i, l| {
        let day = short_date(days[i].0);
        if l.rest {
            format!("{day}: {}", tx::FOCUS_REST)
        } else {
            format!("{day}: {} {} · {} {}", l.reviews, tx::REVIEWS, l.new, tx::NEW_ITEMS)
        }
    });
    let peak = days.iter().max_by_key(|(_, l)| l.reviews + l.new).map(|(d, l)| (*d, l.reviews + l.new));
    rsx! {
        section { class: "card chart-card",
            h2 { {tx::LOAD_TITLE} }
            div { class: "chart-box", dangerous_inner_html: "{svg}" }
            div { class: "chart-key small muted",
                span { span { class: "key key-bar" } {tx::LOAD_KEY_REVIEWS} }
                span { span { class: "key key-new" } {tx::LOAD_KEY_NEW} }
                span { span { class: "key key-cap" } "{tx::LOAD_KEY_CAP} ({cap})" }
            }
            if let Some((d, n)) = peak.filter(|(_, n)| *n > 0) {
                p { class: "muted small", "{tx::LOAD_PEAK}: {short_date(d)} ({n})" }
            }
        }
    }
}

#[component]
fn PaceCard(p: Plan, past: Vec<(i32, u32)>) -> Element {
    if p.days.is_empty() || p.total == 0 {
        return rsx! {};
    }
    let today = p.days[0].day;
    let mut planned = vec![(0, p.seen)];
    let mut seen = p.seen;
    let mut done_on = (p.seen >= p.total).then_some(today);
    for (i, d) in p.days.iter().enumerate() {
        seen += d.new;
        if done_on.is_none() && seen >= p.total {
            done_on = Some(d.day);
        }
        planned.push((i as i32 + 1, seen));
    }
    let to = p.days.len() as i32;
    let svg = charts::pace(
        &charts::Pace {
            past: &past,
            planned: &planned,
            total: p.total,
            from: past.first().map_or(0, |(o, _)| *o),
            to,
            final_from: p.final_from.map(|f| today.until(f)),
        },
        tx::PACE_LABEL,
    );
    rsx! {
        section { class: "card chart-card",
            h2 { {tx::PACE_TITLE} }
            div { class: "chart-box", dangerous_inner_html: "{svg}" }
            div { class: "chart-key small muted",
                span { span { class: "key key-past" } {tx::PACE_KEY_PAST} }
                span { span { class: "key key-plan" } {tx::PACE_KEY_PLAN} }
                span { span { class: "key key-final" } {tx::PACE_KEY_FINAL} }
            }
            p { class: "small",
                match done_on {
                    Some(d) => format!("{} {}", tx::PACE_DONE_ON, long_date(d)),
                    None => tx::PACE_NOT_DONE.to_string(),
                }
            }
        }
    }
}

#[component]
fn ProgressCard(p: Plan) -> Element {
    let share = |n: u32| if p.total == 0 { 0.0 } else { n as f32 * 100.0 / p.total as f32 };
    rsx! {
        section { class: "card",
            h2 { {tx::PROGRESS} }
            for (label, n) in [(tx::FIRST_PASS, p.seen), (tx::SECOND_PASS, p.seen_twice)] {
                div { class: "bar", key: "{label}",
                    span { class: "bar-label small", {label} }
                    span { class: "bar-track", span { class: "bar-fill", style: "inline-size: {share(n):.0}%" } }
                    span { class: "bar-value small", bdi { "{n}/{p.total}" } }
                }
            }
        }
    }
}

#[component]
fn WeekCard(p: Plan) -> Element {
    let week: Vec<DayPlan> = p.days.iter().skip(1).take(7).cloned().collect();
    let rest: Vec<DayPlan> = p.days.iter().skip(8).cloned().collect();
    rsx! {
        section { class: "card",
            h2 { {tx::WEEK} }
            ol { class: "plan-days",
                for d in week {
                    li { key: "{d.day.0}", DayLine { d } }
                }
            }
            if !rest.is_empty() {
                details { class: "plan-all",
                    summary { {tx::ALL_DAYS} }
                    ol { class: "plan-days",
                        for d in rest {
                            li { key: "{d.day.0}", DayLine { d } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DayLine(d: DayPlan) -> Element {
    rsx! {
        div { class: "plan-day", "data-focus": "{d.focus:?}",
            span { class: "plan-date", {short_date(d.day)} }
            span { class: "plan-focus", {focus_label(d.focus)} }
            if d.focus != Focus::Rest {
                DayDetail { d: d.clone() }
            }
        }
    }
}

#[component]
fn DayDetail(d: DayPlan) -> Element {
    rsx! {
        span { class: "plan-detail muted small",
            if d.new > 0 {
                "{d.new} {tx::NEW_ITEMS} {tx::FROM} "
                bdi { {lectures(&d.new_lectures)} }
                " · "
            }
            if !d.revisit.is_empty() {
                "{tx::REVISIT} "
                bdi { {lectures(&d.revisit)} }
                " · "
            }
            "{d.reviews} {tx::REVIEWS}"
        }
    }
}

/// `["hb.L01", "hb.L02"]` → `L1, L2`.
fn lectures(ids: &[String]) -> String {
    ids.iter().map(|id| format!("L{}", lecture_number(id))).collect::<Vec<_>>().join(", ")
}

fn focus_label(f: Focus) -> &'static str {
    match f {
        Focus::Learn => tx::FOCUS_LEARN,
        Focus::Revisit => tx::FOCUS_REVISIT,
        Focus::Mock => tx::FOCUS_MOCK,
        Focus::WeakTopics => tx::FOCUS_WEAK,
        Focus::Rest => tx::FOCUS_REST,
    }
}

/// `الأربع 20 يناير 2027`.
fn long_date(d: Day) -> String {
    let (y, m, day) = d.ymd();
    format!("{} {day} {} {y}", tx::WEEKDAYS[d.weekday()], tx::MONTHS[m as usize - 1])
}

/// `أربع 20 يناير`.
fn short_date(d: Day) -> String {
    let (_, m, day) = d.ymd();
    format!("{} {day} {}", tx::WEEKDAYS_SHORT[d.weekday()], tx::MONTHS[m as usize - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_dates() {
        let d = Day::from_ymd(2027, 1, 20).unwrap();
        assert_eq!(long_date(d), "الأربع 20 يناير 2027");
        assert_eq!(short_date(d), "أربع 20 يناير");
        assert_eq!(lectures(&["hb.L01".into(), "hb.L12".into()]), "L1, L12");
    }
}
