//! The review state the screens share: the log replayed through FSRS, the
//! verdicts, and the settings, kept in memory over the store. It implements
//! [`StudyStore`] for the study player and the other study screens.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use batna_core::content::{Body, Item};
use batna_core::review::verify::{self, Check, Edit, Judgement, Verdict, Verdicts};
use batna_core::review::{LoggedReview, ReviewEvent, StudyStore, StudyStoreHandle};
use batna_core::srs::{self, CardState, ReviewSettings, Scheduler};
use dioxus::prelude::*;

use super::{Store, now_ms, utc_offset_minutes};
use crate::content::Library;

/// One study item with the lecture it belongs to.
#[derive(Clone, Debug)]
pub struct Entry {
    pub lecture: String,
    pub item: Item,
}

/// Every item in the built-in content, by id, in book order of lectures.
pub type Catalog = BTreeMap<String, Entry>;

pub fn catalog(lib: &Library) -> Catalog {
    lib.lectures
        .values()
        .flat_map(|l| {
            l.items.iter().map(|i| (i.id.clone(), Entry { lecture: l.lecture.clone(), item: i.clone() }))
        })
        .collect()
}

pub struct ReviewBackend {
    store: &'static Store,
    catalog: Catalog,
    pub settings: ReviewSettings,
    scheduler: Scheduler,
    log: Vec<LoggedReview>,
    states: BTreeMap<String, CardState>,
    judgements: BTreeMap<String, Judgement>,
    /// Bumps the screens' revision signal after every write.
    on_change: Option<Box<dyn FnMut()>>,
}

/// Rough time per item, for "about N minutes" on Today.
pub fn seconds_for(body: &Body) -> u32 {
    match body {
        Body::Card { .. } | Body::Cloze { .. } => 15,
        Body::Mcq { .. } => 45,
        Body::Enumerate { .. } | Body::Order { .. } => 60,
        Body::Table { .. } => 90,
        Body::Essay { .. } | Body::Calc { .. } => 120,
        Body::Case { .. } => 180,
    }
}

/// What Today shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overview {
    pub due: usize,
    pub due_minutes: u32,
    /// New items he can still start today (the daily cap and what's written).
    pub new_today: usize,
    /// New items written and not started yet, ignoring the cap.
    pub unseen: usize,
    pub due_tomorrow: u32,
    /// Items he has checked (✓ or ✎), and items he has seen at all.
    pub checked: usize,
    pub seen: usize,
    pub lectures: Vec<LectureProgress>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LectureProgress {
    pub id: String,
    pub title: String,
    pub total: usize,
    pub seen: usize,
    pub due: usize,
}

impl ReviewBackend {
    pub fn new(store: &'static Store, catalog: Catalog) -> Self {
        let mut b = Self {
            store,
            catalog,
            settings: ReviewSettings::default(),
            scheduler: Scheduler::default(),
            log: Vec::new(),
            states: BTreeMap::new(),
            judgements: BTreeMap::new(),
            on_change: None,
        };
        b.reload();
        b
    }

    /// Re-reads everything from the store.
    pub fn reload(&mut self) {
        self.settings = self.store.review_settings();
        self.scheduler = Scheduler::from_settings(&self.settings, utc_offset_minutes());
        self.log = self.store.reviews().unwrap_or_default();
        self.judgements = self.store.judgements().unwrap_or_default();
        self.replay();
    }

    /// Re-reads everything and tells the screens (after an import).
    pub fn reload_and_notify(&mut self) {
        self.reload();
        self.changed();
    }

    fn replay(&mut self) {
        self.states = self.scheduler.replay_all(&self.log);
    }

    fn changed(&mut self) {
        if let Some(f) = &mut self.on_change {
            f();
        }
    }

    pub fn entry(&self, item_id: &str) -> Option<&Entry> {
        self.catalog.get(item_id)
    }

    pub fn judgement(&self, item_id: &str) -> Option<&Judgement> {
        self.judgements.get(item_id)
    }

    /// Whether to ask ✓ / ✎ / ✗ for an item. Unknown items never ask.
    pub fn check(&self, item_id: &str) -> Check {
        match self.entry(item_id) {
            Some(e) => verify::check(&e.item, self.judgement(item_id)),
            None => Check::Settled,
        }
    }

