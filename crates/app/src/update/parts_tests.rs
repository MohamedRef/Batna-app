//! The content pack fetched in parts (#1354): text every time, pictures only
//! when they changed, and the full pack whenever the parts don't join into
//! the signed bytes.

use super::tests::{serve_logged, updater};
use super::*;

/// A small real pack with pictures: one written lecture and its lessons.
fn pack(title: &str) -> Pack {
    let lib = crate::content::Library::get();
    let file = lib
        .lessons
        .values()
        .find(|f| {
            !Pack::new(lib.index.clone(), Vec::new()).with_lessons(vec![(*f).clone()]).pictures().0.is_empty()
        })
        .expect("a lesson file with a picture")
        .clone();
    let mut index = lib.index.clone();
    index.lectures.retain(|e| e.id == file.lecture);
    index.lectures[0].title = title.into();
    let lecture = lib.lectures.get(&file.lecture).unwrap().clone();
    let mut p = Pack::new(index, vec![lecture]).with_lessons(vec![file]);
    p.program = None;
    p
}

fn asset(platform: UpdatePlatform, file: &str, bytes: &[u8]) -> UpdateAsset {
    UpdateAsset {
        platform,
        file: file.into(),
        size: bytes.len() as u64,
        sha256: sha256_hex(bytes),
        version_code: None,
        build: None,
    }
}

/// A signed preview feed of `build` with the full pack and its two parts
/// (`text` and `pictures` as given, so a test can break them).
fn feed(
    signer: &ed25519_dalek::SigningKey,
    build: u64,
    full: &[u8],
    text: &[u8],
    pictures: &[u8],
) -> tempfile::TempDir {
    use base64::Engine as _;
    use ed25519_dalek::Signer as _;
    let dir = tempfile::tempdir().unwrap();
    let files = [
        (UpdatePlatform::Content, format!("full-{build}.pack"), full),
        (UpdatePlatform::ContentText, format!("text-{build}.pack"), text),
        (UpdatePlatform::ContentPictures, format!("pictures-{build}.pack"), pictures),
    ];
    let manifest = UpdateManifest {
        format: batna_core::update::UPDATE_FORMAT,
        channel: Channel::Preview,
        version: "0.1.0".into(),
        build,
        commit: None,
        published_at: format!("2026-11-16T08:{:02}:00Z", build % 60),
        notes: UpdateNotes::default(),
        history: Vec::new(),
        assets: files.iter().map(|(p, f, b)| asset(*p, f, b)).collect(),
        ui_build: Some(build),
    };
    let json = serde_json::to_vec(&manifest).unwrap();
    fs::write(dir.path().join(MANIFEST_FILE), &json).unwrap();
    let sig = base64::engine::general_purpose::STANDARD.encode(signer.sign(&json).to_bytes());
    fs::write(dir.path().join(SIGNATURE_FILE), sig).unwrap();
    for (_, f, b) in files {
        fs::write(dir.path().join(f), b).unwrap();
    }
    dir
}

fn fetched(seen: &super::tests::Seen) -> Vec<String> {
    let mut out: Vec<String> =
        seen.lock().unwrap().iter().map(|(p, _)| p.clone()).filter(|p| p.ends_with(".pack")).collect();
    out.sort();
    out
}

#[test]
fn a_text_fix_downloads_the_text_and_keeps_the_pictures() {
    let signer = ed25519_dalek::SigningKey::from_bytes(&[9; 32]);
    let info = BuildInfo { key: Some(signer.verifying_key()), ..super::tests::build("0.1.0", 40, true) };
    let first = pack("First title");
    let (text, pictures) = first.split();
    let (full, text, pictures) = (first.encode(), text.encode(), pictures.encode());
    let served = feed(&signer, 45, &full, &text, &pictures);
    let (url, seen) = serve_logged(served.path().to_path_buf(), None);
    let (u, data) = updater(&url, info.clone());
    u.check(true);
    assert_eq!(u.status().content_check, Some(ContentCheck::New));
    assert_eq!(fetched(&seen), ["pictures-45.pack", "text-45.pack"], "the parts, not the full pack");
    assert_eq!(fs::read(data.path().join("updates").join(PACK_FILE)).unwrap(), full, "kept as the full pack");
    assert!(u.apply_content());

    // A one-word fix: new text, the same pictures.
    let second = pack("Second title");
    let (text2, pictures2) = second.split();
    let (full2, text2, pictures2) = (second.encode(), text2.encode(), pictures2.encode());
    assert_eq!(pictures2, pictures);
    let served2 = feed(&signer, 46, &full2, &text2, &pictures2);
    let (url2, seen2) = serve_logged(served2.path().to_path_buf(), None);
    let u2 = super::tests::updater_in(data.path(), &url2, info);
    u2.check(true);
    assert_eq!(u2.status().content_check, Some(ContentCheck::New));
    assert_eq!(fetched(&seen2), ["text-46.pack"], "the pictures come from this device");
    assert_eq!(fs::read(data.path().join("updates").join(PACK_FILE)).unwrap(), full2);
}

#[test]
fn parts_that_do_not_join_into_the_signed_pack_fall_back_to_it() {
    let signer = ed25519_dalek::SigningKey::from_bytes(&[10; 32]);
    let info = BuildInfo { key: Some(signer.verifying_key()), ..super::tests::build("0.1.0", 40, true) };
    let p = pack("Title");
    let (_, pictures) = p.split();
    let full = p.encode();
    // A text part from other content: it joins, but not into these bytes.
    let other = pack("Other").split().0.encode();
    let served = feed(&signer, 45, &full, &other, &pictures.encode());
    let (url, seen) = serve_logged(served.path().to_path_buf(), None);
    let (u, data) = updater(&url, info);
    u.check(true);
    assert_eq!(u.status().content_check, Some(ContentCheck::New));
    assert!(fetched(&seen).contains(&"full-45.pack".to_string()), "{:?}", fetched(&seen));
    assert_eq!(fs::read(data.path().join("updates").join(PACK_FILE)).unwrap(), full);
    // Pictures from a failed join are not kept.
    assert!(
        fs::read_dir(data.path().join("updates"))
            .unwrap()
            .flatten()
            .all(|e| !e.file_name().to_string_lossy().starts_with("pictures-"))
    );
}
