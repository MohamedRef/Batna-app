//! Shared pieces of the 2026-10 redesign that need code, not only a class
//! (the classes are in assets/parts.css, listed in docs/DESIGN.md).

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Ui;
use crate::i18n::views as tx;

/// "3 / 12" that reads left to right inside Arabic text; without the
/// isolation the bidi algorithm shows it as "12 / 3".
#[component]
pub fn Count(done: usize, total: usize) -> Element {
    rsx! {
        span { class: "num-ltr", "{done} / {total}" }
    }
}

/// A course's heading over its share of a list (`Library::by_course`): its
/// Arabic name, then its title. Lectures under it show only their number
/// and real title, so every list reads course → lecture → lesson. Tapping it
/// opens the course map.
#[component]
pub fn CourseHead(short: String, title: String) -> Element {
    let ui = use_context::<Ui>();
    rsx! {
        h3 { class: "course-head",
            button { class: "tap tap-plain course-head-btn", title: crate::i18n::courses::TITLE, onclick: move |_| crate::courses::open_map(ui),
                span { class: "course-head-short", "{short}" }
                bdi { class: "course-head-title", dir: "ltr", lang: "en", "{title}" }
                span { class: "chev", aria_hidden: "true" }
            }
        }
    }
}

/// Focused study: while the calling component is mounted (a running
/// session, a lesson, a question set), the phone's bottom bar hides so the
/// task gets the screen. The component must show its own way out (a close
/// or finish button with `"data-back": "true"`, so Android Back takes it).
/// From 600dp the rail stays: there is room for both.
pub fn use_focus_mode(on: bool) {
    let mut focus = use_context::<Ui>().focus;
    use_effect(use_reactive!(|on| {
        if *focus.peek() != on {
            focus.set(on);
        }
    }));
    use_drop(move || focus.set(false));
}

/// How a list screen lays its items out, Files-app style (Mohamed,
/// 2026-10-29): cards in a grid, a list of rows with a second line, or a
/// compact list of one line each. The markup is the same in all three;
/// `data-layout` on the `ul.coll` picks the look (assets/parts.css).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Cards,
    List,
    Compact,
}

impl Layout {
    pub const ALL: [Self; 3] = [Self::Cards, Self::List, Self::Compact];
    /// Starting layout for collections; saved choices still win.
    pub const DEFAULT: Self = Self::List;

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cards => "cards",
            Self::List => "list",
            Self::Compact => "compact",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Cards => tx::CARDS,
            Self::List => tx::LIST,
            Self::Compact => tx::COMPACT,
        }
    }

    fn icon(self) -> Element {
        match self {
            Self::Cards => crate::icons::layout_cards(),
            Self::List => crate::icons::layout_list(),
            Self::Compact => crate::icons::layout_compact(),
        }
    }
}

/// The order a list screen shows its items in. Each screen offers the ones
/// that mean something for it ([`Collection::sorts`]); the first is its
/// default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    /// The book's order (lecture, then lesson number).
    Book,
    /// What still has work left first, the rest after, each part in book order.
    Left,
    /// Alphabetical by title.
    Name,
    /// Due for review first.
    Due,
}

impl Sort {
    fn label(self) -> &'static str {
        match self {
            Self::Book => tx::SORT_BOOK,
            Self::Left => tx::SORT_LEFT,
            Self::Name => tx::SORT_NAME,
            Self::Due => tx::SORT_DUE,
        }
    }
}

/// One list screen's view choice, kept on this device (a phone and a
/// tablet may well want different ones) as the local setting `view.<key>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewPick {
    pub layout: Layout,
    pub sort: Sort,
    /// Section headers by lecture, where the screen offers them.
    #[serde(default)]
    pub grouped: bool,
}

/// A list screen that offers view choices: its key, its default layout,
/// the orders it offers (first = default) and whether it can group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Collection {
    pub key: &'static str,
    pub layout: Layout,
    pub sorts: &'static [Sort],
    pub groups: bool,
}

/// Choices made while the app runs, so every list re-renders at once.
static PICKS: GlobalSignal<std::collections::BTreeMap<&'static str, ViewPick>> =
    Signal::global(std::collections::BTreeMap::new);

