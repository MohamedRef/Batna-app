//! Plans the next few study reminders on the device, so they work offline.
//! Pattern adapted from Kabar's `crates/app/src/coach/notify.rs` (main):
//! Rust words and times them, and the platform arms each one from its
//! wall-clock `date` and `time` in the device's own zone, so Egypt's
//! daylight-saving switch keeps 19:00 at 19:00.
//!
//! A reminder belongs to the study day its time falls in (from 4 am, like
//! an answer): after midnight, today's calendar date is already the next
//! study day, so "today's target is met" and today's due count stay with
//! the evening before (#1384).

use batna_core::plan::Day;
use batna_core::srs::ROLLOVER_HOUR;
use batna_core::zone::Zone;
use serde::{Deserialize, Serialize};

/// Reminders armed ahead: a week or more even at three a day. After the
/// last one the platform keeps going from [`Repeat`] (issue #26).
pub const UPCOMING: usize = 24;

/// How far ahead to look for study days, in days.
const HORIZON: i32 = 14;

const DAY_MS: i64 = 86_400_000;

/// One notification, as `ReminderBridge.schedule` reads it. `at` is the same
/// moment in UTC, by the offset in force then; the platforms arm from `date`
/// and `time` and read `at` only when those don't parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Scheduled {
    pub at: u64,
    pub date: String,
    pub time: String,
    pub title: String,
    pub text: String,
}

/// What the platform arms by itself once the planned reminders run out
/// (the app not opened for a week): the same times and study days, with a
/// general line, since the due counts would be stale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repeat {
    pub times: Vec<String>,
    /// Study days, Saturday first, as `1` and `0`.
    pub days: String,
    pub title: String,
    pub text: String,
    /// Android's notification channel name and description.
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub channel_about: String,
}

/// The fallback for these inputs; `None` where nothing should ring.
pub fn repeat(i: &Inputs) -> Option<Repeat> {
    if !i.here || i.times.is_empty() || !i.weekdays.contains(&true) {
        return None;
    }
    let mut times = i.times.to_vec();
    times.sort_unstable();
    times.dedup();
    Some(Repeat {
        times: times.iter().map(|&t| format!("{:02}:{:02}", t / 60, t % 60)).collect(),
        days: i.weekdays.iter().map(|&on| if on { '1' } else { '0' }).collect(),
        title: crate::i18n::remind::TITLE.into(),
        text: crate::i18n::remind::text(None),
        channel: crate::i18n::remind::CHANNEL.into(),
        channel_about: crate::i18n::remind::CHANNEL_ABOUT.into(),
    })
}

/// The next `n` fallback reminders after the day `last` (`YYYY-MM-DD`), as
/// `(date, time)`. For a platform that can't wake Batna when the planned
/// reminders run out (the Mac), so it arms these ahead instead.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn after(last: &str, r: &Repeat, n: usize) -> Vec<(String, String)> {
    let Some(last) = Day::parse(last) else { return Vec::new() };
    let days: Vec<bool> = r.days.chars().map(|c| c == '1').collect();
    let mut out = Vec::new();
    // A year is plenty: an empty week or no times would loop forever.
    for d in 1..=366 {
        let day = last.plus(d);
        if !days.get(day.weekday()).copied().unwrap_or(false) {
            continue;
        }
        for t in &r.times {
            if out.len() == n {
                return out;
            }
            out.push((day.iso(), t.clone()));
        }
    }
    out
}

/// Everything [`schedule`] reads.
#[derive(Clone, Debug)]
pub struct Inputs<'a> {
    /// This device rings (phone by default).
    pub here: bool,
    /// Reminder times as minutes after local midnight.
    pub times: &'a [u16],
    /// Study days, Saturday first (`PlanSettings::weekdays`).
    pub weekdays: [bool; 7],
    /// Reviews due per study day, today's study day first (overdue counted
    /// today), as `due_forecast` counts them.
    pub due: &'a [u32],
    /// Today's study day's target is already met, so it stays quiet.
    pub target_met: bool,
    pub now_ms: u64,
    /// The device's time zone (`store::zone`), with its summer-time changes.
    pub zone: Zone,
}

