//! The course chip and sheet, and the MSc map, tapped the way he uses them
//! (`test_dom`).

use batna_core::program::CourseStatus;

use super::*;
use crate::i18n::courses as tx;
use crate::prefs::DevicePrefs;
use crate::test_dom::{El, Screen as Dom, norm, shows};
use crate::{Screen, Ui};

fn is(class: &'static str) -> impl Fn(&El) -> bool {
    move |e| e.has_class(class)
}

fn frame(children: Element) -> Element {
    let ui = use_context_provider(|| Ui {
        screen: Signal::new(Screen::Learn),
        prefs: Signal::new(DevicePrefs::default()),
        focus: Signal::new(false),
    });
    crate::store::use_provide_review();
    use_hook(|| focus::set(None));
    let shown = crate::lecture::shown(Screen::Learn);
    rsx! {
        p { class: "t-screen", "{ui.screen:?}" }
        p { class: "t-shown", "{shown:?}" }
        {children}
    }
}

fn sheet() -> Element {
    frame(rsx! {
        CourseChip {}
        Picker {}
    })
}

fn map() -> Element {
    frame(rsx! { Map {} })
}

/// A course with lectures that is not hepatology, so changing it leaves
/// the lessons other tests use in view.
fn other_course() -> String {
    let lib = Library::get();
    summaries(lib, |c| c.status)
        .into_iter()
        .find(|s| s.lectures > 0 && s.course.id != "hb")
        .expect("a second course")
        .course
        .id
}

#[test]
fn the_chip_opens_the_sheet_and_a_pick_narrows_the_screens() {
    crate::store::use_memory_store();
    let mut d = Dom::new(sheet);
    let chip = d.find(is("cf-chip")).remove(0);
    assert_eq!(chip.attr("data-all"), Some("true"));
    assert_eq!(chip.attr("aria-expanded"), Some("false"));
    assert!(chip.text.contains(tx::ALL));
    assert!(!shows(&d, "cf-list"));
    d.click(is("cf-chip"));
    assert_eq!(d.find(is("cf-chip")).remove(0).attr("aria-expanded"), Some("true"));
    // Every course with lectures, plus "all courses", checked now.
    let lib = Library::get();
    let listed = focus::listed(summaries(lib, |c| c.status));
    let rows = d.find(is("cf-row"));
    assert_eq!(rows.len(), listed.len() + 1);
    assert_eq!(rows[0].attr("aria-checked"), Some("true"));
    assert!(rows[0].text.contains(&tx::all_sub(listed.len())));
    // Pick hepatology: the chip names it, the sheet closes, the store keeps it.
    let hb = lib.program.courses.iter().find(|c| c.id == "hb").unwrap();
    d.click(|e| e.has_class("cf-row") && e.text.contains(&hb.short));
    assert_eq!(d.read(focus::current), Some("hb".to_string()));
    assert!(!shows(&d, "cf-list"));
    let chip = d.find(is("cf-chip")).remove(0);
    assert_eq!(chip.attr("data-all"), Some("false"));
    assert!(norm(&chip.text).contains(&norm(&hb.title)));
    assert!(d.read(|| focus::shows("hb.L01")));
    assert!(!d.read(|| focus::shows(&format!("{}.L01", other_course()))));
    // Opening a lecture of another course moves the focus there.
    let other = other_course();
    d.read(|| focus::follow(&format!("{other}.L01")));
    assert_eq!(d.read(focus::current), Some(other.clone()));
    // A course that isn't in the program is refused; all courses again.
    d.read(|| focus::set(Some("nope".into())));
    assert_eq!(d.read(focus::current), None);
    d.read(|| focus::follow("hb.L01"));
    assert_eq!(d.read(focus::current), None, "with all in view, following changes nothing");
}

#[test]
fn the_sheet_links_to_the_map_and_closes() {
    crate::store::use_memory_store();
    let mut d = Dom::new(sheet);
    d.click(is("cf-chip"));
    d.click(is("cf-map"));
    assert!(!shows(&d, "cf-list"));
    assert_eq!(d.find(is("t-shown")).remove(0).text, format!("{:?}", Some(MAP)));
    // Tapping the chip twice opens and closes it.
    d.click(is("cf-chip"));
    assert!(shows(&d, "cf-list"));
    d.click(is("cf-chip"));
    assert!(!shows(&d, "cf-list"));
}

