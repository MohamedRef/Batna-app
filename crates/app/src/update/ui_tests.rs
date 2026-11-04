//! What Settings › Updates, the banner, the Today note and the "what's
//! new" sheet show for each state of the updater (Dioxus SSR).

use batna_core::update::{BuildChanges, Change, ChangeKind, Channel, UpdateNotes};
use dioxus::prelude::*;

use super::sheet::{SheetMode, SheetView, UpdateSheetNav};
use super::tests::{offer, status};
use super::view::{BannerView, ContentNoteView, UpdatesView};
use super::*;
use crate::i18n::update as tx;

#[derive(Clone, Copy, PartialEq)]
enum Which {
    Section,
    Banner,
    Note,
    Sheet(SheetMode),
}

#[derive(Clone)]
struct Case {
    status: Option<Status>,
    which: Which,
    /// Provide the sheet's navigation (the shell always does).
    nav: bool,
}

fn root(c: Case) -> Element {
    let status = use_signal(|| c.status.clone());
    let open = match c.which {
        Which::Sheet(m) => Some(m),
        _ => None,
    };
    let nav = use_hook(|| UpdateSheetNav(Signal::new(open)));
    if c.nav {
        use_context_provider(|| nav);
    }
    match c.which {
        Which::Section => rsx! { UpdatesView { status } },
        Which::Banner => rsx! { BannerView { status } },
        Which::Note => rsx! { ContentNoteView { status } },
        Which::Sheet(_) => rsx! { SheetView { status } },
    }
}

fn html(which: Which, status: Option<Status>) -> String {
    render(Case { status, which, nav: true })
}

fn render(case: Case) -> String {
    let mut dom = VirtualDom::new_with_props(root, case);
    dom.rebuild_in_place();
    without_styles(&dioxus::ssr::render(&dom))
}

/// The markup alone: the inline stylesheets mention classes and states too.
fn without_styles(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find("<style>") {
        out.push_str(&rest[..at]);
        rest = rest[at..].split_once("</style>").map_or("", |(_, r)| r);
    }
    out.push_str(rest);
    out
}

/// Whether the page shows `text` (SSR escapes quotes).
fn has(page: &str, text: &str) -> bool {
    page.contains(&text.replace('"', "&#34;"))
}

fn change(kind: ChangeKind, ar: &str) -> Change {
    Change { kind, text: UpdateNotes { en: String::new(), ar_eg: ar.into() } }
}

fn build_changes(build: u64, changes: Vec<Change>, content_only: bool) -> BuildChanges {
    BuildChanges {
        version: "0.1.0".into(),
        build,
        published_at: "2026-11-01T10:00:00Z".into(),
        changes,
        content_only,
    }
}

fn with_changes(mut o: Offer, changes: Vec<BuildChanges>) -> Offer {
    o.changes = changes;
    o
}

#[test]
fn a_build_without_updates_says_why_and_offers_no_buttons() {
    let page = html(Which::Section, None);
    assert!(has(&page, tx::OFF_DEV), "no updater counts as a dev build: {page}");
    assert!(!has(&page, "btn-primary"), "no Check now: {page}");
    for (off, line) in [
        (Off::NoKey, tx::OFF_KEY),
        (Off::NoPlatform, tx::OFF_PLATFORM),
        (Off::NotInBundle, tx::OFF_BUNDLE),
        (Off::DevBuild, tx::OFF_DEV),
    ] {
        let mut s = status(Phase::Idle, Mode::Auto);
        s.off = Some(off);
        let page = html(Which::Section, Some(s));
        assert!(has(&page, line), "{off:?}: {page}");
        assert!(!has(&page, tx::MODE), "no mode picker while off: {page}");
    }
}

