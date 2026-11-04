//! `batna-tools ids`: the content ids Mohamed's records point at
//! (`batna_core::ids`, `docs/CONTENT-IDS.md`).
//!
//! - `ids check --base REF` compares the working tree's content with the
//!   content at that git commit (CI: `HEAD^1`), and fails on a removed or
//!   repurposed id that `content/ids-retired.txt` doesn't name.
//! - `ids check --base-pack FILE` compares with a published content pack.
//! - `ids list` prints every id and what it hangs under.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use batna_core::ids::{Registry, compare, parse_retired, registry};
use batna_core::pack::Pack;

pub const RETIRED: &str = "content/ids-retired.txt";

type Res<T> = Result<T, Vec<String>>;

pub fn run(root: &Path, args: &[String]) -> Res<String> {
    match args.first().map(String::as_str) {
        Some("list") => {
            let ids = registry(&pack_at(root)?);
            Ok(ids.iter().map(|(id, parent)| format!("{id}\t{parent}")).collect::<Vec<_>>().join("\n"))
        }
        Some("check") => {
            let base = match (crate::flag(args, "--base"), crate::flag(args, "--base-pack")) {
                (Some(rev), None) => registry(&pack_at_rev(root, rev)?),
                (None, Some(file)) => {
                    let bytes = std::fs::read(file).map_err(|e| vec![format!("{file}: {e}")])?;
                    registry(&Pack::decode(&bytes).map_err(|e| vec![format!("{file}: {e}")])?)
                }
                _ => return Err(vec!["usage: batna-tools ids check (--base REF | --base-pack FILE)".into()]),
            };
            check(root, &base, &registry(&pack_at(root)?))
        }
        _ => Err(vec!["usage: batna-tools ids <list | check (--base REF | --base-pack FILE)>".into()]),
    }
}