    /// Records ✓ / ✎ / ✗ on the item as it is now.
    pub fn judge(&mut self, item_id: &str, verdict: Verdict, edit: Edit, note: Option<String>) {
        let Some(entry) = self.entry(item_id) else { return };
        let j = Judgement {
            verdict,
            edit: if verdict == Verdict::Edited { edit } else { Edit::new() },
            note: note.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()),
            fingerprint: verify::fingerprint(&entry.item),
            at_ms: now_ms(),
        };
        if self.store.set_judgement(item_id, Some(&j)).is_ok() {
            self.judgements.insert(item_id.to_string(), j);
            self.changed();
        }
    }

    /// Takes a verdict back: the item is unverified again.
    pub fn undo_judgement(&mut self, item_id: &str) {
        if self.store.set_judgement(item_id, None).is_ok() {
            self.judgements.remove(item_id);
            self.changed();
        }
    }

    pub fn set_settings(&mut self, settings: ReviewSettings) {
        if self.store.put_review_settings(&settings).is_ok() {
            self.settings = settings;
            self.scheduler = Scheduler::from_settings(&self.settings, utc_offset_minutes());
            self.replay();
            self.changed();
        }
    }

    /// Items falling due on each of the next `days` study days (index 0 is
    /// today, overdue included). Items marked ✗ are left out.
    #[allow(dead_code)] // reminders and recap (chunk 20)
    pub fn due_forecast(&self, days: usize, now_ms: i64) -> Vec<u32> {
        self.scheduler.forecast(self.states.values().filter(|s| !self.hidden(&s.item_id)), now_ms, days)
    }

    /// Answers logged today (every screen, evidence included).
    #[allow(dead_code)] // reminders and recap (chunk 20)
    pub fn answers_today(&self, now_ms: i64) -> usize {
        let today = self.scheduler.day.day(now_ms);
        self.log.iter().filter(|l| self.scheduler.day.day(l.event.at_ms) == today).count()
    }

    /// New items started on each of the last `days` study days, oldest
    /// first (the last entry is today).
    #[allow(dead_code)] // reminders and recap (chunk 20)
    pub fn learned_per_day(&self, days: usize, now_ms: i64) -> Vec<u32> {
        let today = self.scheduler.day.day(now_ms);
        let mut out = vec![0; days];
        for first in self.states.values().filter_map(|s| s.first_review_ms) {
            let ago = today - self.scheduler.day.day(first);
            if (0..days as i64).contains(&ago) {
                out[days - 1 - ago as usize] += 1;
            }
        }
        out
    }

    pub fn reviews_logged(&self) -> usize {
        self.log.len()
    }

    /// The verdicts export (`batna.verdicts/v1`), every verdict on every item.
    pub fn verdicts(&self, now_ms: i64) -> Verdicts {
        Verdicts::from_judgements(self.judgements.iter().map(|(id, j)| (id.as_str(), j)), now_ms)
    }

    pub fn verdict_counts(&self) -> HashMap<Verdict, usize> {
        let mut out = HashMap::new();
        for j in self.judgements.values() {
            *out.entry(j.verdict).or_default() += 1;
        }
        out
    }

    fn hidden(&self, item_id: &str) -> bool {
        self.check(item_id) == Check::Hidden
    }

    fn due_states(&self, now_ms: i64) -> Vec<&CardState> {
        self.scheduler.due(
            self.states
                .values()
                .filter(|s| self.catalog.contains_key(&s.item_id) && !self.hidden(&s.item_id)),
            now_ms,
        )
    }

    pub fn overview(&self, lib: &Library, now_ms: i64) -> Overview {
        let due = self.due_states(now_ms);
        let due_minutes = due
            .iter()
            .filter_map(|s| self.entry(&s.item_id))
            .map(|e| seconds_for(&e.item.body))
            .sum::<u32>()
            .div_ceil(60);
        let unseen =
            self.catalog.keys().filter(|id| !self.states.contains_key(*id) && !self.hidden(id)).count();
        let tomorrow = self.scheduler.day.day_start_ms(self.scheduler.day.day(now_ms) + 1);
        let due_tomorrow = self
            .states
            .values()
            .filter(|s| s.due_ms.is_some_and(|d| d > now_ms && d <= tomorrow) && !self.hidden(&s.item_id))
            .count() as u32;
        let checked = self
            .catalog
            .iter()
            .filter(|(id, e)| verify::check(&e.item, self.judgement(id)) == Check::Settled)
            .count();
        let mut lectures = Vec::new();
        for entry in &lib.index.lectures {
            let Some(lec) = lib.lectures.get(&entry.id) else { continue };
            let seen = lec.items.iter().filter(|i| self.states.contains_key(&i.id)).count();
            if seen == 0 {
                continue;
            }
            let lec_due =
                due.iter().filter(|s| self.entry(&s.item_id).is_some_and(|e| e.lecture == entry.id)).count();
            lectures.push(LectureProgress {
                id: entry.id.clone(),
                title: entry.title.clone(),
                total: lec.items.len(),
                seen,
                due: lec_due,
            });
        }
        Overview {
            due: due.len(),
            due_minutes,
            new_today: self.new_left_today(now_ms).min(unseen),
            unseen,
            due_tomorrow,
            checked,
            seen: self.states.keys().filter(|id| self.catalog.contains_key(*id)).count(),
            lectures,
        }
    }
}

