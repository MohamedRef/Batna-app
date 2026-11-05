//! The study session: which items to show, in what order, and how a typed
//! answer is checked. Pure logic; the app's `study` screen drives it.
//!
//! Order: due reviews first, interleaved across topics (mixing helps
//! retrieval), then new items blocked by topic in book order (one idea at a
//! time while it is first learned). Everything is capped at a time budget.

use std::collections::{BTreeMap, HashSet};

use crate::content::{Body, Item, Lecture, OutlineEntry, Status};
use crate::review::Grade;

mod compose;
pub use compose::*;

mod running;
pub use running::*;

mod typed;
pub use typed::*;

#[cfg(test)]
mod tests;