#[test]
fn the_section_shows_the_app_row_the_checks_and_every_choice() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.channel = Channel::Preview;
    let page = html(Which::Section, Some(s.clone()));
    for text in [
        tx::GROUP_HAVE,
        tx::ROW_APP,
        tx::ROW_CONTENT,
        tx::UP_TO_DATE,
        tx::NEVER_CHECKED,
        tx::CHECK_NOW,
        tx::HISTORY,
        tx::MODE,
        tx::NOTIFY,
        tx::CHANNEL,
        tx::PREVIEW_CHANNEL_NOTE,
    ] {
        assert!(has(&page, text), "missing {text}: {page}");
    }
    for (_, (title, sub)) in Mode::ALL.iter().map(|m| (m, mode_text(*m))) {
        assert!(has(&page, title) && has(&page, sub));
    }
    for label in [tx::PREVIEW, tx::ALPHA, tx::BETA, tx::STABLE] {
        assert!(has(&page, label), "{label}");
    }
    assert!(has(&page, "✓ "), "up to date carries a tick");
    // Nothing offered: no "what's new?" button.
    assert!(!has(&page, tx::WHATS_NEW_ASK));
    // The mode and the channel picked are the checked radios.
    assert_eq!(page.matches("aria-checked=\"true\"").count(), 2, "{page}");

    // "Only when I check" hides the notification switch; alpha hides the
    // preview note.
    s.prefs.mode = Mode::Off;
    s.channel = Channel::Alpha;
    s.prefs.last_check_ms = Some(now_ms());
    let page = html(Which::Section, Some(s));
    assert!(!has(&page, tx::NOTIFY), "{page}");
    assert!(!has(&page, tx::PREVIEW_CHANNEL_NOTE));
    assert!(has(&page, tx::LAST_CHECK) && !has(&page, tx::NEVER_CHECKED));
}

fn mode_text(m: Mode) -> (&'static str, &'static str) {
    match m {
        Mode::Auto => (tx::MODE_AUTO, tx::MODE_AUTO_SUB),
        Mode::Notify => (tx::MODE_NOTIFY, tx::MODE_NOTIFY_SUB),
        Mode::Off => (tx::MODE_OFF, tx::MODE_OFF_SUB),
    }
}

#[test]
fn without_the_sheet_the_rows_cannot_open_it() {
    let s = status(Phase::Available(offer("0.2.0", 0)), Mode::Auto);
    let page = render(Case { status: Some(s), which: Which::Section, nav: false });
    assert_eq!(page.matches("disabled=true").count(), 2, "both rows disabled: {page}");
    assert!(!has(&page, tx::HISTORY), "{page}");
}

#[test]
fn each_phase_has_its_headline_and_its_button() {
    let o = || offer("0.2.0", 0);
    let cases: Vec<(Phase, &str, Option<&str>)> = vec![
        (Phase::Checking, tx::CHECKING, None),
        (Phase::NotForThisDevice, tx::NOT_FOR_DEVICE, None),
        (Phase::Available(o()), tx::AVAILABLE, Some(tx::DOWNLOAD)),
        (Phase::Downloading { offer: o(), done: 0 }, tx::DOWNLOADING, None),
        (Phase::Ready(o()), tx::READY, Some(tx::INSTALL_NOW)),
        (Phase::Installing(o()), tx::INSTALLING, None),
        (Phase::NeedsPermission(o()), tx::NEEDS_PERMISSION, Some(tx::OPEN_PERMISSION)),
        (Phase::Installed(o()), tx::INSTALLED, Some(tx::RESTART_NOW)),
        (Phase::Failed { offer: None, why: Failure::TooNew }, tx::TOO_NEW, None),
        (Phase::Failed { offer: Some(o()), why: Failure::Network }, tx::FAILED_NETWORK, Some(tx::TRY_AGAIN)),
        (
            Phase::Failed { offer: Some(o()), why: Failure::Verify("sha mismatch".into()) },
            tx::FAILED_VERIFY,
            Some(tx::TRY_AGAIN),
        ),
        (
            Phase::Failed { offer: Some(o()), why: Failure::Install("refused".into()) },
            tx::FAILED_INSTALL,
            Some(tx::TRY_AGAIN),
        ),
    ];
    for (phase, headline, button) in cases {
        let name = format!("{phase:?}");
        let page = html(Which::Section, Some(status(phase.clone(), Mode::Notify)));
        assert!(has(&page, headline), "{name}: {page}");
        if let Some(b) = button {
            assert!(has(&page, b), "{name} needs {b}: {page}");
        }
        let failed = matches!(phase, Phase::Failed { .. });
        let shows_label = matches!(&phase, Phase::Available(_) | Phase::Ready(_)) && !failed;
        if shows_label {
            assert!(has(&page, "0.2.0"), "{name}: the offer's version: {page}");
        }
    }
    // A failure keeps its detail, apart and left-to-right.
    let page = html(
        Which::Section,
        Some(status(
            Phase::Failed { offer: Some(o()), why: Failure::Verify("sha mismatch".into()) },
            Mode::Auto,
        )),
    );
    assert!(has(&page, "sha mismatch") && page.contains("dir=\"ltr\""), "{page}");
    // Busy phases lock "Check now" and the channels.
    let page = html(Which::Section, Some(status(Phase::Checking, Mode::Auto)));
    assert!(page.matches("disabled=true").count() >= 5, "{page}");
}

