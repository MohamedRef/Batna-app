//! `batna-tools allam`: Dr. Mahmoud Allam's revision course, sealed in git
//! (docs/ALLAM.md).
//!
//! Mohamed's transcripts of the course (86 lessons, 101 videos, Egyptian
//! Arabic) live in the repo encrypted, as Steps keeps its VQ bank
//! (Mohamed, 2026-11-21). The repo is public, so only the sealed bytes go
//! in; the key stays with Mohamed.
//!
//! - `allam keygen` prints a new key and its key id. Nothing is written.
//! - `allam seal DIR` (with `BATNA_ALLAM_KEY`) seals every file under DIR to
//!   `content/allam/<path>.yaml` and removes the sealed files DIR no longer
//!   has. DIR must be outside the repo.
//! - `allam open DIR` (with the key) writes every file back to `DIR/<path>`.
//!   DIR must be outside the repo, and new or empty.
//!
//! The crypto is adapted from steps-app main: crates/core/src/sealed.rs.
//! The key is 32 random bytes, written as base32 without padding in groups
//! of four. Subkeys are keyed BLAKE2b-256 of the labels `batna sealed enc`
//! and `batna sealed nonce`; the key id is the first 4 bytes of the same over
//! `batna sealed key id`. A blob is `0x01 ‖ nonce ‖ ciphertext+tag` in
//! base64url without padding: XChaCha20-Poly1305 over the file's bytes
//! compressed with DEFLATE, with the file's path as associated data, so a
//! blob moved to another path doesn't open. The nonce is keyed BLAKE2b
//! (nonce subkey) of `path ‖ 0x00 ‖ file bytes` (SIV style), so the same
//! file under the same key always gives the same bytes and an untouched file
//! never shows in a diff.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use blake2::Blake2bMac;
use blake2::digest::Mac as _;
use blake2::digest::consts::{U24, U32};
use chacha20poly1305::aead::{Aead as _, Payload};
use chacha20poly1305::{KeyInit as _, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};

type Res<T> = Result<T, Vec<String>>;

pub const KEY_VAR: &str = "BATNA_ALLAM_KEY";
/// `content/allam/<path>.yaml`.
pub const SCHEMA: &str = "batna.sealed/v1";
/// Where the sealed files live, under `content/`.
const FOLDER: &str = "allam";

/// The first byte of every blob: the layout above.
const VERSION: u8 = 1;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
/// `scripts/check-tracked-files.sh` refuses a tracked file over 5 MB.
const MAX_FILE_BYTES: usize = 4 * 1024 * 1024;

/// One sealed file, as `content/allam/<path>.yaml` holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sealed {
    pub schema: String,
    /// The file's path in the opened folder (`lessons/HEP-L01.md`), in clear.
    pub path: String,
    /// Which key sealed it ([`Key::id`]); never the key.
    pub key: String,
    /// The blob.
    pub sealed: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SealError {
    KeyText,
    Blob,
    Open,
}

impl fmt::Display for SealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::KeyText => "not a key: it is 52 letters and digits (A–Z, 2–7) in groups of four",
            Self::Blob => "not a sealed blob",
            Self::Open => "doesn't open with this key (another key, a changed byte, or another file's blob)",
        })
    }
}

/// A key. Its `Debug` shows only the key id, so a key never lands in a log;
/// [`Key::text`] is the one way to write it out.
#[derive(Clone, PartialEq, Eq)]
pub struct Key([u8; 32]);

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key(id {})", self.id())
    }
}

const BASE32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
/// 256 bits in 5-bit letters, the last with 4 bits to spare.
const KEY_LETTERS: usize = 52;

