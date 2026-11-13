use super::*;

/// Mohamed's screenshot read "7 ساعة بتذاكرها" (2026-11-02).
#[test]
fn course_map_lines_agree() {
    assert_eq!(courses::hours_now("7"), "7 ساعات بتذاكرها الترم ده");
    assert_eq!(courses::hours_now("2"), "ساعتين بتذاكرها الترم ده");
    assert_eq!(courses::hours_now("12"), "12 ساعة بتذاكرها الترم ده");
    assert_eq!(courses::hours_done("0"), "0 ساعة خلصت امتحانها");
    assert_eq!(courses::hours_line("42", 9), "42 ساعة معتمدة في 9 مقررات");
    assert_eq!(courses::hours_line("6", 2), "6 ساعات معتمدة في مقررين");
    assert_eq!(courses::all_sub(9), "9 مقررات مع بعض");
    assert_eq!(courses::all_sub(12), "12 مقرر مع بعض");
    assert_eq!(courses::lessons_done(3, 7), "3 من 7 دروس");
    assert_eq!(courses::lessons_done(3, 30), "3 من 30 درس");
    assert_eq!(courses::lectures_only(5), "5 محاضرات، الدروس جاية");
    assert_eq!(courses::ready_lectures(1, 2), "مكتوب 1 من محاضرتين");
    assert_eq!(courses::content_line(3, 4, 120, 6), "مكتوب 3 من 4 محاضرات · 120 كارت · 6 دروس");
}

#[test]
fn other_counted_lines_agree() {
    assert_eq!(today::exam_in(5), "فاضل 5 أيام على الامتحان");
    assert_eq!(today::exam_in(40), "فاضل 40 يوم على الامتحان");
    assert_eq!(today::plan_line(3, 4), "3 جديد + 4 مراجعات");
    assert_eq!(stats::day_bar(5, 2), "5 دقايق، إجابتين");
    assert_eq!(table::mark_each(2), "فاضل خانتين: علّم على كل واحدة عرفتها ولا لأ.");
    assert_eq!(table::recalled(1, 12), "افتكرت 1 من 12 خانة.");
    assert_eq!(order::moves_to_fix(2), "الترتيب قريب: نقل خطوتين بس بيظبطه.");
    assert_eq!(learn::misses(2), "وصّلتهم كلهم، بغلطتين في السكة.");
    assert_eq!(learn::misses(4), "وصّلتهم كلهم، بـ 4 غلطات في السكة.");
    assert_eq!(update::notice_more(1), "وحاجة واحدة كمان");
    assert_eq!(update::notice_more(4), "و4 حاجات كمان");
    assert_eq!(cloud::pending(2), "تغييرين مستنيين يترفعوا");
    assert_eq!(cloud::pending(14), "14 تغيير مستني يترفع");
    assert_eq!(data::import_partial(3).split(" (").next().unwrap(), "بس فيه 3 صفوف ما اتدمجوش");
    assert_eq!(study::enum_goal(8, 5), "فيه 8 نقط، ولو افتكرت 5 منهم تعدّي.");
    assert_eq!(plan::days_left(40), "فاضل 40 يوم مذاكرة");
    assert_eq!(anki::contents(12, 2), "12 محاضرة · سؤالين");
}

/// The catch-up plan and weak-fact lines came in after the sweep above
/// (#801): «2 أيام» and «حوالي 5 دقيقة» on screen.
#[test]
fn catchup_and_weak_lines_agree() {
    assert_eq!(catchup::days(1), "يوم");
    assert_eq!(catchup::days(2), "يومين");
    assert_eq!(catchup::days(5), "5 أيام");
    assert_eq!(catchup::days(14), "14 يوم");
    assert_eq!(catchup::offer_line(1, 1), "اتجمّع عندك مراجعة واحدة، حوالي دقيقة. عادي، بيحصل، ونقسمهم على كام يوم.");
    assert_eq!(catchup::offer_line(120, 2), "اتجمّع عندك 120 مراجعة، حوالي دقيقتين. عادي، بيحصل، ونقسمهم على كام يوم.");
    assert_eq!(catchup::offer_line(5, 5), "اتجمّع عندك 5 مراجعات، حوالي 5 دقايق. عادي، بيحصل، ونقسمهم على كام يوم.");
    assert_eq!(catchup::per_day(8), "يعني حوالي 8 دقايق مراجعة في اليوم، الأقدم الأول.");
    assert_eq!(catchup::per_day(14), "يعني حوالي 14 دقيقة مراجعة في اليوم، الأقدم الأول.");
    assert_eq!(catchup::today_left(2, 4), "فاضل النهارده مراجعتين، حوالي 4 دقايق.");
    assert_eq!(catchup::today_left(14, 1), "فاضل النهارده 14 مراجعة، حوالي دقيقة.");
    assert_eq!(weak::questions(1), "وفيه سؤال واحد من الصفحة دي محتاج تبص عليه تاني.");
    assert_eq!(weak::questions(2), "وفيه سؤالين من الصفحة دي محتاجين تبص عليهم تاني.");
    assert_eq!(weak::questions(5), "وفيه 5 أسئلة من الصفحة دي محتاجين تبص عليهم تاني.");
    assert_eq!(weak::questions(14), "وفيه 14 سؤال من الصفحة دي محتاج تبص عليه تاني.");
    assert_eq!(weak::study(2), "ذاكر كارتين دلوقتي");
    assert_eq!(weak::study(5), "ذاكر 5 كروت دلوقتي");
}
