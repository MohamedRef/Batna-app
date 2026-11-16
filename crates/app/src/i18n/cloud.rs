pub const SECTION: &str = "المزامنة";
pub const SECTION_SUB: &str = "حسابك وأجهزتك";
pub const TITLE: &str = "المزامنة بين أجهزتك";
pub const NOT_CONFIGURED: &str = "المزامنة لسه ما اتفعّلتش في النسخة دي. مذاكرتك محفوظة على الجهاز، وتقدر تنقلها بنسخة احتياطية من «بياناتك».";
pub const SIGN_IN: &str = "سجّل دخول بجوجل";
pub const SIGNING_IN: &str = "كمّل تسجيل الدخول في المتصفح، وبعدين ارجع هنا.";
/// The browser page after Google sends him back (HTML).
pub const BROWSER_DONE: &str = "<h1>✓</h1><p>تمام، ارجع لباطنة.</p>";
pub const BROWSER_CANCELLED: &str = "<p>ما اتسجلتش. ارجع لباطنة وجرّب تاني.</p>";
/// The browser page's button back to the app (Android).
pub const BROWSER_BACK: &str = "ارجع لباطنة";
/// Google said yes; Batna is finishing with the server.
pub const FINISHING: &str = "بيكمّل تسجيل الدخول…";
/// "من" in "من 3 دقايق".
pub const AGO: &str = "من";
pub const SIGNED_IN_AS: &str = "داخل بـ";
pub const SIGN_OUT: &str = "اخرج من الجهاز ده";
/// Settings › Sync: connect and sync right now.
pub const SYNC_NOW: &str = "زامن دلوقتي";
pub const SYNCING: &str = "بيزامن…";
pub const CONNECTING: &str = "بيتصل";
pub const CATCHING_UP: &str = "بيجيب اللي فاتك";
pub const LIVE: &str = "متزامن";
pub const OFFLINE: &str = "مش متصل، هيحاول تاني لوحده";
pub const STOPPED: &str = "واقف";
pub const PENDING: &str = "تغيير مستني يترفع";
/// "3 تغييرات مستنية تترفع".
pub fn pending(n: u64) -> String {
    match n {
        1 => "تغيير واحد مستني يترفع".to_string(),
        2 => "تغييرين مستنيين يترفعوا".to_string(),
        _ if (3..=10).contains(&(n % 100)) => format!("{n} تغييرات مستنية تترفع"),
        _ => format!("{n} {PENDING}"),
    }
}
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
pub const P_SIGN_IN_WAITING: &str =
    "جوجل وافقت، بس النت مش واصل لباطنة دلوقتي. هيكمّل الدخول لوحده أول ما النت يرجع، أو دوس «جرّب دلوقتي».";
pub const TRY_NOW: &str = "جرّب دلوقتي";
pub const P_SERVER: &str = "السيرفر فيه مشكلة دلوقتي. هيحاول تاني لوحده.";
pub const P_OUTDATED: &str = "السيرفر أحدث من النسخة دي. حدّث باطنة.";
pub const P_CLOCK: &str =
    "ساعة الجهاز سابقة الوقت الحقيقي، فالتغييرات الجديدة مستنية. ظبط الساعة على التلقائي وهتتبعت لوحدها.";

// Username + password (docs/CLOUD.md §4.1).
pub const PW_TITLE: &str = "الدخول باسم مستخدم وباسورد";
pub const PW_INTRO: &str = "على جهاز مش بتاعك ومن غير جيميلك: ادخل باسم المستخدم والباسورد اللي ظبطتهم من جهازك، وهتلاقي مذاكرتك كلها وآخر حتة وقفت عندها.";
pub const USERNAME: &str = "اسم المستخدم";
pub const PASSWORD: &str = "الباسورد";
pub const NOT_MINE: &str = "الجهاز ده مش بتاعي";
pub const NOT_MINE_HINT: &str = "هتخرج لوحدك أول ما تقفل التاب أو التطبيق، ومذاكرتك مش هتفضل على الجهاز ده.";
pub const PW_SIGN_IN: &str = "ادخل";
pub const PW_CHECKING: &str = "بيدخل…";
pub const PW_YOURS: &str = "بتدخل من أي جهاز بـ";
pub const PW_HOW: &str = "على أي جهاز: افتح باطنة، ومن «المزامنة» اكتب اسم المستخدم والباسورد.";
pub const PW_NONE: &str =
    "اختار اسم مستخدم وباسورد عشان تدخل من أي جهاز من غير جيميل. حسابك هو هو، فكل حاجة بتتزامن.";