impl Key {
    #[cfg(test)]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// A new random key.
    pub fn generate() -> Result<Self, String> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| format!("no randomness: {e}"))?;
        Ok(Self(bytes))
    }

    /// Reads the text form; spaces, dashes and case don't matter.
    pub fn parse(text: &str) -> Result<Self, SealError> {
        let letters: Vec<u8> = text
            .bytes()
            .filter(|b| !b.is_ascii_whitespace() && *b != b'-')
            .map(|b| b.to_ascii_uppercase())
            .collect();
        if letters.len() != KEY_LETTERS {
            return Err(SealError::KeyText);
        }
        let (mut out, mut acc, mut bits) = (Vec::with_capacity(32), 0u32, 0u32);
        for b in letters {
            let v = BASE32.iter().position(|&c| c == b).ok_or(SealError::KeyText)? as u32;
            acc = (acc << 5) | v;
            bits += 5;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
                acc &= (1 << bits) - 1;
            }
        }
        // The spare bits are zero, so each key has exactly one text form.
        if acc != 0 {
            return Err(SealError::KeyText);
        }
        out.try_into().map(Self).map_err(|_| SealError::KeyText)
    }

    /// `ABCD-EFGH-…`: base32, uppercase, in groups of four.
    pub fn text(&self) -> String {
        let (mut letters, mut acc, mut bits) = (Vec::with_capacity(KEY_LETTERS), 0u32, 0u32);
        for &b in &self.0 {
            acc = (acc << 8) | u32::from(b);
            bits += 8;
            while bits >= 5 {
                bits -= 5;
                letters.push(BASE32[((acc >> bits) & 31) as usize]);
            }
            acc &= (1 << bits) - 1;
        }
        if bits > 0 {
            letters.push(BASE32[((acc << (5 - bits)) & 31) as usize]);
        }
        letters.chunks(4).map(|c| String::from_utf8_lossy(c).into_owned()).collect::<Vec<_>>().join("-")
    }

    /// 8 hex digits naming the key in files; it tells nothing about the key.
    pub fn id(&self) -> String {
        self.subkey(b"batna sealed key id")[..4].iter().map(|b| format!("{b:02x}")).collect()
    }

    fn subkey(&self, label: &[u8]) -> [u8; 32] {
        let mut mac = Blake2bMac::<U32>::new_from_slice(&self.0).expect("a 32-byte key fits BLAKE2b");
        mac.update(label);
        mac.finalize().into_bytes().into()
    }
}

/// Seals the file at `path` (its bytes `plain`).
pub fn seal(key: &Key, path: &str, plain: &[u8]) -> String {
    let nonce = nonce(key, path, plain);
    let packed = miniz_oxide::deflate::compress_to_vec(plain, 9);
    let cipher = XChaCha20Poly1305::new(&key.subkey(b"batna sealed enc").into());
    let sealed = cipher
        .encrypt(&XNonce::from(nonce), Payload { msg: &packed, aad: path.as_bytes() })
        .expect("a transcript is far below the cipher's length limit");
    let mut blob = Vec::with_capacity(1 + NONCE_LEN + sealed.len());
    blob.push(VERSION);
    blob.extend_from_slice(&nonce);
    blob.extend(sealed);
    URL_SAFE_NO_PAD.encode(blob)
}

/// Opens a blob [`seal`] made for the file at `path`.
pub fn open(key: &Key, path: &str, blob: &str) -> Result<Vec<u8>, SealError> {
    let bytes = URL_SAFE_NO_PAD.decode(blob.trim()).map_err(|_| SealError::Blob)?;
    if bytes.len() < 1 + NONCE_LEN + TAG_LEN || bytes[0] != VERSION {
        return Err(SealError::Blob);
    }
    let (nonce, sealed) = bytes[1..].split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| SealError::Blob)?;
    let cipher = XChaCha20Poly1305::new(&key.subkey(b"batna sealed enc").into());
    let packed = cipher
        .decrypt(&XNonce::from(nonce), Payload { msg: sealed, aad: path.as_bytes() })
        .map_err(|_| SealError::Open)?;
    let plain = miniz_oxide::inflate::decompress_to_vec(&packed).map_err(|_| SealError::Open)?;
    // Only `seal` makes this nonce for these bytes.
    if self::nonce(key, path, &plain) != nonce {
        return Err(SealError::Open);
    }
    Ok(plain)
}

fn nonce(key: &Key, path: &str, plain: &[u8]) -> [u8; NONCE_LEN] {
    let mut mac = Blake2bMac::<U24>::new_from_slice(&key.subkey(b"batna sealed nonce"))
        .expect("a 32-byte key fits BLAKE2b");
    mac.update(path.as_bytes());
    mac.update(&[0]);
    mac.update(plain);
    mac.finalize().into_bytes().into()
}

/// Whether `path` may name a sealed file: relative, `/`-separated parts of
/// A–Z a–z 0–9 `.` `_` `-`, none of them empty, `.` or `..`.
pub fn is_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 200
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
        })
}

/// The YAML text of a sealed file: the same bytes for the same input.
pub fn file_text(s: &Sealed) -> String {
    format!(
        "# Dr. Mahmoud Allam's course, one file, sealed (docs/ALLAM.md).\nschema: {}\npath: {}\nkey: \"{}\"  # key id: which key sealed it (never the key)\nsealed: {}\n",
        s.schema, s.path, s.key, s.sealed
    )
}

