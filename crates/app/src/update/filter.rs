//! Search and filters for the changelog (Settings › Updates › سجل
//! التغييرات, #1340): words (Arabic marks and letter variants ignored), a
//! course or the app, and a kind. While any is set, the matching lines of
//! every build kept on this device replace the timeline, newest build
//! first, each line still opening its lesson.

use batna_core::update::{BuildChanges, Change, ChangeKind};
use dioxus::prelude::*;

use super::changelog::label;
use super::groups::{groups, line_item};
use super::sheet::{kind_id, kind_label};
use crate::content::Library;
use crate::i18n::changelog as tx;

/// What the changelog is narrowed to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub words: String,
    /// A course id, `app`, or "" for lines nothing places.
    pub group: Option<String>,
    pub kind: Option<ChangeKind>,
}

impl Filter {
    pub fn is_on(&self) -> bool {
        !self.words.trim().is_empty() || self.group.is_some() || self.kind.is_some()
    }

    fn keeps(&self, c: &Change) -> bool {
        self.kind.is_none_or(|k| c.kind == k)
            && self.group.as_deref().is_none_or(|g| c.group().unwrap_or("") == g)
            && (self.words.trim().is_empty()
                || crate::settings::search::matches(&self.words, &format!("{} {}", c.text.ar_eg, c.text.en)))
    }
}

static FILTER: GlobalSignal<Filter> = Signal::global(Filter::default);

/// The changelog is narrowed: the sheet shows [`HistoryFilter`]'s results
/// instead of its timeline.
pub fn filtering() -> bool {
    FILTER.read().is_on()
}

/// Forgets the search (the sheet closed).
pub fn clear() {
    if FILTER.peek().is_on() {
        *FILTER.write() = Filter::default();
    }
}

/// The builds with a line `f` keeps, newest first, with those lines.
pub fn matching<'a>(history: &'a [BuildChanges], f: &Filter) -> Vec<(&'a BuildChanges, Vec<&'a Change>)> {
    history
        .iter()
        .map(|b| (b, b.changes.iter().filter(|c| f.keeps(c)).collect::<Vec<_>>()))
        .filter(|(_, lines)| !lines.is_empty())
        .collect()
}

/// The groups any kept build has, in the sheet's order (app, courses, rest).
fn group_choices(history: &[BuildChanges]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let lib = Library::get();
    let mut all = BuildChanges {
        version: String::new(),
        build: 0,
        published_at: String::new(),
        changes: history.iter().flat_map(|b| b.changes.iter().cloned()).collect(),
        content_only: false,
    };
    all.changes.dedup();
    for g in groups(&all, super::groups::course_rank(lib)) {
        out.push(g.id);
    }
    out
}

