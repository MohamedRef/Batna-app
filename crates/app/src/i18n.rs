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

pub mod bank {
    pub const TITLE: &str = "بنك الأسئلة";
    pub const LEAD: &str = "اختار المحاضرات والمواضيع، وجاوب بالشرح بعد كل سؤال أو بالوقت زي الامتحان.";
    pub const EMPTY_TITLE: &str = "لسه مفيش أسئلة اختيار من متعدد";
    pub const EMPTY_BODY: &str = "أول ما محاضرة فيها أسئلة MCQ تخلص، هتلاقيها هنا على طول.";
    pub const LECTURES: &str = "المحاضرات";
    pub const TOPICS: &str = "المواضيع";
    pub const PICK_LECTURE_FOR_TOPICS: &str = "اختار محاضرة أو أكتر عشان تظهر مواضيعها.";
    pub const TAGS: &str = "نوع السؤال";
    pub const TAG_HIGH_YIELD: &str = "مهم جدًا";
    pub const TAG_BOOK: &str = "أسئلة الكتاب";
    pub const TAG_NUMBERS: &str = "أرقام";
    pub const ONLY_WRONG: &str = "الغلط واللي ما اتشافش بس";
    pub const ALL: &str = "الكل";
    pub const COUNT: &str = "عدد الأسئلة";
    pub const AVAILABLE: &str = "سؤال متاح";
    pub const NONE_MATCH: &str = "مفيش أسئلة بالاختيارات دي. جرّب توسّع الفلتر.";
    pub const START_TUTOR: &str = "ابدأ بالشرح";
    pub const START_TIMED: &str = "ابدأ بالوقت";
    pub const TUTOR_HINT: &str = "الشرح وصفحة الكتاب بعد كل سؤال";
    pub const TIMED_HINT: &str = "دقيقة وخُمس للسؤال، والمراجعة في الآخر";
    pub const MOCK_TITLE: &str = "امتحان تجريبي";
    pub const MOCK_LEAD: &str =
        "أسئلة متلخبطة من كل المحاضرات اللي خلصت، بالوقت، ودرجتك لكل محاضرة ولكل موضوع.";
    pub const START_MOCK: &str = "ابدأ الامتحان";
    pub const MINUTES: &str = "دقيقة";
    pub const HISTORY: &str = "امتحاناتك اللي فاتت";
    pub const HISTORY_NOTE: &str = "السجل ده محفوظ لحد ما تقفل التطبيق. الحفظ الدائم جاي مع شاشة المراجعة.";
    pub const MODE_TIMED: &str = "بالوقت";
    pub const MODE_MOCK: &str = "تجريبي";

    pub const QUESTION: &str = "سؤال";
    pub const OF: &str = "من";
    pub const NEXT: &str = "التالي";
    pub const PREV: &str = "السابق";
    pub const FLAG: &str = "علّم للمراجعة";
    pub const FLAGGED: &str = "متعلّم";
    pub const SUBMIT: &str = "سلّم";
    pub const FINISH: &str = "خلّص";
    pub const LEAVE: &str = "اخرج";
    pub const LEFT_ABOUT: &str = "فاضل حوالي";
    pub const UNANSWERED: &str = "من غير إجابة";
    pub const BOOK_PAGE: &str = "Book p.";
    pub const UPDATE: &str = "تحديث من الإرشادات الحالية";
    pub const UPDATE_NOTE: &str = "إجابة الامتحان هي إجابة الكتاب.";
    pub const RIGHT: &str = "صح";
    pub const YOUR_ANSWER: &str = "إجابتك";
    pub const NOT_FOUND: &str = "السؤال ده اتشال من المحتوى.";

    pub const RESULT: &str = "النتيجة";
    pub const TOOK: &str = "الوقت";
    pub const BY_LECTURE: &str = "حسب المحاضرة";
    pub const BY_TOPIC: &str = "حسب الموضوع";
    pub const WEAK: &str = "ابدأ مراجعتك بدول";
    pub const ALL_STRONG: &str = "مفيش موضوع تحت ٦٠٪. شغل حلو.";
    pub const REVIEW: &str = "راجع الأسئلة";
    pub const SHOW_ALL: &str = "الكل";
    pub const SHOW_WRONG: &str = "الغلط";
    pub const SHOW_FLAGGED: &str = "المتعلّم عليها";
    pub const RETRY_WRONG: &str = "جاوب الغلط تاني بالشرح";
    pub const BACK: &str = "رجوع للبنك";
    pub const SENT_TO_REVIEW: &str = "الأسئلة اللي غلطت فيها هترجعلك في المراجعة قريب.";
}
