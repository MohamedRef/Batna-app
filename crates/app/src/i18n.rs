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
    pub const TITLE: &str = "النهارده";
    pub const DUE: &str = "مستحق مراجعة";
    pub const NEW: &str = "جديد متاح";
    pub const CHECKED: &str = "راجعت صحته";
    pub const DUE_TIME_1: &str = "المستحق ياخد حوالي";
    pub const DUE_TIME_2: &str = "دقيقة. الجلسة بتبدأ بالأضعف.";
    pub const START: &str = "ابدأ جلسة ١٠ دقايق";
    pub const ALL_DONE: &str = "خلّصت المستحق النهارده. شغل حلو.";
    pub const NEW_CAP_REACHED: &str = "وخلّصت الجديد اللي محدّده لليوم";
    pub const TOMORROW: &str = "بكرة:";
    pub const NOTHING_WRITTEN: &str = "المحتوى لسه بيتكتب. أول ما محاضرة تجهز هتلاقيها هنا.";
    pub const STORE_FAILED: &str = "مش قادر أفتح قاعدة البيانات، فالمراجعات مش هتتحفظ بعد ما تقفل التطبيق:";
    pub const IN_PROGRESS: &str = "محاضرات شغّال فيها";
    pub const OF: &str = "من";
    pub const STARTED: &str = "بدأتهم";
    pub const DUE_CHIP: &str = "مستحق";
    pub const LIBRARY: &str = "محاضرات الفصل الأول";
    pub const CHAPTER: &str = "Hepato-Biliary Pancreatic Unit";
    pub const READY: &str = "جاهزة";
    pub const WRITING: &str = "بتتكتب";
    pub const ITEMS: &str = "سؤال وكارت";
    pub const PAGES: &str = "صفحات";
    pub const UNVERIFIED_NOTE: &str = "كل المحتوى لسه ما اتراجعش. أول ما تشوف أي سؤال، علّم عليه صح أو غلط.";
}

pub mod study_settings {
    pub const STORE_MEMORY: &str =
        "مذاكرتك مش بتتحفظ: قاعدة البيانات ما اتفتحتش، فاللي بتذاكره هيروح لما تقفل التطبيق.";
    pub const SECTION: &str = "المراجعة";
    pub const SECTION_SUB: &str = "نسبة التذكّر والجديد كل يوم";
    pub const RETENTION: &str = "نسبة التذكّر المطلوبة";
    pub const RETENTION_BODY: &str =
        "قد إيه عايز تكون فاكر السؤال لما ييجي ميعاده. نسبة أعلى يعني مراجعات أكتر.";
    pub const R85: &str = "أخف";
    pub const R90: &str = "متوازن";
    pub const R95: &str = "قبل الامتحان";
    pub const PERSONAL_1: &str = "الجدول متظبط على مذاكرتك أنت، من";
    pub const PERSONAL_2: &str = "مراجعة. بيتحدّث كل أسبوع.";
    pub const DEFAULT_PARAMS_1: &str = "الجدول هيتظبط على مذاكرتك لما توصل";
    pub const DEFAULT_PARAMS_2: &str = "مراجعة. لحد دلوقتي:";
    pub const NEW_PER_DAY: &str = "أسئلة جديدة في اليوم";
    pub const NEW_PER_DAY_BODY: &str = "الجديد بيزوّد مراجعات الأيام الجاية، فاختار رقم تقدر تكمّل عليه.";
}

pub mod study {
    pub const TITLE: &str = "جلسة مذاكرة";
    pub const SETUP_INTRO: &str =
        "اختار المحاضرات والوقت. هنبدأ بالمراجعات اللي عليها الدور، وبعدها الجديد موضوع موضوع.";
    pub const EMPTY_LIBRARY: &str = "لسه مفيش محاضرة جاهزة للمذاكرة هنا. أول ما محاضرة تتكتب هتلاقيها في القايمة، ولحد ده ما يحصل تقدر تذاكر من Anki.";
    pub const LECTURES: &str = "المحاضرات";
    pub const PICK_ALL: &str = "اختار الكل";
    pub const CLEAR_ALL: &str = "امسح الاختيار";
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
    pub const REPORT: &str = "بلّغ عن مشكلة في السؤال ده";
    pub const CLOSE: &str = "اقفل";
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
    pub const PACE_KEY_FINAL: &str = "آخر ١٠ أيام";
    pub const PACE_DONE_ON: &str = "على الخطة دي، هتكون شفت كل الأسئلة يوم";
    pub const PACE_NOT_DONE: &str = "على الخطة دي، فيه أسئلة مش هتلحق تتشاف قبل آخر الأيام.";
    pub const ALL_DAYS: &str = "الخطة كلها لحد الامتحان";
}

