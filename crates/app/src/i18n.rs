//! UI text, in Egyptian Arabic. One module per feature: a feature chunk adds
//! its own `pub mod <feature>` block and touches nothing else here.
//! Medical terms stay in English (wrap them in `bdi` in the markup).

pub mod app {
    pub const NAME: &str = "باطنة";
    pub const TAGLINE: &str = "مذاكرة الباطنة، جلسة قصيرة كل مرة";
}

pub mod nav {
    pub const TODAY: &str = "النهارده";
    pub const STUDY: &str = "مذاكرة";
    pub const BANK: &str = "الأسئلة";
    pub const CASES: &str = "حالات";
    pub const ORAL: &str = "شفوي";
    pub const STATS: &str = "تقدّمي";
    pub const SETTINGS: &str = "الإعدادات";
}

pub mod today {
    pub const LIBRARY: &str = "محاضرات الفصل الأول";
    pub const CHAPTER: &str = "Hepato-Biliary Pancreatic Unit";
    pub const READY: &str = "جاهزة";
    pub const WRITING: &str = "بتتكتب";
    pub const ITEMS: &str = "سؤال وكارت";
    pub const PAGES: &str = "صفحات";
    pub const UNVERIFIED_NOTE: &str = "كل المحتوى لسه ما اتراجعش. أول ما تشوف أي سؤال، علّم عليه صح أو غلط.";
}

pub mod soon {
    pub const TITLE: &str = "الشاشة دي بتتبني دلوقتي";
    pub const BODY: &str = "هتظهر هنا أول ما تخلص. لحد ما ده يحصل، تقدر تذاكر نفس المحتوى من Anki.";
}

pub mod anki {
    pub const TITLE: &str = "ذاكر في Anki";
    pub const BODY: &str =
        "زرار واحد بيعمل ملف Anki فيه كل المحاضرات، كل محاضرة في ديك لوحدها، ويفتحه في AnkiDroid علطول.";
    pub const LECTURES: &str = "محاضرة";
    pub const ITEMS: &str = "سؤال وكارت";
    pub const BUTTON: &str = "صدّر لـ Anki";
    pub const WORKING: &str = "بجهّز الملف…";
    pub const DONE: &str = "الملف جاهز";
    pub const NOTES: &str = "كارت";
    pub const SAVED_IN: &str = "اتحفظ في";
    pub const REIMPORT: &str = "لو استوردته قبل كده، الكروت هتتحدّث مكانها ومراجعاتك هتفضل زي ما هي.";
    pub const FAILED: &str = "معرفتش أكمّل التصدير:";
}
