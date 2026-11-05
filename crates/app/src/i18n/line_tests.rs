//! Every counted or filled-in line, one test per family, read the way an
//! Egyptian would say it: 1 singular, 2 dual, 3–10 plural, 11+ singular.

use super::*;

#[test]
fn citations_name_the_page_or_slide() {
    assert_eq!(cite::page(179), "ص 179");
    assert_eq!(cite::slide(25), "سلايد 25");
}

#[test]
fn todays_lines_follow_the_counts() {
    assert_eq!(today::exam_in(1), "الامتحان بكرة. إنت جاهز أكتر مما فاكر.");
    assert_eq!(today::exam_in(2), "فاضل يومين على الامتحان");
    assert_eq!(today::due_cases(1), "حالة واحدة مستنية مراجعة في شاشة الحالات");
    assert_eq!(today::due_cases(3), "3 حالات مستنية مراجعة في شاشة الحالات");
    assert!(today::tight_days(1).contains("يوم مذاكرة زيادة"));
    assert!(today::tight_days(3).contains("3 أيام مذاكرة زيادة"), "{}", today::tight_days(3));
    assert!(today::tight_cap(220).contains("220"));
    assert_eq!(today::headline(0, 5), "5 حاجات جديدة، على قد وقتك.");
    assert_eq!(today::headline(2, 0), "مراجعتين، وبس.");
    assert_eq!(today::headline(12, 1), "12 مراجعة، وحاجة جديدة.");
    assert_ne!(today::hero_body(0, 3), today::hero_body(3, 0));
    assert_ne!(today::hero_body(3, 0), today::hero_body(3, 3));
    assert_eq!(today::plan_line(4, 2), format!("4 {} + مراجعتين", today::PLAN_NEW));
    assert_eq!(today::plan_reviews_only(7), "7 مراجعات بس");
    assert_eq!(today::checked_sub(1), "راجعت صحة سؤال واحد لحد دلوقتي");
    assert_eq!(today::minutes_short(15), "15 د");
    assert_eq!(today::start_session(10), "ابدأ الجلسة · 10 دقايق");
    assert_eq!(today::start_session(2), "ابدأ الجلسة · دقيقتين");
    assert_eq!(today::minutes_all_label(45), "45 دقيقة للكل تقريبًا");
    assert_eq!(today::minutes_all(5), "دقايق للكل تقريبًا");
    assert_eq!(today::minutes_all(45), "دقيقة للكل تقريبًا");
}

#[test]
fn review_settings_say_how_far_the_schedule_is_personal() {
    assert_eq!(
        study_settings::personal(520),
        format!("{} 520 مراجعة. بيتحدّث كل أسبوع.", study_settings::PERSONAL_1)
    );
    assert_eq!(
        study_settings::default_params(400, 37),
        format!("{} 400 مراجعة. لحد دلوقتي: 37.", study_settings::DEFAULT_PARAMS_1)
    );
    assert_eq!(study::due_elsewhere(1), "فيه كارت واحد مستحق في مقرر تاني.");
    assert_eq!(study::due_elsewhere(2), "فيه كارتين مستحقين في مقررات تانية.");
    assert_eq!(study::due_elsewhere(4), "فيه 4 كروت مستحقة في مقررات تانية.");
    assert_eq!(study::due_elsewhere(40), "فيه 40 كارت مستحق في مقررات تانية.");
}