pub mod table {
    pub const HINT_TAP: &str = "دوس على كل ؟ تشوف الإجابة، وبعدين علّم: عرفتها ✓ ولا لأ ✗.";
    pub const HINT_TYPE: &str = "اكتب كل خانة فاضية، وبعدين دوس صحّح.";
    pub const REVEAL_CELL: &str = "اكشف الخانة";
    pub const TYPE_CELL: &str = "اكتب الخانة";
    pub const REVEAL_ALL: &str = "اكشف الكل";
    pub const CHECK_TYPED: &str = "صحّح";
    pub const TYPING_ON: &str = "عايز أكتب";
    pub const TYPING_OFF: &str = "من غير كتابة";
    pub const KNEW_IT: &str = "عرفتها";
    pub const MISSED_IT: &str = "ماعرفتهاش";
    pub const MARK_RIGHT: &str = "اعتبرها صح";
    pub const MARK_WRONG: &str = "اعتبرها غلط";

    pub fn mark_each(left: usize) -> String {
        if left == 1 {
            "فاضل خانة واحدة: عرفتها ولا لأ؟".to_string()
        } else {
            format!("فاضل {left} خانات: علّم على كل واحدة عرفتها ولا لأ.")
        }
    }

    pub fn recalled(right: usize, total: usize) -> String {
        if right == total {
            format!("كل الخانات صح ({total} من {total}). حلو أوي!")
        } else {
            format!("افتكرت {right} من {total} خانات.")
        }
    }
}

pub mod order {
    pub const HINT: &str =
        "دوس على الخطوات بالترتيب. تقدر ترجّع خطوة بالدوس عليها، أو تحركها بالأسهم أو بالسحب.";
    pub const PLACED: &str = "ترتيبك";
    pub const YOUR_ORDER: &str = "ترتيبك";
    pub const POOL: &str = "الخطوات";
    pub const NEXT_SLOT: &str = "الخطوة الجاية هنا";
    pub const CORRECT_ORDER: &str = "الترتيب في الكتاب";
    pub const CHECK: &str = "صحّح";
    pub const CLEAR: &str = "ابدأ من الأول";
    pub const SEND_BACK: &str = "رجّعها للخطوات";
    pub const MOVE_UP: &str = "حركها لفوق";
    pub const MOVE_DOWN: &str = "حركها لتحت";
    pub const IN_PLACE: &str = "في مكانها";
    pub const ALL_IN_PLACE: &str = "كل الخطوات في مكانها. ترتيب ممتاز!";

    pub fn in_place(right: usize, total: usize) -> String {
        format!("{right} من {total} في مكانها. الخطوات المعلّمة بالأحمر محتاجة نظرة تانية.")
    }

    pub fn should_be(position: usize) -> String {
        format!("مكانها {position}")
    }
}

pub mod oral {
    pub const TITLE: &str = "امتحان شفوي";
    pub const INTRO: &str = "التطبيق يسألك بصوت عالي، وانت تجاوب بصوتك. بعد وقت التفكير يقولك الإجابة، وانت تقيّم نفسك. ينفع وانت ماشي.";
    pub const LECTURE: &str = "المحاضرة";
    pub const ALL_LECTURES: &str = "كل المحاضرات الجاهزة";
    pub const SIZE: &str = "عدد الأسئلة";
    pub const THINK: &str = "وقت التفكير";
    pub const SECONDS: &str = "ثانية";
    pub const VOICE: &str = "اقرا الأسئلة بصوت عالي";
    pub const RATE: &str = "سرعة الصوت";
    pub const RATES: [&str; 3] = ["أبطأ", "عادي", "أسرع"];
    pub const AUTO: &str = "كمّل لوحدك بعد الإجابة";
    pub const AUTO_HINT: &str =
        "لو ما ضغطتش على تقييم في ٨ ثواني، يعدّي للسؤال اللي بعده من غير ما يسجّل حاجة.";
    pub const AVAILABLE: &str = "سؤال ينفع يتسأل شفوي";
    pub const START: &str = "ابدأ";
    pub const EMPTY: &str = "لسه مفيش محاضرات جاهزة فيها أسئلة شفوي. أول ما محاضرة تخلص هتلاقيها هنا.";
    pub const NO_VOICE: &str = "مفيش صوت إنجليزي شغّال على الجهاز ده، فالأسئلة هتظهر مكتوبة بس. على أندرويد: الإعدادات ← تحويل النص إلى كلام ← نزّل صوت English.";

    pub const LISTEN: &str = "اسمع السؤال";
    pub const THINKING: &str = "فكّر وجاوب بصوتك";
    pub const WHAT_ELSE: &str = "وإيه كمان؟";
    pub const ANSWER: &str = "الإجابة";
    pub const FOLLOW_UP: &str = "سؤال الممتحن بعد الكارت";
    pub const AGAIN_BADGE: &str = "تاني مرة";
    pub const PASS: &str = "كفاية تذكر";
    pub const OF: &str = "من";
    pub const PAGE: &str = "صفحة";
    pub const LEFT: &str = "فاضل";
    pub const SHOW: &str = "اعرض الإجابة";
    pub const REPEAT: &str = "عيد السؤال";
    pub const AGAIN: &str = "تاني";
    pub const AGAIN_HINT: &str = "هيرجعلك بعد ٣ أسئلة";
    pub const GOOD: &str = "تمام";
    pub const GOOD_HINT: &str = "جاوبت صح";
    pub const END: &str = "إنهاء";

