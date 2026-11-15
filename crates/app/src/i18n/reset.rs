//! «ابدأ من الأول»: restarting a course or the whole curriculum (#1341).

use super::count::{CARD, LESSON, PAPER, count};

pub const TITLE: &str = "ابدأ من الأول";
pub const INTRO: &str =
    "لو عايز تلف لفة تانية قبل الامتحان: الكروت والدروس ترجع جديدة، وكل حاجة كتبتها تفضل زي ما هي.";
pub const PICK: &str = "تبدأ إيه من الأول؟";
pub const ALL: &str = "المنهج كله";
pub const COURSE_BUTTON: &str = "ابدأ المادة دي من الأول";
/// "في الكبد" / "في المنهج كله".
pub fn in_scope(name: &str) -> String {
    format!("في {name}")
}
pub const RESTARTS: &str = "هيرجع جديد:";
/// "412 كارت ذاكرتهم و37 درس خلصتهم".
pub fn restarts_line(cards: usize, lessons: usize) -> String {
    match (cards, lessons) {
        (0, 0) => "مفيش تقدّم لسه هنا، فمفيش حاجة هتتغيّر.".to_string(),
        (c, 0) => format!("{} ذاكرتهم", count(c, &CARD)),
        (0, l) => format!("{} خلصتهم", count(l, &LESSON)),
        (c, l) => format!("{} ذاكرتهم و{} خلصتهم", count(c, &CARD), count(l, &LESSON)),
    }
}
pub const STAYS: &str = "هيفضل زي ما هو:";
pub const STAYS_LINE: &str = "ملاحظاتك، الهايلايت، كروتك، علاماتك على المحتوى، سجل مذاكرتك، والإعدادات.";
pub const STAYS_OWN: &str = "جدول كروتك انت بيبدأ من الأول هو كمان، والكروت نفسها بتفضل.";
/// "والامتحانات اللي خلصتها كمان (3 امتحانات)".
pub fn papers_too(n: usize) -> String {
    format!("والامتحانات اللي خلصتها كمان ({})", count(n, &PAPER))
}
pub const BACKUP_FIRST: &str = "هحفظ نسخة احتياطية من كل حاجة الأول، ولو ما عرفتش مش هبدأ.";
pub const UNDO_HINT: &str = "وتقدر ترجّع كل حاجة زي ما كانت من هنا في أي وقت.";
pub const GO: &str = "ابدأ من الأول";
/// The second tap for one course: "أيوه، ابدأ الكبد من الأول".
pub fn confirm(name: &str) -> String {
    format!("أيوه، ابدأ {name} من الأول")
}
pub const CANCEL: &str = "لأ، سيبها";
pub const TYPE_TO_CONFIRM: &str = "للمنهج كله، اكتب «ابدأ» علشان نتأكد:";
pub const TYPE_WORD: &str = "ابدأ";
pub const BLOCKED_SESSION: &str = "فيه مذاكرة شغالة دلوقتي. خلّصها أو اقفلها الأول، وبعدين ابدأ من الأول.";
pub const BLOCKED_PAPER: &str = "فيه امتحان مفتوح دلوقتي. سلّمه أو اقفله الأول، وبعدين ابدأ من الأول.";
pub const BACKUP_FAILED: &str = "ما عرفتش أحفظ النسخة الاحتياطية، فما بدأتش حاجة:";
pub const FAILED: &str = "ما عرفتش أبدأ من الأول، ومفيش حاجة اتغيّرت:";
/// "تمام، الكبد بدأ من الأول."
pub fn done(name: &str) -> String {
    format!("تمام، {name} بدأ من الأول.")
}
pub const BACKUP_AT: &str = "النسخة الاحتياطية اتحفظت في:";
pub const UNDO: &str = "رجّع زي ما كان";
pub const UNDONE: &str = "رجع كل حاجة زي ما كانت.";
pub const IN_FORCE: &str = "اللي بدأته من الأول";
/// "رجّع تقدّم الكبد".
pub fn undo_scope(name: &str) -> String {
    format!("رجّع تقدّم {name}")
}
