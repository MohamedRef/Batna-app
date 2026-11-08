pub const TITLE: &str = "الإعدادات";
pub const CLOSE: &str = "اقفل";
pub const BACK: &str = "رجوع";
pub const SECTIONS: &str = "أقسام الإعدادات";
pub const SEARCH: &str = "دوّر في الإعدادات";
pub const NO_RESULTS: &str = "مفيش قسم مناسب. جرّب كلمة تانية زي الخط أو التذكيرات.";
pub const CLEAR_SEARCH: &str = "امسح البحث";
pub const GROUP_STUDY: &str = "المذاكرة والتذكير";
pub const GROUP_DEVICE: &str = "الشكل والجهاز";
pub const GROUP_DATA: &str = "الحساب والبيانات";
pub const THIS_DEVICE: &str = "للجهاز ده بس";

pub const LOOK: &str = "الشكل";
pub const LOOK_SUB: &str = "الألوان، نمط الواجهة وحجمها";
pub const DATA: &str = "بياناتك";
pub const DATA_SUB: &str = "فين بتتحفظ، والتصدير";
pub const ABOUT: &str = "عن باطنة";
pub const ABOUT_SUB: &str = "النسخة والمحتوى";

pub const THEME: &str = "الألوان";
pub const THEME_SYSTEM: &str = "زي الجهاز";
pub const THEME_SYSTEM_SUB: &str = "فاتح بالنهار، وغامق لما الجهاز يبقى غامق";
pub const THEME_PAPER: &str = "ورق";
pub const THEME_PAPER_SUB: &str = "فاتح وهادي";
pub const THEME_NIGHT: &str = "ليل";
pub const THEME_NIGHT_SUB: &str = "غامق";
pub const THEME_SYSTEM_BLACK: &str = "زي الجهاز، وبالليل أسود";
pub const THEME_SYSTEM_BLACK_SUB: &str = "فاتح بالنهار، وأسود خالص لما الجهاز يبقى غامق";
pub const THEME_BLACK: &str = "أسود AMOLED";
pub const THEME_BLACK_SUB: &str = "أسود خالص، فبكسلات شاشة الأموليد بتطفي: أريح للعين بالليل وبتوفّر بطارية";
pub const THEME_SHIFT: &str = "نبطشية";
pub const THEME_SHIFT_SUB: &str = "غامق ودافي من غير ضوء أزرق، للمذاكرة متأخر قبل النوم";
pub const THEME_STUDY: &str = "مذاكرة";
pub const THEME_STUDY_SUB: &str = "فاتح دافي هادي من غير أبيض ساطع، للقراية الطويلة في النور";
pub const THEME_STUDY_DIM: &str = "مذاكرة خافت";
pub const THEME_STUDY_DIM_SUB: &str = "رمادي دافي وخط مش أبيض ساطع، للمذاكرة في أوضة ضلمة";
pub const THEME_SYSTEM_STUDY: &str = "مذاكرة زي الجهاز";
pub const THEME_SYSTEM_STUDY_SUB: &str = "مذاكرة بالنهار، ومذاكرة خافت لما الجهاز يبقى غامق";

pub const APPEARANCE: &str = "نمط الواجهة";
pub const APPEARANCE_CALM: &str = "هادئ";
pub const APPEARANCE_CALM_SUB: &str = "أسطح سادة وهادية، من غير لمعة";
pub const APPEARANCE_GLOSSY: &str = "Glossy · Material You";
pub const APPEARANCE_GLOSSY_SUB: &str =
    "أسطح ملونة فيها لمعة خفيفة وأزرار مدوّرة. ألوانها من ألوان باطنة، مش من خلفية الموبايل";
pub const APPEARANCE_LIQUID_GLASS: &str = "Liquid Glass · iOS";
pub const APPEARANCE_LIQUID_GLASS_SUB: &str =
    "خلفية وكروت زجاجية ناعمة وشريط تنقّل مطفي، مع الحفاظ على وضوح أسئلة المذاكرة";

pub const TEXT_SIZE: &str = "حجم الواجهة";
pub const TEXT_SMALLER: &str = "أصغر";
pub const TEXT_SMALL: &str = "صغير";
pub const TEXT_NORMAL: &str = "عادي";
pub const TEXT_LARGE: &str = "كبير";
pub const TEXT_LARGER: &str = "أكبر";
pub const TEXT_SAMPLE_AR: &str = "إيه أول علاج لـ";
pub const TEXT_SAMPLE_TERM: &str = "variceal bleeding";

pub const DATA_FOLDER: &str = "فولدر البيانات";
pub const DATA_NOWHERE: &str = "مش معروف على الجهاز ده";
pub const DATA_IN_BROWSER: &str =
    "في المتصفح ده (IndexedDB). مسح بيانات الموقع بيمسحها، فخد نسخة احتياطية من فوق من وقت للتاني.";
pub const DATA_GROUP_BACKUP: &str = "احفظ ورجّع";
pub const DATA_GROUP_EXPORT: &str = "صدّر لبرّه";
pub const DATA_GROUP_DEVICE: &str = "على الجهاز ده";

pub const VERSION: &str = "النسخة";
pub const CONTENT: &str = "المحتوى";
pub const LECTURES_WRITTEN: &str = "المحاضرات المكتوبة:";
pub const SOURCE: &str = "المصدر";
pub const BOOK: &str = "Internal Medicine Postgraduate Lectures, Alexandria Faculty of Medicine (2025), parts 1–5; Basics of Research lecture slides";

pub const DIAG: &str = "اللي حصل من مشاكل";
pub const DIAG_EMPTY: &str = "مفيش مشاكل لحد دلوقتي.";
pub fn diag_lines(n: usize) -> String {
    super::count::count(n, &super::count::LINE)
}
pub const DIAG_REFRESH: &str = "حدّث";
pub const DIAG_COPY: &str = "انسخ";
pub const DIAG_COPIED: &str = "اتنسخ";
