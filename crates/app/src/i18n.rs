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

pub mod settings {
    pub const TITLE: &str = "الإعدادات";
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
    pub const DATA: &str = "مذاكرتك";
    pub const DATA_AT: &str = "محفوظة على الجهاز ده في:";
    pub const DATA_MEMORY: &str =
        "مش محفوظة: قاعدة البيانات ما اتفتحتش، فاللي بتذاكره هيروح لما تقفل التطبيق.";
    pub const DEVICE: &str = "اسم الجهاز في المزامنة:";
    pub const NEW_PER_DAY_BODY: &str = "الجديد بيزوّد مراجعات الأيام الجاية، فاختار رقم تقدر تكمّل عليه.";
}

pub mod soon {
    pub const TITLE: &str = "الشاشة دي بتتبني دلوقتي";
    pub const BODY: &str = "هتظهر هنا أول ما تخلص. لحد ما ده يحصل، تقدر تذاكر نفس المحتوى من Anki.";
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
    pub const IMPORT_TITLE: &str = "استرجع نسخة";
    pub const IMPORT_BODY: &str =
        "بيدمج النسخة مع اللي على الجهاز ده. مفيش حاجة بتتمسح، واللي اتعدّل بعدين هو اللي بيكسب.";
    pub const IMPORT_FOUND: &str = "نسخ لقيتها على الجهاز:";
    pub const IMPORT_PICK: &str = "أو اختار ملف";
    pub const IMPORT_PICK_FIRST: &str = "اختار ملف النسخة";
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
