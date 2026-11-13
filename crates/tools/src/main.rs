//! batna-tools: content checks for Batna.
//!
//! ```text
//! batna-tools validate [--root DIR]   check content/ against the rules (SPLIT.md §4–5)
//! batna-tools anki [--lecture hb.L01] [--verdicts FILE] [--out DIR]   Anki deck (docs/ANKI.md)
//! batna-tools packs build [--out DIR] [--check]   the offline content pack
//! batna-tools verdicts FILE [--approve]   fix list from the app's ✎/✗ export
//! batna-tools update <keygen | manifest | sign | verify>   update feeds (docs/RELEASE.md §5)
//! batna-tools refs [--lecture hb.L01] [--write]   page references against the book (refs.rs)
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use batna_core::content::{Index, Lecture, SCHEMA};
use batna_core::validate::{self, Context};

mod anki;
mod changes;
mod content;
mod ids;
mod lessons;
mod packs;
#[cfg(test)]
mod readme_tests;
mod refs;
#[cfg(test)]
mod testkit;
mod update;
mod verdicts;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out, err) = cli(&args);
    if let Some(out) = out {
        println!("{out}");
    }
    for e in &err {
        eprintln!("{e}");
    }
    code
}

/// One command line: the exit code, what goes to stdout, and the lines for
/// stderr.
fn cli(args: &[String]) -> (ExitCode, Option<String>, Vec<String>) {
    let root = match args.iter().position(|a| a == "--root") {
        Some(i) => match args.get(i + 1) {
            Some(r) => PathBuf::from(r),
            None => return usage(),
        },
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    };
    match args.first().map(String::as_str) {
        Some("validate") => match validate_all(&root) {
            Ok(report) => (ExitCode::SUCCESS, Some(report), Vec::new()),
            Err(mut problems) => {
                problems.push(format!("{} problem(s)", problems.len()));
                (ExitCode::FAILURE, None, problems)
            }
        },
        Some("anki") => report(anki::run(&root, &args[1..])),
        Some("packs") => report(packs::run(&root, &args[1..])),
        Some("verdicts") => report(verdicts::run(&root, &args[1..])),
        Some("update") => report(update::run(&root, &args[1..])),
        Some("refs") => report(refs::run(&root, &args[1..])),
        Some("ids") => report(ids::run(&root, &args[1..])),
        Some("content") => report(content::run(&root, &args[1..])),
        _ => usage(),
    }
}

fn report(result: Result<String, Vec<String>>) -> (ExitCode, Option<String>, Vec<String>) {
    match result {
        Ok(out) => (ExitCode::SUCCESS, Some(out), Vec::new()),
        Err(problems) => (ExitCode::FAILURE, None, problems),
    }
}

/// The value after `name`, if given.
fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn usage() -> (ExitCode, Option<String>, Vec<String>) {
    let text = "usage: batna-tools <validate | anki | packs build | verdicts FILE | update … | refs | ids <list | check --base REF> | content <next-id | new-item | diff | preflight>> [--root DIR] (see the top of crates/tools/src/main.rs)";
    (ExitCode::from(2), None, vec![text.to_owned()])
}