#[test]
fn a_download_shows_its_size_its_progress_and_what_it_brings() {
    let mut o = offer("0.2.0", 0);
    o.asset.size = 3 * 1024 * 1024;
    o.notes = UpdateNotes { en: "Faster cards".into(), ar_eg: String::new() };
    let page = html(Which::Section, Some(status(Phase::Available(o.clone()), Mode::Notify)));
    assert!(has(&page, "3.0 ") && has(&page, tx::MB), "{page}");
    assert!(has(&page, tx::WHATS_NEW) && has(&page, "Faster cards"), "{page}");
    assert!(has(&page, tx::WHATS_NEW_ASK), "an offer opens the sheet: {page}");

    let page =
        html(Which::Section, Some(status(Phase::Downloading { offer: o.clone(), done: 1024 }, Mode::Auto)));
    assert!(has(&page, "<progress") && page.contains("value=\"1024\""), "{page}");
    assert!(page.contains(&format!("max=\"{}\"", 3 * 1024 * 1024)), "{page}");

    // Changelog lines replace the old notes, without the "What's new:" label.
    let o = with_changes(
        o,
        vec![build_changes(
            50,
            vec![change(ChangeKind::New, "بطاقات أسرع"), change(ChangeKind::Fixed, "زرار")],
            false,
        )],
    );
    let page = html(Which::Section, Some(status(Phase::Ready(o), Mode::Auto)));
    assert!(has(&page, "بطاقات أسرع"), "{page}");
    assert!(!has(&page, "Faster cards"));

    let page = html(Which::Section, Some(status(Phase::Installed(offer("0.2.0", 0)), Mode::Auto)));
    assert!(has(&page, tx::NEXT_LAUNCH), "{page}");
}

#[test]
fn a_renewed_preview_key_lists_the_steps_to_reinstall() {
    let s = status(Phase::Failed { offer: None, why: Failure::KeyChanged }, Mode::Auto);
    let page = html(Which::Section, Some(s));
    assert!(has(&page, tx::KEY_CHANGED) && has(&page, tx::GET_PREVIEW), "{page}");
    for step in tx::KEY_CHANGED_STEPS {
        assert!(has(&page, step), "{step}");
    }
}

#[test]
fn the_content_half_reports_its_own_check() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    let page = html(Which::Section, Some(s.clone()));
    assert_eq!(
        page.matches("class=\"update-state\"").count(),
        1,
        "only the app's half before the first check: {page}"
    );
    for (check, line) in [
        (ContentCheck::Current, tx::CONTENT_CURRENT),
        (ContentCheck::New, tx::CONTENT_NEW),
        (ContentCheck::Failed, tx::CONTENT_FAILED),
    ] {
        s.content_check = Some(check);
        let page = html(Which::Section, Some(s.clone()));
        assert!(has(&page, line), "{check:?}: {page}");
        assert_eq!(page.matches("class=\"update-state\"").count(), 2, "{page}");
        assert!(has(&page, tx::RESULT_APP), "the app half is labelled beside it");
    }
    // A waiting pack says so whatever the last check found.
    s.content_waiting = true;
    s.content_check = Some(ContentCheck::Failed);
    assert!(has(&html(Which::Section, Some(s.clone())), tx::CONTENT_NEW));

    // The feed out of reach for both halves is one unlabelled line.
    s.content_waiting = false;
    s.phase = Phase::Failed { offer: None, why: Failure::Network };
    let page = html(Which::Section, Some(s));
    assert!(has(&page, tx::FAILED_NETWORK), "{page}");
    assert!(!has(&page, tx::CONTENT_FAILED), "{page}");
    assert_eq!(page.matches("class=\"update-state\"").count(), 1, "{page}");
}

