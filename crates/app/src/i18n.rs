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
