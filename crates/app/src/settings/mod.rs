//! Settings: one sheet over the current screen, so opening it never loses a
//! study session. The gear in the top bar opens it.
//!
//! A feature chunk that needs settings adds its `Section` variant (in
//! `ALL`, with a title, summary and id) and one line in `SectionView`, and
//! keeps the rows in its own module. Sections with nothing in them yet are
//! simply not listed.
// adapted from kabar-app main: crates/app/src/settings/mod.rs

mod about;
mod data;
mod look;
#[cfg(all(feature = "desktop", not(target_os = "android")))]
pub mod menu;

use dioxus::prelude::*;

use crate::i18n::settings as tx;
use crate::look::{Css, css};

const SETTINGS_CSS: Css = css!("assets/settings.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Study,
    Reminders,
    Sync,
    Look,
    /// Pen hover and the sidebar, on Android (the Tab S11, the S24 Ultra).
    Tablet,
    /// Mouse hover, the sidebar and the keyboard, on the Mac and desktops.
    Mac,
    Data,
    Updates,
    About,
}

impl Section {
    pub const ALL: [Self; 9] = [
        Self::Study,
        Self::Reminders,
        Self::Sync,
        Self::Look,
        Self::Tablet,
        Self::Mac,
        Self::Data,
        Self::Updates,
        Self::About,
    ];

    /// Stable id, for `BATNA_OPEN_SETTINGS=<id>` previews.
    pub fn id(self) -> &'static str {
        match self {
            Self::Study => "study",
            Self::Reminders => "reminders",
            Self::Sync => "sync",
            Self::Look => "look",
            Self::Tablet => "tablet",
            Self::Mac => "mac",
            Self::Data => "data",
            Self::Updates => "updates",
            Self::About => "about",
        }
    }

    /// Only the sections that apply on this device are listed.
    pub fn applies(self) -> bool {
        match self {
            Self::Tablet => cfg!(target_os = "android"),
            Self::Mac => !cfg!(target_os = "android"),
            _ => true,
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }

    fn title(self) -> &'static str {
        match self {
            Self::Study => crate::i18n::study_settings::SECTION,
            Self::Reminders => crate::i18n::remind::SECTION,
            Self::Sync => crate::i18n::cloud::SECTION,
            Self::Look => tx::LOOK,
            Self::Tablet => crate::i18n::desk::TABLET,
            Self::Mac => crate::i18n::desk::MAC,
            Self::Data => tx::DATA,
            Self::Updates => crate::i18n::update::SECTION,
            Self::About => tx::ABOUT,
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Study => crate::i18n::study_settings::SECTION_SUB,
            Self::Reminders => crate::i18n::remind::SECTION_SUB,
            Self::Sync => crate::i18n::cloud::SECTION_SUB,
            Self::Look => tx::LOOK_SUB,
            Self::Tablet => crate::i18n::desk::TABLET_SUB,
            Self::Mac => crate::i18n::desk::MAC_SUB,
            Self::Data => tx::DATA_SUB,
            Self::Updates => crate::i18n::update::SECTION_SUB,
            Self::About => tx::ABOUT_SUB,
        }
    }
}

/// Whether the sheet is open and on which section. `section: None` is the
/// section list on the phone; wider windows show the first section beside it.
#[derive(Clone, Copy)]
pub struct SettingsNav {
    pub open: Signal<bool>,
    pub section: Signal<Option<Section>>,
}

impl SettingsNav {
    pub fn new() -> Self {
        // BATNA_OPEN_SETTINGS=look (or "" for the list) opens the sheet at launch.
        let start = std::env::var("BATNA_OPEN_SETTINGS").ok();
        Self {
            open: Signal::new(start.is_some()),
            section: Signal::new(start.as_deref().and_then(Section::from_id)),
        }
    }

    pub fn show(mut self, section: Option<Section>) {
        self.section.set(section);
        self.open.set(true);
    }

    pub fn close(mut self) {
        self.open.set(false);
    }
}

