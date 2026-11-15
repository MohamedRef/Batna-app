//! A release's lines in the "what's new" sheet, grouped by what they are
//! about (#1340): the app first, then one row per course with its count,
//! the courses he studies now first. A release of 300 lines reads as a
//! dozen rows; a line that names a lesson, a lecture or a screen opens it,
//! and closing that brings the sheet back where it was.
//!
//! A release whose lines all sit in one group (or that predates `scope`)
//! keeps the kind cards: new, improved, fixed.

use batna_core::program::CourseStatus;
use batna_core::update::{APP, BuildChanges, Change, ChangeKind, Open};
use dioxus::prelude::*;

use super::changelog::{line, of_kind};
use super::sheet::{SheetMode, UpdateSheetNav, chevron, kind_icon, kind_id, kind_label};
use crate::content::Library;
use crate::i18n::changelog as tx;
use crate::i18n::update as utx;
use crate::{Screen, Ui};

const CSS: crate::look::Css = crate::look::css!("src/update/groups.css");

/// Lines a card or group shows before "Show all"; it folds only past `FOLD_AT`.
const PREVIEW: usize = 5;
const FOLD_AT: usize = 6;
/// A release this short opens every group; a longer one starts folded.
const OPEN_UPTO: usize = 8;

/// One row of a release: the lines of one course, of the app, or of
/// nothing known.
#[derive(Debug, PartialEq)]
pub struct Group<'a> {
    /// A course id, [`APP`], or "" for lines nothing places.
    pub id: String,
    /// New things first, then improvements, then fixes; written order inside.
    pub lines: Vec<&'a Change>,
}

impl Group<'_> {
    /// How many of each kind, zeros left out.
    pub fn tally(&self) -> Vec<(ChangeKind, usize)> {
        ChangeKind::ALL
            .into_iter()
            .map(|k| (k, self.lines.iter().filter(|c| c.kind == k).count()))
            .filter(|(_, n)| *n > 0)
            .collect()
    }
}

/// The groups of `b`: the app, then courses by `rank` (lower first) and
/// the program's order, then lines nothing places.
pub fn groups<'a>(b: &'a BuildChanges, rank: impl Fn(&str) -> (u8, usize)) -> Vec<Group<'a>> {
    let mut out: Vec<Group<'a>> = Vec::new();
    for c in &b.changes {
        let id = c.group().unwrap_or("");
        match out.iter_mut().find(|g| g.id == id) {
            Some(g) => g.lines.push(c),
            None => out.push(Group { id: id.to_owned(), lines: vec![c] }),
        }
    }
    for g in &mut out {
        g.lines.sort_by_key(|c| ChangeKind::ALL.iter().position(|k| *k == c.kind));
    }
    out.sort_by_key(|g| match g.id.as_str() {
        APP => (0, (0, 0)),
        "" => (2, (0, 0)),
        id => (1, rank(id)),
    });
    out
}

/// Courses he focuses on, then the ones he studies now, then the rest; in
/// the program's order inside each.
pub(super) fn course_rank(lib: &Library) -> impl Fn(&str) -> (u8, usize) + '_ {
    let focus = crate::courses::focus::current();
    move |id: &str| {
        let pos = lib.program.courses.iter().position(|c| c.id == id).unwrap_or(usize::MAX);
        let status = lib.program.course(id).map(crate::courses::status);
        let first = if focus.as_deref() == Some(id) {
            0
        } else if status == Some(CourseStatus::Active) {
            1
        } else {
            2
        };
        (first, pos)
    }
}

/// A group's name: the course's short Arabic name (`كبد`).
pub(super) fn group_name(lib: &Library, id: &str) -> String {
    match id {
        APP => tx::GROUP_APP.to_owned(),
        "" => tx::GROUP_OTHER.to_owned(),
        id => lib.program.course(id).map_or(id, |c| c.short.as_str()).to_owned(),
    }
}

/// Where a line's tap goes, when this build knows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Lesson(String),
    Lecture(String),
    Screen(Screen),
}