    pub const DONE: &str = "خلصت الجلسة";
    pub const DONE_GOOD: &str = "تمام";
    pub const DONE_AGAIN: &str = "تاني";
    pub const DONE_HEARD: &str = "سمعتها بس";
    pub const MISSED: &str = "الأسئلة اللي محتاجة مراجعة";
    pub const NONE_MISSED: &str = "ولا سؤال وقع منك. شغل حلو!";
    pub const ANOTHER: &str = "جلسة كمان";
    pub const BACK: &str = "رجوع للإعدادات";
}

pub mod case {
    pub const TAB_CASES: &str = "حالات إكلينيكية";
    pub const TAB_CALC_ITEMS: &str = "تمارين بالأرقام";
    pub const TAB_CALCULATORS: &str = "الحاسبات";
    pub const INTRO: &str = "كل حالة بتكشف بيانات جديدة خطوة بخطوة، وفي كل خطوة قرار واحد.";
    pub const EMPTY: &str = "لسه مفيش حالات. هتظهر هنا أول ما محاضراتها تتكتب، ولحد ده الحاسبات شغالة.";
    pub const EMPTY_CALC: &str = "لسه مفيش تمارين بالأرقام. هتظهر هنا مع المحاضرات، ولحد ده جرّب الحاسبات.";
    pub const STEPS: &str = "خطوات";
    pub const STEP: &str = "خطوة";
    pub const OF: &str = "من";
    pub const NEW_DATA: &str = "بيانات جديدة";
    pub const RIGHT: &str = "صح، برافو.";
    pub const WRONG: &str = "مش دي. الإجابة الصح متعلّمة.";
    pub const SHOW_ANSWER: &str = "ورّيني الإجابة";
    pub const GOT_IT: &str = "جاوبتها صح";
    pub const MISSED: &str = "فاتتني";
    pub const NEXT: &str = "الخطوة اللي بعدها";
    pub const FINISH: &str = "خلّص الحالة";
    pub const PAGE: &str = "الكتاب صفحة";
    pub const SCORE: &str = "خطوات صح";
    pub const ALL_RIGHT: &str = "كل الخطوات صح. الحالة دي هترجعلك بعد مدة أطول.";
    pub const SOME_RIGHT: &str = "حلو. بص على الخطوات اللي فاتتك، والحالة هترجعلك قريب في المراجعة.";
    pub const FEW_RIGHT: &str = "ولا يهمك، الحالات دي بتتعلم بالتكرار. هترجعلك بكرة.";
    pub const AGAIN: &str = "عيدها";
    pub const BACK: &str = "رجوع للقائمة";
    pub const UNVERIFIED: &str = "لسه ما اتراجعش";
    pub const UPDATE: &str = "تحديث عن الكتاب";
}

pub mod calc {
    pub const PICK: &str = "اختار حاسبة";
    pub const FILL: &str = "املا البيانات وهتطلع النتيجة هنا.";
    pub const MISSING: &str = "ناقص";
    pub const INVALID: &str = "راجع القيمة دي";
    pub const OPTIONAL: &str = "اختياري";
    pub const YES: &str = "أيوه";
    pub const NO: &str = "لأ";
    pub const RESULT: &str = "النتيجة";
    pub const BREAKDOWN: &str = "التفاصيل";
    pub const SOURCE: &str = "المصدر";
    pub const BOOK_PAGE: &str = "الكتاب صفحة";
    pub const RESET: &str = "امسح";
    pub const BACK: &str = "كل الحاسبات";
    pub const AREA_LIVER: &str = "الكبد";
    pub const AREA_ASCITES: &str = "الاستسقاء";
    pub const AREA_TUMOUR: &str = "الأورام";
    pub const AREA_ACUTE: &str = "الفشل الكبدي الحاد";
    pub const AREA_PANCREAS: &str = "البنكرياس";
    // The `calc` item: compute it, then interpret it.
    pub const YOUR_RESULT: &str = "حسبتها كام؟";
    pub const YOUR_RESULT_HINT: &str = "اكتب الرقم أو المرحلة، أو سيبها فاضية واستخدم الحاسبة.";
    pub const CHECK: &str = "اتأكد";
    pub const USE_CALC: &str = "احسبها بالحاسبة";
    pub const MATCH: &str = "مظبوط.";
    pub const NO_MATCH: &str = "مش مظبوط. شوف الحساب تحت.";
    pub const MEANING: &str = "يعني إيه النتيجة دي؟";
    pub const MEANING_HINT: &str = "قولها لنفسك الأول، وبعدين افتح التفسير.";
    pub const SHOW_MEANING: &str = "ورّيني التفسير";
}