#[test]
fn a_content_update_in_use_names_its_build_and_is_new_until_seen() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.content_build = 45;
    s.content_at = Some(1_700_000_000);
    s.history = vec![build_changes(45, vec![change(ChangeKind::New, "درس جديد")], true)];
    s.prefs.content_from = Some("0.1.0+41".into());
    assert!(s.content_news());
    let page = html(Which::Section, Some(s.clone()));
    assert!(has(&page, "45") && has(&page, tx::CONTENT_PUBLISHED), "{page}");
    assert!(has(&page, tx::BADGE_NEW), "{page}");
    s.prefs.content_from = None;
    assert!(!has(&html(Which::Section, Some(s)), tx::BADGE_NEW));
}

#[test]
fn the_banner_shows_only_when_there_is_something_to_do() {
    assert_eq!(html(Which::Banner, None), "");
    assert_eq!(html(Which::Banner, Some(status(Phase::UpToDate, Mode::Auto))), "");
    // Automatic mode downloads by itself: no banner for "available".
    assert_eq!(html(Which::Banner, Some(status(Phase::Available(offer("0.2.0", 0)), Mode::Auto))), "");

    let o = with_changes(
        offer("0.2.0", 0),
        vec![build_changes(
            50,
            vec![
                change(ChangeKind::New, "أول"),
                change(ChangeKind::New, "تاني"),
                change(ChangeKind::Fixed, "تالت"),
            ],
            false,
        )],
    );
    for (phase, headline, button) in [
        (Phase::Available(o.clone()), tx::AVAILABLE, tx::DOWNLOAD),
        (Phase::Ready(o.clone()), tx::READY, tx::INSTALL_NOW),
        (Phase::NeedsPermission(o.clone()), tx::NEEDS_PERMISSION, tx::TRY_AGAIN),
        (Phase::Installed(o.clone()), tx::INSTALLED, tx::RESTART_NOW),
    ] {
        let page = html(Which::Banner, Some(status(phase, Mode::Notify)));
        for text in [headline, button, tx::LATER, tx::WHATS_NEW_ASK, "0.2.0", "أول"] {
            assert!(has(&page, text), "missing {text}: {page}");
        }
        assert!(!has(&page, "تالت"), "the banner keeps to two lines: {page}");
    }
    let mut closed = status(Phase::Ready(o.clone()), Mode::Auto);
    closed.prefs.dismissed = Some(o.key());
    assert_eq!(html(Which::Banner, Some(closed)), "", "Later hides it for this version");

    let page =
        render(Case { status: Some(status(Phase::Ready(o), Mode::Auto)), which: Which::Banner, nav: false });
    assert!(!has(&page, tx::WHATS_NEW_ASK), "no sheet, no button: {page}");
}

#[test]
fn the_today_note_tells_the_first_content_change_only() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    assert_eq!(html(Which::Note, Some(s.clone())), "");
    assert_eq!(html(Which::Note, None), "");
    s.content_build = 45;
    s.history = vec![build_changes(
        45,
        vec![
            Change {
                kind: ChangeKind::New,
                text: UpdateNotes { en: "Renal lessons".into(), ar_eg: String::new() },
            },
            change(ChangeKind::Fixed, "تاني سطر"),
        ],
        true,
    )];
    s.prefs.content_from = Some("0.1.0+41".into());
    let page = html(Which::Note, Some(s.clone()));
    assert!(has(&page, tx::CONTENT_NOTE) && has(&page, ": Renal lessons"), "English when no Arabic: {page}");
    assert!(!has(&page, "تاني سطر"));
    assert!(has(&page, tx::WHATS_NEW_ASK) && has(&page, tx::CLOSE), "{page}");

    // An app update in the news opens the sheet instead.
    s.history.insert(0, build_changes(46, vec![change(ChangeKind::New, "تطبيق")], false));
    s.build = 46;
    s.prefs.whats_new_from = Some("0.1.0+41".into());
    assert_eq!(html(Which::Note, Some(s)), "");
}

#[test]
fn a_closed_sheet_draws_nothing() {
    let s = status(Phase::UpToDate, Mode::Auto);
    let page = render(Case { status: Some(s), which: Which::Section, nav: true });
    assert!(!page.contains("class=\"us-sheet\""));
    // Open, but no status yet.
    assert_eq!(html(Which::Sheet(SheetMode::History), None), "");
}