#[test]
fn counted_nouns_agree_in_every_screen() {
    assert_eq!(abbr::count(1), "اختصار واحد");
    assert_eq!(abbr::count(2), "اختصارين");
    assert_eq!(abbr::count(4), "4 اختصارات");
    assert_eq!(abbr::count_lessons(2), "موجود في درسين:");
    assert_eq!(anki::done(340), format!("{} (340 كارت).", anki::DONE));
    assert_eq!(own::cards(3), "3 كروت");
    assert_eq!(calc::count(44), "44 حاسبة");
    assert_eq!(calc::left(2), "فاضل خانتين");
    assert_eq!(case::steps(5), "5 خطوات");
    assert_eq!(settings::diag_lines(1), "سطر واحد");
    assert_eq!(data::export_saved(1200), format!("{} 1200 حاجة في:", data::EXPORT_SAVED_1));
    assert_eq!(data::import_apply(12), "ادمج 12 تغيير");
    assert_eq!(data::import_done(2), "تمام، اتدمج تغييرين. الجدول اتحدّث.");
    assert!(data::import_partial(3).starts_with("بس فيه 3 صفوف ما اتدمجوش"), "{}", data::import_partial(3));
    assert_eq!(cloud::pending(1), "تغيير واحد مستني يترفع");
    assert_eq!(cloud::pending(2), "تغييرين مستنيين يترفعوا");
    assert_eq!(cloud::pending(7), "7 تغييرات مستنية تترفع");
    assert_eq!(cloud::pending(103), "103 تغييرات مستنية تترفع");
    assert_eq!(cloud::pending(25), format!("25 {}", cloud::PENDING));
    assert_eq!(update::builds_count(1), "نسخة جديدة");
    assert_eq!(update::builds_count(2), "نسختين جداد");
    assert_eq!(update::builds_count(5), "5 نسخ جديدة");
    assert_eq!(update::builds_count(14), "14 نسخة جديدة");
    assert_eq!(update::content_updates_count(1), "تحديث محتوى");
    assert_eq!(update::content_updates_count(2), "تحديثين محتوى");
    assert_eq!(update::content_updates_count(3), "3 تحديثات محتوى");
    assert_eq!(update::content_updates_count(30), "30 تحديث محتوى");
}

#[test]
fn bank_stats_and_plan_lines() {
    assert_eq!(bank::all_of("الكبد"), "كل محاضرات الكبد");
    assert_eq!(bank::checked(true, 'B'), "إجابتك صح.");
    assert_eq!(bank::checked(false, 'C'), "إجابتك غلط. الإجابة الصح C.");
    assert_eq!(stats::better(true, 5), "أحسن من الأسبوع اللي فات بـ 5%");
    assert_eq!(stats::worse(false, 3), "أقل من الشهر اللي فات بـ 3%، عادي يحصل");
    assert_eq!(stats::same(true), "زي الأسبوع اللي فات تقريبًا");
    assert_eq!(plan::tight_cap(200), "لو علّيت حد المراجعات اليومي لـ 200 هتلحق كلها.");
}

