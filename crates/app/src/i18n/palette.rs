//! Quick navigation (Cmd/Ctrl+K on a keyboard, the search button on a phone).

pub const OPEN: &str = "بحث سريع";
pub const TITLE: &str = "الانتقال السريع";
pub const CLOSE: &str = "إغلاق البحث السريع";
pub const FIELD: &str = "ابحث عن قسم أو محاضرة أو درس أو حاسبة أو إعدادات أو أسئلة";
pub const PLACEHOLDER: &str = "قسم، درس، حاسبة، إعدادات، أو questions واسم المحاضرة…";
pub const RESULTS: &str = "نتائج البحث السريع";
pub const EMPTY: &str = "مفيش نتائج مطابقة.";
pub const HINT: &str = "↑ ↓ للتنقل · Enter للفتح · Esc للإغلاق";

// The small line under each result: what kind of place it opens.
pub const KIND_SCREEN: &str = "قسم";
pub const KIND_SETTINGS: &str = "إعدادات التطبيق";
pub const KIND_SETTINGS_SECTION: &str = "قسم من الإعدادات";

pub fn lecture(context: &str) -> String {
    format!("محاضرة · {context}")
}

pub fn lesson(context: &str) -> String {
    format!("درس · {context}")
}

pub fn calculator(area: &str) -> String {
    format!("حاسبة · {area}")
}

pub fn questions(title: &str) -> String {
    format!("أسئلة · {title}")
}

pub fn practice(context: &str) -> String {
    format!("تدريب المحاضرة · {context}")
}

/// "ص 30 · كبد": a book page in the quick search.
pub fn page(cite: &str, course: &str) -> String {
    format!("{cite} · {course}")
}
