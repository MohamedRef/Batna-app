// The "what's new" sheet's course groups and changelog search
// (crates/app/src/update/groups.rs, filter.rs).

use super::count::Noun;

/// The group of lines about the app itself (screens, sync, updates).
pub const GROUP_APP: &str = "التطبيق";
/// Lines nothing places (older builds).
pub const GROUP_OTHER: &str = "حاجات تانية";
/// A line's tap target, for screen readers: "افتح: <line>".
pub const OPEN: &str = "افتح";

pub const CHANGE: Noun =
    Noun { one: "تغيير واحد", two: "تغييرين", few: "تغييرات", many: "تغيير" };
pub const COURSE: Noun =
    Noun { one: "مادة واحدة", two: "مادتين", few: "مواد", many: "مادة" };
/// After the course count when the app changed too.
pub const AND_APP: &str = " والتطبيق";

/// "306 تغيير في 9 مواد والتطبيق؛ الأكتر: كبد (41)".
pub fn digest(changes: &str, courses: &str, and_app: &str, top: &str, n: usize) -> String {
    format!("{changes} في {courses}{and_app}؛ الأكتر: {top} ({n})")
}

/// The head of the box of finished lessons a release changed.
pub fn finished_changed(n: usize) -> String {
    match n {
        1 => "درس خلصته اتغيّر".to_owned(),
        2 => "درسين خلصتهم اتغيّروا".to_owned(),
        _ if (3..=10).contains(&(n % 100)) => format!("{n} دروس خلصتها اتغيّرت"),
        _ => format!("{n} درس خلصتهم اتغيّروا"),
    }
}

// Changelog search (Settings › Updates › سجل التغييرات).
pub const SEARCH: &str = "دوّر في التغييرات";
pub const ALL_GROUPS: &str = "كل المواد";
pub const ALL_KINDS: &str = "كل الأنواع";
pub const CLEAR: &str = "امسح البحث";
pub const NO_MATCH: &str = "مفيش تغيير بالكلام ده";

/// "12 تغيير في 3 نسخ".
pub fn found(changes: usize, builds: usize) -> String {
    let b = match builds {
        1 => "نسخة واحدة".to_owned(),
        2 => "نسختين".to_owned(),
        _ if (3..=10).contains(&(builds % 100)) => format!("{builds} نسخ"),
        _ => format!("{builds} نسخة"),
    };
    format!("{} في {b}", super::count::count(changes, &CHANGE))
}
