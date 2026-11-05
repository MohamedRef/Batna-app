pub const START_HINT: &str = "ابدأ جلسة النهارده";
pub const PROGRESS_HINT: &str = "افتح التقدّم بالتفصيل";
pub const DAY_HINT: &str = "افتح محاضرة اليوم ده";
pub const SETUP_TITLE: &str = "امتحانك إمتى؟";
pub const EXAM_DAY: &str = "يوم الامتحان";
pub const DAYS_LABEL: &str = "أيام المذاكرة في الأسبوع";
pub const CAP_LABEL: &str = "أقصى عدد مراجعات في اليوم";
pub const SAVE: &str = "اعمل الخطة";
pub const EDIT: &str = "عدّل الخطة";
pub const CANCEL: &str = "إلغاء";
pub const PICK_DAY: &str = "اختار يوم بعد النهارده.";
pub const PICK_WEEKDAY: &str = "اختار يوم واحد على الأقل.";
pub const WEEKDAYS: [&str; 7] = ["السبت", "الحد", "الاتنين", "التلات", "الأربع", "الخميس", "الجمعة"];
pub const WEEKDAYS_SHORT: [&str; 7] = ["سبت", "حد", "اتنين", "تلات", "أربع", "خميس", "جمعة"];
pub const MONTHS: [&str; 12] = [
    "يناير",
    "فبراير",
    "مارس",
    "أبريل",
    "مايو",
    "يونيو",
    "يوليو",
    "أغسطس",
    "سبتمبر",
    "أكتوبر",
    "نوفمبر",
    "ديسمبر",
];

pub const EXAM_ON: &str = "الامتحان";
/// "فاضل 40 يوم مذاكرة".
pub fn days_left(n: usize) -> String {
    format!("فاضل {}", super::count::count(n, &super::count::STUDY_DAY))
}
pub const FINAL_FROM: &str = "آخر أيام قبل الامتحان (امتحانات تجريبية ومواضيع ضعيفة، من غير جديد) تبدأ";

pub const TODAY: &str = "النهارده";
pub const NEW_ITEMS: &str = "جديد";
pub const REVIEWS: &str = "مراجعة تقريبًا";
pub const APPROX: &str = "تقريبًا";
pub const FROM: &str = "من";
pub const REVISIT: &str = "لفّة تانية على";
pub const REST_TODAY: &str = "النهارده راحة. المذاكرة الجاية:";

pub const FOCUS_LEARN: &str = "جديد ومراجعة";
pub const FOCUS_REVISIT: &str = "لفّة تانية";
pub const FOCUS_MOCK: &str = "امتحان تجريبي";
pub const FOCUS_WEAK: &str = "المواضيع الضعيفة";
pub const FOCUS_REST: &str = "راحة";

pub const ON_TRACK: &str = "الخطة مريحة: كل المحاضرات هتخلص بدري وتلحق تلفّ عليها تاني.";
pub const SNUG: &str = "الخطة ماشية، بس الجديد هياخد جزء من وقت اللفّة التانية.";
pub const TIGHT: &str = "على الخطة دي، فيه أسئلة مش هتلحق قبل آخر 10 أيام:";
pub const TIGHT_ADD: &str = "لو زوّدت أيام مذاكرة في الأسبوع هتلحق كلها. عدد الأيام:";
pub fn tight_cap(cap: u32) -> String {
    format!("لو علّيت حد المراجعات اليومي لـ {cap} هتلحق كلها.")
}
pub const TIGHT_HIGH_YIELD: &str = "حتى بأيام أكتر وحد مراجعات أعلى مش هتلحق كلها، فركّز على high-yield الأول.";
pub const TIGHT_FINAL: &str =
    "إنت في آخر الأيام، فالوقت للامتحانات التجريبية والمواضيع الضعيفة. في الجديد، ركّز على high-yield بس.";
pub const PICK_DAY_FAR: &str = "اختار يوم في خلال 3 سنين من النهارده.";
pub const FINAL_WINDOW: &str = "إنت في آخر الأيام: امتحانات تجريبية ومراجعة المواضيع الضعيفة. مفيش جديد.";
pub const EXAM_DONE: &str = "الامتحان عدّى. ربنا يوفقك! لو عندك امتحان تاني، حط ميعاده.";

pub const PROGRESS: &str = "وصلت لفين";
pub const FIRST_PASS: &str = "اللفّة الأولى";
pub const SECOND_PASS: &str = "اللفّة التانية";
pub const WEEK: &str = "الأيام الجاية";
pub const LOAD_TITLE: &str = "الحِمل في الأسبوعين الجايين";
pub const LOAD_LABEL: &str = "مراجعات وجديد متوقعين كل يوم، النهارده على اليمين";
pub const LOAD_PEAK: &str = "أتقل يوم";
pub const LOAD_KEY_REVIEWS: &str = "مراجعة";
pub const LOAD_KEY_NEW: &str = "جديد";
pub const LOAD_KEY_CAP: &str = "الحد اليومي";
pub const PACE_TITLE: &str = "السرعة لحد الامتحان";
pub const PACE_LABEL: &str = "اللي شفته لحد النهارده، والخطة لحد الامتحان";
pub const PACE_KEY_PAST: &str = "اللي فات";
pub const PACE_KEY_PLAN: &str = "الخطة";
pub const PACE_KEY_FINAL: &str = "آخر 10 أيام";
pub const PACE_DONE_ON: &str = "على الخطة دي، هتكون شفت كل الأسئلة يوم";
pub const PACE_NOT_DONE: &str = "على الخطة دي، فيه أسئلة مش هتلحق تتشاف قبل آخر الأيام.";
pub const ALL_DAYS: &str = "الخطة كلها لحد الامتحان";
