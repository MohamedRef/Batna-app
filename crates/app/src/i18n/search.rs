//! Full-text search results (#1350).

use super::count::{STEP, count};

pub const FULL_LABEL: &str = "الكلمة جوّه الدروس والكروت";
pub const BY_COURSE: &str = "النتايج حسب المقرر";
pub const ALL: &str = "الكل";
pub const MORE: &str = "اعرض كمان";
pub const OPEN_STEP: &str = "افتح الدرس عند الخطوة دي";
pub const OPEN_ITEM: &str = "افتح السؤال ده";
pub const OPEN_LESSON: &str = "افتح الدرس اللي بيشرحها";

/// "في شرح 19 درس".
pub fn in_lessons(n: usize) -> String {
    format!("في شرح {}", count(n, &super::count::LESSON))
}
/// "في الكروت والأسئلة (43)".
pub fn in_items(n: usize) -> String {
    format!("في الكروت والأسئلة ({n})")
}
/// "خطوة 4 من 12".
pub fn step_n(k: usize, n: usize) -> String {
    format!("خطوة {k} من {n}")
}
/// "وفي خطوتين كمان".
pub fn also_in(n: usize) -> String {
    format!("وفي {} كمان", count(n, &STEP))
}
