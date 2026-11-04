//! A small content tree for the tools' tests: one lecture (`hb.L01`, two
//! items), one lesson, the program, and the page-reference lock, so
//! `validate` passes on it. Tests break one thing at a time and check the
//! problem they get. The "book" page is a sentence written here, not the
//! textbook's text.

use std::path::{Path, PathBuf};
use std::process::Command;

pub const INDEX: &str = "schema: batna.content/v1\nlectures:\n  - { id: hb.L01, title: Liver Cirrhosis, pages: [1, 22], chunk: 10 }\n";

pub const PROGRAM: &str = "schema: batna.program/v1\ntitle: MSc\ncourses:\n  - { id: hb, title: Hepatology, short: كبد, hours: 2, group: subspecialty, semester: 1, status: active }\n";

pub const LECTURE: &str = r#"schema: batna.content/v1
lecture: hb.L01
title: Liver Cirrhosis
author: Dr. Mohamed Bakosh
book: { chapter: 1, pages: [1, 22] }
topics:
  - { id: definition, title: Definition and causes }
items:
  - id: hb.L01.001
    kind: card
    topic: definition
    front: Anatomical definition of cirrhosis?
    back: A diffuse process with fibrosis and regenerative nodules.
    src: { page: 1, quote: "diffuse process with fibrosis and regenerative nodule formation" }
    status: unverified
  - id: hb.L01.002
    kind: card
    topic: definition
    front: Which cells are the major fibrogenic cells of the liver?
    back: The hepatic stellate cells, in the space of Disse.
    src: { page: 1, quote: "The hepatic stellate cells, the major fibrogenic cells in the liver" }
    status: unverified
"#;

pub const LESSON: &str = r#"schema: batna.lesson/v1
lecture: hb.L01
lessons:
  - id: hb.L01.lesson1
    title: "يعني إيه cirrhosis؟"
    hook: "حكاية الـ cirrhosis من أولها."
    minutes: 3
    pages: [1, 2]
    topics: [definition]
    goals: ["تعرّف الـ cirrhosis بالكلمتين"]
    status: unverified
    steps:
      - kind: explain
        page: 1
        title: "التعريف"
        quote: "diffuse process with fibrosis and regenerative nodule formation"
        text: "الـ **cirrhosis** فيها **fibrosis** و **regenerative nodules** مع بعض."
      - kind: explain
        page: 1
        title: "الخلية المسؤولة"
        quote: "The hepatic stellate cells, the major fibrogenic cells in the liver"
        text: "الـ **hepatic stellate cells** هي الـ **major fibrogenic cells** في الكبد."
      - kind: check
        page: 1
        quote: "The hepatic stellate cells, the major fibrogenic cells in the liver"
        question: "Which cells are the **major fibrogenic cells** of the liver?"
        options:
          - "Hepatic stellate cells"
          - "Kupffer cells"
          - "Cholangiocytes"
        answer: 0
        why:
          - "صح: الـ stellate cells هي اللي بتعمل collagen."
          - "الـ Kupffer cells دي macrophages."
          - "الـ cholangiocytes بطانة القنوات المرارية."
    recap: ["fibrosis + nodules = cirrhosis"]
    practice: [hb.L01.001]
"#;

/// One printed page holding both quotes.
pub const BOOK_PAGE: &str = "=== book page 1 ===\nCirrhosis is a diffuse process with fibrosis and regenerative nodule formation.\nThe hepatic stellate cells, the major fibrogenic cells in the liver, sit in the space of Disse.\n";

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn write(root: &Path, path: &str, text: &str) {
    let p = root.join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// The book folder for `refs --book`, outside `content/`.
pub fn book(root: &Path) -> PathBuf {
    root.join("book-text")
}

/// The whole tree, lock included (written by the real `refs --write`).
pub fn tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "content/index.yaml", INDEX);
    write(root, "content/program.yaml", PROGRAM);
    write(root, "content/hb/L01.yaml", LECTURE);
    write(root, "content/lessons/hb/L01.yaml", LESSON);
    write(root, "book-text/L01.txt", BOOK_PAGE);
    let sample = std::fs::read_to_string(repo_root().join("fixtures/content/sample.yaml")).unwrap();
    write(root, "fixtures/content/sample.yaml", &sample);
    relock(root);
    dir
}

/// Re-runs `refs --write` against the tree's book page.
pub fn relock(root: &Path) {
    let book = book(root).display().to_string();
    let args = strings(&["--book", &book, "--ocr", "/nonexistent", "--checked", "/nonexistent", "--write"]);
    crate::refs::run(root, &args).unwrap();
}

pub fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_owned()).collect()
}

pub fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Makes `root` a repository with everything committed.
pub fn commit_all(root: &Path, message: &str) {
    if !root.join(".git").exists() {
        for args in [&["init", "-q"][..], &["config", "user.email", "t@t"], &["config", "user.name", "t"]] {
            git(root, args);
        }
    }
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "--allow-empty", "-m", message]);
}