#[test]
fn the_map_shows_hours_and_a_tile_opens_its_course() {
    crate::store::use_memory_store();
    let mut d = Dom::new(map);
    let lib = Library::get();
    let all = d.read(|| summaries(lib, status));
    let (done, now, total) = hours(&all);
    let html = d.html();
    assert!(html.contains(&format!("{}/{}", fmt_hours(done + now), fmt_hours(total))), "{html}");
    assert!(html.contains(&tx::hours_line(&fmt_hours(total), all.len())));
    assert_eq!(d.find(is("cm-tile")).len(), all.len());
    // A group per status in use.
    let groups = d.find(is("cm-group")).len();
    let used: std::collections::HashSet<_> = all.iter().map(|s| s.status).collect();
    assert_eq!(groups, used.len());
    // A tap opens a tile's detail; a second tap closes it.
    d.click(|e| e.has_class("cm-tile") && e.attr("aria-controls") == Some("cm-detail-hb"));
    assert!(shows(&d, "cm-detail"));
    assert_eq!(d.find(|e| e.has_class("cm-tile") && e.attr("aria-expanded") == Some("true")).len(), 1);
    d.click(|e| e.has_class("cm-tile") && e.attr("aria-controls") == Some("cm-detail-hb"));
    assert!(!shows(&d, "cm-detail"));
    // Open its lectures: Learn, narrowed to the course, page closed.
    d.click(|e| e.has_class("cm-tile") && e.attr("aria-controls") == Some("cm-detail-hb"));
    d.click(is("cm-open"));
    assert_eq!(d.find(is("t-screen")).remove(0).text, "Learn");
    assert_eq!(d.read(focus::current), Some("hb".to_string()));
    assert!(!d.read(crate::lecture::is_open));
}

#[test]
fn a_course_status_changes_on_the_map_and_is_kept() {
    crate::store::use_memory_store();
    let mut d = Dom::new(map);
    let id = other_course();
    let lib = Library::get();
    let before = d.read(|| status(lib.program.courses.iter().find(|c| c.id == id).unwrap()));
    let to = if before == CourseStatus::Archived { CourseStatus::Later } else { CourseStatus::Archived };
    let controls = format!("cm-detail-{id}");
    d.click(|e| e.has_class("cm-tile") && e.attr("aria-controls") == Some(controls.as_str()));
    let label = map::status_label(to);
    d.click_in(is("cm-detail"), |e| e.attr("role") == Some("radio") && e.text == label);
    let course = lib.program.courses.iter().find(|c| c.id == id).unwrap();
    assert_eq!(d.read(|| status(course)), to);
    assert_eq!(crate::store::shared().setting::<String>(&format!("{KEY}{id}")), Some(to.key().to_string()));
    let checked = d.find_in(is("cm-detail"), |e| e.attr("aria-checked") == Some("true")).remove(0);
    assert_eq!(checked.text, label);
    // Its tile moved to that status's group.
    let group = d.find(|e| e.has_class("cm-group") && e.attr("aria-label") == Some(label)).remove(0);
    let tiles = d.find_in(|e| e.has_class("cm-group") && e.attr("aria-label") == Some(label), is("cm-tile"));
    assert!(tiles.iter().any(|t| t.attr("aria-controls") == Some(controls.as_str())), "{}", group.text);
    // A reload (a sync) keeps it; then it goes back.
    d.read(reload);
    assert_eq!(d.read(|| status(course)), to);
    d.read(|| set_status(&id, before));
    assert_eq!(d.read(|| status(course)), before);
}

#[test]
fn every_status_and_group_has_words() {
    use batna_core::program::Group;
    for st in CourseStatus::ALL {
        assert!(!map::status_label(st).is_empty());
    }
    let keys: std::collections::HashSet<_> =
        [Group::Research, Group::Clinical, Group::Subspecialty, Group::BasicScience]
            .into_iter()
            .map(map::group_key)
            .collect();
    assert_eq!(keys.len(), 4);
}