/// Checks `content/index.yaml` and every `content/<chapter>/LNN.yaml`.
/// Returns a one-line summary, or every problem found.
pub fn validate_all(root: &Path) -> Result<String, Vec<String>> {
    let mut problems = Vec::new();
    let content = root.join("content");
    let index: Index = match read_yaml(&content.join("index.yaml")) {
        Ok(i) => i,
        Err(e) => return Err(vec![e]),
    };
    if index.schema != SCHEMA {
        problems.push(format!("content/index.yaml: schema must be {SCHEMA}"));
    }
    // The MSc's courses: every lecture belongs to one.
    let mut courses = 0;
    match read_yaml::<batna_core::program::Program>(&content.join("program.yaml")) {
        Ok(p) => {
            courses = p.courses.len();
            problems.extend(p.problems(&index).into_iter().map(|e| format!("content/program.yaml: {e}")));
        }
        Err(e) => problems.push(e),
    }
    let approved = read_approved(&content.join("approved.txt"));

    let mut items = 0;
    let mut files = 0;
    let mut listed = BTreeSet::new();
    for entry in &index.lectures {
        listed.insert(entry.id.clone());
        let path = lecture_path(&content, &entry.id);
        if !path.exists() {
            continue; // not written yet
        }
        files += 1;
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                problems.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        let lec: Lecture = match serde_yaml_ng::from_str(&text) {
            Ok(l) => l,
            Err(e) => {
                problems.push(format!("{}: {e}", rel(root, &path)));
                continue;
            }
        };
        if lec.lecture != entry.id {
            problems.push(format!(
                "{}: lecture is {}, the file name says {}",
                rel(root, &path),
                lec.lecture,
                entry.id
            ));
        }
        if lec.book.pages != entry.pages {
            problems.push(format!(
                "{}: book pages {:?} differ from content/index.yaml {:?}",
                rel(root, &path),
                lec.book.pages,
                entry.pages
            ));
        }
        problems
            .extend(content::split_options(&text).into_iter().map(|p| format!("{}: {p}", rel(root, &path))));
        let cx = Context { raw_keys: raw_keys(&text), approved: Some(&approved) };
        for p in validate::lecture(&lec, &cx) {
            problems.push(format!("{}: {p}", rel(root, &path)));
        }
        items += lec.items.len();
    }
    // Lecture files that the index doesn't list.
    if let Ok(dirs) = std::fs::read_dir(&content) {
        for dir in dirs.flatten().filter(|d| d.path().is_dir() && d.file_name() != "lessons") {
            let chapter = dir.file_name().to_string_lossy().to_string();
            for f in std::fs::read_dir(dir.path()).into_iter().flatten().flatten() {
                let name = f.file_name().to_string_lossy().to_string();
                if let Some(stem) = name.strip_suffix(".yaml") {
                    let id = format!("{chapter}.{stem}");
                    if !listed.contains(&id) {
                        problems.push(format!("content/{chapter}/{name}: not listed in content/index.yaml"));
                    }
                }
            }
        }
    }
    // The hover glossary (PARITY.md §6.3), when there is one.
    let glossary_path = content.join("glossary.yaml");
    let mut terms = 0;
    let mut glossary = None;
    if glossary_path.exists() {
        match read_yaml::<batna_core::glossary::Glossary>(&glossary_path) {
            Ok(g) => {
                terms = g.entries.len();
                for p in validate::glossary(&g, Some(&approved)) {
                    problems.push(format!("content/glossary.yaml: {p}"));
                }
                glossary = Some(g);
            }
            Err(e) => problems.push(e),
        }
    }
    // Calculators' book quotes (their words are checked by refs, below).
    let calc_quotes = content.join("calc-quotes.yaml");
    if calc_quotes.exists() {
        match read_yaml::<batna_core::content::CalcQuotes>(&calc_quotes) {
            Ok(q) => problems.extend(
                validate::calc_quotes(&q, &index)
                    .into_iter()
                    .map(|p| format!("content/calc-quotes.yaml: {p}")),
            ),
            Err(e) => problems.push(e),
        }
    }
    // Lessons (docs/LESSONS.md), with every abbreviation in the glossary.
    let (_, lessons) = lessons::validate_all(root, &approved, glossary.as_ref(), &mut problems);
    // Nothing left from a template (`batna-tools content new-item`).
    problems.extend(content::unfinished(root));
    // Page references checked against the book (refs.rs).
    problems.extend(refs::check_locks(root, &listed.iter().cloned().collect::<Vec<_>>()));
    if problems.is_empty() {
        Ok(format!(
            "content OK: {courses} courses, {files} of {} lectures written, {items} items, {lessons} lessons, {terms} glossary terms",
            index.lectures.len()
        ))
    } else {
        Err(problems)
    }
}