pub const USERNAME_HINT: &str = "من 3 لـ 32: حروف إنجليزي صغيرة وأرقام و . _ -";
pub const NEW_PASSWORD: &str = "الباسورد الجديد";
pub const PASSWORD_HINT: &str = "10 حروف على الأقل. جملة قصيرة تفتكرها أحسن من كلمة صعبة.";
pub const PASSWORD_AGAIN: &str = "اكتبه تاني";
pub const PW_SAVE: &str = "احفظ";
pub const PW_SAVING: &str = "بيحفظ…";
pub const PW_SAVED: &str = "اتحفظ. تقدر تدخل بيه من أي جهاز دلوقتي.";
pub const PW_CHANGE: &str = "غيّر";
pub const PW_REMOVE: &str = "اقفل الدخول بالباسورد";
pub const CANCEL: &str = "إلغاء";
pub const TEMPORARY: &str =
    "دخول مؤقت: هتخرج لوحدك لما تقفل التاب أو التطبيق. لو خلصت قبلها، دوس «اخرج من الجهاز ده».";
pub const P_WRONG_PASSWORD: &str = "اسم المستخدم أو الباسورد مش مظبوط. جرّب تاني.";
pub const P_CURRENT_WRONG: &str = "الباسورد الحالي مش مظبوط. جرّب تاني.";
pub const CURRENT_PASSWORD: &str = "الباسورد الحالي";
pub const CURRENT_HINT: &str =
    "إنت داخل بالباسورد، فعشان تغيّره أو تقفله اكتب الحالي. نسيته؟ غيّره من جهاز داخل بجيميلك.";
pub const P_LOCKED: &str = "محاولات غلط كتير. استنى شوية وجرّب تاني.";
/// "استنى 5 دقايق": the lock's wait, with the Egyptian number forms.
pub fn locked(minutes: u64) -> String {
    use crate::i18n::count::{MINUTE, count};
    format!("محاولات غلط كتير، فالدخول بالاسم ده واقف شوية. جرّب تاني بعد {}.", count(minutes, &MINUTE))
}
pub const P_TAKEN: &str = "اسم المستخدم ده محجوز. اختار اسم تاني.";
pub const P_BAD_USERNAME: &str = "اسم المستخدم من 3 لـ 32: حروف إنجليزي وأرقام و . _ - بس.";
pub const P_SHORT_PASSWORD: &str = "الباسورد لازم يبقى 10 حروف على الأقل.";
pub const P_MISMATCH: &str = "الباسوردين مش زي بعض.";
/// Changes Batna Cloud refused, or this build couldn't read (`cloud/held_ui.rs`).
pub const HELD_TITLE: &str = "تغييرات ما اترفعتش";
pub const HELD_CHANGE: super::count::Noun =
    super::count::Noun {
        one: "تغيير واحد", two: "تغييرين", few: "تغييرات", many: "تغيير"
    };
/// "{n} … Batna Cloud didn't take"; the changes are safe on this device.
pub fn held(n: u64) -> String {
    format!(
        "{} من الجهاز ده ما اترفعوش على باطنة كلاود. هما محفوظين هنا عادي، بس أجهزتك التانية مش شايفاهم. ابعتهم تاني (بعد تحديث مثلًا)، أو سيبهم.",
        super::count::count(n, &HELD_CHANGE)
    )
}
pub const HELD_SEND_AGAIN: &str = "ابعتهم تاني";
pub const HELD_FORGET: &str = "سيبهم";
pub const HELD_FORGET_SURE: &str = "هيفضلوا على الجهاز ده بس، ومش هيوصلوا لأجهزتك التانية. متأكد؟";
pub const HELD_FORGET_YES: &str = "أيوه، سيبهم";
pub const HELD_FORGET_NO: &str = "لأ";

/// The verdicts read key (`cloud/read_key.rs`, #1352).
pub const KEY_TITLE: &str = "مفتاح قراءة علامات المحتوى";
pub const KEY_ABOUT: &str = "بيخلّي أداة تصليح المحتوى تقرا علامات ✓ ✎ ✗ بتاعتك من باطنة كلاود على طول، من غير ما تبعت ملف. مايقدرش يغيّر حاجة، ولا يقرا أي حاجة تانية.";
pub const KEY_NONE: &str = "مافيش مفتاح دلوقتي.";
/// "عندك مفتاح من 16 نوفمبر."
pub fn key_since(day: u32, month: &str) -> String {
    format!("عندك مفتاح من {day} {month}.")
}
pub const KEY_MAKE: &str = "اعمل مفتاح";
pub const KEY_REMAKE: &str = "اعمل مفتاح جديد";
pub const KEY_REMOVE: &str = "شيل المفتاح";
pub const KEY_REMOVE_SURE: &str = "الأداة مش هتقدر تقرا علاماتك لحد ما تعمل مفتاح جديد. متأكد؟";
pub const KEY_REMOVE_YES: &str = "أيوه، شيله";
pub const KEY_REMOVE_NO: &str = "لأ";
pub const KEY_REMAKE_NOTE: &str = "المفتاح الجديد بيلغي القديم.";
pub const KEY_NEW: &str =
    "ده المفتاح. انسخه دلوقتي وحطه كسرّ عند أداة المحتوى (BATNA_VERDICTS_KEY)، مش هيظهر تاني:";
pub const KEY_COPY: &str = "انسخ";
pub const KEY_COPIED: &str = "اتنسخ ✓";
pub const KEY_BUSY: &str = "لحظة…";
pub const KEY_LOADING: &str = "بشوف لو فيه مفتاح…";
