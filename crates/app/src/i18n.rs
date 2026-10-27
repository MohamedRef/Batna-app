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