/// Compares; Ok with the report, or Err with it when it is breaking.
pub fn check(root: &Path, base: &Registry, head: &Registry) -> Res<String> {
    let retired = match std::fs::read_to_string(root.join(RETIRED)) {
        Ok(t) => parse_retired(&t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
        Err(e) => return Err(vec![format!("{RETIRED}: {e}")]),
    };
    let report = compare(base, head, &retired);
    let mut lines: Vec<String> = report.lines().into_iter().map(|l| format!("ids: {l}")).collect();
    if report.breaking() {
        lines.push(format!(
            "content ids his records point at were removed or repurposed. Keep the id (fix the text instead), \
             or, after a deliberate migration decision, list it in {RETIRED}"
        ));
        return Err(lines);
    }
    lines.push(format!("ids: {} checked, no breaking change", head.len()));
    Ok(lines.join("\n"))
}

/// The content of a checkout as one pack (no stylesheets or images:
/// ids only).
pub fn pack_at(root: &Path) -> Res<Pack> {
    let (index, lectures) = crate::packs::load(root)?;
    let lessons = match root.join("content/lessons").exists() {
        true => crate::lessons::load(root)?.into_iter().map(|(_, f, _)| f).collect(),
        false => Vec::new(),
    };
    let mut pack = Pack::new(index, lectures).with_lessons(lessons);
    if let Ok(p) = crate::read_yaml(&root.join("content/program.yaml")) {
        pack = pack.with_program(p);
    }
    Ok(pack)
}

/// The content at git commit `rev`: `content/` exported to a scratch
/// directory (removed afterwards) and read like a checkout.
pub fn pack_at_rev(root: &Path, rev: &str) -> Res<Pack> {
    let scratch = std::env::temp_dir().join(format!(
        "batna-ids-{}-{}",
        std::process::id(),
        rev.replace(['/', '^', '~'], "_")
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).map_err(|e| vec![format!("{}: {e}", scratch.display())])?;
    let result = (|| {
        let archive = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["archive", "--format=tar", rev, "--", "content"])
            .output()
            .map_err(|e| vec![format!("git archive: {e}")])?;
        if !archive.status.success() {
            return Err(vec![format!(
                "git archive {rev}: {}",
                String::from_utf8_lossy(&archive.stderr).trim()
            )]);
        }
        let tar = scratch.join("content.tar");
        std::fs::write(&tar, &archive.stdout).map_err(|e| vec![e.to_string()])?;
        let status = Command::new("tar")
            .arg("-xf")
            .arg(&tar)
            .arg("-C")
            .arg(&scratch)
            .status()
            .map_err(|e| vec![format!("tar: {e}")])?;
        if !status.success() {
            return Err(vec![format!("tar couldn't unpack {rev}'s content")]);
        }
        pack_at(&scratch)
    })();
    let _ = std::fs::remove_dir_all(&scratch);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_commit_and_the_working_tree_agree() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let Ok(at_head) = pack_at_rev(&root, "HEAD") else { return }; // not a git checkout
        let ids = registry(&at_head);
        assert!(ids.len() > 1000, "{}", ids.len());
        assert!(check(&root, &ids, &ids).is_ok());
    }

    use crate::testkit::{self, strings, write};

    #[test]
    fn check_compares_with_a_commit_or_a_published_pack() {
        let tree = testkit::tree();
        let root = tree.path();
        testkit::commit_all(root, "content");
        let cli = |args: &[&str]| run(root, &strings(args));
        let ok = cli(&["check", "--base", "HEAD"]).unwrap();
        assert_eq!(ok, "ids: 6 checked, no breaking change");
        let built = root.join("dist").display().to_string();
        crate::packs::run(root, &strings(&["build", "--out", &built])).unwrap();
        let pack = format!("{built}/batna-content.pack");
        assert_eq!(cli(&["check", "--base-pack", &pack]).unwrap(), ok);

        // A lesson removed from the working tree breaks both.
        std::fs::remove_file(root.join("content/lessons/hb/L01.yaml")).unwrap();
        let err = cli(&["check", "--base", "HEAD"]).unwrap_err();
        assert!(err.iter().any(|l| l.contains("lesson hb.L01.lesson1")), "{err:?}");
        assert!(err.last().unwrap().contains("list it in content/ids-retired.txt"), "{err:?}");
        assert_eq!(cli(&["check", "--base-pack", &pack]).unwrap_err(), err);

        let usage = |args: &[&str]| cli(args).unwrap_err()[0].clone();
        assert!(usage(&["check"]).starts_with("usage: batna-tools ids check"));
        assert!(
            usage(&["check", "--base", "HEAD", "--base-pack", &pack])
                .starts_with("usage: batna-tools ids check")
        );
        assert!(usage(&[]).starts_with("usage: batna-tools ids <list"));
        assert!(usage(&["check", "--base-pack", "/no/pack"]).starts_with("/no/pack: "));
        write(root, "junk.pack", "junk");
        let junk = root.join("junk.pack").display().to_string();
        assert!(usage(&["check", "--base-pack", &junk]).starts_with(&format!("{junk}: ")));
        assert!(usage(&["check", "--base", "nope"]).starts_with("git archive nope: "));
        // An unreadable retired list is an error, not "nothing retired".
        std::fs::create_dir_all(root.join(RETIRED)).unwrap();
        assert!(check(root, &Registry::new(), &Registry::new()).unwrap_err()[0].starts_with(RETIRED));
    }

    #[test]
    fn a_checkout_without_lessons_or_program_still_packs_its_ids() {
        let tree = testkit::tree();
        let root = tree.path();
        std::fs::remove_dir_all(root.join("content/lessons")).unwrap();
        std::fs::remove_file(root.join("content/program.yaml")).unwrap();
        let pack = pack_at(root).unwrap();
        assert!(pack.program.is_none() && pack.lessons.is_empty());
        let ids = registry(&pack);
        assert!(ids.contains_key("item hb.L01.002") && !ids.contains_key("lesson hb.L01.lesson1"), "{ids:?}");
        write(root, "content/hb/L01.yaml", "lecture: [");
        assert!(pack_at(root).unwrap_err()[0].contains("L01.yaml"));
    }

    #[test]
    fn a_removed_id_fails_until_it_is_retired() {
        let dir = tempfile::tempdir().unwrap();
        let mut base = Registry::new();
        base.insert("item hb.L01.001".into(), "kind mcq".into());
        let err = check(dir.path(), &base, &Registry::new()).unwrap_err();
        assert!(err.last().unwrap().contains(RETIRED), "{err:?}");
        std::fs::create_dir_all(dir.path().join("content")).unwrap();
        std::fs::write(dir.path().join(RETIRED), "item hb.L01.001  # merged\n").unwrap();
        assert!(check(dir.path(), &base, &Registry::new()).is_ok());
    }
}