fn history(n: u64) -> Vec<BuildChanges> {
    (0..n)
        .rev()
        .map(|i| {
            build_changes(
                30 + i,
                vec![
                    change(ChangeKind::New, &format!("جديد {i}")),
                    change(ChangeKind::Improved, &format!("أحسن {i}")),
                ],
                false,
            )
        })
        .collect()
}

#[test]
fn the_changelog_opens_the_newest_build_and_folds_the_rest() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.build = 39;
    s.history = history(10);
    let page = html(Which::Sheet(SheetMode::History), Some(s.clone()));
    assert!(page.contains("role=\"dialog\""), "{page}");
    assert!(has(&page, tx::HISTORY) && has(&page, tx::EARLIER), "{page}");
    // Newest (39, installed) open; four earlier builds listed, the rest behind a button.
    assert!(has(&page, "جديد 9") && has(&page, "أحسن 9"), "{page}");
    assert!(!has(&page, "جديد 8"), "earlier builds start folded: {page}");
    assert!(has(&page, "0.1.0 (38)") && has(&page, "0.1.0 (35)"), "{page}");
    assert!(!has(&page, "0.1.0 (34)"), "{page}");
    assert!(has(&page, tx::SHOW_OLDER) && has(&page, tx::BADGE_INSTALLED), "{page}");
    assert_eq!(page.matches("aria-expanded=\"true\"").count(), 1, "{page}");
    assert!(has(&page, tx::CLOSE) && has(&page, "GitHub"));
    assert!(!has(&page, tx::GOT_IT));

    s.history.clear();
    let page = html(Which::Sheet(SheetMode::History), Some(s));
    assert!(has(&page, tx::HISTORY_EMPTY), "{page}");
}

#[test]
fn a_long_kind_folds_after_five_lines() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    let lines: Vec<Change> = (0..8).map(|i| change(ChangeKind::Fixed, &format!("إصلاح {i}"))).collect();
    s.history = vec![build_changes(41, lines, false)];
    let page = html(Which::Sheet(SheetMode::History), Some(s));
    assert!(has(&page, "إصلاح 4") && !has(&page, "إصلاح 5"), "{page}");
    assert!(has(&page, &tx::SHOW_ALL.replace("{n}", "8")), "{page}");
    assert!(has(&page, tx::KIND_FIXED));
}

#[test]
fn the_offer_sheet_counts_what_the_update_brings() {
    let o = with_changes(
        offer("0.1.0", 45),
        vec![
            build_changes(
                45,
                vec![change(ChangeKind::New, "حاجة جديدة"), change(ChangeKind::Fixed, "إصلاح")],
                false,
            ),
            build_changes(44, vec![change(ChangeKind::Improved, "تحسين")], false),
        ],
    );
    let mut s = status(Phase::Available(o.clone()), Mode::Notify);
    s.history = history(3);
    let page = html(Which::Sheet(SheetMode::Offer), Some(s.clone()));
    for text in
        [tx::AVAILABLE, tx::DOWNLOAD, &tx::builds_count(2), tx::KIND_NEW, tx::KIND_FIXED, tx::KIND_IMPROVED]
    {
        assert!(has(&page, text), "missing {text}: {page}");
    }
    // Same version: the jump is by build number.
    assert!(has(&page, "#41") && has(&page, "#45"), "{page}");
    assert!(has(&page, "حاجة جديدة"), "{page}");
    assert!(has(&page, tx::BADGE_LATEST) || has(&page, tx::BADGE_NEW), "{page}");

    let mut downloading = s.clone();
    downloading.phase = Phase::Downloading { offer: o.clone(), done: 0 };
    let page = html(Which::Sheet(SheetMode::Offer), Some(downloading));
    assert!(has(&page, tx::DOWNLOADING) && has(&page, "<progress"), "{page}");

    for (phase, head) in [
        (Phase::Ready(o.clone()), tx::READY),
        (Phase::Installing(o.clone()), tx::INSTALLING),
        (Phase::NeedsPermission(o.clone()), tx::NEEDS_PERMISSION),
        (Phase::Installed(o.clone()), tx::INSTALLED),
    ] {
        let mut st = s.clone();
        st.phase = phase;
        assert!(has(&html(Which::Sheet(SheetMode::Offer), Some(st)), head), "{head}");
    }

    // An offer without a changelog falls back to the feed's notes.
    let mut bare = offer("0.2.0", 0);
    bare.notes.ar_eg = "ملاحظات قديمة".into();
    s.phase = Phase::Available(bare);
    let page = html(Which::Sheet(SheetMode::Offer), Some(s));
    assert!(has(&page, "ملاحظات قديمة") && has(&page, "v0.1.0 (41)") && has(&page, "v0.2.0"), "{page}");
}