impl StudyStore for ReviewBackend {
    fn record(&mut self, event: ReviewEvent) {
        if let Ok(logged) = self.store.append_review(event) {
            self.log.push(logged);
            self.replay();
            self.changed();
        }
    }

    fn due(&self, now_ms: i64) -> Vec<String> {
        self.due_states(now_ms).into_iter().map(|s| s.item_id.clone()).collect()
    }

    fn is_new(&self, item_id: &str) -> bool {
        self.states.get(item_id).is_none_or(CardState::is_new)
    }

    fn is_hidden(&self, item_id: &str) -> bool {
        self.hidden(item_id)
    }

    fn effective(&self, item: &Item) -> Item {
        verify::effective(item, self.judgement(&item.id))
    }

    fn next_due_ms(&self, now_ms: i64) -> Option<i64> {
        self.states
            .values()
            .filter(|s| !self.hidden(&s.item_id))
            .filter_map(|s| s.due_ms)
            .filter(|d| *d > now_ms)
            .min()
    }

    fn new_left_today(&self, now_ms: i64) -> usize {
        srs::new_left_today(&self.settings, self.scheduler.started_today(self.states.values(), now_ms))
    }

    fn intervals(&self, item_id: &str, now_ms: i64) -> Option<[f32; 4]> {
        let state = self.states.get(item_id).cloned().unwrap_or_else(|| CardState::new(item_id));
        Some(self.scheduler.intervals(&state, now_ms))
    }
}

/// The review state in context, with a revision the screens subscribe to.
#[derive(Clone)]
pub struct Review {
    pub backend: Rc<RefCell<ReviewBackend>>,
    pub revision: Signal<u64>,
}

/// Called once in `App`: opens the store, provides [`Review`] and the
/// [`StudyStoreHandle`] for the study screens, and starts the weekly FSRS
/// refit in the background.
pub fn use_provide_review() {
    let review = use_context_provider(|| {
        let backend = Rc::new(RefCell::new(ReviewBackend::new(super::shared(), catalog(Library::get()))));
        let mut revision = Signal::new(0);
        backend.borrow_mut().on_change = Some(Box::new(move || *revision.write() += 1));
        spawn_refit();
        Review { backend, revision }
    });
    use_context_provider(|| StudyStoreHandle(review.backend.clone() as Rc<RefCell<dyn StudyStore>>));
}

/// The review state, subscribed: the calling component re-renders after
/// every answer, verdict or settings change.
pub fn use_review() -> Review {
    let review = use_context::<Review>();
    let _ = (review.revision)();
    review
}

/// Fits personal FSRS parameters once a week when there are enough reviews
/// (`MIN_REVIEWS_TO_OPTIMIZE`). Off the UI thread; the new parameters apply
/// from the next launch.
fn spawn_refit() {
    let _ = std::thread::Builder::new().name("fsrs-refit".into()).spawn(|| {
        let store = super::shared();
        let now = now_ms();
        let settings = store.review_settings();
        if !settings.refit_due(now) {
            return;
        }
        let Ok(log) = store.reviews() else { return };
        let scheduler = Scheduler::from_settings(&settings, utc_offset_minutes());
        if let Some(params) = scheduler.optimize(&log, now) {
            // Re-read so a settings change made meanwhile isn't lost.
            let mut latest = store.review_settings();
            latest.fsrs = Some(params);
            let _ = store.put_review_settings(&latest);
        }
    });
}

