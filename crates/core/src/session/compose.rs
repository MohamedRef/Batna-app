//! Composing a session: which items, in what order, within the time budget.

use super::*;

/// A rough time for one item, in seconds, used to fit a session to its budget.
pub fn estimate_secs(body: &Body) -> u32 {
    match body {
        Body::Card { .. } => 20,
        Body::Cloze { text, .. } => {
            let blanks = crate::content::cloze_blanks(text).map_or(1, |b| b.len().max(1));
            15 + 10 * blanks as u32
        }
        Body::Mcq { .. } => 45,
        Body::Enumerate { points, .. } => 30 + 6 * points.len() as u32,
        Body::Essay { outline, .. } => 60 + 20 * outline.len() as u32,
        Body::Case { steps, .. } => 30 + 40 * steps.len() as u32,
        Body::Table { blanks, .. } => 15 + 20 * blanks.len() as u32,
        Body::Order { steps, .. } => 10 + 8 * steps.len() as u32,
        Body::Calc { .. } => 90,
    }
}

/// What the composer gets from the review store and the learner's choices.
#[derive(Debug, Clone)]
pub struct Inputs<'a> {
    /// The lectures to study from, in book order.
    pub lectures: &'a [&'a Lecture],
    /// Only these topics (`hb.L01/definition`); empty means every topic.
    pub topics: &'a HashSet<String>,
    /// Item kinds the player can show (`card`, `cloze`, …).
    pub kinds: &'a [&'a str],
    /// Due item ids, most overdue first.
    pub due: &'a [String],
    /// Ids already studied at least once (not new).
    pub seen: &'a HashSet<String>,
    /// Ids the learner marked wrong (✗): kept out until fixed.
    pub hidden: &'a HashSet<String>,
    /// How many new items may still start today.
    pub max_new: usize,
    /// Time budget in seconds.
    pub budget_secs: u32,
}

/// One planned item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub item_id: String,
    pub lecture: String,
    /// `hb.L01/definition`, or the lecture id when the item has no topic.
    pub topic: String,
    pub est_secs: u32,
    /// First time ever seen.
    pub fresh: bool,
}

/// Builds the ordered session. Always returns at least one item when anything
/// is available, even if that one item is longer than the budget.
pub fn compose(input: &Inputs<'_>) -> Vec<Planned> {
    // Every eligible item, with its book position for stable ordering.
    let mut pool: BTreeMap<&str, (usize, Planned)> = BTreeMap::new();
    let mut book_pos = 0usize;
    for lecture in input.lectures {
        for item in &lecture.items {
            book_pos += 1;
            if !eligible(item, input) {
                continue;
            }
            let topic = topic_key(&lecture.lecture, item);
            if !input.topics.is_empty() && !input.topics.contains(&topic) {
                continue;
            }
            let planned = Planned {
                item_id: item.id.clone(),
                lecture: lecture.lecture.clone(),
                topic,
                est_secs: estimate_secs(&item.body),
                fresh: !input.seen.contains(&item.id),
            };
            pool.insert(item.id.as_str(), (book_pos, planned));
        }
    }

    let mut taken: HashSet<String> = HashSet::new();
    let due: Vec<Planned> = input
        .due
        .iter()
        .filter_map(|id| pool.get(id.as_str()))
        .filter(|(_, p)| taken.insert(p.item_id.clone()))
        .map(|(_, p)| Planned { fresh: false, ..p.clone() })
        .collect();

    let mut fresh: Vec<(usize, Planned)> =
        pool.values().filter(|(_, p)| p.fresh && !taken.contains(&p.item_id)).cloned().collect();
    fresh.sort_by_key(|(pos, _)| *pos);
    let fresh = block_by_topic(fresh.into_iter().map(|(_, p)| p).collect());
    let mut new_left = input.max_new;

    let mut out = Vec::new();
    let mut used = 0u32;
    for p in interleave(due).into_iter().chain(fresh) {
        if p.fresh && new_left == 0 {
            continue;
        }
        if out.is_empty() || used + p.est_secs <= input.budget_secs {
            new_left -= usize::from(p.fresh);
            used += p.est_secs;
            out.push(p);
        }
    }
    out
}

fn eligible(item: &Item, input: &Inputs<'_>) -> bool {
    item.status != Status::Rejected
        && !input.hidden.contains(&item.id)
        && input.kinds.contains(&item.body.kind())
}

fn topic_key(lecture: &str, item: &Item) -> String {
    match &item.topic {
        Some(t) => format!("{lecture}/{t}"),
        None => lecture.to_string(),
    }
}

/// Keeps book order but pulls every item of a topic together, topics in the
/// order they first appear.
fn block_by_topic(items: Vec<Planned>) -> Vec<Planned> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<Planned>> = BTreeMap::new();
    for p in items {
        if !groups.contains_key(&p.topic) {
            order.push(p.topic.clone());
        }
        groups.entry(p.topic.clone()).or_default().push(p);
    }
    order.into_iter().flat_map(|t| groups.remove(&t).unwrap_or_default()).collect()
}

/// Reorders so that two neighbours share a topic only when nothing else is
/// left, while keeping the most overdue items near the front.
fn interleave(mut items: Vec<Planned>) -> Vec<Planned> {
    let mut out: Vec<Planned> = Vec::with_capacity(items.len());
    while !items.is_empty() {
        let last = out.last().map(|p| p.topic.clone());
        let pick = items.iter().position(|p| Some(&p.topic) != last.as_ref()).unwrap_or(0);
        out.push(items.remove(pick));
    }
    out
}