pub fn run(root: &Path, args: &[String]) -> Result<String, Vec<String>> {
    match args.first().map(String::as_str) {
        Some("keygen") => keygen(),
        Some("seal") => match args.get(1) {
            Some(dir) => seal_dir(root, Path::new(dir), &env_key()?),
            None => Err(vec!["usage: batna-tools allam seal DIR".into()]),
        },
        Some("open") => match args.get(1) {
            Some(dir) => open_dir(root, Path::new(dir), &env_key()?),
            None => Err(vec!["usage: batna-tools allam open DIR".into()]),
        },
        _ => Err(vec!["usage: batna-tools allam <keygen | seal DIR | open DIR> (docs/ALLAM.md)".into()]),
    }
}

fn keygen() -> Res<String> {
    let key = Key::generate().map_err(|e| vec![e])?;
    Ok(format!(
        "key:    {}\nkey id: {}\nKeep the key in your password manager. It is never written to disk here.",
        key.text(),
        key.id()
    ))
}

/// `BATNA_ALLAM_KEY`; an error that never repeats the key.
fn env_key() -> Res<Key> {
    match std::env::var(KEY_VAR) {
        Ok(text) if !text.trim().is_empty() => Key::parse(&text).map_err(|e| vec![format!("{KEY_VAR}: {e}")]),
        _ => Err(vec![format!("{KEY_VAR} is not set: the key is with Mohamed (docs/ALLAM.md)")]),
    }
}

fn folder(root: &Path) -> PathBuf {
    root.join("content").join(FOLDER)
}

/// Refuses a folder inside the repo: opened files must never land in git.
fn outside(root: &Path, dir: &Path) -> Res<PathBuf> {
    let repo = root.canonicalize().map_err(|e| vec![format!("{}: {e}", root.display())])?;
    let probe =
        if dir.exists() { dir.to_path_buf() } else { dir.parent().unwrap_or(Path::new(".")).to_path_buf() };
    let abs = probe.canonicalize().map_err(|e| vec![format!("{}: {e}", probe.display())])?;
    if abs.starts_with(&repo) {
        return Err(vec![format!("{} is inside the repo: use a folder outside it", dir.display())]);
    }
    Ok(dir.to_path_buf())
}

/// Every file under `dir`, as `/`-separated paths relative to it, sorted.
fn walk(dir: &Path) -> Res<Vec<String>> {
    fn go(base: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                go(base, &path, out)?;
            } else {
                let rel = path.strip_prefix(base).unwrap_or(&path);
                out.push(
                    rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"),
                );
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(dir, dir, &mut out).map_err(|e| vec![format!("{}: {e}", dir.display())])?;
    out.sort();
    Ok(out)
}

/// The sealed files in the repo, as (path in the repo folder, file).
fn sealed_files(root: &Path) -> Res<Vec<(String, Sealed)>> {
    let dir = folder(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for rel in walk(&dir)? {
        let path = dir.join(&rel);
        let text = std::fs::read_to_string(&path).map_err(|e| vec![format!("{}: {e}", path.display())])?;
        match serde_yaml_ng::from_str::<Sealed>(&text) {
            Ok(s) => out.push((rel, s)),
            Err(e) => problems.push(format!("content/{FOLDER}/{rel}: not a sealed file ({SCHEMA}): {e}")),
        }
    }
    if problems.is_empty() { Ok(out) } else { Err(problems) }
}

fn seal_dir(root: &Path, dir: &Path, key: &Key) -> Res<String> {
    let dir = outside(root, dir)?;
    let paths = walk(&dir)?;
    let mut problems: Vec<String> = paths
        .iter()
        .filter(|p| !is_path(p))
        .map(|p| format!("{p}: rename it (A-Z a-z 0-9 . _ - and / only)"))
        .collect();
    if paths.is_empty() {
        problems.push(format!("{} has no files", dir.display()));
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let out = folder(root);
    let mut written = 0;
    let mut kept = BTreeSet::new();
    for p in &paths {
        let bytes = std::fs::read(dir.join(p)).map_err(|e| vec![format!("{p}: {e}")])?;
        let s =
            Sealed { schema: SCHEMA.into(), path: p.clone(), key: key.id(), sealed: seal(key, p, &bytes) };
        let text = file_text(&s);
        if text.len() > MAX_FILE_BYTES {
            return Err(vec![format!(
                "{p}: sealed it is {} bytes, over {MAX_FILE_BYTES}: split it",
                text.len()
            )]);
        }
        let target = out.join(format!("{p}.yaml"));
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| vec![format!("{}: {e}", parent.display())])?;
        }
        if std::fs::read_to_string(&target).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&target, text).map_err(|e| vec![format!("{}: {e}", target.display())])?;
            written += 1;
        }
        kept.insert(format!("{p}.yaml"));
    }
    let mut removed = 0;
    for (rel, _) in sealed_files(root)? {
        if !kept.contains(&rel) {
            let path = out.join(&rel);
            std::fs::remove_file(&path).map_err(|e| vec![format!("{}: {e}", path.display())])?;
            removed += 1;
        }
    }
    Ok(format!(
        "sealed {} files under content/{FOLDER}/ with key {}: {written} written, {} unchanged, {removed} removed",
        paths.len(),
        key.id(),
        paths.len() - written
    ))
}

