pub const TITLE: &str = "الحاسبات";
/// "فاضل 3 خانات"
pub fn left(n: usize) -> String {
    format!("فاضل {}", super::count::count(n, &super::count::FIELD))
}
pub const PICK: &str = "اختار حاسبة";
pub const FILL: &str = "املا البيانات وهتطلع النتيجة هنا.";
pub const MISSING: &str = "ناقص";
pub const INVALID: &str = "راجع القيمة دي";
/// A number outside what a body can show (`calc::IMPOSSIBLE`).
pub fn impossible(range: &str, unit: &str) -> String {
    let unit = if unit.is_empty() { String::new() } else { format!(" {unit}") };
    // The range is isolated (FSI…PDI) so it keeps its LTR order.
    format!("الرقم ده مش ممكن هنا. اكتب رقم في المدى \u{2068}{range}{unit}\u{2069}، وراجع الوحدة.")
}
/// Other refusals a calculator gives, by its English reason.
pub fn reason(reason: &str) -> Option<&'static str> {
    Some(match reason {
        "higher than the serum albumin" => {
            "ألبومين الاستسقاء مايبقاش أعلى من ألبومين الدم. راجع الرقمين والوحدة."
        }
        "lower than the ascitic albumin" => "البروتين الكلي في الاستسقاء مايبقاش أقل من الألبومين اللي فيه.",
        _ => return None,
    })
}
/// Under a number box: its normal range, or which way it is out of it.
pub const NORMAL: &str = "الطبيعي:";
pub const HIGH: &str = "↑ أعلى من الطبيعي:";
pub const LOW: &str = "↓ أقل من الطبيعي:";
/// "رجالة 13.5–17.5 · ستات 12–16 g/dL" until he picks the sex.
pub const MEN: &str = "رجالة";
pub const WOMEN: &str = "ستات";
pub const OPTIONAL: &str = "اختياري";
pub const YES: &str = "أيوه";
pub const NO: &str = "لأ";
pub const RESULT: &str = "النتيجة";
/// Over the interpretation under the result bar.
pub const MEANS: &str = "يعني إيه";
pub const BREAKDOWN: &str = "التفاصيل";
pub const SOURCE: &str = "المصدر";
/// Over the next steps for a result (`calc::guide`).
pub const NEXT_STEP: &str = "الخطوة الجاية";
/// Over the differential diagnosis for a result (`calc::guide`).
pub const DD: &str = "التشخيص التفريقي";
pub const RESET: &str = "امسح";
pub const BACK: &str = "كل الحاسبات";
pub const AREA_LIVER: &str = "الكبد";
pub const AREA_ASCITES: &str = "الاستسقاء";
pub const AREA_TUMOUR: &str = "الأورام";
pub const AREA_ACUTE: &str = "الفشل الكبدي الحاد";
pub const AREA_PANCREAS: &str = "البنكرياس";
pub const AREA_GI: &str = "الجهاز الهضمي";
pub const AREA_RENAL: &str = "الكلى والأملاح";
pub const AREA_ELDERLY: &str = "المسنين";
pub const AREA_RHEUM: &str = "الروماتيزم";
pub const AREA_DIABETES: &str = "السكر";
pub const AREA_BLOOD: &str = "الدم";
pub const AREA_RESEARCH: &str = "الريسيرش";
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