pub mod desk {
    pub const TABLET: &str = "التابلت والقلم";
    pub const TABLET_SUB: &str = "القايمة الجانبية ولمحة القلم";
    pub const MAC: &str = "الماك والكيبورد";
    pub const MAC_SUB: &str = "القايمة الجانبية ولمحة الماوس والاختصارات";
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
    pub const TITLE: &str = "اللمحة";
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

pub mod verify {
    pub const GROUP: &str = "مراجعة المحتوى";
    pub const QUESTION: &str = "المحتوى ده مظبوط؟";
    pub const RIGHT: &str = "✓ صح";
    pub const EDIT: &str = "✎ عدّل";
    pub const WRONG: &str = "✗ غلط";
    pub const UNDO: &str = "رجوع";
    pub const CANCEL: &str = "إلغاء";
    pub const SAVE: &str = "احفظ تعديلي";
    pub const SAVED_APPROVED: &str = "✓ اتسجّل إنه صح، ومش هيسألك عليه تاني.";
    pub const SAVED_EDITED: &str = "✎ اتسجّل تعديلك، وهتذاكره بكلامك من هنا ورايح.";
    pub const SAVED_WRONG: &str = "✗ اتشال من المراجعة لحد ما يتصلّح.";
    pub const WRONG_NOTE: &str = "إيه الغلط؟ (اختياري)";
    pub const WRONG_PLACEHOLDER: &str = "مثلاً: الكتاب صفحة ١٤ بيقول ٧–٩ نقط";
    pub const WRONG_CONFIRM: &str = "شيله من المراجعة";
    pub const EDITOR_HINT: &str = "عدّل اللي محتاج يتعدّل بس. الكتاب هو المرجع.";
    pub const EDIT_NOTE: &str = "ملاحظة للي هيصلّح المحتوى (اختياري)";
    pub const NOTHING_CHANGED: &str = "ما غيّرتش حاجة لسه. عدّل حقل أو اكتب ملاحظة.";
    pub const F_FRONT: &str = "الوش";
    pub const F_BACK: &str = "الضهر";
    pub const F_TEXT: &str = "النص بالفراغات";
    pub const F_EXTRA: &str = "معلومة زيادة";
    pub const F_STEM: &str = "السؤال";
    pub const F_OPTIONS: &str = "الاختيارات، كل واحد في سطر";
    pub const F_ANSWER: &str = "الإجابة الصح";
    pub const F_WHY: &str = "الشرح، سطر لكل اختيار بنفس الترتيب";
    pub const F_PROMPT: &str = "السؤال";
    pub const F_POINTS: &str = "النقط، كل نقطة في سطر";
    pub const F_STEPS: &str = "الخطوات بالترتيب، كل خطوة في سطر";
    pub const F_TITLE: &str = "العنوان";
    pub const F_QUESTION: &str = "المطلوب";
    pub const F_EXPECT: &str = "النتيجة";
    pub const F_INTERPRET: &str = "التفسير";
    pub const EXPORT_TITLE: &str = "صدّر مراجعاتك للمحتوى";
    pub const EXPORT_BODY: &str =
        "ملف فيه كل ✓ و✎ و✗ علّمتها. ابعته في المحادثة عشان المحتوى يتصلّح والموافقات تتسجّل.";
    pub const EXPORT_BUTTON: &str = "صدّر الملف";
    pub const EXPORT_SAVED: &str = "اتحفظ في:";
    pub const EXPORT_SHOW: &str = "اعرض النص (تقدر تنسخه)";
    pub const EXPORT_FAILED: &str = "ما عرفتش أحفظ الملف:";
}

pub mod data {
    pub const EXPORT_TITLE: &str = "نسخة احتياطية من كل حاجة";
    pub const EXPORT_BODY: &str =
        "ملف واحد فيه مراجعاتك وعلاماتك على المحتوى وإعداداتك. تقدر ترجّعه على أي جهاز من «استرجع نسخة».";
    pub const EXPORT_BUTTON: &str = "احفظ نسخة";
    pub const EXPORT_SAVED_1: &str = "اتحفظت";
    pub const EXPORT_SAVED_2: &str = "حاجة في:";
    pub const EXPORT_ANDROID_HINT: &str =
        "تلاقيه من تطبيق الملفات أو من الكمبيوتر بالكابل، في Android/data/app.batna/files.";
    pub const EXPORT_FAILED: &str = "ما عرفتش أحفظ النسخة:";
    pub const EXPORT_SAVE_TO: &str = "احفظها في Downloads أو Drive";
    pub const EXPORT_SAVING: &str = "اختار مكان النسخة…";
    pub const EXPORT_SAVED_AS: &str = "اتحفظت برّه التطبيق باسم";
    pub const EXPORT_SAVED_OUT: &str = "اتحفظت في المكان اللي اخترته.";
    pub const EXPORT_SAVE_CANCELLED: &str = "ما اخترتش مكان، النسخة لسه في فولدر التطبيق.";
    pub const EXPORT_SAVE_FAILED: &str = "ما عرفتش أنقلها، النسخة لسه في فولدر التطبيق.";
    pub const IMPORT_TITLE: &str = "استرجع نسخة";
    pub const IMPORT_BODY: &str =
        "بيدمج النسخة مع اللي على الجهاز ده. مفيش حاجة بتتمسح، واللي اتعدّل بعدين هو اللي بيكسب.";
    pub const IMPORT_FOUND: &str = "نسخ لقيتها على الجهاز:";
    pub const IMPORT_PICK: &str = "أو اختار ملف";
    pub const IMPORT_PICK_FIRST: &str = "اختار ملف النسخة";
    pub const IMPORT_PICKING: &str = "اختار الملف…";
    pub const IMPORT_PICK_FAILED: &str = "ما عرفتش أفتح اختيار الملفات";
    pub const IMPORT_PASTE: &str = "أو الزق النص";
    pub const IMPORT_PASTED: &str = "نص ملزوق";
    pub const IMPORT_READ: &str = "اقرا النص";
    pub const IMPORT_FILE: &str = "الملف:";
    pub const IMPORT_FROM: &str = "من جهاز";
    pub const IMPORT_AT: &str = "اتعمل";
    pub const IMPORT_NOTHING: &str = "كل اللي في النسخة دي موجود هنا بالفعل.";
    pub const IMPORT_SAFE: &str = "الجديد هيتضاف، والأحدث هيحل محل القديم. مفيش حاجة هتتمسح.";
    pub const IMPORT_APPLY_1: &str = "ادمج";
    pub const IMPORT_APPLY_2: &str = "تغيير";
    pub const IMPORT_CANCEL: &str = "إلغاء";
    pub const IMPORT_BACK: &str = "رجوع";
    pub const IMPORT_AGAIN: &str = "استرجع نسخة تانية";
    pub const IMPORT_DONE_1: &str = "تمام، اتدمج";
    pub const IMPORT_DONE_2: &str = "تغيير. الجدول اتحدّث.";
    pub const IMPORT_FAILED: &str = "ما عرفتش أقرا النسخة دي:";
    pub const COL_WHAT: &str = "إيه";
    pub const COL_IN_FILE: &str = "في الملف";
    pub const COL_NEW: &str = "جديد";
    pub const COL_NEWER: &str = "أحدث";
    pub const T_REVIEWS: &str = "المراجعات";
    pub const T_CHECKS: &str = "علاماتك على المحتوى";
    pub const T_SETTINGS: &str = "الإعدادات";
}

pub mod cloud {
    pub const SECTION: &str = "المزامنة";
    pub const SECTION_SUB: &str = "حسابك وأجهزتك";
    pub const TITLE: &str = "المزامنة بين أجهزتك";
    pub const INTRO: &str = "سجّل دخول بجوجل على الموبايل والتابلت والماك، واللي بتذاكره وعلاماتك على المحتوى هيبقوا على التلاتة. مجاني، وعلى سيرفر باطنة لوحده.";
    pub const NOT_CONFIGURED: &str = "المزامنة لسه ما اتفعّلتش في النسخة دي. مذاكرتك محفوظة على الجهاز، وتقدر تنقلها بنسخة احتياطية من «بياناتك».";
    pub const SIGN_IN: &str = "سجّل دخول بجوجل";
    pub const SIGNING_IN: &str = "كمّل تسجيل الدخول في المتصفح، وبعدين ارجع هنا.";
    pub const SIGNED_IN_AS: &str = "داخل بـ";
    pub const SIGN_OUT: &str = "اخرج من الجهاز ده";
    pub const CONNECTING: &str = "بيتصل";
    pub const CATCHING_UP: &str = "بيجيب اللي فاتك";
    pub const LIVE: &str = "متزامن";
    pub const OFFLINE: &str = "مش متصل، هيحاول تاني لوحده";
    pub const STOPPED: &str = "واقف";
    pub const PENDING: &str = "تغيير مستني يترفع";
    pub const LAST: &str = "آخر مزامنة";
    pub const JUST_NOW: &str = "دلوقتي";
    pub const DEVICES: &str = "أجهزتك";
    pub const REFRESH: &str = "حدّث";
    pub const THIS_DEVICE: &str = "الجهاز ده";
    pub const DEVICE_OUT: &str = "خرج";
    pub const NEVER_SYNCED: &str = "لسه ما اتزامنش";
    pub const SIGN_OUT_DEVICE: &str = "خرّجه";
    pub const LOADING: &str = "بيحمّل…";

