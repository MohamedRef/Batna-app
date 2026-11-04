//! Daily study reminders: planned here from the review forecast, armed as
//! local notifications by the platform, so they work offline. Only one
//! device rings (the phone by default); a met target keeps today quiet.

pub mod plan;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::platform::reminders as native;
use crate::store::{now_ms, shared, use_review, utc_offset_minutes};

/// At most this many reminder times a day.
pub const MAX_TIMES: usize = 3;

/// Settings › Reminders, synced as the `remind` setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemindPrefs {
    pub on: bool,
    /// Minutes after local midnight, soonest first.
    pub times: Vec<u16>,
}

impl Default for RemindPrefs {
    fn default() -> Self {
        Self { on: true, times: vec![19 * 60] }
    }
}

impl RemindPrefs {
    const KEY: &str = "remind";

    fn load() -> Self {
        shared().setting(Self::KEY).unwrap_or_default()
    }

    pub fn save(self) {
        let prefs = self.normalized();
        crate::store::save_setting(Self::KEY, &prefs);
        *PREFS.write() = prefs;
    }

    /// Times soonest first, each once, at most [`MAX_TIMES`].
    fn normalized(mut self) -> Self {
        self.times.sort_unstable();
        self.times.dedup();
        self.times.truncate(MAX_TIMES);
        self
    }

    /// "Add a time": two hours after the last one (7 pm with none), never
    /// past 11 pm.
    fn with_added_time(mut self) -> Self {
        let last = self.times.last().copied().unwrap_or(17 * 60);
        self.times.push((last + 120).min(23 * 60));
        self
    }

    /// The `i`th time changed to what the time input says, if it reads.
    fn with_time(mut self, i: usize, input: &str) -> Option<Self> {
        let v = parse_hhmm(input)?;
        *self.times.get_mut(i)? = v;
        Some(self)
    }

    fn without_time(mut self, i: usize) -> Self {
        if i < self.times.len() {
            self.times.remove(i);
        }
        self
    }
}

pub static PREFS: GlobalSignal<RemindPrefs> = Signal::global(RemindPrefs::load);

/// Re-reads the reminder times after a sync or restore (see `crate::synced`).
pub fn reload() {
    let fresh = RemindPrefs::load();
    if *PREFS.peek() != fresh {
        *PREFS.write() = fresh;
    }
}

/// This device rings. Device-only, never synced. Until he chooses, devices
/// with a vibration motor ring (his phone), and tablets and the Mac stay
/// quiet; it's a guess about the hardware, not the device's name.
pub static HERE: GlobalSignal<bool> =
    Signal::global(|| shared().local_setting(HERE_KEY).unwrap_or_else(crate::platform::haptics::available));

const HERE_KEY: &str = "remind_here";

pub fn set_here(here: bool) {
    crate::persistence::save("local.remind_here", move || {
        shared().put_local_setting(HERE_KEY, &here).map_err(|e| e.to_string())
    });
    *HERE.write() = here;
}

/// Called once in `App`: re-plans this device's reminders whenever an
/// answer is logged or the reminder or planner settings change.
pub fn use_local_reminders() {
    let review = use_review();
    use_effect(move || {
        let _ = (review.revision)();
        let prefs = PREFS.read().clone();
        let here = *HERE.read();
        let weekdays = crate::plan::SETTINGS.read().weekdays;
        if !native::local() {
            return;
        }
        let now = now_ms();
        let (due, answered) = {
            let b = review.backend.borrow();
            (b.due_forecast(14, now), b.answers_today(now))
        };
        let (repeat, items) = armed(&prefs, here, weekdays, &due, answered, now);
        native::repeat(&repeat);
        native::schedule(&items);
    });
}