/// The Arabic hint under a calculator field (core keeps the English one).
pub fn hint(calc: &str, field: &str) -> Option<&'static str> {
    Some(match (calc, field) {
        ("sample_size_survey", "expected_prevalence") => {
            "من دراسات قبل كده أو pilot أو experts؛ 50% لو مفيش أي بيانات."
        }
        ("sample_size_survey", "design_effect") => {
            "1 للـ simple random sampling؛ غالبًا 2 للـ cluster sampling."
        }
        ("sample_size_two_proportions", "power") => "80% على الأقل؛ 85% أو 90% أحسن.",
        ("sample_size_two_proportions", "ratio") => "1 لو المجموعتين متساويين.",
        ("predictive_values", "prevalence") => {
            "بتاعة الناس اللي هتكشف عليهم أنت، مش بتاعة الدراسة اللي عملت الـ test."
        }
        ("post_test_probability", "pretest") => "الـ prevalence في عيانين زي ده، أو تقديرك الإكلينيكي.",
        ("post_test_probability", "lr") => {
            "نتيجة positive: LR+ = sensitivity ÷ (1 − specificity). نتيجة negative: LR− = (1 − sensitivity) ÷ specificity."
        }
        ("child_pugh", "pt_prolonged_s") => "دي طريقة الكتاب. سيبها فاضية لو هتستخدم الـ INR.",
        ("child_pugh", "inr") => "بيتحسب بس لو خانة الـ PT فاضية.",
        ("apri", "ast_uln_u_l") => "غالبًا 40 U/L.",
        ("maddrey", "pt_control_s") => "غالبًا 12–13 ثانية.",
        ("glasgow_ah", "urea_mmol_l") => "اليوريا بالـ mg/dL ÷ 6 = mmol/L.",
        ("ascitic_pmn", "pmn_per_mm3") => "عدد كرات الدم البيضا الكلي × نسبة النيوتروفيل.",
        ("bisap", "bun_mg_dl") => "اليوريا ÷ 2.14.",
        ("atlanta_2012", "organ_failure") => {
            "صدمة (SBP أقل من 90 أو MAP أقل من 70)، PaO2 أقل من 60 أو جهاز تنفّس، كرياتينين فوق 2 بعد السوايل أو غسيل، نزيف أكتر من 500 مل في اليوم، نبض فوق 90."
        }
        ("glasgow_blatchford", "bun_mg_dl") => "اليوريا ÷ 2.14.",
        ("oakland", "hb_g_dl") => "جدول الكتاب بالـ g/L: ‏7 g/dL = 70 g/L.",
        ("kyoto_gastritis", "diffuse_redness") => {
            "الكتاب بيدرّجها بشفافية الـ collecting venules في الـ body."
        }
        ("osmolal_gap", "bun_mg_dl") => "اليوريا ÷ 2.14.",
        ("acr_eular_2010_ra", "large_joints") => "الكتف والكوع والحوض والركبة والكاحل.",
        ("acr_eular_2010_ra", "small_joints") => {
            "الـ MCP والـ PIP والـ MTP من 2 لـ 5 والـ IP بتاع الإبهام والرسغ."
        }
        ("caspar", "inflammatory_disease") => "شرط الدخول: CASPAR بتتطبّق عليه بس.",
        ("asas_axial_spa", "back_pain_3_months_onset_45") => {
            "شرط الدخول (ASAS 2009)؛ الكتاب ص295 بيقول بدأ قبل 40–45."
        }
        ("asas_axial_spa", "sacroiliitis_imaging") => {
            "التهاب نشط على الـ MRI، أو أشعة grade 2 في الناحيتين أو 3–4 في ناحية."
        }
        ("asas_axial_spa", "ibp") => {
            "بدأ قبل 45، بالتدريج، أكتر من 3 شهور، بالليل، بيتحسن بالحركة مش بالراحة."
        }
        ("asas_peripheral_spa", "preceding_infection") => {
            "التهاب مجرى البول أو عنق الرحم أو إسهال قبل المفاصل."
        }
        ("modified_new_york", "si_grade_right") => "الأشعة: 0 طبيعي، 1 مشكوك، 2 بسيط، 3 متوسط، 4 التحام.",
        ("sle_acr_1997", "renal") => "بروتين في البول أكتر من 0.5 جم/يوم، أو خلايا أو casts.",
        ("sle_acr_1997", "neurological") => "تشنجات أو ذهان.",
        ("sle_acr_1997", "haematological") => "أنيميا انحلالية، أو نقص كرات بيضا أو ليمفاوية أو صفايح.",
        ("sle_acr_1997", "immunological") => "Anti-dsDNA أو anti-Sm أو anticardiolipin.",
        ("sle_eular_acr_2019", "ana_1_80") => "شرط الدخول.",
        ("sle_eular_acr_2019", "antiphospholipid") => {
            "Anticardiolipin IgG أكتر من 40، أو anti-β2GP1 IgG أكتر من 40، أو lupus anticoagulant."
        }
        ("effective_osmolality", "urea_mmol_l") => "للأوزمولاليتي الكلية. اليوريا بالـ mg/dL ÷ 6.",
        ("dka_severity", "bhb_mmol_l") => "سيبها فاضية لو معاك شريط بول بس، وعلّم الخانة اللي تحت.",
        ("feurea", "urine_urea_mg_dl") => "يوريا أو BUN، بس بنفس وحدة البلازما.",
        ("bun_creatinine", "bun_mg_dl") => "اليوريا ÷ 2.14.",
        ("kdigo_ckd", "acr_mg_g") => "في جدول الكتاب: 3 mg/mmol = 30 mg/g.",
        ("aa_gradient", "fio2") => "هوا الغرفة = 0.21.",
        ("plasmic", "platelets_10e9_l") => "30 × 10⁹/L = 30,000/µL.",
        ("plasmic", "hemolysis") => {
            "الريتيكيولوسايت فوق 2.5%، أو الهابتوجلوبين مش ظاهر، أو الـ indirect bilirubin فوق 2 mg/dL."
        }
        ("anion_gap", "albumin_g_dl") => "بيصحح الـ gap لو الألبومين واطي.",
        ("ranson", "bun_rise_mg_dl") => "زيادة اليوريا ÷ 2.14.",
        ("relative_risk", "a") => "معرّض وعنده المرض (أو case)",
        ("relative_risk", "b") => "معرّض ومعندوش المرض (أو control)",
        ("relative_risk", "c") => "مش معرّض وعنده المرض (أو case)",
        ("relative_risk", "d") => "مش معرّض ومعندوش المرض (أو control)",
        ("odds_ratio", "a") => "معرّض وعنده المرض (أو case)",
        ("odds_ratio", "b") => "معرّض ومعندوش المرض (أو control)",
        ("odds_ratio", "c") => "مش معرّض وعنده المرض (أو case)",
        ("odds_ratio", "d") => "مش معرّض ومعندوش المرض (أو control)",
        ("diagnostic_accuracy", "tp") => "التحليل + والمرض +",
        ("diagnostic_accuracy", "fp") => "التحليل + والمرض −",
        ("diagnostic_accuracy", "fn") => "التحليل − والمرض +",
        ("diagnostic_accuracy", "tn") => "التحليل − والمرض −",
        ("rpi", "rbc_10e12_l") => "بس عشان العدد المطلق للـ reticulocytes.",
        ("binet_cll", "areas") => {
            "غدد الرقبة والإبط والأُربية (كل منطقة بواحدة حتى لو الناحيتين)، والطحال، والكبد."
        }
        ("ipi_nhl", "ecog_2_plus") => "2 = بيقوم ويتحرك أكتر من نص اليوم بس مش قادر يشتغل.",
        ("who_pmf", "clonal") => "ماركرز تانية: ASXL1 وEZH2 وTET2 وIDH1/IDH2 وSRSF2 وSF3B1.",
        ("sar", "primary_cases") => "بيتشالوا من اللي ممكن يتعدوا.",
        ("lawton_iadl", "food_preparation" | "housekeeping" | "laundry") => {
            "للستات بس (من 8)، الراجل بيتحسب من 5."
        }
        ("clinical_frailty", "level") => "في الخرف، درجة الهشاشة بتمشي مع درجة الخرف.",
        ("sarc_f", "falls") => "الكتاب كاتب «أقل من 3» = 1. في الأصل من 1 لـ 3 وقعات = 1.",
        ("abi", "dp_mm_hg") => "ضغط الكاحل بالدوبلر، وبيتاخد الأعلى من الاتنين.",
        ("abi", "brachial_right_mm_hg") => "بيتاخد الأعلى من الدراعين.",
        _ => return None,
    })
}