    pub const P_NOT_CONFIGURED: &str = "تسجيل الدخول لسه ما اتظبطش في النسخة دي.";
    pub const P_NOT_ALLOWED: &str = "الحساب ده مش مسموح له. ادخل بالجيميل اللي متسجل في السيرفر.";
    pub const P_CANCELLED: &str = "تسجيل الدخول اتلغى.";
    pub const P_FAILED: &str = "تسجيل الدخول ما نجحش. جرّب تاني.";
    pub const P_REMOTE: &str = "الجهاز ده اتخرّج من جهاز تاني. سجّل دخول تاني وقت ما تحب.";
    pub const P_NETWORK: &str = "مش قادر يوصل للسيرفر دلوقتي. هيحاول تاني لوحده، ومذاكرتك محفوظة على الجهاز.";
    pub const P_SERVER: &str = "السيرفر فيه مشكلة دلوقتي. هيحاول تاني لوحده.";
    pub const P_OUTDATED: &str = "السيرفر أحدث من النسخة دي. حدّث باطنة.";
}

pub mod settings {
    pub const TITLE: &str = "الإعدادات";
    pub const CLOSE: &str = "اقفل";
    pub const BACK: &str = "رجوع";
    pub const SECTIONS: &str = "أقسام الإعدادات";
    pub const THIS_DEVICE: &str = "للجهاز ده بس";

