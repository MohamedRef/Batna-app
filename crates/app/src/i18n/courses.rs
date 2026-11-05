pub const TITLE: &str = "المقررات";
pub const ACTIVE: &str = "الترم ده";
pub const MAINTENANCE: &str = "صيانة";
pub const ARCHIVED: &str = "أرشيف";
pub const LATER: &str = "لسه";
pub const ACTIVE_HINT: &str = "اللي بتذاكره دلوقتي.";
pub const MAINTENANCE_HINT: &str = "خلص امتحانه. مراجعات خفيفة عشان يفضل حاضر لامتحان آخر الماستر.";
pub const ARCHIVED_HINT: &str = "مستخبي من اليوم، بس موجود في المراجع وفي ملف Anki.";
pub const LATER_HINT: &str = "ترم جاي. أول ما كتابه يوصل، المحاضرات بتاعته تظهر هنا.";
pub const G_RESEARCH: &str = "ريسيرش";
pub const G_CLINICAL: &str = "إكلينيكي";
pub const G_SUBSPECIALTY: &str = "تخصص";
pub const G_BASIC: &str = "علوم أساسية";
pub const STATUS: &str = "المقرر ده فين؟";
pub const OPEN_LECTURES: &str = "افتح المحاضرات";
pub const NO_CONTENT: &str = "لسه مفيش محاضرات. هتنزل لما المنهج يوصل.";
pub const ALL: &str = "كل المقررات";
pub const PICK_TITLE: &str = "تذاكر إيه دلوقتي؟";
pub const MAP_LINK: &str = "خريطة المقررات والساعات";

pub fn chip_label(name: &str) -> String {
    format!("المقرر المعروض: {name}. دوس عشان تبدّل")
}
use super::count::{CARD, COURSE, CREDIT_HOUR, HOUR, LECTURE, LESSON, count, count_text};

pub fn all_sub(courses: usize) -> String {
    format!("{} مع بعض", count(courses, &COURSE))
}
/// "3 من 12 درس", "1 من درسين".
pub fn lessons_done(done: usize, all: usize) -> String {
    format!("{done} من {}", count(all, &LESSON))
}
pub fn lectures_only(n: usize) -> String {
    format!("{}، الدروس جاية", count(n, &LECTURE))
}

/// "42 ساعة معتمدة في 9 مقررات".
pub fn hours_line(total: &str, courses: usize) -> String {
    format!("{} في {}", count_text(total, &CREDIT_HOUR), count(courses, &COURSE))
}
pub fn hours_done(h: &str) -> String {
    format!("{} خلصت امتحانها", count_text(h, &HOUR))
}
/// "7 ساعات بتذاكرها الترم ده".
pub fn hours_now(h: &str) -> String {
    format!("{} بتذاكرها الترم ده", count_text(h, &HOUR))
}
pub fn tile_line(short: &str, hours: &str) -> String {
    format!("{short} · {hours} س")
}
pub fn semester(n: u32) -> String {
    format!("ترم {n}")
}
pub fn content_line(written: usize, lectures: usize, items: usize, lessons: usize) -> String {
    format!("{} · {} · {}", ready_lectures(written, lectures), count(items, &CARD), count(lessons, &LESSON))
}
/// "مكتوب 3 من 12 محاضرة".
pub fn ready_lectures(written: usize, total: usize) -> String {
    format!("مكتوب {written} من {}", count(total, &LECTURE))
}