/// `hb.L01` → `content/hb/L01.yaml`.
fn lecture_path(content: &Path, id: &str) -> PathBuf {
    let (chapter, num) = id.split_once('.').unwrap_or((id, ""));
    content.join(chapter).join(format!("{num}.yaml"))
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_yaml_ng::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// One item id per line; `#` starts a comment. Missing file = nothing approved.
fn read_approved(path: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// The keys written on each item, by id, so typos like `anwser:` are caught.
fn raw_keys(text: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut out = std::collections::BTreeMap::new();
    let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text) else {
        return out;
    };
    let Some(items) = doc.get("items").and_then(|i| i.as_sequence()) else {
        return out;
    };
    for item in items {
        let Some(map) = item.as_mapping() else {
            continue;
        };
        let Some(id) = map.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let keys = map.keys().filter_map(|k| k.as_str().map(String::from)).collect();
        out.insert(id.to_string(), keys);
    }
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn repo_content_is_valid() {
        if let Err(problems) = validate_all(&root()) {
            panic!("{problems:#?}");
        }
    }

    #[test]
    fn a_tree_with_problems_is_reported() {
        let dir = std::env::temp_dir().join(format!("batna-validate-{}", std::process::id()));
        let hb = dir.join("content/hb");
        std::fs::create_dir_all(&hb).unwrap();
        std::fs::write(
            dir.join("content/index.yaml"),
            "schema: batna.content/v1\nlectures:\n  - { id: hb.L01, title: Liver Cirrhosis, pages: [1, 22], chunk: 10 }\n",
        )
        .unwrap();
        let sample = std::fs::read_to_string(root().join("fixtures/content/sample.yaml")).unwrap();
        std::fs::write(
            hb.join("L01.yaml"),
            sample.replace("front: Anatomical", "fornt: x\n    front: Anatomical"),
        )
        .unwrap();
        std::fs::write(hb.join("L99.yaml"), "").unwrap();
        let problems = validate_all(&dir).unwrap_err();
        assert!(problems.iter().any(|p| p.contains("unknown key `fornt`")), "{problems:#?}");
        assert!(problems.iter().any(|p| p.contains("L99.yaml: not listed")), "{problems:#?}");
        assert!(problems.iter().any(|p| p.contains("program.yaml")), "{problems:#?}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    use crate::testkit::{self, strings, write};

    fn run(args: &[&str]) -> (ExitCode, String, Vec<String>) {
        let (code, out, err) = cli(&strings(args));
        (code, out.unwrap_or_default(), err)
    }

    #[test]
    fn the_command_line_reports_and_exits_like_ci_expects() {
        let tree = testkit::tree();
        let root = tree.path().display().to_string();
        let (code, out, err) = run(&["validate", "--root", &root]);
        assert_eq!(
            (code, out.as_str(), err.len()),
            (
                ExitCode::SUCCESS,
                "content OK: 1 courses, 1 of 1 lectures written, 2 items, 1 lessons, 0 glossary terms",
                0
            )
        );

        // Each subcommand reaches its module and its answer goes to stdout.
        let (code, out, _) = run(&["packs", "build", "--check", "--root", &root]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.starts_with("pack OK: 1 lecture(s), 2 items, 1 lesson(s)"), "{out}");
        let (_, out, _) = run(&["ids", "list", "--root", &root]);
        assert!(out.lines().any(|l| l == "item hb.L01.002\tkind card"), "{out}");
        assert!(out.lines().any(|l| l == "lesson hb.L01.lesson1\tlecture hb.L01"), "{out}");
        let (_, out, _) = run(&["content", "next-id", "hb.L01", "--root", &root]);
        assert_eq!(out, "next item id: hb.L01.003\nnext lesson id: hb.L01.lesson2");
        let (_, out, _) = run(&["update", "channels", "--version", "1.0.0-beta.1", "--root", &root]);
        assert_eq!(out, "alpha\nbeta");
        let book = testkit::book(tree.path()).display().to_string();
        let (_, out, _) = run(&["refs", "--root", &root, "--book", &book, "--ocr", "/x", "--checked", "/x"]);
        assert_eq!(out, "hb.L01: 5 refs, 5 quoted, 0 book figures, 0 checked by hand, 0 to look at");
        let deck = tree.path().join("deck").display().to_string();
        let (code, out, _) = run(&["anki", "--merged-only", "--out", &deck, "--root", &root]);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.contains("2 notes") && out.contains("hb.L01: 2 notes"), "{out}");
        let (code, out, err) = run(&["verdicts", "/no/such/export.json", "--root", &root]);
        assert_eq!((code, out.as_str()), (ExitCode::FAILURE, ""));
        assert!(err[0].contains("/no/such/export.json"), "{err:?}");

        // A broken tree fails with every problem and their count last.
        write(
            tree.path(),
            "content/hb/L01.yaml",
            &testkit::LECTURE.replace("pages: [1, 22]", "pages: [1, 20]"),
        );
        let (code, _, err) = run(&["validate", "--root", &root]);
        assert_eq!(code, ExitCode::FAILURE);
        assert!(err[0].contains("book pages [1, 20] differ from content/index.yaml [1, 22]"), "{err:?}");
        assert_eq!(err.last().unwrap(), &format!("{} problem(s)", err.len() - 1));
    }

    #[test]
    fn a_bad_command_line_prints_the_usage() {
        for args in [&[][..], &["frobnicate"], &["validate", "--root"]] {
            let (code, out, err) = run(args);
            assert_eq!((code, out.as_str()), (ExitCode::from(2), ""), "{args:?}");
            assert!(err[0].starts_with("usage: batna-tools <validate"), "{args:?}");
        }
    }

    #[test]
    fn validate_names_what_is_wrong_with_a_lecture_file() {
        let tree = testkit::tree();
        let root = tree.path();
        let problems = |change: &dyn Fn(&Path)| {
            let t = testkit::tree();
            change(t.path());
            validate_all(t.path()).unwrap_err()
        };
        let has = |p: &[String], s: &str| assert!(p.iter().any(|x| x.contains(s)), "{s}: {p:#?}");

        let p = problems(&|r| write(r, "content/index.yaml", INDEX_V0));
        has(&p, "content/index.yaml: schema must be batna.content/v1");
        let p = problems(&|r| {
            write(r, "content/hb/L01.yaml", &testkit::LECTURE.replace("lecture: hb.L01", "lecture: hb.L02"))
        });
        has(&p, "content/hb/L01.yaml: lecture is hb.L02, the file name says hb.L01");
        let p = problems(&|r| write(r, "content/hb/L01.yaml", "lecture: [\n"));
        has(&p, "content/hb/L01.yaml: ");
        let p = problems(&|r| {
            std::fs::remove_file(r.join("content/hb/L01.yaml")).unwrap();
            std::fs::create_dir(r.join("content/hb/L01.yaml")).unwrap();
        });
        has(&p, "L01.yaml: ");
        let p =
            problems(&|r| write(r, "content/glossary.yaml", "schema: batna.glossary/v1\nentries: oops\n"));
        has(&p, "glossary.yaml: ");
        let p = problems(&|r| {
            write(
                r,
                "content/glossary.yaml",
                "schema: batna.glossary/v1\nentries:\n  - { term: HSC, kind: abbr, text: \"\", status: unverified }\n",
            )
        });
        has(&p, "content/glossary.yaml: ");

        // A glossary that knows every abbreviation keeps the tree valid and is counted.
        write(
            root,
            "content/glossary.yaml",
            "schema: batna.glossary/v1\nentries:\n  - { term: HSC, kind: abbr, text: Hepatic stellate cell, status: unverified }\n",
        );
        let ok = validate_all(root).unwrap();
        assert!(ok.ends_with("1 lessons, 1 glossary terms"), "{ok}");
    }

    const INDEX_V0: &str = "schema: batna.content/v0\nlectures:\n  - { id: hb.L01, title: Liver Cirrhosis, pages: [1, 22], chunk: 10 }\n";

    #[test]
    fn approved_ids_and_raw_keys_read_what_the_files_say() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("approved.txt");
        std::fs::write(&f, "# header\nhb.L01.001  # card\n\n  hb.L01.002\n").unwrap();
        assert_eq!(read_approved(&f).into_iter().collect::<Vec<_>>(), ["hb.L01.001", "hb.L01.002"]);
        assert!(read_approved(&dir.path().join("missing.txt")).is_empty());

        let keys =
            raw_keys("items:\n  - { id: a, kind: card, fornt: x }\n  - just text\n  - { kind: card }\n");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys["a"], ["id", "kind", "fornt"]);
        assert!(raw_keys("items: [").is_empty());
        assert!(raw_keys("title: no items\n").is_empty());
    }
}
