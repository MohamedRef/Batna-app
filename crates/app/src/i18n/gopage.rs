//! Go to a book page (`crate::gopage`). Adapted from Steps-app main:
//! crates/app/src/i18n/gopage.rs.
pub const TITLE: &str = "روح لصفحة في الكتاب";
pub const SLIDE_TITLE: &str = "روح لسلايد في المحاضرة";
pub const TOOL: &str = "روح لصفحة";
pub const BOOKS: &str = "كتاب أنهي مادة";
pub const LECTURE: &str = "المحاضرة";
pub const HINT: &str = "اختار المادة، واكتب رقم الصفحة المطبوع في كتابها (أو مدى زي 30–35).";
pub const SLIDE_HINT: &str = "اختار المحاضرة، واكتب رقم السلايد.";
pub const PLACEHOLDER: &str = "مثلاً 30";
pub const GO: &str = "افتح";
pub const INVALID: &str = "اكتب رقم صفحة بس.";
pub fn outside(page: u32, slides: bool) -> String {
    if slides {
        format!("سلايد {page} مش موجودة في المحاضرة دي.")
    } else {
        format!("صفحة {page} مش جوه أي محاضرة من كتاب المادة دي في باطنة.")
    }
}
pub fn nearest(page: u32) -> String {
    format!("افتح أقرب صفحة: {page}")
}
