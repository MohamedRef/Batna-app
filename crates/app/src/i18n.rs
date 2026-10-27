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

pub mod study {
    pub const TITLE: &str = "جلسة مذاكرة";
    pub const SETUP_INTRO: &str =
        "اختار المحاضرات والوقت. هنبدأ بالمراجعات اللي عليها الدور، وبعدها الجديد موضوع موضوع.";
    pub const EMPTY_LIBRARY: &str = "لسه مفيش محاضرة جاهزة للمذاكرة هنا. أول ما محاضرة تتكتب هتلاقيها في القايمة، ولحد ده ما يحصل تقدر تذاكر من Anki.";
    pub const LECTURES: &str = "المحاضرات";
    pub const TOPICS: &str = "المواضيع";
    pub const TOPICS_HINT: &str = "سيبها فاضية عشان تذاكر المحاضرة كلها.";
    pub const TIME: &str = "الوقت";
    pub const MINUTES: &str = "دقيقة";
    pub const HOURS: &str = "ساعة";
    pub const DAYS: &str = "أيام";
    pub const ITEMS: &str = "سؤال";
    pub const REVIEWS: &str = "مراجعة";
    pub const NEW: &str = "جديد";
    pub const ABOUT: &str = "حوالي";
    pub const START: &str = "يلا نبدأ";
    pub const NOTHING_NOW: &str =
        "مفيش حاجة عليها الدور في اللي اخترته دلوقتي. جرّب محاضرة تانية أو ارجع بعدين.";
    pub const END: &str = "إنهاء الجلسة";
    pub const OF: &str = "من";

    pub const BADGE_NEW: &str = "جديد";
    pub const BADGE_REVIEW: &str = "مراجعة";
    pub const BADGE_RETRY: &str = "تاني مرة";
    pub const UNVERIFIED: &str = "لسه ما اتراجعش";
    pub const UNVERIFIED_HINT: &str = "اتكتب من الكتاب ولسه إنت ما أكدتوش.";
    pub const KIND_CARD: &str = "كارت";
    pub const KIND_CLOZE: &str = "كمّل الفراغ";
    pub const KIND_ENUMERATE: &str = "عدّد";
    pub const KIND_ESSAY: &str = "مقالي";
    pub const KIND_MCQ: &str = "اختيار";
    pub const KIND_CASE: &str = "حالة";
    pub const KIND_TABLE: &str = "جدول";
    pub const KIND_ORDER: &str = "رتّب";
    pub const KIND_CALC: &str = "حساب";
    pub const BOOK_PAGE: &str = "الكتاب ص";
    pub const BOOK_QUOTE: &str = "من الكتاب";
    pub const UPDATE_TITLE: &str = "تحديث عن الكتاب";
    pub const UPDATE_AFTER: &str = "افتحه بعد ما تجاوب";
    pub const UPDATE_BOOK_WINS: &str = "في الامتحان، إجابة الكتاب هي اللي بتتحسب.";
    pub const NOT_HERE_YET: &str = "النوع ده من الأسئلة ليه شاشة لسه بتتبني.";
    pub const SKIP: &str = "عدّي";

    pub const THINK_FIRST: &str = "جاوب في دماغك الأول، وبعدين اكشف.";
    pub const REVEAL: &str = "اكشف الإجابة";
    pub const BLANK: &str = "الفراغ";
    pub const CHECK: &str = "صحّح";
    pub const REVEAL_BLANKS: &str = "مش فاكر، وريني";
    pub const CLOZE_TIP: &str = "الأرقام بتتصحح من غير الوحدات والمسافات.";
    pub const CLOZE_RIGHT: &str = "مظبوط.";
    pub const CLOZE_CLOSE: &str = "قريب جدًا. بص على الإجابة كاملة.";
    pub const CLOZE_WRONG: &str = "الإجابة الصح مكتوبة فوق. هترجعلك تاني قريب.";

    pub const ENUM_GOAL_A: &str = "فيه";
    pub const ENUM_GOAL_B: &str = "نقط، ولو افتكرت";
    pub const ENUM_GOAL_C: &str = "منهم تعدّي.";
    pub const ENUM_PLACEHOLDER: &str = "اكتب اللي فاكره، أو قوله لنفسك بصوت عالي (اختياري)";
    pub const SHOW_POINTS: &str = "خلصت، وريني النقط";
    pub const TICK_WHAT_YOU_GOT: &str = "علّم على اللي افتكرته.";
    pub const PRE_TICKED: &str = "علّمت لك على اللي لقيته في كلامك، وغيّر براحتك.";
    pub const ENUM_PASS: &str = "عدّيت.";
    pub const ENUM_SHORT_A: &str = "فاضلك";
    pub const ENUM_SHORT_B: &str = "عشان تعدّي.";
    pub const WHAT_I_WROTE: &str = "اللي كتبته";

    pub const ESSAY_HOW: &str = "اكتب العناوين بس، كل عنوان في سطر، زي ما هتكتبهم في الامتحان.";
    pub const ESSAY_PLACEHOLDER: &str = "Definition\nPathogenesis\nClinical picture\n…";
    pub const COMPARE: &str = "قارن بالإجابة النموذجية";
    pub const MODEL_OUTLINE: &str = "الإجابة النموذجية";
    pub const TICK_HEADINGS: &str = "علّم على العناوين اللي جت في بالك.";
    pub const HEADINGS_GOT: &str = "العناوين اللي جبتها:";

    pub const GRADE_LABEL: &str = "قيّم نفسك";
    pub const SUGGESTED: &str = "اقتراحي متعلّم عليه، والقرار ليك.";
    pub const AGAIN: &str = "تاني";
    pub const HARD: &str = "صعب";
    pub const GOOD: &str = "تمام";
    pub const EASY: &str = "سهل";
    pub const UNIT_MIN: &str = "د";
    pub const UNIT_HOUR: &str = "س";
    pub const UNIT_DAY: &str = "ي";
    pub const UNIT_MONTH: &str = "ش";

    pub const SUMMARY_TITLE: &str = "خلصت الجلسة، برافو عليك";
    pub const SUMMARY_EMPTY: &str = "الجلسة خلصت";
    pub const SUM_ITEMS: &str = "سؤال";
    pub const SUM_RIGHT: &str = "صح من أول مرة";
    pub const SUM_NEW: &str = "جديد";
    pub const SUM_ALL_SOLID: &str = "كله كان ثابت. المراجعة الجاية هتيجي بعد فترة أطول.";
    pub const SUM_BACK_SOON: &str = "هيرجعولك قريب";
    pub const SUM_BACK_SOON_HINT: &str = "دول عادي يتلخبطوا أول مرة. هيتكرروا لحد ما يثبتوا.";
    pub const NEXT_SESSION: &str = "الجلسة الجاية:";
    pub const NEXT_TOMORROW: &str = "ارجع بكرة لجلسة قصيرة، وهتلاقي المراجعات مستنياك.";
    pub const ANOTHER: &str = "جلسة كمان";
    pub const NOW: &str = "دلوقتي";
    pub const AFTER: &str = "بعد";
    pub const TOMORROW: &str = "بكرة";
}

pub mod soon {
    pub const TITLE: &str = "الشاشة دي بتتبني دلوقتي";
    pub const BODY: &str = "هتظهر هنا أول ما تخلص. لحد ما ده يحصل، تقدر تذاكر نفس المحتوى من Anki.";
}