/// The search box, the course and kind pickers, and (while narrowed) the
/// lines that match.
#[component]
pub(super) fn HistoryFilter(history: Vec<BuildChanges>) -> Element {
    let lib = Library::get();
    let f = FILTER.read().clone();
    let choices = group_choices(&history);
    let found = matching(&history, &f);
    let n: usize = found.iter().map(|(_, l)| l.len()).sum();
    let group_value = f.group.clone().map_or("*".to_owned(), |g| if g.is_empty() { "-".into() } else { g });
    rsx! {
        div { class: "us-filter",
            input {
                r#type: "search",
                class: "us-search",
                placeholder: "{tx::SEARCH}",
                aria_label: "{tx::SEARCH}",
                value: "{f.words}",
                oninput: move |e| FILTER.write().words = e.value(),
            }
            div { class: "us-filter-row",
                select {
                    class: "us-select",
                    aria_label: "{tx::ALL_GROUPS}",
                    value: "{group_value}",
                    onchange: move |e| {
                        FILTER.write().group = match e.value().as_str() {
                            "*" => None,
                            "-" => Some(String::new()),
                            g => Some(g.to_owned()),
                        };
                    },
                    option { value: "*", selected: f.group.is_none(), "{tx::ALL_GROUPS}" }
                    for g in choices {
                        {
                            let value = if g.is_empty() { "-".to_owned() } else { g.clone() };
                            let selected = f.group.as_deref() == Some(g.as_str());
                            rsx! {
                                option { key: "{value}", value: "{value}", selected, "{super::groups::group_name(lib, &g)}" }
                            }
                        }
                    }
                }
                div { class: "us-kind-chips", role: "group", aria_label: "{tx::ALL_KINDS}",
                    for k in ChangeKind::ALL {
                        button {
                            key: "{kind_id(k)}",
                            class: "us-kind-chip",
                            "data-kind": "{kind_id(k)}",
                            aria_pressed: if f.kind == Some(k) { "true" } else { "false" },
                            onclick: move |_| {
                                let mut w = FILTER.write();
                                w.kind = if w.kind == Some(k) { None } else { Some(k) };
                            },
                            "{kind_label(k)}"
                        }
                    }
                }
            }
        }
        if f.is_on() {
            div { class: "us-found",
                p { class: "us-found-count",
                    if n == 0 { "{tx::NO_MATCH}" } else { "{tx::found(n, found.len())}" }
                    button { class: "us-more", onclick: move |_| *FILTER.write() = Filter::default(), "{tx::CLEAR}" }
                }
                for (b, lines) in found {
                    section { key: "{b.key()}", class: "us-found-build",
                        h3 { bdi { dir: "ltr", class: "latin", "v{label(&b.version, b.build)}" } }
                        ul { class: "us-items",
                            for (j, c) in lines.into_iter().enumerate() {
                                {line_item(lib, c, j)}
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

    fn c(kind: ChangeKind, ar: &str, scope: Option<&str>) -> Change {
        Change {
            scope: scope.map(str::to_owned),
            ..Change::new(kind, UpdateNotes { en: String::new(), ar_eg: ar.into() })
        }
    }

    fn b(build: u64, changes: Vec<Change>) -> BuildChanges {
        BuildChanges {
            version: "1.0.0".into(),
            build,
            published_at: String::new(),
            changes,
            content_only: false,
        }
    }

    #[test]
    fn words_course_and_kind_narrow_every_build() {
        use ChangeKind::*;
        let h = vec![
            b(9, vec![c(New, "صورة جديدة في الكبد", Some("hb")), c(Fixed, "البنك أسرع", Some("bank"))]),
            b(8, vec![c(Improved, "إصلاح صورة الدم", Some("hem"))]),
        ];
        let f = |words: &str, group: Option<&str>, kind: Option<ChangeKind>| Filter {
            words: words.into(),
            group: group.map(str::to_owned),
            kind,
        };
        let builds = |f: Filter| matching(&h, &f).iter().map(|(b, l)| (b.build, l.len())).collect::<Vec<_>>();
        assert_eq!(builds(f("صورة", None, None)), [(9, 1), (8, 1)]);
        assert_eq!(builds(f("اصلاح", None, None)), [(8, 1)], "إ and ا match");
        assert_eq!(builds(f("", Some("app"), None)), [(9, 1)]);
        assert_eq!(builds(f("", None, Some(Improved))), [(8, 1)]);
        assert_eq!(builds(f("صورة", Some("hb"), Some(Fixed))), []);
        assert!(!Filter::default().is_on() && f(" ", None, None).words.trim().is_empty());
    }

    #[test]
    fn the_changelog_search_narrows_and_clears() {
        use super::super::sheet::{SheetMode, SheetView, UpdateSheetNav};
        use super::super::tests::status;
        use super::super::{Mode, Phase};

        fn app() -> Element {
            use_context_provider(|| UpdateSheetNav(Signal::new(Some(SheetMode::History))));
            let status = use_signal(|| {
                let mut s = status(Phase::UpToDate, Mode::Auto);
                s.history = vec![
                    b(
                        9,
                        vec![
                            c(ChangeKind::New, "صورة جديدة في الكبد", Some("hb")),
                            c(ChangeKind::Fixed, "البنك أسرع", Some("bank")),
                        ],
                    ),
                    b(8, vec![c(ChangeKind::Improved, "إصلاح صورة الدم", Some("hem"))]),
                ];
                Some(s)
            });
            rsx! { SheetView { status } }
        }
        let mut t = crate::tap_dom::TestDom::of(app);
        assert!(t.html().contains("us-search") && !t.html().contains("us-found"), "{}", t.html());
        t.input(&format!("aria-label=\"{}\"", tx::SEARCH), "صورة");
        let page = t.html();
        assert!(page.contains(&tx::found(2, 2)) && page.contains("us-timeline\" hidden"), "{page}");
        t.click(&format!("data-kind=\"{}\"", kind_id(ChangeKind::Improved)));
        assert!(t.html().contains(&tx::found(1, 1)), "{}", t.html());
        t.click(tx::CLEAR);
        assert!(!t.html().contains("us-found"), "{}", t.html());
        t.input_event("change", &format!("aria-label=\"{}\"", tx::ALL_GROUPS), "app");
        assert!(t.html().contains(&tx::found(1, 1)) && t.html().contains("البنك أسرع"), "{}", t.html());
    }
}