    pub const LOOK: &str = "الشكل";
    pub const LOOK_SUB: &str = "الألوان وحجم الخط";
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
    pub const THEME_SHIFT: &str = "نبطشية";
    pub const THEME_SHIFT_SUB: &str = "غامق ودافي من غير ضوء أزرق، للمذاكرة متأخر قبل النوم";

    pub const TEXT_SIZE: &str = "حجم الخط";
    pub const TEXT_SMALL: &str = "صغير";
    pub const TEXT_NORMAL: &str = "عادي";
    pub const TEXT_LARGE: &str = "كبير";
    pub const TEXT_LARGER: &str = "أكبر";
    pub const TEXT_SAMPLE_AR: &str = "إيه أول علاج لـ";
    pub const TEXT_SAMPLE_TERM: &str = "variceal bleeding";

    pub const DATA_FOLDER: &str = "فولدر البيانات";
    pub const DATA_NOWHERE: &str = "مش معروف على الجهاز ده";
    pub const DATA_LOCAL: &str = "كل اللي بتذاكره متسجل على الجهاز ده. النسخة الاحتياطية والمزامنة جايين.";

    pub const VERSION: &str = "النسخة";
    pub const CONTENT: &str = "المحتوى";
    pub const LECTURES_WRITTEN: &str = "محاضرة مكتوبة من";
    pub const ITEMS: &str = "سؤال وكارت";
    pub const SOURCE: &str = "المصدر";
    pub const BOOK: &str =
        "Internal Medicine Postgraduate Lectures, Alexandria Faculty of Medicine (2025), part 1";
    pub const PERSONAL: &str = "نسخة شخصية، مش منشورة.";

    pub const DIAG: &str = "اللي حصل من مشاكل";
    pub const DIAG_BODY: &str = "لو حاجة اشتغلت غلط، انسخ السطور دي وابعتها. مفيهاش أي حاجة من مذاكرتك.";
    pub const DIAG_EMPTY: &str = "مفيش مشاكل لحد دلوقتي.";
    pub const DIAG_LINES: &str = "سطر";
    pub const DIAG_REFRESH: &str = "حدّث";
    pub const DIAG_COPY: &str = "انسخ";
    pub const DIAG_COPIED: &str = "اتنسخ";
}

pub mod update {
    // Settings › Updates (crates/app/src/update/view.rs).
    pub const SECTION: &str = "التحديثات";
    pub const SECTION_SUB: &str = "نسخة جديدة ومحتوى جديد";
    pub const TITLE: &str = "التحديثات";
    pub const THIS_BUILD: &str = "النسخة اللي معاك";
    pub const BUILD: &str = "بناء";

    // Why updates are off here.
    pub const OFF_DEV: &str =
        "دي نسخة تطوير، فمش بتدوّر على تحديثات. النسخ اللي على الموبايل والتابلت والماك هي اللي بتتحدّث.";
    pub const OFF_KEY: &str = "التحديثات لسه مش شغالة: مفتاح التوقيع لسه ما اتعملش. أول نسخة ألفا هتيجي بيه.";
    pub const OFF_PLATFORM: &str = "مفيش تحديثات منشورة للجهاز ده.";
    pub const OFF_BUNDLE: &str = "باطنة شغال من فولدر البناء، مش من Batna.app، فمش هيتحدّث لوحده.";

    // Where things stand.
    pub const CHECKING: &str = "بيدوّر على تحديث…";
    pub const CHECK_NOW: &str = "دوّر دلوقتي";
    pub const UP_TO_DATE: &str = "معاك آخر نسخة";
    pub const NOT_FOR_DEVICE: &str = "فيه نسخة أحدث، بس لسه مش منشورة للجهاز ده.";
    pub const AVAILABLE: &str = "فيه نسخة جديدة";
    pub const DOWNLOAD: &str = "نزّلها";
    pub const DOWNLOADING: &str = "بينزّل";
    pub const READY: &str = "النسخة الجديدة جاهزة";
    pub const INSTALL_NOW: &str = "ثبّتها";
    pub const INSTALL_RESTART: &str = "ثبّتها وافتح تاني";
    pub const INSTALLING: &str = "بيثبّت…";
    pub const INSTALLING_ANDROID: &str = "أندرويد ممكن يسألك توافق. مذاكرتك محفوظة ومش هتضيع.";
    pub const NEEDS_PERMISSION: &str = "أندرويد محتاج إذن \"تثبيت تطبيقات غير معروفة\" لباطنة";
    pub const OPEN_PERMISSION: &str = "افتح الإذن";
    pub const TRY_AGAIN: &str = "جرّب تاني";
    pub const INSTALLED: &str = "اتثبّتت";
    pub const RESTART_NOW: &str = "افتح باطنة تاني";
    pub const NEXT_LAUNCH: &str = "النسخة الجديدة هتشتغل أول ما تفتح باطنة تاني.";
    pub const FAILED_NETWORK: &str = "مقدرش يوصل للتحديثات. هيجرّب تاني بعدين.";
    pub const FAILED_VERIFY: &str = "التحديث ما عداش من الفحص، فما اتثبّتش. مفيش حاجة اتغيّرت.";
    pub const FAILED_INSTALL: &str = "التثبيت ما كملش. النسخة اللي معاك شغالة زي ما هي.";
    pub const TOO_NEW: &str = "التحديث ده محتاج نسخة أحدث من باطنة. ثبّتها بإيدك مرة واحدة.";
    pub const KEY_CHANGED: &str = "مفتاح نسخ التجربة اتغيّر. ثبّت آخر نسخة تجربة بإيدك مرة واحدة.";
    pub const WAITING_WIFI: &str = "هينزّلها لوحده أول ما تبقى على واي فاي.";
    pub const LATER: &str = "بعدين";
    pub const WHATS_NEW: &str = "الجديد";
    pub const CONTENT_WAITING: &str = "محتوى جديد نزل، وهيظهر أول ما تفتح باطنة تاني.";

