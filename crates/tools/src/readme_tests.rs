//! README.md states content numbers by hand; these tests keep them equal
//! to the repo (#803, #818), so a PR that adds a lecture or a calculator
//! fails until the README says so.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn readme() -> String {
    std::fs::read_to_string(root().join("README.md")).expect("README.md")
}

fn lecture_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| {
                    let n = e.file_name();
                    let n = n.to_string_lossy();
                    n.starts_with('L') && n.ends_with(".yaml")
                })
                .count()
        })
        .unwrap_or(0)
}

/// `| Hepatology (`hb`) | 20 | 20 |` → ("hb", 20, 20); the Total row → ("", …).
fn content_rows(readme: &str) -> Vec<(String, usize, usize)> {
    readme
        .lines()
        .skip_while(|l| !l.starts_with("| Course | Lectures | Lesson files |"))
        .skip(2)
        .take_while(|l| l.starts_with('|'))
        .map(|l| {
            let cells: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
            let id = cells[0].split('`').nth(1).unwrap_or("").to_owned();
            let num = |s: &str| s.trim_matches('*').parse::<usize>().unwrap_or_else(|_| panic!("README row {l}"));
            (id, num(cells[1]), num(cells[2]))
        })
        .collect()
}

#[test]
fn readme_lecture_counts_match_content() {
    let rows = content_rows(&readme());
    assert!(rows.len() > 1, "README content table not found");
    let (mut lectures, mut lessons) = (0, 0);
    for (id, l, f) in rows.iter().filter(|r| !r.0.is_empty()) {
        let have = lecture_files(&root().join("content").join(id));
        let have_lessons = lecture_files(&root().join("content/lessons").join(id));
        assert_eq!(*l, have, "README says {l} lectures for {id}, content/{id} has {have}");
        assert_eq!(*f, have_lessons, "README says {f} lesson files for {id}, there are {have_lessons}");
        lectures += have;
        lessons += have_lessons;
    }
    let total = rows.iter().find(|r| r.0.is_empty()).expect("Total row");
    assert_eq!((total.1, total.2), (lectures, lessons), "README Total row");
    for entry in std::fs::read_dir(root().join("content")).unwrap().filter_map(Result::ok) {
        let id = entry.file_name().to_string_lossy().into_owned();
        if lecture_files(&entry.path()) > 0 {
            assert!(rows.iter().any(|r| r.0 == id), "content/{id} has lectures but no README row");
        }
    }
}

#[test]
fn readme_calculator_count_matches_registry() {
    let n = batna_core::calc::all().len();
    let line = format!("Plus {n} calculators");
    assert!(readme().contains(&line), "README should say \"{line}\"");
}
