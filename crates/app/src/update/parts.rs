//! This device's copy of the content pack's pictures part (#1354): one
//! file, `content.pictures-<sha256>.pack`, beside the pack in use. A feed
//! whose pictures part has the same hash is joined with it instead of
//! downloading 16 MB of pictures again.
//!
//! The name starts with `content.` on purpose: `Updater::clear_downloads`
//! runs on every check that finds the app up to date, when an app download
//! starts, after an install and on a channel switch, and it removes
//! everything in the folder except the `content.*` files. Under any other
//! name the kept part was gone before the next content update (#1367).

use std::fs;
use std::path::Path;

use batna_core::update::UpdateAsset;

use super::sha256_hex;
use crate::diag::report;

pub(super) const PREFIX: &str = "content.pictures-";

fn name(sha: &str) -> String {
    format!("{PREFIX}{sha}.pack")
}

/// The kept pictures part when it is the feed's (same hash, still whole).
pub(super) fn kept_pictures(dir: &Path, asset: &UpdateAsset) -> Option<Vec<u8>> {
    let bytes = fs::read(dir.join(name(&asset.sha256))).ok()?;
    (sha256_hex(&bytes) == asset.sha256).then_some(bytes)
}

/// Keeps `bytes` as the pictures part and lets any older one go.
pub(super) fn keep_pictures(dir: &Path, sha: &str, bytes: &[u8]) {
    let keep = name(sha);
    let tmp = dir.join(format!("{keep}.part"));
    let written = fs::create_dir_all(dir)
        .and_then(|()| fs::write(&tmp, bytes))
        .and_then(|()| fs::rename(&tmp, dir.join(&keep)));
    if let Err(e) = written {
        let _ = fs::remove_file(&tmp);
        report!("pictures part not kept: {e}");
        return;
    }
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        if file.starts_with(PREFIX) && file != keep {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use batna_core::update::UpdatePlatform;

    fn asset(bytes: &[u8]) -> UpdateAsset {
        UpdateAsset {
            platform: UpdatePlatform::ContentPictures,
            file: "p.pack".into(),
            size: bytes.len() as u64,
            sha256: sha256_hex(bytes),
            version_code: None,
            build: None,
        }
    }

    #[test]
    fn one_pictures_part_is_kept_and_found_by_its_hash() {
        let dir = tempfile::tempdir().unwrap();
        let (old, new) = (b"old pictures".as_slice(), b"new pictures".as_slice());
        assert_eq!(kept_pictures(dir.path(), &asset(old)), None);
        keep_pictures(dir.path(), &sha256_hex(old), old);
        assert_eq!(kept_pictures(dir.path(), &asset(old)).as_deref(), Some(old));
        keep_pictures(dir.path(), &sha256_hex(new), new);
        assert_eq!(kept_pictures(dir.path(), &asset(old)), None, "the older one is gone");
        assert_eq!(kept_pictures(dir.path(), &asset(new)).as_deref(), Some(new));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        // A damaged copy is not used.
        fs::write(dir.path().join(name(&sha256_hex(new))), b"cut").unwrap();
        assert_eq!(kept_pictures(dir.path(), &asset(new)), None);
    }
}
