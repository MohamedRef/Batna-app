//! The store couldn't be opened, so this session runs on an in-memory one
//! (`store::shared`). That is said on every screen, not only on Today
//! (engineering audit F11): what will disappear, and the way to keep it.

use dioxus::prelude::*;

use crate::i18n::saving as tx;

/// Shown above every screen while the session's data lives only in memory.
#[component]
pub fn VolatileNotice() -> Element {
    let Some(error) = crate::store::open_error() else {
        return rsx! {};
    };
    let settings = try_use_context::<crate::settings::SettingsNav>();
    rsx! {
        div { class: "save-notice volatile-notice", role: "alert", aria_atomic: "true",
            p {
                "{tx::VOLATILE} "
                bdi { dir: "ltr", class: "muted small", "{error}" }
            }
            if let Some(settings) = settings {
                button {
                    class: "secondary volatile-export",
                    onclick: move |_| settings.show(Some(crate::settings::Section::Data)),
                    {tx::VOLATILE_EXPORT}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdom::TestDom;

    #[component]
    fn Root() -> Element {
        let settings = use_context_provider(crate::settings::SettingsNav::new);
        let section = (*settings.section.read()).map(|s| s.id()).unwrap_or_default();
        rsx! {
            VolatileNotice {}
            p { class: "t-section", "{section}" }
        }
    }

    #[test]
    fn an_unopened_store_is_said_with_the_way_to_keep_the_work() {
        crate::store::OPEN_ERROR_TEST.set(None);
        assert_eq!(TestDom::new(Root).count("div.volatile-notice"), 0, "a normal store says nothing");
        crate::store::OPEN_ERROR_TEST.set(Some("/data: disk I/O error"));
        let mut t = TestDom::new(Root);
        assert!(t.text().contains(tx::VOLATILE) && t.text().contains("disk I/O error"));
        t.click("button.volatile-export");
        assert_eq!(t.texts("p.t-section"), ["data"]);
        crate::store::OPEN_ERROR_TEST.set(None);
    }
}
