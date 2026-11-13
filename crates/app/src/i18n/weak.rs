//! Weak-fact marks (`crate::weak`). Adapted from Steps-app main:
//! crates/app/src/i18n/weak.rs.
use super::count::{CARD, QUESTION, count};

pub const MARK: &str = "راجعها تاني";
pub const WHY: &str = "فيه حاجة من الصفحة دي غلطت فيها آخر مرة. العلامة بتمشي أول ما تجاوبها صح.";
pub const SETTING_TITLE: &str = "علامات المراجعة";
pub const SETTING_LINE: &str = "علّم في الدروس وصفحات الكتاب الحاجات اللي غلطت فيها آخر مرة";
/// "ذاكر الـ 3 كروت دول"
pub fn study(n: usize) -> String {
    format!("ذاكر {} دلوقتي", count(n, &CARD))
}
/// "وفيه سؤال واحد من الصفحة دي محتاج تبص عليه تاني."; the verb and
/// pronoun follow the number like the noun does («سؤالين … محتاجين تبص
/// عليهم», as `cloud::pending`).
pub fn questions(n: usize) -> String {
    let plural = n == 2 || (3..=10).contains(&(n % 100));
    let tail = if plural { "محتاجين تبص عليهم" } else { "محتاج تبص عليه" };
    format!("وفيه {} من الصفحة دي {tail} تاني.", count(n, &QUESTION))
}
