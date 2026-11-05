//! A running session: outcomes, undo and the summary.

use super::*;

/// How one item went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub item_id: String,
    pub grade: Grade,
    pub fresh: bool,
    /// Graded again later in the same session after an Again.
    pub retry: bool,
}

/// A session in progress. An item graded Again comes back once at the end of
/// the session, so it is retrieved again while it is still fresh.
#[derive(Debug, Clone, Default)]
pub struct Session {
    queue: Vec<Planned>,
    pos: usize,
    retried: HashSet<String>,
    pub outcomes: Vec<Outcome>,
    /// One per grade, newest last, so [`Self::undo`] can put it all back.
    steps: Vec<Step>,
}

/// What one grade changed, to take it back exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Step {
    /// The position before the grade (hidden items it skipped included).
    pos: usize,
    /// Where the grade queued its Again retry, if it did.
    retry_at: Option<usize>,
    /// The review event the grade logged, for the store to take back.
    event: Option<String>,
}

/// A grade [`Session::undo`] took back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undone {
    pub outcome: Outcome,
    /// The logged review event to retract (`StudyStore::retract`).
    pub event: Option<String>,
}

impl Session {
    pub fn new(plan: Vec<Planned>) -> Self {
        Self { queue: plan, ..Self::default() }
    }

    pub fn current(&self) -> Option<&Planned> {
        self.queue.get(self.pos)
    }

    /// 0-based position and the current queue length (it grows on Again).
    pub fn progress(&self) -> (usize, usize) {
        (self.pos, self.queue.len())
    }

    pub fn is_done(&self) -> bool {
        self.pos >= self.queue.len()
    }

    /// Whether the current item is a same-session retry.
    pub fn is_retry(&self) -> bool {
        self.current().is_some_and(|p| self.queue[..self.pos].iter().any(|q| q.item_id == p.item_id))
    }

    pub fn grade(&mut self, grade: Grade) {
        self.grade_unless(grade, |_| false);
    }

    /// Grades the current item, then skips every item `hidden` says he
    /// marked ✗ (here or by sync) from what's left (#246). A hidden item is
    /// not queued for a retry either.
    pub fn grade_unless(&mut self, grade: Grade, hidden: impl Fn(&str) -> bool) {
        self.grade_logged(grade, None, hidden);
    }

    /// [`Self::grade_unless`], remembering the review event the grade was
    /// logged as, so [`Self::undo`] can hand it back for retraction.
    pub fn grade_logged(&mut self, grade: Grade, event: Option<String>, hidden: impl Fn(&str) -> bool) {
        let Some(p) = self.current().cloned() else { return };
        let retry = self.is_retry();
        let mut step = Step { pos: self.pos, retry_at: None, event };
        self.outcomes.push(Outcome { item_id: p.item_id.clone(), grade, fresh: p.fresh, retry });
        if grade == Grade::Again && !hidden(&p.item_id) && self.retried.insert(p.item_id.clone()) {
            step.retry_at = Some(self.queue.len());
            self.queue.push(Planned { fresh: false, ..p });
        }
        self.steps.push(step);
        self.pos += 1;
        while self.current().is_some_and(|q| hidden(&q.item_id)) {
            self.pos += 1;
        }
    }

    /// Whether there is a grade [`Self::undo`] can take back.
    pub fn can_undo(&self) -> bool {
        !self.steps.is_empty()
    }

    /// Takes back the newest grade: the item is current again, its outcome
    /// and its Again retry are gone, and the event it was logged as comes
    /// back for the store to retract. Grades before it stay; calling it again
    /// takes those back one by one.
    pub fn undo(&mut self) -> Option<Undone> {
        let step = self.steps.pop()?;
        let outcome = self.outcomes.pop()?;
        if let Some(at) = step.retry_at {
            if at < self.queue.len() {
                self.queue.remove(at);
            }
            self.retried.remove(&outcome.item_id);
        }
        self.pos = step.pos;
        Some(Undone { outcome, event: step.event })
    }

    /// Queues more items after what's left, skipping any still to come, so
    /// "study this page" during a session adds to it instead of replacing it.
    /// Returns how many were added.
    pub fn append(&mut self, more: Vec<Planned>) -> usize {
        let mut added = 0;
        for p in more {
            if !self.queue[self.pos..].iter().any(|q| q.item_id == p.item_id) {
                self.queue.push(p);
                added += 1;
            }
        }
        added
    }

    pub fn summary(&self) -> Summary {
        // The first grade of each item is the one that counts for "right".
        let mut first: Vec<&Outcome> = Vec::new();
        for o in &self.outcomes {
            if !first.iter().any(|f| f.item_id == o.item_id) {
                first.push(o);
            }
        }
        let right = first.iter().filter(|o| matches!(o.grade, Grade::Good | Grade::Easy)).count();
        let shaky: Vec<String> = first
            .iter()
            .filter(|o| matches!(o.grade, Grade::Again | Grade::Hard))
            .map(|o| o.item_id.clone())
            .collect();
        Summary { items: first.len(), right, fresh: first.iter().filter(|o| o.fresh).count(), shaky }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// Distinct items answered.
    pub items: usize,
    /// Answered Good or Easy the first time.
    pub right: usize,
    /// Seen for the first time.
    pub fresh: usize,
    /// Graded Again or Hard the first time: these come back sooner.
    pub shaky: Vec<String>,
}