    // Mode and channel.
    pub const MODE: &str = "إزاي يتحدّث";
    pub const MODE_AUTO: &str = "لوحده";
    pub const MODE_AUTO_SUB: &str = "ينزّل على الواي فاي ويثبّت في وقت هادي، عمره ما يقطع مذاكرة.";
    pub const MODE_NOTIFY: &str = "قولّي الأول";
    pub const MODE_NOTIFY_SUB: &str = "يقولك إن فيه نسخة جديدة، وإنت تختار امتى.";
    pub const MODE_OFF: &str = "لأ";
    pub const MODE_OFF_SUB: &str = "بس لما تدوس \"دوّر دلوقتي\".";
    pub const CHANNEL: &str = "القناة";
    pub const CHANNEL_SUB: &str = "ألفا بتوصل الأول، والثابتة بتوصل آخر حاجة.";
    pub const ALPHA: &str = "ألفا";
    pub const BETA: &str = "بيتا";
    pub const STABLE: &str = "ثابتة";
    pub const PREVIEW: &str = "تجربة";
    pub const PREVIEW_SUB: &str = "نسخة تجربة من CI: بتتحدّث من نسخ التجربة بس.";
    pub const SAFETY: &str =
        "كل تحديث متوقّع بمفتاح باطنة، وكل ملف بيتفحص قبل ما يتثبّت. مذاكرتك بتفضل على الجهاز زي ما هي.";
    pub const CONTENT: &str = "المحاضرات الجديدة بتيجي مع التحديثات، حتى لو تحديث التطبيق نفسه مقفول.";
    pub const MB: &str = "ميجا";

    // The notifications Android shows around an install.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub const DONE_TITLE: &str = "باطنة اتحدّث";
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub const DONE_BODY: &str = "افتحه وكمّل من مكانك.";
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub const READY_TITLE: &str = "تحديث باطنة جاهز: دوس عشان تثبّته";
}

pub mod sound {
    pub const TITLE: &str = "الأصوات والاهتزاز";
    pub const BODY: &str = "صوت هادي مع الإجابة الصح ونغمة في آخر الجلسة. الإجابة الغلط مالهاش صوت.";
    pub const SOUNDS: &str = "الأصوات";
    pub const ON: &str = "شغّالة";
    pub const OFF: &str = "مقفولة";
    pub const RIGHT: &str = "صوت الإجابة الصح";
    pub const END: &str = "نغمة آخر الجلسة";
    pub const PICK_HINT: &str = "دوس على أي واحد تسمعه وتختاره.";
    pub const VARIATION: &str = "صوت";
    pub const VIBRATE: &str = "الاهتزاز";
    pub const VIBRATE_BODY: &str = "لمسة خفيفة مع كل إجابة. بيمشي على إعداد الاهتزاز في الموبايل.";
}

pub mod remind {
    pub const TITLE: &str = "باطنة";
    pub const SECTION: &str = "التذكير";
    pub const SECTION_SUB: &str = "تذكير هادي بمراجعة اليوم";
    pub const PHONE_ONLY: &str = "التذكير بيرن على الموبايل. على الماك مفيش تذكير لسه.";
    pub const DAILY: &str = "تذكير المذاكرة اليومي";
    pub const DAILY_BODY: &str = "إشعار واحد في المعاد اللي تختاره، فيه عدد المراجعات المستنياك.";
    pub const ON: &str = "شغّال";
    pub const OFF: &str = "مقفول";
    pub const TIMES: &str = "المواعيد";
    pub const TIME: &str = "معاد التذكير";
    pub const ADD: &str = "زوّد معاد";
    pub const REMOVE: &str = "شيل";
    pub const RULES: &str =
        "بيرن بس في أيام المذاكرة اللي في خطة الامتحان، ولو خلّصت مراجعات النهارده مش هيرن تاني لحد بكرة.";
    pub const HERE: &str = "يرن على الجهاز ده";
    pub const HERE_BODY: &str = "جهاز واحد بس يرن، عشان الموبايل والتابلت ما يرنّوش مع بعض.";
    pub const BLOCKED: &str = "الإشعارات مقفولة لباطنة، فالتذكير مش هيظهر.";
    pub const ALLOW: &str = "اسمح بالإشعارات";
    pub const OPEN_NOTIFICATIONS: &str = "افتح إعدادات الإشعارات";
    pub const BATTERY: &str =
        "لو التذكير ما رنّش، خلّي البطارية لباطنة «من غير قيود» عشان الموبايل ما ينيّمهاش.";
    pub const OPEN_APP: &str = "افتح إعدادات التطبيق";