// The picker (calc/picker.rs): search, area filter, favourites, recents.
pub const SEARCH: &str = "دوّر على حاسبة…";
pub const SEARCH_LABEL: &str = "دوّر على حاسبة";
pub const ALL_AREAS: &str = "كل الأقسام";
pub const AREA_FILTER: &str = "القسم";
pub const FAVOURITES: &str = "المفضلة";
pub const RECENT: &str = "آخر ما فتحت";
pub const RESULTS: &str = "النتايج";
pub const NOTHING_FOUND: &str = "مفيش حاسبة بالاسم ده.";
pub const CLEAR_SEARCH: &str = "امسح البحث";
pub const PIN: &str = "ثبّتها في المفضلة";
pub const UNPIN: &str = "شيلها من المفضلة";
pub const PINNED: &str = "في المفضلة";
/// "44 حاسبة"
pub fn count(n: usize) -> String {
    super::count::count(n, &super::count::CALC)
}

/// One line under a calculator's name: what it is for. Every calculator
/// has one (test `every_calculator_has_a_blurb`).
pub fn blurb(calc: &str) -> Option<&'static str> {
    Some(match calc {
        "child_pugh" => "شدة تليّف الكبد (A وB وC) والتوقّع",
        "meld" => "خطورة السيروزس خلال 3 شهور وأولوية زرع الكبد",
        "meld_na" => "الـ MELD ومعاه الصوديوم: أدق في أولوية الزرع",
        "albi" => "وظيفة الكبد في الـ HCC من الألبومين والبيليروبين",
        "fib4" => "التليّف من غير بيوبسي: السن والإنزيمات والصفايح",
        "apri" => "التليّف من غير بيوبسي: الـ AST والصفايح",
        "fli" => "الكبد الدهني من الدهون الثلاثية والوزن والوسط والـ GGT",
        "baveno_vi" => "مين في السيروزس المعوّضة يستغنى عن منظار الدوالي",
        "r_ratio" => "نوع أذية الكبد من الدوا: خلوي ولا ركودي ولا مختلط",
        "maddrey" => "شدة الـ alcoholic hepatitis ومين ياخد ستيرويد",
        "glasgow_ah" => "التوقّع في الـ alcoholic hepatitis وفايدة الستيرويد",
        "saag" => "سبب الاستسقاء: من الـ portal hypertension ولا لأ",
        "ascitic_pmn" => "تشخيص الـ SBP من عدد الـ PMN في سائل البطن",
        "sos_criteria" => "تشخيص الـ SOS (VOD) بعد زرع النخاع",
        "cha2ds2_vasc" => "خطر الجلطات في الـ AF: وقف المسيّل قبل المنظار والـ bridging",
        "bclc" => "مرحلة الـ HCC والعلاج المناسب ليها",
        "milan" => "الـ HCC ينفع معاه زرع كبد ولا لأ",
        "okuda" => "مرحلة الـ HCC من حجم الورم ووظيفة الكبد",
        "page_b" => "خطر الـ HCC في فيروس B: مين يتعمله مسح",
        "kings_college" => "مين محتاج زرع كبد في الفشل الكبدي الحاد",
        "clichy" => "معايير الزرع في الفشل الكبدي الحاد بالـ factor V والوعي",
        "ranson" => "شدة التهاب البنكرياس الحاد عند الدخول وبعد 48 ساعة",
        "bisap" => "خطورة التهاب البنكرياس الحاد في أول 24 ساعة",
        "ctsi" => "شدة التهاب البنكرياس من الأشعة المقطعية والـ necrosis",
        "atlanta_2012" => "شدة التهاب البنكرياس من فشل الأعضاء والمضاعفات",
        "glasgow_blatchford" => "نزيف الجهاز الهضمي العلوي: مين محتاج تدخل ومين يروّح",
        "shock_index" => "النبض ÷ الضغط: النزيف مستقر ولا لأ",
        "oakland" => "نزيف الجهاز الهضمي السفلي: مين يروّح بأمان ومين يتحجز",
        "uc_activity" => "شدة الـ ulcerative colitis: خفيفة ولا متوسطة ولا شديدة",
        "kyoto_gastritis" => "المنظار بيقول إيه عن الـ H. pylori وخطر سرطان المعدة",
        "rome4_ibs" => "تشخيص الـ IBS بمعايير Rome IV ونوعه",
        "anion_gap" => "سبب الـ metabolic acidosis: الـ gap عالي ولا طبيعي",
        "delta_ratio" => "فيه اضطراب تاني مع الـ high anion gap acidosis؟",
        "expected_compensation" => "التعويض المتوقّع: الاضطراب لوحده ولا مخلوط؟",
        "osmolal_gap" => "فيه مادة سامة زي الميثانول أو الإيثيلين جلايكول؟",
        "kdigo_aki" => "مرحلة الـ AKI من الكرياتينين وكمية البول",
        "fena" => "الـ AKI قبل الكلى (prerenal) ولا من الكلى نفسها (ATN)",
        "feurea" => "زي الـ FENa، بس بينفع والعيان على مدرّات",
        "bun_creatinine" => "نسبة الـ BUN للكرياتينين: prerenal ولا ATN؟",
        "ckd_epi_2021" => "الـ eGFR بمعادلة CKD-EPI 2021 من الكرياتينين والسن",
        "kdigo_ckd" => "مرحلة الـ CKD بالـ G والـ A ولون الخطر والمتابعة",
        "aa_gradient" => "نقص الأكسجين من الرئة نفسها ولا من قلة التهوية؟",
        "free_water_deficit" => "نقص الميه في الـ hypernatraemia وحجم المحلول المطلوب",
        "plasmic" => "احتمال إن الـ TMA تكون TTP قبل نتيجة الـ ADAMTS13",
        "cockcroft_gault" => "تصفية الكرياتينين لظبط جرعات الدوا، خصوصًا في المسنين",
        "katz_adl" => "الأنشطة اليومية الأساسية: مستقل ولا محتاج مساعدة؟",
        "lawton_iadl" => "الأنشطة الأعلى زي الفلوس والأدوية والمواصلات",
        "gds15" => "فحص اكتئاب المسنين: 15 سؤال أيوه ولا لأ",
        "mna_sf" => "فحص سوء التغذية في المسنين من 14 نقطة",
        "clinical_frailty" => "درجة الهشاشة من 1 لـ 9 بوصف الكتاب",
        "cam" => "تشخيص الـ delirium بأربع علامات",
        "sarc_f" => "فحص ضعف وضمور العضلات (sarcopenia)",
        "mmse" => "فحص الذاكرة والإدراك من 30 درجة",
        "abi" => "ضغط الكاحل على ضغط الدراع: فيه PAD؟",
        "acr_eular_2010_ra" => "تشخيص الـ rheumatoid arthritis: 6 من 10 أو أكتر",
        "caspar" => "تشخيص الـ psoriatic arthritis: 3 نقط أو أكتر",
        "asas_axial_spa" => "الـ axial SpA بالأشعة أو بالـ HLA-B27، حتى قبل ما تبان في الأشعة",
        "asas_peripheral_spa" => "الـ SpA في المفاصل الطرفية: arthritis أو enthesitis أو dactylitis",
        "modified_new_york" => "تشخيص الـ ankylosing spondylitis بالأشعة والكشف",
        "sle_acr_1997" => "تشخيص الذئبة الحمراء: 4 من 11 أو أكتر",
        "sle_eular_acr_2019" => "تشخيص الذئبة بالنقط: ANA وبعدين 10 نقط أو أكتر",
        "gpa_2022" => "تصنيف الـ GPA (Wegener) بالنقط: 5 أو أكتر",
        "mpa_2022" => "تصنيف الـ microscopic polyangiitis بالنقط: 5 أو أكتر",
        "corrected_sodium" => "الصوديوم الحقيقي لما السكر عالي",
        "effective_osmolality" => "أوزمولاليتي الدم لتشخيص الـ HHS",
        "dka_severity" => "هل هي DKA؟ وشدتها قد إيه ومكان العلاج",
        "premixed_insulin" => "أول جرعة إنسولين premixed: التلتين الصبح والتلت بالليل",
        "bmi" => "مؤشر كتلة الجسم وإمتى نفحص أو ندّي دوا أو نفكر في جراحة",
        "ipss_mf" => "التوقّع في الـ primary myelofibrosis",
        "rpi" => "النخاع بيعوّض الأنيميا ولا لأ: نقص إنتاج ولا تكسير ونزيف",
        "rai_cll" => "مرحلة الـ CLL (من 0 لـ IV) بالغدد والطحال والهيموجلوبين والصفايح",
        "binet_cll" => "مرحلة الـ CLL (A وB وC) بعدد المناطق والهيموجلوبين والصفايح",
        "ann_arbor" => "مرحلة الليمفوما من I لـ IV، وA أو B حسب الأعراض",
        "ipi_nhl" => "التوقّع في الـ non-Hodgkin lymphoma من 0 لـ 5",
        "who_pv" => "تشخيص الـ polycythaemia vera بمعايير الـ WHO",
        "who_et" => "تشخيص الـ essential thrombocythaemia بمعايير الـ WHO",
        "who_pmf" => "تشخيص الـ primary myelofibrosis بمعايير الـ WHO",
        "incidence" => "معدل الحالات الجديدة في فترة معيّنة",
        "prevalence" => "نسبة كل الحالات الموجودة في وقت معيّن",
        "prevalence_id" => "العلاقة بين الانتشار والحدوث ومدة المرض",
        "attack_rate" => "نسبة اللي عيّوا في وباء أو تفشّي",
        "sar" => "نسبة العدوى بين المخالطين بعد أول حالة",
        "cfr" => "نسبة الوفيات بين المصابين بالمرض",
        "pmr" => "نسبة الوفيات بسبب معيّن من كل الوفيات",
        "imr" => "وفيات الأطفال تحت سنة لكل 1000 مولود حي",
        "nmr" => "وفيات أول 28 يوم لكل 1000 مولود حي",
        "relative_risk" => "الخطر في المعرّضين قصاد غير المعرّضين (cohort)",
        "odds_ratio" => "قوة العلاقة في دراسة الـ case-control",
        "attributable_risk" => "الزيادة في الخطر اللي سببها التعرّض",
        "nnt" => "تعالج كام واحد عشان واحد يستفيد (أو يتأذى)",
        "diagnostic_accuracy" => "الحساسية والنوعية والـ PPV والـ NPV من جدول 2×2",
        "z_score" => "القيمة بعيدة عن المتوسط بكام انحراف معياري",
        "sample_size_survey" => "محتاج كام واحد في الـ survey عشان تقدّر الـ prevalence",
        "sample_size_two_proportions" => {
            "محتاج كام case و control (أو exposed و unexposed) بالـ power اللي عايزه"
        }
        "predictive_values" => "الـ PPV والـ NPV لنفس الـ test في population بـ prevalence تانية",
        "post_test_probability" => "الاحتمال بعد النتيجة من الـ pre-test probability والـ likelihood ratio",
        _ => return None,
    })
}