pub fn target(lib: &Library, c: &Change) -> Option<Target> {
    match Open::parse(c.open.as_deref()?)? {
        Open::Lesson(id) => lib.lesson(id).map(|_| Target::Lesson(id.to_owned())),
        Open::Lecture(id) => {
            lib.index.lectures.iter().any(|e| e.id == id).then(|| Target::Lecture(id.to_owned()))
        }
        Open::Screen(name) => [Screen::NAV.as_slice(), &[Screen::Settings]]
            .concat()
            .into_iter()
            .find(|s| format!("{s:?}").eq_ignore_ascii_case(name))
            .map(Target::Screen),
    }
}

/// The sheet to bring back once what a line opened is closed.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Parked {
    mode: SheetMode,
    /// The screen the sheet sat over: closing the lesson or lecture comes
    /// back to it; leaving for another screen drops the sheet.
    screen: Screen,
    /// Settings was open under the sheet (Settings › Updates › changelog).
    settings: bool,
    lesson: bool,
}

static PARKED: GlobalSignal<Option<Parked>> = Signal::global(|| None);

/// Opens `t` and puts the sheet away until it's closed.
fn follow(t: &Target) {
    let (Some(ui), Some(mut nav)) = (try_consume_context::<Ui>(), try_consume_context::<UpdateSheetNav>())
    else {
        return;
    };
    let settings = try_consume_context::<crate::settings::SettingsNav>();
    let Some(mode) = *nav.0.peek() else { return };
    let under_settings = settings.is_some_and(|s| *s.open.peek());
    let screen = *ui.screen.peek();
    if let Some(s) = settings {
        s.close();
    }
    nav.0.set(None);
    let lesson = match t {
        Target::Lesson(id) => {
            crate::learn::open(ui, id);
            true
        }
        Target::Lecture(id) => {
            crate::lecture::open(ui, id);
            false
        }
        Target::Screen(s) => {
            let mut ui = ui;
            match s {
                Screen::Settings => {
                    if let Some(s) = settings {
                        s.show(None);
                    }
                }
                s => ui.screen.set(*s),
            }
            return;
        }
    };
    *PARKED.write() = Some(Parked { mode, screen, settings: under_settings, lesson });
}

/// Brings the sheet back once the lesson or lecture a line opened is
/// closed; called by the sheet, so it lives as long as the shell.
pub(super) fn use_return(mut mode: Signal<Option<SheetMode>>) {
    let ui = try_use_context::<Ui>();
    let settings = try_use_context::<crate::settings::SettingsNav>();
    use_effect(move || {
        let lesson_open = crate::learn::OPEN.read().is_some();
        let lecture_open = crate::lecture::is_open();
        let screen = ui.map(|u| *u.screen.read());
        // A closed sheet forgets its search (not while a lesson it opened is up).
        let parked = PARKED.peek().is_some();
        if mode.read().is_none() && !parked {
            super::filter::clear();
        }
        let Some(p) = *PARKED.peek() else { return };
        // Opened before it was parked, so "not open" means closed.
        let open = if p.lesson { lesson_open && screen == Some(Screen::Learn) } else { lecture_open };
        if open {
            return;
        }
        *PARKED.write() = None;
        if screen == Some(p.screen) && mode.peek().is_none() {
            if p.settings
                && let Some(s) = settings
            {
                s.show(Some(crate::settings::Section::Updates));
            }
            mode.set(Some(p.mode));
        }
    });
}

/// An arrow towards the reading end: the line opens something.
fn go_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M9 6l6 6-6 6" }
        }
    }
}

/// One line: plain, or a button when it opens something.
pub(super) fn line_item(lib: &Library, c: &Change, key: usize) -> Element {
    let (text, latin) = line(c);
    let kind = kind_id(c.kind);
    let words = rsx! {
        if latin {
            bdi { dir: "ltr", class: "latin", "{text}" }
        } else {
            "{text}"
        }
    };
    match target(lib, c) {
        Some(t) => rsx! {
            li { key: "{key}", "data-kind": "{kind}", class: "us-linked",
                button { class: "us-link", aria_label: "{tx::OPEN}: {text}", onclick: move |_| follow(&t),
                    span { class: "us-link-text", {words} }
                    span { class: "us-go-icon", {go_icon()} }
                }
            }
        },
        None => rsx! {
            li { key: "{key}", "data-kind": "{kind}", {words} }
        },
    }
}