/// What to arm now, soonest first. Empty on a quiet device or with no times.
pub fn schedule(i: &Inputs) -> Vec<Scheduled> {
    if !i.here || i.times.is_empty() || !i.weekdays.contains(&true) {
        return Vec::new();
    }
    let mut times = i.times.to_vec();
    times.sort_unstable();
    times.dedup();
    let now = i.now_ms as i64;
    let local_now = i.zone.local_ms(now);
    let today = i.zone.calendar_day(now);
    let study_today = i.zone.study_day(now);
    let rollover = u16::from(ROLLOVER_HOUR) * 60;
    let midnight = local_now - local_now.rem_euclid(DAY_MS);
    let mut out = Vec::new();
    for d in 0..HORIZON {
        let day = today.plus(d);
        for &t in &times {
            // Before 4 am the reminder still belongs to the evening before.
            let study = if t < rollover { day.plus(-1) } else { day };
            let ahead = study_today.until(study);
            if !i.weekdays[study.weekday()] || (ahead == 0 && i.target_met) {
                continue;
            }
            let local = midnight + i64::from(d) * DAY_MS + i64::from(t) * 60_000;
            if local <= local_now {
                continue;
            }
            let due = usize::try_from(ahead).ok().and_then(|s| i.due.get(s)).copied();
            out.push(Scheduled {
                at: i.zone.utc_from_local(local).max(0) as u64,
                date: day.iso(),
                time: format!("{:02}:{:02}", t / 60, t % 60),
                title: crate::i18n::remind::TITLE.into(),
                text: crate::i18n::remind::text(due),
            });
            if out.len() == UPCOMING {
                return out;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAIRO: i32 = 180;
    /// 2026-10-27 (a Tuesday) 08:00 Cairo summer time.
    const NOW: u64 = 1_793_077_200_000;
    const ALL: [bool; 7] = [true; 7];

    fn inputs<'a>(times: &'a [u16], due: &'a [u32]) -> Inputs<'a> {
        Inputs {
            here: true,
            times,
            weekdays: ALL,
            due,
            target_met: false,
            now_ms: NOW,
            zone: Zone::fixed(CAIRO),
        }
    }

    #[test]
    fn now_is_a_tuesday_in_cairo() {
        let today = Day::from_unix_ms(NOW as i64, CAIRO, 0);
        assert_eq!(today.iso(), "2026-10-27");
        assert_eq!(today.weekday(), 3, "Saturday-first: a Tuesday");
    }

    #[test]
    fn reminders_follow_the_times_and_skip_past_ones() {
        let items = schedule(&inputs(&[19 * 60, 7 * 60, 9 * 60], &[12, 30]));
        assert_eq!(items.len(), UPCOMING);
        let when: Vec<_> = items.iter().take(6).map(|s| (s.date.as_str(), s.time.as_str())).collect();
        assert_eq!(
            when,
            [
                ("2026-10-27", "09:00"),
                ("2026-10-27", "19:00"),
                ("2026-10-28", "07:00"),
                ("2026-10-28", "09:00"),
                ("2026-10-28", "19:00"),
                ("2026-10-29", "07:00"),
            ]
        );
        assert_eq!(items[0].at, NOW + 3_600_000);
        assert!(items[0].text.contains("12"), "{}", items[0].text);
        assert!(items[2].text.contains("30"), "{}", items[2].text);
        assert!(items.iter().all(|s| !s.title.is_empty() && !s.text.contains('{')));
    }

    #[test]
    fn today_stays_quiet_once_the_target_is_met() {
        let items = schedule(&Inputs { target_met: true, ..inputs(&[19 * 60], &[5]) });
        assert_eq!(items[0].date, "2026-10-28");
    }

    #[test]
    fn after_the_clocks_go_back_19_00_is_still_19_00() {
        // Egypt leaves summer time at 24:00 on 2026-10-29 (21:00 UTC).
        static CHANGES: [(i64, i32); 1] = [(1_793_307_600_000, 120)];
        let zone = Zone { base: CAIRO, changes: &CHANGES };
        let items = schedule(&Inputs { zone, ..inputs(&[19 * 60], &[]) });
        let at = |date: &str| items.iter().find(|s| s.date == date).map(|s| s.at).unwrap();
        // 19:00 is 16:00 UTC in summer time and 17:00 UTC after the change (#1384).
        assert_eq!(at("2026-10-29"), 1_793_289_600_000);
        assert_eq!(at("2026-10-30"), 1_793_289_600_000 + 25 * 3_600_000);
        assert_eq!(at("2026-11-02"), 1_793_289_600_000 + (4 * 24 + 1) * 3_600_000);
        assert!(items.iter().all(|s| s.time == "19:00"));
    }

    /// Friday 2026-11-06 01:00 Cairo (UTC+2): Thursday's study day runs to 4 am.
    const FRIDAY_1_AM: u64 = 1_793_919_600_000;

    #[test]
    fn after_midnight_the_new_study_day_still_rings() {
        // Thursday evening's target is met; Friday's study day has 7 due.
        let i = Inputs {
            now_ms: FRIDAY_1_AM,
            zone: Zone::fixed(120),
            target_met: true,
            ..inputs(&[19 * 60], &[0, 7])
        };
        let items = schedule(&i);
        assert_eq!((items[0].date.as_str(), items[0].time.as_str()), ("2026-11-06", "19:00"), "Friday rings");
        assert!(items[0].text.contains('7'), "Friday's own count: {}", items[0].text);
    }

    #[test]
    fn a_reminder_before_4_am_belongs_to_the_evening_before() {
        let i = Inputs {
            now_ms: FRIDAY_1_AM,
            zone: Zone::fixed(120),
            target_met: true,
            ..inputs(&[3 * 60, 19 * 60], &[0, 7, 9])
        };
        let when: Vec<_> = schedule(&i).into_iter().take(3).map(|s| (s.date, s.time, s.text)).collect();
        // Friday 03:00 is still Thursday's study day, which is done: it stays quiet.
        assert_eq!((when[0].0.as_str(), when[0].1.as_str()), ("2026-11-06", "19:00"));
        // Saturday 03:00 is Friday's study day, with Friday's count.
        assert_eq!((when[1].0.as_str(), when[1].1.as_str()), ("2026-11-07", "03:00"));
        assert!(when[1].2.contains('7') && when[2].2.contains('9'), "{when:?}");
    }

    #[test]
    fn rest_days_are_skipped() {
        // Tuesday and Wednesday off (Saturday first).
        let weekdays = [true, true, true, false, false, true, true];
        let items = schedule(&Inputs { weekdays, ..inputs(&[19 * 60], &[]) });
        assert_eq!(items[0].date, "2026-10-29");
    }

    #[test]
    fn a_quiet_device_or_no_times_arms_nothing() {
        assert!(schedule(&Inputs { here: false, ..inputs(&[19 * 60], &[]) }).is_empty());
        assert!(schedule(&inputs(&[], &[])).is_empty());
        assert!(schedule(&Inputs { weekdays: [false; 7], ..inputs(&[19 * 60], &[]) }).is_empty());
    }

    #[test]
    fn three_times_a_day_still_cover_a_week() {
        let items = schedule(&inputs(&[7 * 60, 13 * 60, 21 * 60], &[]));
        assert_eq!(items.len(), UPCOMING);
        let last = Day::parse(&items[UPCOMING - 1].date).unwrap();
        assert!(Day::from_unix_ms(NOW as i64, CAIRO, 0).until(last) >= 7, "{}", items[UPCOMING - 1].date);
    }

    #[test]
    fn the_fallback_keeps_the_times_and_study_days() {
        let weekdays = [true, true, true, false, false, true, false];
        let r = repeat(&Inputs { weekdays, ..inputs(&[19 * 60, 7 * 60], &[]) }).unwrap();
        assert_eq!(r.times, ["07:00", "19:00"]);
        assert_eq!(r.days, "1110010");
        assert!(!r.text.chars().any(|c| c.is_ascii_digit()), "no stale count: {}", r.text);
        assert_eq!(repeat(&Inputs { here: false, ..inputs(&[19 * 60], &[]) }), None);
        assert_eq!(repeat(&inputs(&[], &[])), None);
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""channel":""#) && json.contains(r#""channel_about":""#), "{json}");
    }

    #[test]
    fn the_mac_fallback_picks_up_after_the_last_planned_day() {
        let weekdays = [true, false, true, true, true, true, true];
        let r = repeat(&Inputs { weekdays, ..inputs(&[19 * 60, 7 * 60], &[]) }).unwrap();
        // 2026-10-30 is a Friday, so Saturday comes next and Sunday is a rest day.
        let next = after("2026-10-30", &r, 5);
        let want = [
            ("2026-10-31", "07:00"),
            ("2026-10-31", "19:00"),
            ("2026-11-02", "07:00"),
            ("2026-11-02", "19:00"),
            ("2026-11-03", "07:00"),
        ];
        assert_eq!(next, want.map(|(d, t)| (d.to_string(), t.to_string())));
        assert!(after("2026-10-30", &Repeat { days: "0000000".into(), ..r.clone() }, 5).is_empty());
        assert!(after("not a day", &r, 5).is_empty());
    }

    #[test]
    fn the_bridge_reads_this_json_shape() {
        let items = schedule(&inputs(&[19 * 60], &[3]));
        let json = serde_json::to_string(&items[..1]).unwrap();
        assert!(json.starts_with(r#"[{"at":"#) && json.contains(r#""time":"19:00""#), "{json}");
    }
}