#[cfg(test)]
mod tests {
    use batna_core::content::Lecture;
    use batna_core::review::{Grade, Origin};

    use super::*;

    const T0: i64 = 1_793_610_000_000;

    fn backend() -> ReviewBackend {
        let lec: Lecture =
            serde_yaml_ng::from_str(include_str!("../../../../fixtures/content/sample.yaml")).unwrap();
        let catalog = lec
            .items
            .iter()
            .map(|i| (i.id.clone(), Entry { lecture: lec.lecture.clone(), item: i.clone() }))
            .collect();
        let store: &'static Store = Box::leak(Box::new(Store::open_in_memory().unwrap()));
        ReviewBackend::new(store, catalog)
    }

    fn answer(b: &mut ReviewBackend, id: &str, grade: Grade, at_ms: i64) {
        StudyStore::record(
            b,
            ReviewEvent { item_id: id.into(), grade, origin: Origin::Study, at_ms, took_ms: 5000 },
        );
    }

    #[test]
    fn answers_schedule_and_wrong_items_hide() {
        let mut b = backend();
        assert!(b.is_new("hb.L01.001"));
        answer(&mut b, "hb.L01.001", Grade::Again, T0);
        answer(&mut b, "hb.L01.002", Grade::Good, T0);
        assert!(!b.is_new("hb.L01.001"));
        // FSRS brings an Again back about five hours later.
        let later = T0 + 6 * 3_600_000;
        assert_eq!(b.due(later), vec!["hb.L01.001".to_string()]);
        assert!(b.next_due_ms(T0).is_some());

        b.judge("hb.L01.001", Verdict::Wrong, Edit::new(), Some("  page 2 says otherwise ".into()));
        assert!(b.is_hidden("hb.L01.001"));
        assert!(b.due(later).is_empty());
        assert_eq!(b.judgement("hb.L01.001").unwrap().note.as_deref(), Some("page 2 says otherwise"));
        b.undo_judgement("hb.L01.001");
        assert_eq!(b.check("hb.L01.001"), Check::Ask);
    }

    #[test]
    fn edits_override_what_he_studies() {
        let mut b = backend();
        let edit = Edit::from([("back".into(), verify::FieldValue::Text("Fibrosis + nodules".into()))]);
        b.judge("hb.L01.001", Verdict::Edited, edit, None);
        let item = b.entry("hb.L01.001").unwrap().item.clone();
        let Body::Card { back, .. } = b.effective(&item).body else { panic!() };
        assert_eq!(back, "Fibrosis + nodules");
        assert_eq!(b.check("hb.L01.001"), Check::Settled);
        assert_eq!(b.verdicts(T0).verdicts.len(), 1);
    }

    #[test]
    fn counts_for_reminders_and_recap() {
        let mut b = backend();
        let day = 86_400_000;
        answer(&mut b, "hb.L01.001", Grade::Good, T0 - day);
        answer(&mut b, "hb.L01.002", Grade::Good, T0);
        answer(&mut b, "hb.L01.002", Grade::Good, T0 + 1000);
        assert_eq!(b.answers_today(T0 + 2000), 2);
        assert_eq!(b.learned_per_day(3, T0 + 2000), vec![0, 1, 1]);
        assert_eq!(b.due_forecast(30, T0).iter().sum::<u32>(), 2);
    }

    #[test]
    fn the_daily_new_cap_counts_items_started_today() {
        let mut b = backend();
        b.set_settings(ReviewSettings { new_per_day: 15, ..Default::default() });
        answer(&mut b, "hb.L01.001", Grade::Good, T0);
        answer(&mut b, "hb.L01.001", Grade::Good, T0 + 1000);
        assert_eq!(b.new_left_today(T0 + 2000), 14);
        let i = b.intervals("hb.L01.002", T0).unwrap();
        assert!(i[0] <= i[3]);
    }
}