/// One sentence for a release spread over several courses: how many
/// changes, over how many courses, and where most of them are.
pub fn digest(lib: &Library, total: usize, groups: &[Group<'_>]) -> Option<String> {
    let courses: Vec<&Group<'_>> = groups.iter().filter(|g| g.id != APP && !g.id.is_empty()).collect();
    if courses.len() < 2 {
        return None;
    }
    let app = groups.iter().any(|g| g.id == APP);
    let top = courses.iter().max_by_key(|g| (g.lines.len(), std::cmp::Reverse(g.id.as_str())))?;
    let changes = crate::i18n::count::count(total, &tx::CHANGE);
    let over = crate::i18n::count::count(courses.len(), &tx::COURSE);
    let and_app = if app { tx::AND_APP } else { "" };
    Some(tx::digest(&changes, &over, and_app, &group_name(lib, &top.id), top.lines.len()))
}

/// Lessons he finished before the day `b` came out that its lines open
/// (a lesson, or every finished lesson of a lecture), in order, once each.
/// `done` is when a lesson was finished, if it was.
pub fn finished_changed(lib: &Library, b: &BuildChanges, done: impl Fn(&str) -> Option<i64>) -> Vec<String> {
    let Some(day) = super::changelog::day_of(&b.published_at) else { return Vec::new() };
    let before = |id: &str| done(id).is_some_and(|ms| ms.div_euclid(86_400_000) < day);
    let mut out: Vec<String> = Vec::new();
    for c in &b.changes {
        let ids: Vec<String> = match target(lib, c) {
            Some(Target::Lesson(id)) => vec![id],
            Some(Target::Lecture(id)) => lib
                .lessons
                .get(&id)
                .map(|f| f.lessons.iter().map(|l| l.id.clone()).collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        for id in ids {
            if before(&id) && !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

/// What leads an open release: lessons he finished that it changed, then
/// the digest of a release over several courses.
fn release_head(lib: &Library, b: &BuildChanges, groups: &[Group<'_>]) -> Element {
    let changed = finished_changed(lib, b, |id| crate::learn::mark(id).map(|m| m.done_ms));
    let digest = digest(lib, b.changes.len(), groups);
    rsx! {
        if let Some(d) = digest {
            p { class: "us-digest", "{d}" }
        }
        if !changed.is_empty() {
            section { class: "us-changed",
                h3 { "{tx::finished_changed(changed.len())}" }
                ul { class: "us-items",
                    for id in changed {
                        {
                            let title = lib.lesson(&id).map_or(id.clone(), |(_, l)| l.title.clone());
                            let crumb = lib.lecture_tag(&id);
                            let t = Target::Lesson(id.clone());
                            rsx! {
                                li { key: "{id}", class: "us-linked",
                                    button { class: "us-link", aria_label: "{tx::OPEN}: {title}", onclick: move |_| follow(&t),
                                        span { class: "us-link-text",
                                            span { class: "us-crumb", "{crumb} · " }
                                            "{title}"
                                        }
                                        span { class: "us-go-icon", {go_icon()} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The open part of a release on the timeline: kind cards, or course rows
/// when its lines span more than one group.
#[component]
pub(super) fn ReleaseBody(release: BuildChanges) -> Element {
    let lib = Library::get();
    let key = release.key();
    let flipped = use_signal(Vec::<String>::new);
    let all = use_signal(Vec::<String>::new);
    let toggle = move |mut list: Signal<Vec<String>>, k: String| {
        let mut v = list.write();
        match v.iter().position(|x| *x == k) {
            Some(i) => {
                v.remove(i);
            }
            None => v.push(k),
        }
    };
    let groups = groups(&release, course_rank(lib));
    let start_open = release.changes.len() <= OPEN_UPTO;
    let head = release_head(lib, &release, &groups);

    if groups.len() < 2 {
        return rsx! {
            style { {CSS.text()} }
            {head}
            div { class: "us-kinds",
                for k in ChangeKind::ALL {
                    {
                        let items = of_kind(&release, k);
                        let kkey = format!("{key}:{}", kind_id(k));
                        let shows_all = all.read().contains(&kkey);
                        let folds = items.len() > FOLD_AT;
                        let n = items.len();
                        let all_label = utx::SHOW_ALL.replace("{n}", &n.to_string());
                        let show: Vec<&Change> =
                            items.into_iter().take(if folds && !shows_all { PREVIEW } else { usize::MAX }).collect();
                        rsx! {
                            if n > 0 {
                                section { key: "{kkey}", class: "us-kind", "data-kind": "{kind_id(k)}",
                                    div { class: "us-kind-head",
                                        span { class: "us-kind-icon", {kind_icon(k)} }
                                        h3 { "{kind_label(k)}" }
                                        span { class: "us-chip", "{n}" }
                                    }
                                    ul { class: "us-items",
                                        for (j, c) in show.into_iter().enumerate() {
                                            {line_item(lib, c, j)}
                                        }
                                    }
                                    if folds {
                                        button { class: "us-more", onclick: move |_| toggle(all, kkey.clone()),
                                            if shows_all { "{utx::SHOW_LESS}" } else { "{all_label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        style { {CSS.text()} }
        {head}
        div { class: "us-groups",
            for g in groups {
                {
                    let gkey = format!("{key}:{}", g.id);
                    let open = start_open != flipped.read().contains(&gkey);
                    let shows_all = all.read().contains(&gkey);
                    let n = g.lines.len();
                    let folds = n > FOLD_AT;
                    let all_label = utx::SHOW_ALL.replace("{n}", &n.to_string());
                    let tally = g.tally();
                    let name = group_name(lib, &g.id);
                    let show: Vec<&Change> =
                        g.lines.iter().copied().take(if folds && !shows_all { PREVIEW } else { usize::MAX }).collect();
                    let (head_key, more_key) = (gkey.clone(), gkey.clone());
                    let group_id = if g.id.is_empty() { "other".to_owned() } else { g.id.clone() };
                    rsx! {
                        section { key: "{gkey}", class: "us-group", "data-group": "{group_id}",
                            button {
                                class: "us-group-head",
                                aria_expanded: if open { "true" } else { "false" },
                                onclick: move |_| toggle(flipped, head_key.clone()),
                                h3 { class: "us-group-name", "{name}" }
                                span { class: "us-group-kinds",
                                    for (k, c) in tally {
                                        span { key: "{kind_id(k)}", class: "us-mini", "data-kind": "{kind_id(k)}",
                                            title: "{kind_label(k)}",
                                            {kind_icon(k)}
                                            "{c}"
                                        }
                                    }
                                }
                                span { class: "us-chip", "{n}" }
                                span { class: "us-chevron", {chevron()} }
                            }
                            if open {
                                ul { class: "us-items",
                                    for (j, c) in show.into_iter().enumerate() {
                                        {line_item(lib, c, j)}
                                    }
                                }
                                if folds {
                                    button { class: "us-more", onclick: move |_| toggle(all, more_key.clone()),
                                        if shows_all { "{utx::SHOW_LESS}" } else { "{all_label}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use batna_core::update::UpdateNotes;

    use super::*;

    fn c(kind: ChangeKind, ar: &str, scope: Option<&str>, open: Option<&str>) -> Change {
        Change {
            scope: scope.map(str::to_owned),
            open: open.map(str::to_owned),
            ..Change::new(kind, UpdateNotes { en: String::new(), ar_eg: ar.into() })
        }
    }

    fn build(changes: Vec<Change>) -> BuildChanges {
        BuildChanges {
            version: "1.0.0".into(),
            build: 9,
            published_at: "2026-11-15T00:00:00Z".into(),
            changes,
            content_only: false,
        }
    }

    #[test]
    fn the_app_comes_first_then_courses_by_rank_then_the_rest() {
        use ChangeKind::*;
        let b = build(vec![
            c(Fixed, "كبد ١", Some("hb"), None),
            c(New, "قديم", None, None),
            c(New, "دم", None, Some("hem.L02.lesson1")),
            c(New, "كبد ٢", Some("hb"), None),
            c(Improved, "بنك", Some("bank"), None),
            c(Fixed, "سينك", Some("sync"), None),
        ]);
        // hem ranks before hb (say, the course in focus).
        let rank = |id: &str| if id == "hem" { (0, 1) } else { (1, 0) };
        let got: Vec<(String, Vec<&str>)> = groups(&b, rank)
            .into_iter()
            .map(|g| (g.id, g.lines.iter().map(|l| l.text.ar_eg.as_str()).collect()))
            .collect();
        assert_eq!(
            got,
            [
                ("app".to_owned(), vec!["بنك", "سينك"]),
                ("hem".to_owned(), vec!["دم"]),
                ("hb".to_owned(), vec!["كبد ٢", "كبد ١"]),
                (String::new(), vec!["قديم"]),
            ]
        );
        let hb = groups(&b, rank).remove(2);
        assert_eq!(hb.tally(), [(New, 1), (Fixed, 1)]);
    }

    #[test]
    fn a_release_over_several_courses_gets_one_sentence() {
        use ChangeKind::*;
        let lib = Library::get();
        let b = build(vec![
            c(New, "١", Some("hb"), None),
            c(New, "٢", Some("hb"), None),
            c(Fixed, "٣", Some("hem"), None),
            c(Fixed, "٤", Some("bank"), None),
        ]);
        let g = groups(&b, |_| (0, 0));
        let short = |id: &str| lib.program.course(id).unwrap().short.clone();
        assert_eq!(
            digest(lib, 4, &g).unwrap(),
            format!("4 تغييرات في مادتين والتطبيق؛ الأكتر: {} (2)", short("hb"))
        );
        let one = build(vec![c(New, "١", Some("hb"), None), c(New, "٢", Some("bank"), None)]);
        assert_eq!(digest(lib, 2, &groups(&one, |_| (0, 0))), None, "one course needs no summary");
    }

    #[test]
    fn finished_lessons_a_release_changed_are_named_once() {
        let lib = Library::get();
        let (lecture, file) = lib.lessons.iter().find(|(_, f)| f.lessons.len() >= 2).unwrap();
        let (first, second) = (file.lessons[0].id.clone(), file.lessons[1].id.clone());
        let mut b = build(vec![
            c(ChangeKind::New, "س", None, Some(&second)),
            c(ChangeKind::New, "ص", None, Some(lecture)),
            c(ChangeKind::New, "ع", Some("hb"), None),
        ]);
        b.published_at = "2026-11-15T05:00:00Z".into();
        let day = super::super::changelog::day_of(&b.published_at).unwrap() * 86_400_000;
        // The first was finished the day before, the second a week before;
        // the rest of the lecture not at all.
        let done = |id: &str| {
            if id == first {
                Some(day - 3_600_000)
            } else if id == second {
                Some(day - 7 * 86_400_000)
            } else {
                None
            }
        };
        assert_eq!(finished_changed(lib, &b, done), [second.clone(), first.clone()]);
        // Finished on the day it came out (maybe after): not named.
        assert_eq!(finished_changed(lib, &b, |id| (id == first).then_some(day + 1)), Vec::<String>::new());
    }

    #[test]
    fn a_line_opens_only_what_this_build_knows() {
        let lib = Library::get();
        let lesson =
            lib.lessons.values().next().and_then(|f| f.lessons.first()).map(|l| l.id.clone()).unwrap();
        let lecture = lib.index.lectures[0].id.clone();
        let t = |open: &str| target(lib, &c(ChangeKind::New, "س", None, Some(open)));
        assert_eq!(t(&lesson), Some(Target::Lesson(lesson.clone())));
        assert_eq!(t(&lecture), Some(Target::Lecture(lecture.clone())));
        assert_eq!(t("bank"), Some(Target::Screen(Screen::Bank)));
        assert_eq!(t("settings"), Some(Target::Screen(Screen::Settings)));
        assert_eq!(t("hb.L99.lesson1"), None);
        assert_eq!(t("zz.L01"), None);
        assert_eq!(target(lib, &c(ChangeKind::New, "س", Some("hb"), None)), None);
    }

    /// Lets the sheet's effects run, then draws.
    fn flush(t: &mut crate::tap_dom::TestDom) {
        let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
        rt.block_on(async {
            for _ in 0..5 {
                let wait = tokio::time::timeout(std::time::Duration::from_millis(20), t.dom.wait_for_work());
                if wait.await.is_err() {
                    break;
                }
                t.settle();
            }
        });
    }

    #[test]
    fn a_line_opens_its_lesson_and_closing_it_brings_the_sheet_back() {
        use super::super::sheet::SheetView;
        use super::super::tests::status;
        use super::super::{Mode, Phase};

        fn first_lesson() -> String {
            let lib = Library::get();
            lib.lessons.values().next().and_then(|f| f.lessons.first()).map(|l| l.id.clone()).unwrap()
        }
        fn app() -> Element {
            use_context_provider(|| Ui {
                screen: Signal::new(Screen::Today),
                prefs: Signal::new(crate::prefs::DevicePrefs::default()),
                focus: Signal::new(false),
            });
            use_context_provider(|| UpdateSheetNav(Signal::new(Some(SheetMode::History))));
            let settings = use_context_provider(crate::settings::SettingsNav::new);
            use_hook(|| settings.show(Some(crate::settings::Section::Updates)));
            let status = use_signal(|| {
                let mut s = status(Phase::UpToDate, Mode::Auto);
                s.history = vec![build(vec![
                    c(ChangeKind::New, "صورة جديدة في الدرس", None, Some(&first_lesson())),
                    c(ChangeKind::Fixed, "البنك بقى أسرع", Some("bank"), None),
                ])];
                Some(s)
            });
            rsx! { SheetView { status } }
        }
        let mut t = crate::tap_dom::TestDom::of(app);
        let page = t.html();
        assert!(page.contains("us-group") && page.contains(tx::GROUP_APP), "{page}");
        assert!(page.find(tx::GROUP_APP) < page.find("صورة جديدة"), "the app comes first: {page}");
        assert!(!page.contains(&format!("{}: البنك", tx::OPEN)), "a line with no target is plain: {page}");

        flush(&mut t);
        t.click(&format!("{}: صورة جديدة", tx::OPEN));
        flush(&mut t);
        assert!(!t.html().contains("role=\"dialog\""), "the sheet steps aside: {}", t.html());
        let (open, screen, settings) = t.with(|| {
            let s = consume_context::<crate::settings::SettingsNav>();
            (crate::learn::OPEN.peek().clone(), *consume_context::<Ui>().screen.peek(), *s.open.peek())
        });
        assert_eq!((open, screen, settings), (Some(first_lesson()), Screen::Learn, false));

        // Closing the lesson comes back to Today: the sheet, over Settings, returns.
        t.with(|| crate::learn::close(consume_context::<Ui>()));
        flush(&mut t);
        assert!(t.html().contains("role=\"dialog\""), "{}", t.html());
        let settings = t.with(|| *consume_context::<crate::settings::SettingsNav>().open.peek());
        assert!(settings, "Settings › Updates is back under it");
    }

    #[test]
    fn leaving_for_another_screen_drops_the_sheet() {
        use super::super::sheet::SheetView;
        use super::super::tests::status;
        use super::super::{Mode, Phase};

        fn app() -> Element {
            use_context_provider(|| Ui {
                screen: Signal::new(Screen::Today),
                prefs: Signal::new(crate::prefs::DevicePrefs::default()),
                focus: Signal::new(false),
            });
            use_context_provider(|| UpdateSheetNav(Signal::new(Some(SheetMode::History))));
            let status = use_signal(|| {
                let lesson = Library::get().lessons.values().next().unwrap().lessons[0].id.clone();
                let mut s = status(Phase::UpToDate, Mode::Auto);
                s.history = vec![build(vec![
                    c(ChangeKind::New, "صورة جديدة", None, Some(&lesson)),
                    c(ChangeKind::New, "شاشة الإحصائيات", Some("stats"), Some("stats")),
                ])];
                Some(s)
            });
            rsx! { SheetView { status } }
        }
        let mut t = crate::tap_dom::TestDom::of(app);
        flush(&mut t);
        t.click(&format!("{}: صورة جديدة", tx::OPEN));
        flush(&mut t);
        t.with(|| consume_context::<Ui>().screen.set(Screen::Bank));
        flush(&mut t);
        t.with(|| *crate::learn::OPEN.write() = None);
        flush(&mut t);
        assert!(!t.html().contains("role=\"dialog\""), "{}", t.html());
        assert_eq!(t.with(|| *PARKED.peek()), None);

        // A screen target just goes there.
        t.with(|| consume_context::<UpdateSheetNav>().open(SheetMode::History));
        t.settle();
        t.click(&format!("{}: شاشة الإحصائيات", tx::OPEN));
        assert_eq!(t.with(|| *consume_context::<Ui>().screen.peek()), Screen::Stats);
        assert!(!t.html().contains("role=\"dialog\""));
    }
}