/// The Settings sheet; renders nothing while closed.
#[component]
pub fn SettingsSheet() -> Element {
    let nav = use_context::<SettingsNav>();
    // Escape steps back out of a section on the phone, else closes. Listened
    // for on the document: WebKit doesn't focus buttons on click.
    use_future(move || async move {
        let mut eval = document::eval(
            r#"
            if (!window.__batnaSettingsEsc) {
              window.__batnaSettingsEsc = true;
              document.addEventListener("keydown", (e) => {
                if (e.key === "Escape" && document.querySelector(".settings-window")) {
                  e.preventDefault();
                  e.stopPropagation();
                  dioxus.send(window.matchMedia("(max-width: 599.98px)").matches);
                }
              }, true);
            }
            await new Promise(() => {});
            "#,
        );
        let mut section = nav.section;
        while let Ok(compact) = eval.recv::<bool>().await {
            if compact && section.peek().is_some() {
                section.set(None);
            } else {
                nav.close();
            }
        }
    });
    if !*nav.open.read() {
        return rsx! {};
    }
    let chosen = *nav.section.read();
    // Phones show the list until a section is picked; wider windows show both.
    let current = chosen.unwrap_or(Section::ALL[0]);
    let drilled = chosen.is_some();
    let mut section_sig = nav.section;

    rsx! {
        style { {SETTINGS_CSS.text()} }
        div { class: "settings-backdrop", onclick: move |_| nav.close() }
        div {
            class: "settings-window",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "settings-title",
            tabindex: "-1",
            "data-drilled": if drilled { "true" } else { "false" },
            onmounted: move |e: MountedEvent| async move {
                let _ = e.set_focus(true).await;
            },
            header { class: "settings-head",
                h2 { id: "settings-title",
                    span { class: "settings-title-list", {tx::TITLE} }
                    span { class: "settings-title-section", {current.title()} }
                }
                button {
                    class: "icon-button",
                    "data-back": "true",
                    aria_label: tx::CLOSE,
                    title: tx::CLOSE,
                    onclick: move |_| nav.close(),
                    span { aria_hidden: "true", "✕" }
                }
                // Last in the header, so system Back picks it over Close
                // whenever it shows (the phone, inside a section).
                button {
                    class: "icon-button settings-back",
                    "data-back": "true",
                    aria_label: tx::BACK,
                    onclick: move |_| section_sig.set(None),
                    span { aria_hidden: "true", "→" }
                }
            }
            div { class: "settings-body",
                nav { class: "settings-sections", aria_label: tx::SECTIONS,
                    for s in Section::ALL.into_iter().filter(|s| s.applies()) {
                        button {
                            key: "{s.id()}",
                            aria_current: if current == s { "true" } else { "false" },
                            onclick: move |_| section_sig.set(Some(s)),
                            span { class: "ss-title", {s.title()} }
                            span { class: "ss-sub", {s.summary()} }
                        }
                    }
                }
                div { class: "settings-pane", key: "{current.id()}",
                    SectionView { section: current }
                }
            }
        }
    }
}

#[component]
fn SectionView(section: Section) -> Element {
    match section {
        Section::Study => {
            rsx! {
                crate::today::StudySection {}
                crate::learn::ViewSection {}
                crate::sounds::SoundSection {}
                crate::onboarding::ReopenCard {}
            }
        }
        Section::Reminders => rsx! { crate::reminders::RemindSection {} },
        Section::Sync => rsx! { crate::cloud::SyncSection {} },
        Section::Look => rsx! { look::LookSection {} },
        Section::Tablet | Section::Mac => rsx! { crate::desk::DeviceSettings { section } },
        Section::Data => rsx! { data::DataSection {} },
        Section::Updates => rsx! { crate::update::view::UpdatesSection {} },
        Section::About => rsx! { about::AboutSection {} },
    }
}

/// A small pill after a field label: this choice stays on this device.
#[component]
pub fn DeviceOnly() -> Element {
    rsx! { span { class: "chip settings-scope", {tx::THIS_DEVICE} } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_ids_round_trip() {
        for s in Section::ALL {
            assert_eq!(Section::from_id(s.id()), Some(s));
        }
        assert_eq!(Section::from_id("nope"), None);
    }
}