    /// The reminder line, with the reviews due that day when known.
    pub fn text(due: Option<u32>) -> String {
        match due {
            Some(0) | None => "وقت جلسة قصيرة. كارت واحد أحسن من مفيش.".into(),
            Some(1) => "عندك مراجعة واحدة النهارده. دقيقة وتخلص.".into(),
            Some(n) => format!("عندك {n} مراجعة النهارده. يلا نبدأ بجلسة قصيرة."),
        }
    }
}

pub mod onboard {
    pub const TITLE: &str = "أهلاً في باطنة";
    pub const STEP: &str = "خطوة";
    pub const OF: &str = "من";
    pub const NEXT: &str = "التالي";
    pub const BACK: &str = "رجوع";
    pub const DONE: &str = "يلا نبدأ";
    pub const SKIP_ALL: &str = "بعدين";
    pub const EXAM_TITLE: &str = "امتحانك إمتى؟";
    pub const EXAM_BODY: &str =
        "عشان أوزّع المحاضرات والمراجعة على الأيام اللي فاضلة بهدوء. لو لسه مش متأكد، سيبها فاضية.";
    pub const EXAM_DAY: &str = "يوم الامتحان";
    pub const PICK_FUTURE: &str = "اختار يوم بعد النهارده.";
    pub const DAYS_TITLE: &str = "بتذاكر أنهي أيام في الأسبوع؟";
    pub const DAYS_BODY: &str = "الخطة والتذكير هيمشوا على الأيام دي بس. أيام الراحة راحة.";
    pub const PICK_A_DAY: &str = "اختار يوم واحد على الأقل.";
    pub const MINUTES_TITLE: &str = "قد إيه في اليوم؟";
    pub const MINUTES_BODY: &str = "طول الجلسة العادي. تقدر تغيّره قبل أي جلسة.";
    pub const MIN: &str = "دقيقة";
    pub const STUDIED_TITLE: &str = "ذاكرت أنهي محاضرات قبل كده؟";
    pub const STUDIED_BODY: &str = "علّم اللي قريته قبل كده. لو مفيش، كمّل عادي.";
    pub const REMIND_TITLE: &str = "أفكّرك إمتى؟";
    pub const REMIND_BODY: &str = "إشعار هادي فيه عدد المراجعات المستنياك. بيرن على الموبايل بس.";
    pub const ADD_TIME: &str = "زوّد معاد";
    pub const REMOVE: &str = "شيل";
    pub const NO_REMINDERS: &str = "من غير تذكير. تقدر تشغّله بعدين من الإعدادات.";
    pub const THEME_TITLE: &str = "تحب الألوان إزاي؟";
    pub const THEME_BODY: &str = "«نبطشية» غامق ودافي، للمذاكرة بالليل.";
    pub const REOPEN_TITLE: &str = "أسئلة البداية";
    pub const REOPEN_BODY: &str = "يوم الامتحان، أيام المذاكرة، طول الجلسة، التذكير والألوان، في مكان واحد.";
    pub const REOPEN: &str = "افتحها تاني";
}

pub mod recap {
    pub const TODAY_1: &str = "النهارده جاوبت";
    pub const TODAY_ANSWERS: &str = "سؤال";
    pub const AND: &str = "واتعلمت";
    pub const TODAY_NEW: &str = "حاجة جديدة";
    pub const DONE_REST: &str = "مراجعة النهارده خلصت، استريح. بكرة عندك";
    pub const REVIEWS: &str = "مراجعة.";
    pub const LEFT_1: &str = "فاضل";
    pub const LEFT_2: &str = "مراجعة لو عندك وقت، ولو لأ بكرة موجودة.";
    pub const WEEK_TITLE: &str = "آخر ٧ أيام";
    pub const WEEK_LEARNED_1: &str = "اتعلمت";
    pub const WEEK_LEARNED_2: &str = "حاجة جديدة في";
    pub const WEEK_DAYS: &str = "أيام.";
    pub const WEEK_RETENTION: &str = "بتفتكر صح:";
    pub const WEEK_WEAK: &str = "يستاهلوا رجوع:";
    pub const WEEK_EXAM_1: &str = "فاضل على الامتحان";
    pub const WEEK_EXAM_2: &str = "يوم.";
}