impl Collection {
    fn default_pick(&self) -> ViewPick {
        ViewPick {
            layout: self.layout,
            sort: self.sorts.first().copied().unwrap_or(Sort::Book),
            grouped: false,
        }
    }

    /// The current choice: this run's, else the saved one, else the default.
    /// A saved order this screen no longer offers falls back to its default.
    /// `BATNA_VIEW=cards|list|compact` forces a layout (previews, screenshots).
    pub fn pick(&self) -> ViewPick {
        let saved = PICKS
            .read()
            .get(self.key)
            .copied()
            .or_else(|| crate::store::shared().local_setting::<ViewPick>(&format!("view.{}", self.key)));
        let mut p = saved.unwrap_or_else(|| self.default_pick());
        if !self.sorts.contains(&p.sort) {
            p.sort = self.default_pick().sort;
        }
        p.grouped &= self.groups;
        if let Ok(v) = std::env::var("BATNA_VIEW")
            && let Some(l) = Layout::ALL.into_iter().find(|l| l.as_str() == v)
        {
            p.layout = l;
        }
        p
    }

    pub fn set(&self, p: ViewPick) {
        let key = format!("view.{}", self.key);
        crate::persistence::save(format!("local.{key}"), move || {
            crate::store::shared().put_local_setting(&key, &p).map_err(|e| e.to_string())
        });
        PICKS.write().insert(self.key, p);
    }
}

/// Sorts `items` by `sort`, stably, so each part keeps book order. `left`
/// says whether an item still has work, `due` whether it is due, `name`
/// gives its title.
pub fn sort_by<T>(
    items: &mut [T],
    sort: Sort,
    left: impl Fn(&T) -> bool,
    due: impl Fn(&T) -> bool,
    name: impl Fn(&T) -> String,
) {
    match sort {
        Sort::Book => {}
        Sort::Left => items.sort_by_key(|i| !left(i)),
        Sort::Due => items.sort_by_key(|i| !due(i)),
        Sort::Name => items.sort_by_cached_key(|i| name(i).to_lowercase()),
    }
}