fn open_dir(root: &Path, dir: &Path, key: &Key) -> Res<String> {
    let dir = outside(root, dir)?;
    if dir.exists() && std::fs::read_dir(&dir).map(|mut d| d.next().is_some()).unwrap_or(true) {
        return Err(vec![format!("{} is not empty: open into a new or empty folder", dir.display())]);
    }
    let files = sealed_files(root)?;
    let mut problems = Vec::new();
    for (rel, s) in &files {
        match open(key, &s.path, &s.sealed) {
            Ok(bytes) => {
                let target = dir.join(&s.path);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| vec![format!("{}: {e}", parent.display())])?;
                }
                std::fs::write(&target, bytes).map_err(|e| vec![format!("{}: {e}", target.display())])?;
            }
            Err(e) => problems.push(format!("content/{FOLDER}/{rel}: {e}")),
        }
    }
    if problems.is_empty() {
        Ok(format!("opened {} files to {}", files.len(), dir.display()))
    } else {
        Err(problems)
    }
}

/// `validate`'s part: each file's shape without the key, and with
/// `BATNA_ALLAM_KEY` set, that every file opens. Returns the problems and a
/// line for the summary.
pub fn check(root: &Path) -> (Vec<String>, String) {
    let files = match sealed_files(root) {
        Ok(f) => f,
        Err(p) => return (p, String::new()),
    };
    if files.is_empty() {
        return (Vec::new(), String::new());
    }
    let mut problems = Vec::new();
    let mut keys = BTreeSet::new();
    for (rel, s) in &files {
        let at = format!("content/{FOLDER}/{rel}");
        if s.schema != SCHEMA {
            problems.push(format!("{at}: schema must be {SCHEMA}"));
        }
        if !is_path(&s.path) || format!("{}.yaml", s.path) != *rel {
            problems.push(format!("{at}: path `{}` must be this file's path without .yaml", s.path));
        }
        let blob_ok = URL_SAFE_NO_PAD
            .decode(s.sealed.trim())
            .is_ok_and(|b| b.len() >= 1 + NONCE_LEN + TAG_LEN && b[0] == VERSION);
        if !blob_ok {
            problems.push(format!("{at}: {}", SealError::Blob));
        }
        keys.insert(s.key.clone());
    }
    if keys.len() > 1 {
        problems.push(format!(
            "content/{FOLDER}/: sealed with {} keys ({}): seal them all again with one",
            keys.len(),
            keys.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    let line = match env_key() {
        Ok(key) => {
            for (rel, s) in &files {
                if s.key != key.id() {
                    problems.push(format!(
                        "content/{FOLDER}/{rel}: sealed with key {}, but {KEY_VAR} is {}",
                        s.key,
                        key.id()
                    ));
                } else if let Err(e) = open(&key, &s.path, &s.sealed) {
                    problems.push(format!("content/{FOLDER}/{rel}: {e}"));
                }
            }
            format!("; Allam course: {} sealed files, all opened", files.len())
        }
        Err(_) => format!("; Allam course: {} sealed files ({KEY_VAR} not set, not opened)", files.len()),
    };
    (problems, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(b: u8) -> Key {
        Key::from_bytes([b; 32])
    }

    #[test]
    fn the_key_reads_back_from_its_text_in_any_case_and_spacing() {
        let k = Key::from_bytes(std::array::from_fn(|i| (i * 37 + 11) as u8));
        let text = k.text();
        assert_eq!(text.len(), 52 + 12, "{text}");
        assert_eq!(Key::parse(&text).unwrap(), k);
        assert_eq!(Key::parse(&text.to_lowercase().replace('-', " ")).unwrap(), k);
        assert_eq!(Key::parse("ABCD"), Err(SealError::KeyText));
        assert_eq!(Key::parse(&text.replace('A', "1")).is_err(), text.contains('A'));
        assert_eq!(k.id().len(), 8);
        assert!(format!("{k:?}").starts_with("Key(id "), "Debug never shows the key");
        assert!(!format!("{k:?}").contains(&text[..4]) || k.id().contains(&text[..4]));
    }

    #[test]
    fn a_file_opens_only_with_its_key_and_at_its_path_and_seals_the_same_twice() {
        let plain = "## 03:39\nأول سؤال معانا هو كلمة Dysphagia\n".repeat(50);
        let blob = seal(&key(1), "lessons/GIT-L01.md", plain.as_bytes());
        assert_eq!(blob, seal(&key(1), "lessons/GIT-L01.md", plain.as_bytes()), "byte-stable");
        assert!(blob.len() < plain.len(), "compressed before sealing");
        assert_eq!(open(&key(1), "lessons/GIT-L01.md", &blob).unwrap(), plain.as_bytes());
        assert_eq!(open(&key(2), "lessons/GIT-L01.md", &blob), Err(SealError::Open));
        assert_eq!(open(&key(1), "lessons/GIT-L02.md", &blob), Err(SealError::Open));
        assert_eq!(open(&key(1), "lessons/GIT-L01.md", "not base64!"), Err(SealError::Blob));
        let mut bytes = URL_SAFE_NO_PAD.decode(&blob).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert_eq!(open(&key(1), "lessons/GIT-L01.md", &URL_SAFE_NO_PAD.encode(bytes)), Err(SealError::Open));
        assert_ne!(seal(&key(1), "a.md", b"x"), seal(&key(1), "b.md", b"x"));
    }

    #[test]
    fn paths_are_plain_relative_names() {
        assert!(is_path("lessons/HEP-L01.md"));
        assert!(is_path("subtitles/BL-044.vtt"));
        for bad in ["", "/etc/passwd", "a//b", "../x", "a/./b", "a b.md", "دروس.md"] {
            assert!(!is_path(bad), "{bad}");
        }
    }

    #[test]
    fn seal_then_open_round_trips_and_validate_checks_the_files() {
        let repo = tempfile::tempdir().unwrap();
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("lessons")).unwrap();
        std::fs::write(src.path().join("lessons/HEP-L01.md"), "شرح الـ LFTs").unwrap();
        std::fs::write(src.path().join("index.json"), "{}").unwrap();
        let k = key(7);
        let out = seal_dir(repo.path(), src.path(), &k).unwrap();
        assert!(out.contains("2 written"), "{out}");
        let again = seal_dir(repo.path(), src.path(), &k).unwrap();
        assert!(again.contains("0 written, 2 unchanged"), "{again}");
        let text =
            std::fs::read_to_string(repo.path().join("content/allam/lessons/HEP-L01.md.yaml")).unwrap();
        assert!(!text.contains("LFTs"), "nothing in clear: {text}");
        assert!(text.contains(&k.id()));
        // Without the key validate checks the shape only.
        let (problems, _) = check(repo.path());
        assert!(problems.is_empty(), "{problems:?}");
        // Open, then seal again from the opened folder: nothing changes.
        let opened = tempfile::tempdir().unwrap();
        let target = opened.path().join("open");
        open_dir(repo.path(), &target, &k).unwrap();
        assert_eq!(std::fs::read_to_string(target.join("lessons/HEP-L01.md")).unwrap(), "شرح الـ LFTs");
        assert!(open_dir(repo.path(), &target, &k).unwrap_err()[0].contains("not empty"));
        // A file gone from the folder is removed from the repo.
        std::fs::remove_file(src.path().join("index.json")).unwrap();
        assert!(seal_dir(repo.path(), src.path(), &k).unwrap().contains("1 removed"));
        assert!(!repo.path().join("content/allam/index.json.yaml").exists());
        // A folder inside the repo is refused both ways.
        let inside = repo.path().join("open");
        assert!(open_dir(repo.path(), &inside, &k).unwrap_err()[0].contains("inside the repo"));
        // A file moved to another path is caught.
        std::fs::rename(
            repo.path().join("content/allam/lessons/HEP-L01.md.yaml"),
            repo.path().join("content/allam/lessons/HEP-L02.md.yaml"),
        )
        .unwrap();
        let (problems, _) = check(repo.path());
        assert!(problems.iter().any(|p| p.contains("must be this file's path")), "{problems:?}");
    }
}
