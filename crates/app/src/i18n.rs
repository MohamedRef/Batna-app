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

pub mod stats {
    pub const TAB_PROGRESS: &str = "تقدّمي";
    pub const TAB_MISTAKES: &str = "الغلطات";
    pub const TAB_PLAN: &str = "خطة الامتحان";
    pub const TABS_LABEL: &str = "أقسام التقدّم";

    pub const SUMMARY: &str = "لحد النهارده";
    pub const SEEN: &str = "شفتها";
    pub const SEEN_TWICE: &str = "شفتها مرتين";
    pub const APPROVED: &str = "وافقت عليها";
    pub const OF: &str = "من";
    pub const NOTHING_YET: &str = "لسه ما ذاكرتش حاجة هنا. أول جلسة هتملا الأرقام دي.";

    pub const COL_SEEN: &str = "اتشافت";
    pub const COL_RETENTION: &str = "بتفتكر";
    pub const COL_MCQ: &str = "MCQ صح";
    pub const COL_APPROVED: &str = "متراجعة";
    pub const NO_DATA: &str = "—";
    pub const TOPICS: &str = "المواضيع";
    pub const UNTAGGED: &str = "من غير موضوع";
    pub const NOT_WRITTEN: &str = "المحاضرات اللي لسه بتتكتب هتظهر هنا أول ما تنزل.";
    pub const RETENTION_HINT: &str =
        "«بتفتكر» = نسبة الأسئلة اللي جاوبتها صح لما رجعتلك بعد يوم أو أكتر، آخر ٣٠ يوم.";

    pub const WEAK_TITLE: &str = "أضعف ٣ مواضيع دلوقتي";
    pub const WEAK_EMPTY: &str = "لسه بدري نحكم. بعد كام مراجعة هنعرف إيه اللي محتاج شغل زيادة.";
    pub const WEAK_START: &str = "ذاكر دول";

    pub const MISTAKES_TITLE: &str = "اللي غلطت فيه آخر ١٤ يوم";
    pub const MISTAKES_NOTE: &str = "الغلطة بتختفي من هنا لما تجاوبها صح مرتين.";
    pub const MISTAKES_EMPTY: &str = "مفيش غلطات مفتوحة. كمّل كده.";
    pub const MISTAKES_START: &str = "جلسة من الغلطات";
    pub const MISSED: &str = "غلطت";
    pub const ONCE: &str = "مرة";
    pub const TWICE: &str = "مرتين";
    pub const TIMES: &str = "مرات";
    pub const TIMES_MANY: &str = "مرة";
    pub const LECTURE: &str = "محاضرة";

    pub const SESSION_SOON: &str = "الأسئلة جاهزة. الزرار هيشتغل أول ما شاشة المذاكرة تنزل.";
    pub const SESSION_COUNT: &str = "سؤال في الجلسة";
    pub const DEMO: &str = "أرقام تجريبية للعرض بس، مش مذاكرتك.";
}

pub mod plan {
    pub const SETUP_TITLE: &str = "امتحانك إمتى؟";
    pub const SETUP_BODY: &str =
        "قولّي يوم الامتحان والأيام اللي تقدر تذاكر فيها، وأنا أوزّع المحاضرات عليها بهدوء.";
    pub const EXAM_DAY: &str = "يوم الامتحان";
    pub const DAYS_LABEL: &str = "أيام المذاكرة في الأسبوع";
    pub const CAP_LABEL: &str = "أقصى عدد مراجعات في اليوم";
    pub const SAVE: &str = "اعمل الخطة";
    pub const EDIT: &str = "عدّل الخطة";
    pub const CANCEL: &str = "إلغاء";
    pub const PICK_DAY: &str = "اختار يوم بعد النهارده.";
    pub const PICK_WEEKDAY: &str = "اختار يوم واحد على الأقل.";
    pub const NOT_SAVED: &str = "لحد ما التخزين ينزل، الخطة بتتحفظ طول ما التطبيق مفتوح بس.";
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
    pub const DAYS_LEFT: &str = "يوم مذاكرة فاضل";
    pub const FINAL_FROM: &str = "آخر أيام قبل الامتحان (امتحانات تجريبية ومواضيع ضعيفة، من غير جديد) تبدأ";

    pub const TODAY: &str = "النهارده";
    pub const NEW_ITEMS: &str = "جديد";
    pub const REVIEWS: &str = "مراجعة تقريبًا";
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
    pub const TIGHT: &str = "عشان الحد اللي حاطه للمراجعات، فيه أسئلة مش هتلحق قبل آخر ١٠ أيام:";
    pub const TIGHT_ADD: &str = "لو زوّدت أيام مذاكرة في الأسبوع هتلحق كلها. عدد الأيام:";
    pub const TIGHT_CAP: &str =
        "زيادة الأيام لوحدها مش كفاية. ممكن تعلّي حد المراجعات اليومي، أو تركّز على high-yield الأول.";
    pub const FINAL_WINDOW: &str = "إنت في آخر الأيام: امتحانات تجريبية ومراجعة المواضيع الضعيفة. مفيش جديد.";
    pub const EXAM_DONE: &str = "الامتحان عدّى. ربنا يوفقك! لو عندك امتحان تاني، حط ميعاده.";

    pub const PROGRESS: &str = "وصلت لفين";
    pub const FIRST_PASS: &str = "اللفّة الأولى";
    pub const SECOND_PASS: &str = "اللفّة التانية";
    pub const WEEK: &str = "الأيام الجاية";
    pub const ALL_DAYS: &str = "الخطة كلها لحد الامتحان";
}
