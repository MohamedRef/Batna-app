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

pub mod desk {
    pub const SECTION: &str = "التابلت والماك";
    pub const SECTION_NOTE: &str = "الإعدادات دي للجهاز ده بس، ومش بتتزامن.";
    pub const NAV_STYLE: &str = "القايمة الجانبية في الشاشة العريضة";
    pub const NAV_LABELS: &str = "أيقونات وأسامي";
    pub const NAV_SLIM: &str = "أيقونات بس";
    // The pane labels are read by desk::Panes / ListDetail, which feature
    // screens opt into one by one.
    #[allow(dead_code)]
    pub const SIDE_WAITING: &str = "الشرح وصفحة الكتاب هيظهروا هنا أول ما تجاوب.";
    #[allow(dead_code)]
    pub const SIDE_LABEL: &str = "الشرح";
    #[allow(dead_code)]
    pub const LIST_LABEL: &str = "القايمة";
}

pub mod hover {
    pub const PEN: &str = "لمحة بقلم S Pen";
    pub const PEN_NOTE: &str = "قرّب القلم من اختصار أو اسم دوا من غير ما تلمس الشاشة، هيظهرلك معناه.";
    pub const MOUSE: &str = "لمحة بالماوس";
    pub const MOUSE_NOTE: &str =
        "سيب الماوس ثانية على الكلمة، أو على رقم صفحة الكتاب، أو على اختيار بعد ما تجاوب.";
    pub const UNVERIFIED: &str = "لسه ما اتراجعش";
    pub const TERMS: &str = "مصطلح في القاموس";
}

pub mod keys {
    pub const TITLE: &str = "اختصارات الكيبورد";
    pub const OPEN: &str = "اعرض الاختصارات";
    pub const CLOSE: &str = "اقفل";
    pub const NOTE: &str = "بتشتغل على كيبورد التابلت والماك. على الماك استخدم ⌘ بدل Ctrl.";
    pub const SCREENS: &str = "تروح لشاشة";
    pub const SETTINGS: &str = "الإعدادات";
    pub const REVEAL: &str = "تكشف الإجابة";
    pub const RATE: &str = "تقيّم نفسك (من نسيت لسهل)";
    pub const OPTIONS: &str = "تختار إجابة في سؤال الاختيارات";
    pub const NEXT: &str = "اللي بعده";
    pub const ESC: &str = "تقفل أي حاجة مفتوحة";
    pub const HELP: &str = "القايمة دي";
}