#[test]
fn after_an_update_the_sheet_says_what_came_and_got_it_closes() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.build = 45;
    s.history = history(16);
    s.prefs.whats_new_from = Some("0.1.0+43".into());
    let page = html(Which::Sheet(SheetMode::Updated), Some(s.clone()));
    assert!(has(&page, tx::UPDATED_TITLE) && has(&page, tx::GOT_IT), "{page}");
    assert!(has(&page, "#43") && has(&page, "#45"), "{page}");
    assert!(has(&page, &tx::builds_count(2)), "{page}");

    // A content-only update: its own title and the content line, no jump.
    let mut c = status(Phase::UpToDate, Mode::Auto);
    c.content_build = 45;
    c.history = vec![build_changes(45, vec![change(ChangeKind::New, "محاضرة")], true)];
    c.prefs.content_from = Some("0.1.0+41".into());
    let page = html(Which::Sheet(SheetMode::Updated), Some(c));
    assert!(has(&page, tx::CONTENT_UPDATED_TITLE), "{page}");
    assert!(has(&page, tx::CONTENT_SHORT) && has(&page, tx::BADGE_CONTENT), "{page}");
    assert!(has(&page, tx::BADGE_CONTENT_IN_USE), "{page}");
    assert!(has(&page, &tx::content_updates_count(1)), "{page}");
    assert!(!page.contains("class=\"us-jump\""), "{page}");

    // Nothing to tell.
    s.prefs.whats_new_from = None;
    let page = html(Which::Sheet(SheetMode::Updated), Some(s));
    assert!(has(&page, tx::NO_DETAILS), "{page}");
}

#[test]
fn megabytes_and_summary_lines_read_naturally() {
    assert_eq!(view::megabytes(1024 * 1024 + 1024 * 512), format!("1.5 {}", tx::MB));
    let o = offer("0.2.0", 0);
    for (phase, line) in [
        (Phase::Available(o.clone()), tx::AVAILABLE),
        (Phase::Downloading { offer: o.clone(), done: 1 }, tx::AVAILABLE),
        (Phase::Ready(o.clone()), tx::READY),
        (Phase::NeedsPermission(o.clone()), tx::READY),
        (Phase::Failed { offer: None, why: Failure::Network }, tx::SUMMARY_FAILED),
    ] {
        assert_eq!(view::summary(&status(phase, Mode::Auto)).as_deref(), Some(line));
    }
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.content_build = 45;
    s.history = vec![build_changes(45, vec![change(ChangeKind::New, "x")], true)];
    s.prefs.content_from = Some("0.1.0+41".into());
    assert_eq!(view::summary(&s).as_deref(), Some(tx::CONTENT_NOTE));
}