/// What this device arms: the fallback repeat and the scheduled reminders,
/// as the JSON the native side reads (`{}` and `[]` when nothing rings).
fn armed(
    prefs: &RemindPrefs,
    here: bool,
    weekdays: [bool; 7],
    due: &[u32],
    answered: usize,
    now: i64,
) -> (String, String) {
    let times = if prefs.on { prefs.times.clone() } else { Vec::new() };
    let inputs = plan::Inputs {
        here,
        times: &times,
        weekdays,
        due,
        // He studied today and nothing is left due: today's done.
        target_met: answered > 0 && due.first().is_none_or(|&d| d == 0),
        now_ms: now.max(0) as u64,
        utc_offset_min: utc_offset_minutes(),
    };
    // The fallback first, so a reminder firing in between finds it.
    let repeat = plan::repeat(&inputs).and_then(|r| serde_json::to_string(&r).ok());
    let items = plan::schedule(&inputs);
    (repeat.unwrap_or_else(|| "{}".into()), serde_json::to_string(&items).unwrap_or_else(|_| "[]".into()))
}

pub(crate) fn hhmm(t: u16) -> String {
    format!("{:02}:{:02}", t / 60, t % 60)
}

pub(crate) fn parse_hhmm(s: &str) -> Option<u16> {
    let (h, m) = s.split_once(':')?;
    let (h, m): (u16, u16) = (h.parse().ok()?, m.get(..2)?.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// Settings › Reminders.
#[component]
pub fn RemindSection() -> Element {
    // Android: re-read when the sheet re-renders (e.g. after he returns from
    // its settings). The Mac answers later, so it is polled while open.
    let mut mac_allowed = use_signal(native::enabled);
    use_future(move || async move {
        if cfg!(target_os = "macos") {
            loop {
                crate::platform::time::sleep(std::time::Duration::from_secs(1)).await;
                let now = native::enabled();
                if *mac_allowed.peek() != now {
                    mac_allowed.set(now);
                }
            }
        }
    });
    let allowed = if cfg!(target_os = "macos") { mac_allowed() } else { native::enabled() };
    rsx! {
        RemindView {
            local: native::local(),
            mac: cfg!(target_os = "macos"),
            allowed,
            unrestricted: native::battery_unrestricted(),
        }
    }
}

/// Settings › Reminders on a device that can (`local`) or can't ring, with
/// what the system said about notifications and the battery.
#[component]
fn RemindView(local: bool, mac: bool, allowed: Option<bool>, unrestricted: Option<bool>) -> Element {
    use crate::i18n::remind as tx;
    use crate::settings::DeviceOnly;
    let prefs = PREFS();
    let here = HERE();
    let on_off = |on: bool| if on { tx::ON } else { tx::OFF };

    if !local {
        return rsx! {
            crate::ui::Card {
                h2 { {tx::SECTION} }
                p { class: "muted", {tx::PHONE_ONLY} }
            }
        };
    }
    rsx! {
        crate::ui::Card {
            h2 { {tx::DAILY} }
            div { class: "segmented", role: "radiogroup", aria_label: tx::DAILY,
                for on in [true, false] {
                    button {
                        key: "{on}",
                        role: "radio",
                        "aria-checked": prefs.on == on,
                        onclick: {
                            let prefs = prefs.clone();
                            move |_| {
                                if on {
                                    native::request_permission();
                                }
                                RemindPrefs { on, ..prefs.clone() }.save();
                            }
                        },
                        {on_off(on)}
                    }
                }
            }
            if prefs.on {
                h3 { {tx::TIMES} }
                div { class: "remind-times",
                    for (i , t) in prefs.times.iter().copied().enumerate() {
                        div { key: "{i}-{t}", class: "row",
                            input {
                                r#type: "time",
                                value: hhmm(t),
                                aria_label: tx::TIME,
                                onchange: {
                                    let prefs = prefs.clone();
                                    move |e: FormEvent| {
                                        if let Some(p) = prefs.clone().with_time(i, &e.value()) {
                                            p.save();
                                        }
                                    }
                                },
                            }
                            if prefs.times.len() > 1 {
                                button {
                                    class: "btn btn-secondary",
                                    onclick: {
                                        let prefs = prefs.clone();
                                        move |_| prefs.clone().without_time(i).save()
                                    },
                                    {tx::REMOVE}
                                }
                            }
                        }
                    }
                    if prefs.times.len() < MAX_TIMES {
                        button {
                            class: "btn btn-secondary",
                            onclick: {
                                let prefs = prefs.clone();
                                move |_| prefs.clone().with_added_time().save()
                            },
                            {tx::ADD}
                        }
                    }
                }
            }
        }
        if prefs.on {
            crate::ui::Card {
                h2 { {tx::HERE} DeviceOnly {} }
                div { class: "segmented", role: "radiogroup", aria_label: tx::HERE,
                    for on in [true, false] {
                        button {
                            key: "{on}",
                            role: "radio",
                            "aria-checked": here == on,
                            onclick: move |_| {
                                if on {
                                    native::request_permission();
                                }
                                set_here(on);
                            },
                            {on_off(on)}
                        }
                    }
                }
                if here && allowed == Some(false) {
                    p { {tx::BLOCKED} }
                    div { class: "row",
                        // macOS asks only once; after a no, only System Settings can change it.
                        if !mac {
                            button { class: "btn btn-primary", onclick: move |_| native::request_permission(), {tx::ALLOW} }
                        }
                        button { class: "btn btn-secondary", onclick: move |_| native::open_notification_settings(), {tx::OPEN_NOTIFICATIONS} }
                    }
                }
                if here && mac && allowed != Some(false) {
                    p { class: "muted small", {tx::MAC_HINT} }
                    button { class: "btn btn-secondary", onclick: move |_| native::open_notification_settings(), {tx::OPEN_NOTIFICATIONS} }
                }
                if here && unrestricted == Some(false) {
                    p { class: "muted small", {tx::BATTERY} }
                    button { class: "btn btn-secondary", onclick: move |_| native::open_app_settings(), {tx::OPEN_APP} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that save to the shared test store take turns.
    static SAVING: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn times_round_trip_through_the_time_input() {
        assert_eq!(parse_hhmm("19:00"), Some(19 * 60));
        assert_eq!(parse_hhmm("07:45:00"), Some(7 * 60 + 45));
        assert_eq!(hhmm(7 * 60 + 5), "07:05");
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("nope"), None);
    }

    #[test]
    fn editing_times_keeps_them_sorted_unique_and_at_most_three() {
        let p =
            RemindPrefs { on: true, times: vec![20 * 60, 8 * 60, 20 * 60, 12 * 60, 22 * 60] }.normalized();
        assert_eq!(p.times, vec![8 * 60, 12 * 60, 20 * 60]);
        // Added: two hours after the last, never past 11 pm; 7 pm with none.
        assert_eq!(
            RemindPrefs { on: true, times: vec![18 * 60] }.with_added_time().times,
            vec![18 * 60, 20 * 60]
        );
        assert_eq!(RemindPrefs { on: true, times: vec![22 * 60] }.with_added_time().times[1], 23 * 60);
        assert_eq!(RemindPrefs { on: true, times: vec![] }.with_added_time().times, vec![19 * 60]);
        // Changed from the time input, only when it reads.
        let two = RemindPrefs { on: true, times: vec![8 * 60, 19 * 60] };
        assert_eq!(two.clone().with_time(1, "21:30").unwrap().times, vec![8 * 60, 21 * 60 + 30]);
        assert_eq!(two.clone().with_time(1, "nope"), None);
        assert_eq!(two.clone().with_time(5, "10:00"), None);
        assert_eq!(two.clone().without_time(0).times, vec![19 * 60]);
        assert_eq!(two.clone().without_time(9), two);
    }

    #[derive(Props, Clone, PartialEq)]
    struct Case {
        prefs: RemindPrefs,
        here: bool,
        local: bool,
        mac: bool,
        allowed: Option<bool>,
        unrestricted: Option<bool>,
    }

    fn root(c: Case) -> Element {
        use_hook(|| {
            *PREFS.write() = c.prefs.clone();
            *HERE.write() = c.here;
        });
        rsx! { RemindView { local: c.local, mac: c.mac, allowed: c.allowed, unrestricted: c.unrestricted } }
    }

    fn render(c: Case) -> String {
        crate::store::use_memory_store();
        let mut dom = VirtualDom::new_with_props(root, c);
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    fn phone() -> Case {
        Case {
            prefs: RemindPrefs { on: true, times: vec![8 * 60, 19 * 60] },
            here: true,
            local: true,
            mac: false,
            allowed: Some(true),
            unrestricted: Some(true),
        }
    }

    #[test]
    fn a_device_that_cannot_ring_says_the_phone_does() {
        use crate::i18n::remind as tx;
        let page = render(Case { local: false, ..phone() });
        assert!(page.contains(tx::PHONE_ONLY) && !page.contains(tx::DAILY), "{page}");
        // This desktop build arms nothing and knows nothing about permissions.
        assert!(!native::local());
        assert_eq!(native::enabled(), None);
        assert_eq!(native::battery_unrestricted(), None);
    }

    #[test]
    fn the_phone_lists_its_times_and_who_rings() {
        use crate::i18n::remind as tx;
        let page = render(phone());
        for text in [tx::DAILY, tx::TIMES, tx::ADD, tx::REMOVE, tx::HERE] {
            assert!(page.contains(text), "missing {text}: {page}");
        }
        assert!(page.contains("value=\"08:00\"") && page.contains("value=\"19:00\""), "{page}");
        assert_eq!(page.matches(tx::REMOVE).count(), 2, "{page}");
        assert!(!page.contains(tx::BLOCKED) && !page.contains(tx::BATTERY), "all allowed: {page}");

        // Three times: no more to add. One time: nothing to remove.
        let three = render(Case { prefs: RemindPrefs { on: true, times: vec![1, 2, 3] }, ..phone() });
        assert!(!three.contains(tx::ADD), "{three}");
        let one = render(Case { prefs: RemindPrefs::default(), ..phone() });
        assert!(!one.contains(tx::REMOVE), "{one}");

        // Off: no times and no device choice.
        let off = render(Case { prefs: RemindPrefs { on: false, times: vec![60] }, ..phone() });
        assert!(off.contains(tx::DAILY) && !off.contains(tx::TIMES) && !off.contains(tx::HERE), "{off}");
    }

    #[test]
    fn blocked_notifications_and_battery_limits_say_how_to_fix_them() {
        use crate::i18n::remind as tx;
        let page = render(Case { allowed: Some(false), unrestricted: Some(false), ..phone() });
        for text in [tx::BLOCKED, tx::ALLOW, tx::OPEN_NOTIFICATIONS, tx::BATTERY, tx::OPEN_APP] {
            assert!(page.contains(text), "missing {text}: {page}");
        }
        // The Mac asks once: only System Settings can change a no.
        let mac = render(Case { mac: true, allowed: Some(false), unrestricted: None, ..phone() });
        assert!(mac.contains(tx::BLOCKED) && !mac.contains(tx::ALLOW), "{mac}");
        let mac_ok = render(Case { mac: true, allowed: None, unrestricted: None, ..phone() });
        assert!(mac_ok.contains(tx::MAC_HINT), "{mac_ok}");
        // Another device rings: nothing to fix here.
        let elsewhere =
            render(Case { here: false, allowed: Some(false), unrestricted: Some(false), ..phone() });
        assert!(!elsewhere.contains(tx::BLOCKED) && !elsewhere.contains(tx::BATTERY), "{elsewhere}");
    }

    #[test]
    fn old_or_partial_settings_fall_back_to_defaults() {
        let p: RemindPrefs = serde_json::from_str(r#"{"on":false}"#).unwrap();
        assert_eq!(p, RemindPrefs { on: false, ..RemindPrefs::default() });
    }

    #[test]
    fn only_a_device_that_rings_with_times_on_arms_anything() {
        let now = 20_400 * 86_400_000 + 9 * 3_600_000;
        let all = [true; 7];
        let on = RemindPrefs { on: true, times: vec![20 * 60, 23 * 60] };
        let (repeat, items) = armed(&on, true, all, &[5, 3], 0, now);
        assert!(repeat.contains("\"20:00\"") && repeat.contains("\"23:00\""), "{repeat}");
        let items: Vec<serde_json::Value> = serde_json::from_str(&items).unwrap();
        assert!(!items.is_empty(), "today's evening reminders at least");

        for (prefs, here, days) in [
            (RemindPrefs { on: false, ..on.clone() }, true, all),
            (on.clone(), false, all),
            (on.clone(), true, [false; 7]),
        ] {
            assert_eq!(armed(&prefs, here, days, &[5], 0, now), ("{}".to_string(), "[]".to_string()));
        }
        // Studied today with nothing left: today stays quiet, later days still ring.
        let (_, met) = armed(&on, true, all, &[0, 4], 12, now);
        let (_, open) = armed(&on, true, all, &[2, 4], 12, now);
        let first = |j: &str| serde_json::from_str::<Vec<serde_json::Value>>(j).unwrap()[0]["date"].clone();
        assert!(first(&met).as_str() > first(&open).as_str(), "{met} vs {open}");
    }

    #[test]
    fn taps_change_the_times_and_who_rings() {
        let _one_at_a_time = SAVING.lock().unwrap_or_else(|e| e.into_inner());
        use crate::i18n::remind as tx;
        use crate::test_dom::TestDom;
        crate::store::use_memory_store();
        let mut t = TestDom::new(VirtualDom::new_with_props(root, phone()));
        t.click(tx::ADD);
        assert!(t.html().contains("value=\"21:00\""), "two hours after the last: {}", t.html());
        assert!(!t.html().contains(tx::ADD), "three is the most");
        t.input_event("change", "value=\"08:00\"", "06:30");
        assert!(t.html().contains("value=\"06:30\""), "{}", t.html());
        t.input_event("change", "value=\"06:30\"", "not a time");
        assert!(t.html().contains("value=\"06:30\""), "an unreadable time changes nothing");
        t.click(tx::REMOVE);
        assert!(!t.html().contains("value=\"06:30\"") && t.html().contains(tx::ADD), "{}", t.html());
        assert_eq!(t.with(RemindPrefs::load).times, vec![19 * 60, 21 * 60], "saved as shown");

        // This device stops ringing, then the reminders go off altogether.
        // The second On/Off pair is this device's.
        t.click_nth(tx::OFF, 1);
        assert!(!t.with(|| *HERE.peek()));
        t.click_nth(tx::ON, 1);
        assert!(t.with(|| *HERE.peek()));
        t.click(tx::OFF);
        assert!(!t.html().contains(tx::TIMES) && !t.with(|| PREFS.peek().on), "{}", t.html());
        t.click(tx::ON);
        assert!(t.html().contains(tx::TIMES) && t.with(|| PREFS.peek().on));
    }

    #[test]
    fn the_fix_buttons_do_nothing_harmful_on_a_desktop() {
        use crate::i18n::remind as tx;
        use crate::test_dom::TestDom;
        crate::store::use_memory_store();
        let case = Case { allowed: Some(false), unrestricted: Some(false), ..phone() };
        let mut t = TestDom::new(VirtualDom::new_with_props(root, case));
        for button in [tx::ALLOW, tx::OPEN_NOTIFICATIONS, tx::OPEN_APP] {
            t.click(button);
            assert!(t.html().contains(button), "still offered: {button}");
        }
    }

    #[test]
    fn a_settings_section_and_the_planner_mount_off_device() {
        let _one_at_a_time = SAVING.lock().unwrap_or_else(|e| e.into_inner());
        use crate::i18n::remind as tx;
        crate::store::use_memory_store();
        fn app() -> Element {
            crate::store::use_provide_review();
            use_local_reminders();
            rsx! { RemindSection {} }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let page = dioxus::ssr::render(&dom);
        assert!(page.contains(tx::PHONE_ONLY), "this desktop build can't ring: {page}");
        dom.in_runtime(|| {
            let saved = RemindPrefs { on: true, times: vec![7 * 60] };
            saved.clone().save();
            *PREFS.write() = RemindPrefs::default();
            reload();
            assert_eq!(*PREFS.peek(), saved);
            set_here(false);
            assert!(!*HERE.peek());
        });
    }
}