#[test]
fn table_order_and_sort_feedback() {
    assert_eq!(table::mark_each(1), "فاضل خانة واحدة: عرفتها ولا لأ؟");
    assert_eq!(table::mark_each(3), "فاضل 3 خانات: علّم على كل واحدة عرفتها ولا لأ.");
    assert_eq!(table::recalled(4, 4), "كل الخانات صح (4 من 4). حلو أوي!");
    assert_eq!(table::recalled(2, 5), "افتكرت 2 من 5 خانات.");
    assert_eq!(order::in_place(3, 6), "3 من 6 في مكانها. الخطوات المعلّمة بالأحمر محتاجة نظرة تانية.");
    assert_eq!(order::should_be(2), "مكانها 2");
    assert_eq!(order::moves_to_fix(1), "الترتيب قريب: نقل خطوة واحدة بس بيظبطه.");
    assert_eq!(order::moves_to_fix(2), "الترتيب قريب: نقل خطوتين بس بيظبطه.");
    assert_eq!(learn::placed_of(2, 5), "اخترت 2 من 5");
    assert_eq!(learn::slot(3), "الخطوة رقم 3");
    assert_eq!(learn::should_be(4), "مكانها رقم 4");
    assert_eq!(learn::sort_left(1, 6), "فاضل 1 من 6");
    assert_eq!(learn::in_place(5, 6), "5 من 6 في مكانهم.");
    assert_eq!(learn::misses(1), "وصّلتهم كلهم، بغلطة واحدة في السكة.");
    assert_eq!(learn::misses(2), "وصّلتهم كلهم، بغلطتين في السكة.");
    assert_eq!(learn::misses(4), "وصّلتهم كلهم، بـ 4 غلطات في السكة.");
    assert_eq!(learn::score_line(5, 5), "ولا غلطة. الدرس ده بقى بتاعك.");
    assert_eq!(learn::score_line(3, 5), "كويس جدًا. اللي غلطت فيه هيرجعلك في المراجعة.");
    assert_eq!(learn::score_line(1, 5), "عادي جدًا من أول مرة، ده درس جديد. الكروت هتثبّته.");
    assert_eq!(
        learn::no_match("ascites"),
        "مفيش درس فيه «ascites». جرّب كلمة تانية أو اسم المحاضرة بالإنجليزي."
    );
    assert_eq!(learn::lecture_n("كبد", "3"), "كبد › المحاضرة 3");
    assert_eq!(learn::lessons_n(1), "درس واحد");
    assert_eq!(learn::lessons_n(2), "درسين");
    assert_eq!(learn::lessons_n(6), "6 دروس");
    assert_eq!(learn::lessons_n(11), "11 درس");
    assert_eq!(learn::done_of(2, 5), "2 من 5 خلّصتهم");
    assert_eq!(learn::resume_at(4, 9), "هتكمّل من خطوة 4 من 9");
    assert_eq!(courses::chip_label("الكلى"), "المقرر المعروض: الكلى. دوس عشان تبدّل");
}

#[test]
fn calculator_texts_refuse_quietly_when_unknown() {
    assert!(calc::impossible("10–200", "mg/dL").contains("\u{2068}10–200 mg/dL\u{2069}"));
    assert!(calc::impossible("1–5", "").contains("\u{2068}1–5\u{2069}"));
    assert!(calc::reason("higher than the serum albumin").is_some());
    assert!(calc::reason("lower than the ascitic albumin").is_some());
    assert_eq!(calc::reason("something else"), None);
    assert!(calc::hint("child_pugh", "inr").is_some());
    assert_eq!(calc::hint("child_pugh", "nope"), None);
    assert!(calc::blurb("meld").is_some());
    assert_eq!(calc::blurb("no_such_calc"), None);
}

#[test]
fn reminders_onboarding_and_recap_lines() {
    assert_eq!(remind::text(None), remind::text(Some(0)));
    assert_eq!(remind::text(Some(1)), "عندك مراجعة واحدة النهارده. دقيقة وتخلص.");
    assert_eq!(remind::text(Some(8)), "عندك 8 مراجعات النهارده. يلا نبدأ بجلسة قصيرة.");
    assert_eq!(onboard::progress(2, 3), "البداية · 2 من 3");
    assert_eq!(onboard::days_sub(7), "كل أيام الأسبوع");
    assert_eq!(onboard::days_sub(1), "يوم واحد في الأسبوع");
    assert_eq!(onboard::days_sub(2), "يومين في الأسبوع");
    assert_eq!(onboard::days_sub(5), "5 أيام في الأسبوع");
    assert_eq!(onboard::studied_sub(0), "ولا محاضرة لسه");
    assert_eq!(onboard::studied_sub(1), "محاضرة واحدة");
    assert_eq!(onboard::studied_sub(2), "محاضرتين");
    assert_eq!(onboard::studied_sub(4), "4 محاضرات");
    assert_eq!(onboard::studied_sub(12), "12 محاضرة");
    assert_eq!(recap::week_learned(10, 3), "اتعلمت 10 حاجات جديدة في 3 أيام.");
    assert_eq!(recap::exam_in(0), "الامتحان النهارده. بالتوفيق.");
    assert_eq!(recap::exam_in(12), "فاضل على الامتحان 12 يوم.");
}
