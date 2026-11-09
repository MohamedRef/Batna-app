// adapted from kabar-app: crates/app/src/settings/browser.rs
//! Browser fixtures render production Settings with the app's own preferences.
//! Rust callbacks are not hydrated; this suite checks layout and the production
//! keyboard controller, not application click or persistence integration.

use super::*;
use crate::{
    Screen, Ui,
    prefs::{Appearance, DevicePrefs, TextSize},
};
use batna_core::theme::Theme;
use std::cell::RefCell;

#[derive(Clone, Copy)]
pub(super) struct Case {
    pub(super) theme: Theme,
    pub(super) appearance: Appearance,
    pub(super) size: TextSize,
}
thread_local! {
    static CASE: RefCell<Option<(Case, Section)>> = const { RefCell::new(None) };
    /// What the search box holds, and whether the sheet is open.
    static SEARCH: RefCell<(String, bool)> = const { RefCell::new((String::new(), true)) };
}

fn fixture() -> Element {
    let (case, section) = CASE.with(|c| c.borrow().unwrap());
    crate::persistence::use_provide_saves();
    crate::store::use_memory_store();
    crate::store::use_provide_review();
    crate::cloud::use_unconfigured_cloud();
    let prefs = DevicePrefs {
        theme: case.theme,
        appearance: case.appearance,
        text_size: case.size,
        ..DevicePrefs::default()
    };
    use_context_provider(|| Ui {
        screen: Signal::new(Screen::Today),
        prefs: Signal::new(prefs),
        focus: Signal::new(false),
    });
    let (query, open) = SEARCH.with(|q| q.borrow().clone());
    use_context_provider(|| SettingsNav {
        open: Signal::new(open),
        section: Signal::new(Some(section)),
        query: Signal::new(query),
    });
    rsx! {
        style { {crate::fonts::css()} }
        style { {crate::MAIN_CSS.text()} }
        style { {crate::SHAPE_CSS.text()} }
        style { {crate::PARTS_CSS.text()} }
        style { {crate::APPEARANCE_CSS.text()} }
        style { {crate::LIQUID_GLASS_CSS.text()} }
        style { {prefs.css()} }
        div { class: "app", lang: "ar", dir: "rtl", "data-appearance": case.appearance.as_str(),
            main { id: "background", button { id: "opener", r#type: "button", "Settings" } }
            SettingsSheet {}
        }
    }
}

/// Settings open on `section`, with the case's look.
pub(super) fn render(case: Case, section: Section) -> String {
    CASE.with(|c| *c.borrow_mut() = Some((case, section)));
    let mut dom = VirtualDom::new(fixture);
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

#[test]
fn look_shows_the_device_s_own_choices_as_checked() {
    let case = Case { theme: Theme::NightShift, appearance: Appearance::Glossy, size: TextSize::Larger };
    let html = render(case, Section::Look);
    assert!(html.contains("settings-window") && html.contains("data-drilled=\"true\""), "{html}");
    // Every theme, appearance and size, and the three lesson reading rows;
    // exactly one of each checked. (The sheet's CSS names [role="radio"]
    // too, so count the buttons.)
    use crate::prefs::{ReadSize, ReadSpacing, ReadWidth};
    let reading = ReadSize::ALL.len() + ReadSpacing::ALL.len() + ReadWidth::ALL.len();
    assert_eq!(
        html.matches("role=\"radio\" aria-checked=").count()
            + html.matches("role=\"radio\" data-size=").count(),
        Theme::ALL.len() + Appearance::ALL.len() + TextSize::ALL.len() + reading
    );
    assert_eq!(
        html.matches("role=\"radio\" aria-checked=\"true\"").count()
            + html.matches("aria-checked=\"true\" aria-label=").count(),
        6
    );
    let checked = |label: &str| {
        let at = html.find(label).unwrap();
        html[..at].rfind("aria-checked=").map(|i| &html[i..i + 19]) == Some("aria-checked=\"true\"")
    };
    assert!(checked(tx::THEME_SHIFT) && checked(tx::APPEARANCE_GLOSSY) && checked(tx::TEXT_LARGER));
    assert!(!checked(tx::THEME_PAPER));
    // Choices here stay on this device, and the sample shows the term in <bdi>.
    assert_eq!(html.matches(tx::THIS_DEVICE).count(), 4);
    assert!(html.contains(&format!("<bdi>{}</bdi>", tx::TEXT_SAMPLE_TERM)));
    assert!(html.contains("--sw-a:") && html.contains("data-swatch=\"liquid-glass\""), "{html}");
}

#[test]
#[ignore = "exports production HTML for the dedicated browser CI job"]
fn export_browser_fixtures() {
    let dir =
        std::path::PathBuf::from(std::env::var_os("APP_BROWSER_FIXTURES").expect("APP_BROWSER_FIXTURES"));
    std::fs::create_dir_all(&dir).unwrap();
    let mut manifest = Vec::new();
    for width in [320, 384, 600, 840, 1280, 1600] {
        for theme in Theme::ALL {
            for appearance in Appearance::ALL.iter().copied() {
                for size in [TextSize::Normal, TextSize::Larger] {
                    // One wide, representative Mac capture; keep the full
                    // theme/size/appearance matrix at regular widths.
                    if width == 1600
                        && (theme != Theme::Paper
                            || appearance != Appearance::Calm
                            || size != TextSize::Normal)
                    {
                        continue;
                    }
                    let html = render(Case { theme, appearance, size }, Section::Look);
                    assert!(html.contains("settings-window") && html.contains("text-sample"));
                    let file =
                        format!("{width}-{}-{}-{}.html", theme.as_str(), appearance.as_str(), size.as_str());
                    std::fs::write(dir.join(&file), format!("<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"></head><body>{html}</body></html>")).unwrap();
                    manifest.push(serde_json::json!({"file":file,"width":width,"dir":"rtl","theme":theme.as_str(),"appearance":appearance.as_str(),"size":size.as_str()}));
                }
            }
        }
    }
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

#[test]
fn every_section_opens_with_its_title_and_the_list_beside_it() {
    let case = Case { theme: Theme::Paper, appearance: Appearance::Calm, size: TextSize::Normal };
    for section in Section::ALL.into_iter().filter(|s| s.applies()) {
        let html = render(case, section);
        assert!(
            html.contains(&format!("<h3 id=\"settings-section-title\">{}</h3>", section.title())),
            "{section:?}"
        );
        let marker = match section {
            Section::Study => crate::i18n::onboard::REOPEN_TITLE,
            Section::Sync => crate::i18n::cloud::NOT_CONFIGURED,
            Section::Look => tx::THEME,
            Section::Mac | Section::Tablet => crate::i18n::desk::NAV_STYLE,
            Section::Data => tx::DATA_FOLDER,
            Section::About => tx::LECTURES_WRITTEN,
            Section::Reminders | Section::Updates => "settings-pane",
        };
        assert!(html.contains(marker), "{section:?} shows {marker}");
        // Every section stays listed beside it.
        for s in Section::ALL.into_iter().filter(|s| s.applies()) {
            assert!(html.contains(s.title()), "{section:?} list misses {s:?}");
        }
    }
}

#[test]
fn search_narrows_the_list_and_says_when_nothing_matches() {
    let case = Case { theme: Theme::Night, appearance: Appearance::Calm, size: TextSize::Normal };
    let search = |q: &str| {
        SEARCH.with(|s| *s.borrow_mut() = (q.into(), true));
        let html = render(case, Section::About);
        SEARCH.with(|s| *s.borrow_mut() = (String::new(), true));
        html
    };
    let html = search("نسخة احتياطية");
    assert!(html.contains(tx::DATA) && !html.contains(tx::LOOK), "only Your data matches");
    assert!(!html.contains(tx::NO_RESULTS));
    let html = search("zzzz");
    assert!(html.contains(tx::NO_RESULTS) && html.contains(tx::CLEAR_SEARCH), "{html}");
    // The open section still shows beside an empty list.
    assert!(html.contains(tx::LECTURES_WRITTEN));
}

#[test]
fn a_closed_sheet_renders_nothing_and_show_opens_it_fresh() {
    let case = Case { theme: Theme::Night, appearance: Appearance::Calm, size: TextSize::Normal };
    SEARCH.with(|s| *s.borrow_mut() = (String::new(), false));
    let html = render(case, Section::Look);
    SEARCH.with(|s| *s.borrow_mut() = (String::new(), true));
    assert!(!html.contains("class=\"settings-window\""));
    let dom = VirtualDom::new(|| rsx! {});
    dom.in_scope(ScopeId::ROOT, || {
        let mut nav = SettingsNav::new();
        nav.query.set("old search".into());
        nav.show(Some(Section::Data));
        assert!(*nav.open.peek());
        assert_eq!(*nav.section.peek(), Some(Section::Data));
        assert_eq!(*nav.query.peek(), "", "a fresh sheet starts unfiltered");
        nav.close();
        assert!(!*nav.open.peek());
    });
}