/// The view switcher above a list: the order (a small menu, when the
/// screen offers more than one) and the three layouts as icon buttons.
#[component]
pub fn ViewBar(of: Collection) -> Element {
    let pick = of.pick();
    let mut menu = use_signal(|| false);
    let several = of.sorts.len() > 1 || of.groups;
    rsx! {
        div { class: "view-bar",
            if several {
                div { class: "view-sort",
                    button {
                        class: "view-sort-btn",
                        "aria-haspopup": "menu",
                        "aria-expanded": if menu() { "true" } else { "false" },
                        title: tx::SORT_TITLE,
                        onclick: move |_| menu.toggle(),
                        {crate::icons::sort()}
                        span { {pick.sort.label()} }
                        if pick.grouped { span { class: "view-sort-more", " · {tx::GROUPED_SHORT}" } }
                    }
                    if menu() {
                        // A tap outside closes the menu.
                        button { class: "view-scrim", aria_label: tx::CLOSE_MENU, tabindex: "-1", onclick: move |_| menu.set(false) }
                        div { class: "view-menu", role: "menu", aria_label: tx::SORT_TITLE,
                            for s in of.sorts.iter().copied() {
                                button {
                                    key: "{s:?}",
                                    class: "view-menu-item",
                                    role: "menuitemradio",
                                    "aria-checked": if pick.sort == s { "true" } else { "false" },
                                    onclick: move |_| {
                                        of.set(ViewPick { sort: s, ..of.pick() });
                                        menu.set(false);
                                    },
                                    span { class: "view-tick", aria_hidden: "true", if pick.sort == s { {crate::icons::check()} } }
                                    {s.label()}
                                }
                            }
                            if of.groups {
                                button {
                                    class: "view-menu-item view-menu-sep",
                                    role: "menuitemcheckbox",
                                    "aria-checked": if pick.grouped { "true" } else { "false" },
                                    onclick: move |_| {
                                        let p = of.pick();
                                        of.set(ViewPick { grouped: !p.grouped, ..p });
                                        menu.set(false);
                                    },
                                    span { class: "view-tick", aria_hidden: "true", if pick.grouped { {crate::icons::check()} } }
                                    {tx::GROUPED}
                                }
                            }
                        }
                    }
                }
            }
            div { class: "view-seg", role: "radiogroup", aria_label: tx::LAYOUT_TITLE,
                for l in Layout::ALL {
                    button {
                        key: "{l:?}",
                        role: "radio",
                        "aria-checked": if pick.layout == l { "true" } else { "false" },
                        aria_label: l.label(),
                        title: l.label(),
                        onclick: move |_| of.set(ViewPick { layout: l, ..of.pick() }),
                        {l.icon()}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod view_tests {
    use super::*;

    const DEMO: Collection = Collection {
        key: "test.demo",
        layout: Layout::Cards,
        sorts: &[Sort::Book, Sort::Name],
        groups: false,
    };

    #[test]
    fn a_pick_round_trips_and_old_or_odd_values_fall_back() {
        let p = ViewPick { layout: Layout::Compact, sort: Sort::Left, grouped: true };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<ViewPick>(&json).unwrap(), p);
        // No `grouped` (a value saved before groups existed) reads as off.
        let old: ViewPick = serde_json::from_str(r#"{"layout":"list","sort":"book"}"#).unwrap();
        assert!(!old.grouped);
        assert_eq!(DEMO.default_pick(), ViewPick { layout: Layout::Cards, sort: Sort::Book, grouped: false });
    }

    #[test]
    fn sorting_keeps_book_order_inside_each_part() {
        let mut v = vec![("b", true, false), ("a", false, true), ("c", true, true), ("d", false, false)];
        sort_by(&mut v, Sort::Left, |i| i.1, |i| i.2, |i| i.0.to_string());
        assert_eq!(v.iter().map(|i| i.0).collect::<String>(), "bcad");
        sort_by(&mut v, Sort::Due, |i| i.1, |i| i.2, |i| i.0.to_string());
        assert_eq!(v.iter().map(|i| i.0).collect::<String>(), "cabd");
        sort_by(&mut v, Sort::Name, |i| i.1, |i| i.2, |i| i.0.to_string());
        assert_eq!(v.iter().map(|i| i.0).collect::<String>(), "abcd");
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    const LESSONS: Collection = Collection {
        key: "test.render.lessons",
        layout: Layout::List,
        sorts: &[Sort::Left, Sort::Book, Sort::Name],
        groups: true,
    };
    const ONE_ORDER: Collection =
        Collection { key: "test.render.one", layout: Layout::Cards, sorts: &[Sort::Book], groups: false };

    fn bar(of: Collection, pick: Option<ViewPick>) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct P {
            of: Collection,
            pick: Option<ViewPick>,
        }
        fn app(p: P) -> Element {
            use_hook(|| {
                if let Some(v) = p.pick {
                    PICKS.write().insert(p.of.key, v);
                }
            });
            rsx! { ViewBar { of: p.of } }
        }
        crate::store::use_memory_store();
        let mut dom = VirtualDom::new_with_props(app, P { of, pick });
        dom.rebuild_in_place();
        dioxus::ssr::render(&dom)
    }

    #[test]
    fn the_view_bar_shows_the_order_and_the_three_layouts() {
        let page = bar(LESSONS, None);
        assert!(page.contains(tx::SORT_LEFT), "the first order is the default: {page}");
        for l in Layout::ALL {
            assert!(page.contains(&format!("aria-label=\"{}\"", l.label())), "{page}");
        }
        // The list layout is this screen's default.
        assert!(page.contains(&format!("aria-checked=\"true\" aria-label=\"{}\"", tx::LIST)), "{page}");
        assert!(page.contains("aria-expanded=\"false\"") && !page.contains("role=\"menu\""), "{page}");

        let page = bar(LESSONS, Some(ViewPick { layout: Layout::Compact, sort: Sort::Name, grouped: true }));
        assert!(page.contains(tx::SORT_NAME) && page.contains(tx::GROUPED_SHORT), "{page}");
        assert!(page.contains(&format!("aria-checked=\"true\" aria-label=\"{}\"", tx::COMPACT)), "{page}");

        // One order and no groups: no order menu at all.
        let page = bar(ONE_ORDER, None);
        assert!(!page.contains("view-sort-btn") && page.contains(tx::LAYOUT_TITLE), "{page}");
    }

    #[test]
    fn a_choice_holds_for_the_run_and_odd_ones_fall_back() {
        fn app() -> Element {
            assert_eq!(ONE_ORDER.pick(), ONE_ORDER.default_pick(), "nothing chosen yet");
            // A saved order this screen doesn't offer, and groups it can't show.
            ONE_ORDER.set(ViewPick { layout: Layout::List, sort: Sort::Due, grouped: true });
            assert_eq!(ONE_ORDER.pick(), ViewPick { layout: Layout::List, sort: Sort::Book, grouped: false });
            // A second screen keeps its own (not LESSONS: the store is shared by every test).
            const OTHER: Collection = Collection { key: "test.render.other", ..LESSONS };
            OTHER.set(ViewPick { layout: Layout::Cards, sort: Sort::Book, grouped: true });
            assert_eq!(OTHER.pick(), ViewPick { layout: Layout::Cards, sort: Sort::Book, grouped: true });
            rsx! { "ok" }
        }
        crate::store::use_memory_store();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        assert_eq!(dioxus::ssr::render(&dom), "ok");
        assert_eq!(Layout::Cards.as_str(), "cards");
        assert_eq!(Layout::DEFAULT, Layout::List);
        for s in [Sort::Book, Sort::Left, Sort::Name, Sort::Due] {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn counts_read_left_to_right_and_course_heads_open_the_map() {
        let html = dioxus::ssr::render_element(rsx! { Count { done: 3, total: 12 } });
        assert_eq!(html, "<span class=\"num-ltr\">3 / 12</span>");
        fn app() -> Element {
            rsx! {
                crate::browser_fixture::Shell { screen: crate::Screen::Learn, focus: false,
                    CourseHead { short: "الكبد".to_string(), title: "Hepatology".to_string() }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let page = dioxus::ssr::render(&dom);
        assert!(
            page.contains("الكبد") && page.contains("lang=\"en\"") && page.contains(">Hepatology<"),
            "{page}"
        );
        assert!(page.contains(crate::i18n::courses::TITLE));
    }

    #[test]
    fn the_menu_changes_the_order_the_groups_and_the_layout() {
        const TAPPED: Collection = Collection { key: "test.render.tapped", ..LESSONS };
        fn app() -> Element {
            rsx! { ViewBar { of: TAPPED } }
        }
        crate::store::use_memory_store();
        let mut t = crate::tap_dom::TestDom::of(app);
        t.click("class=\"view-sort-btn\"");
        assert!(t.html().contains("role=\"menu\""), "{}", t.html());
        t.click(&format!("aria-label=\"{}\"", tx::CLOSE_MENU));
        assert!(!t.html().contains("role=\"menu\""));
        t.click("class=\"view-sort-btn\"");
        t.click(&format!(">{}<", tx::SORT_NAME));
        assert_eq!(t.with(|| TAPPED.pick()).sort, Sort::Name);
        let grouped = t.with(|| TAPPED.pick()).grouped;
        t.click("class=\"view-sort-btn\"");
        t.click(&format!(">{}<", tx::GROUPED));
        assert_eq!(t.with(|| TAPPED.pick()).grouped, !grouped, "grouping toggles");
        t.click(&format!("aria-label=\"{}\"", tx::COMPACT));
        assert_eq!(t.with(|| TAPPED.pick()).layout, Layout::Compact);
        assert!(!t.html().contains("role=\"menu\""));
    }

    #[test]
    fn a_course_head_opens_the_course_map() {
        fn app() -> Element {
            rsx! {
                crate::browser_fixture::Shell { screen: crate::Screen::Learn, focus: false,
                    CourseHead { short: "الكبد".to_string(), title: "Hepatology".to_string() }
                }
            }
        }
        crate::store::use_memory_store();
        let mut t = crate::tap_dom::TestDom::of(app);
        t.click("class=\"tap tap-plain course-head-btn\"");
        assert!(t.with(crate::lecture::is_open), "{}", t.html());
        t.with(crate::lecture::close_all);
    }
}