#[test]
fn a_notification_tap_with_nothing_offered_opens_the_changelog() {
    fn app() -> Element {
        let nav = use_hook(UpdateSheetNav::new);
        // No updater in this process: nothing is offered.
        nav.open_offer();
        assert_eq!(*nav.0.peek(), Some(SheetMode::History));
        nav.open(SheetMode::Offer);
        assert_eq!(*nav.0.peek(), Some(SheetMode::Offer));
        rsx! { "ok" }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    assert_eq!(dioxus::ssr::render(&dom), "ok");
}

/// A screen and the sheet over it, as the shell mounts them, to press through.
fn pressable(which: Which, status: Status) -> crate::test_dom::TestDom {
    #[derive(Props, Clone, PartialEq)]
    struct P {
        which: u8,
        open: Option<SheetMode>,
        status: Status,
    }
    fn app(p: P) -> Element {
        let status = use_signal(|| Some(p.status.clone()));
        use_context_provider(|| UpdateSheetNav(Signal::new(p.open)));
        rsx! {
            match p.which {
                0 => rsx! { UpdatesView { status } },
                1 => rsx! { BannerView { status } },
                2 => rsx! { ContentNoteView { status } },
                _ => rsx! {},
            }
            SheetView { status }
        }
    }
    let (which, open) = match which {
        Which::Section => (0, None),
        Which::Banner => (1, None),
        Which::Note => (2, None),
        Which::Sheet(m) => (3, Some(m)),
    };
    crate::test_dom::TestDom::new(VirtualDom::new_with_props(app, P { which, open, status }))
}

fn sheet_open(page: &str) -> bool {
    page.contains("role=\"dialog\"")
}

#[test]
fn the_changelog_opens_builds_and_older_ones_on_tap_and_closes() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.build = 39;
    s.history = history(10);
    let mut t = pressable(Which::Sheet(SheetMode::History), s);
    assert!(!has(&t.html(), "جديد 8"));
    t.click("0.1.0 (38)");
    assert!(has(&t.html(), "جديد 8"), "a tap opens a build: {}", t.html());
    assert_eq!(t.html().matches("aria-expanded=\"true\"").count(), 2);
    t.click("0.1.0 (38)");
    assert!(!has(&t.html(), "جديد 8"), "and closes it");
    t.click(tx::SHOW_OLDER);
    assert!(has(&t.html(), "0.1.0 (34)"), "four more: {}", t.html());
    t.click_nth(tx::CLOSE, 1);
    assert!(!sheet_open(&t.html()), "{}", t.html());
}

#[test]
fn a_folded_kind_shows_all_then_less() {
    let mut s = status(Phase::UpToDate, Mode::Auto);
    let lines: Vec<Change> = (0..8).map(|i| change(ChangeKind::Fixed, &format!("إصلاح {i}"))).collect();
    s.history = vec![build_changes(41, lines, false)];
    let mut t = pressable(Which::Sheet(SheetMode::History), s);
    t.click(&tx::SHOW_ALL.replace("{n}", "8"));
    assert!(has(&t.html(), "إصلاح 7") && has(&t.html(), tx::SHOW_LESS), "{}", t.html());
    t.click(tx::SHOW_LESS);
    assert!(!has(&t.html(), "إصلاح 7"));
}

#[test]
fn settings_rows_open_the_sheet_on_the_right_page() {
    let o = offer("0.2.0", 0);
    let mut s = status(Phase::Available(o), Mode::Notify);
    s.history = history(2);
    let mut t = pressable(Which::Section, s.clone());
    assert!(!sheet_open(&t.html()));
    t.click(tx::WHATS_NEW_ASK);
    assert!(sheet_open(&t.html()) && has(&t.html(), tx::AVAILABLE), "the offer: {}", t.html());
    t.click(&format!("aria-label=\"{}\"", tx::CLOSE));
    assert!(!sheet_open(&t.html()), "{}", t.html());
    t.click(tx::HISTORY);
    assert!(sheet_open(&t.html()), "the changelog: {}", t.html());

    // Without an updater (this test build) the buttons are safe no-ops.
    let mut t = pressable(Which::Section, s);
    for label in [tx::CHECK_NOW, tx::MODE_OFF, tx::BETA, tx::DOWNLOAD] {
        t.click(label);
        assert!(has(&t.html(), label), "{label}");
    }
    t.input_event("change", "role=\"switch\"", "false");
}

#[test]
fn the_banner_and_the_note_open_the_sheet_or_step_aside() {
    let o =
        with_changes(offer("0.2.0", 0), vec![build_changes(50, vec![change(ChangeKind::New, "أول")], false)]);
    let mut t = pressable(Which::Banner, status(Phase::Ready(o), Mode::Notify));
    t.click(tx::INSTALL_NOW);
    t.click(tx::LATER);
    t.click(tx::WHATS_NEW_ASK);
    assert!(sheet_open(&t.html()), "{}", t.html());

    let mut s = status(Phase::UpToDate, Mode::Auto);
    s.content_build = 45;
    s.history = vec![build_changes(45, vec![change(ChangeKind::New, "محاضرة")], true)];
    s.prefs.content_from = Some("0.1.0+41".into());
    let mut t = pressable(Which::Note, s);
    t.click(tx::CLOSE);
    t.click(tx::WHATS_NEW_ASK);
    assert!(sheet_open(&t.html()) && has(&t.html(), tx::CONTENT_UPDATED_TITLE), "{}", t.html());
}
