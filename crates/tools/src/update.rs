//! `batna-tools update …`: update feeds (`docs/RELEASE.md` §5), the signing
//! key, `update.json` and its signature. The checks the app runs live in
//! `batna_core::update`.
//!
//! ```text
//! update keygen [--public-out FILE] [--force]
//! update manifest --channel C [--version V] [--build N] [--commit SHA]
//!                 --asset PLATFORM=PATH … [--notes-en T] [--notes-ar-eg T] [--out DIR]
//!                 [--previous FEED/update.json] [--changes] [--carry PLATFORM …]
//! update changes [--since COMMIT]          what the next build's "what's new" says
//! update changes --gate BASE               fails unless HEAD adds a line (CI, #320)
//! update channels [--version V]             the channels a version is published on
//! update sign DIR [--public-key FILE]       secret from BATNA_UPDATE_SECRET
//! update verify DIR [--public-key FILE] [--carried PLATFORM …]
//! ```
//!
//! - `keygen` makes the Ed25519 key pair. The public half goes into the app
//!   (`crates/app/update-key.pub`, committed); the secret half is printed
//!   once for the Keychain (`just update-keygen`) and never written to a file.
//! - `manifest` hashes the builds and writes `update.json` next to copies of
//!   them, ready to upload. PLATFORM is `android-arm64`, `macos-arm64`,
//!   `macos-x86_64` or `content` (the pack from `batna-tools packs build`).
//! - `sign` signs `update.json` and checks the result against the public key.
//! - `verify` runs the app's checks on a feed folder. `--carried PLATFORM`
//!   names a row `--carry` kept: its file is on the release already, not in
//!   the folder, so only its build is checked (older than the feed's).
//! - `channels` prints one channel per line (an alpha → alpha; a beta → alpha
//!   and beta; a release → all three), for `scripts/update-publish.sh`.
//! - `--previous` keeps the channel's current history (a missing file is
//!   fine); with `--changes`, `changes/` since that feed's commit becomes
//!   this build's entry, and the notes when none are given (`crate::changes`).
//! - `--carry PLATFORM` keeps that platform's file from `--previous` as it
//!   is (same file, same build): a content-only feed carries the app over,
//!   so devices get the new pack without installing anything. Its history
//!   entry is marked `content_only`.
//!
//! Adapted from kabar-app main: `crates/tools/src/update.rs` (clap and
//! anyhow swapped for this crate's plain argument handling).

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use batna_core::update::{
    BuildChanges, Channel, MANIFEST_FILE, SIGNATURE_FILE, UPDATE_FORMAT, UpdateAsset, UpdateManifest,
    UpdateNotes, UpdatePlatform, Version, check_download, key_line, parse_public_key, verify_manifest,
};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::changes;
use crate::flag;

/// The release public key the app builds in.
pub const RELEASE_KEY_FILE: &str = "crates/app/update-key.pub";
/// The secret, base64, for `update sign`.
const SECRET_ENV: &str = "BATNA_UPDATE_SECRET";

const USAGE: &str = "usage: batna-tools update <keygen | manifest | sign DIR | verify DIR | changes | channels> (see crates/tools/src/update.rs)";

type Res<T> = Result<T, String>;

pub fn run(root: &Path, args: &[String]) -> Result<String, Vec<String>> {
    run_inner(root, args, std::env::var(SECRET_ENV).ok()).map_err(|e| vec![e])
}

/// `secret` is [`SECRET_ENV`]'s value, for `sign`.
fn run_inner(root: &Path, args: &[String], secret: Option<String>) -> Res<String> {
    let public_key = |args: &[String]| -> Res<String> {
        let path = flag(args, "--public-key").map_or_else(|| root.join(RELEASE_KEY_FILE), PathBuf::from);
        read_text(&path)
    };
    match args.first().map(String::as_str) {
        Some("keygen") => {
            let out = flag(args, "--public-out").map_or_else(|| root.join(RELEASE_KEY_FILE), PathBuf::from);
            let (public, secret) = keygen(&out, args.iter().any(|a| a == "--force"))?;
            eprintln!("public key {public} → {}", out.display());
            // stdout carries only the secret, for the Keychain.
            Ok(secret)
        }
        Some("manifest") => {
            let channel = flag(args, "--channel").ok_or("--channel is required")?;
            let channel = Channel::parse(channel)
                .ok_or_else(|| format!("unknown channel {channel:?} (preview, alpha, beta, stable)"))?;
            let version = match flag(args, "--version") {
                Some(v) => v.to_owned(),
                None => workspace_version(root)?,
            };
            let build = match flag(args, "--build") {
                Some(b) => b.parse().map_err(|_| format!("--build {b:?} isn't a number"))?,
                None => 0,
            };
            let assets = values(args, "--asset").map(parse_asset).collect::<Res<Vec<_>>>()?;
            if assets.is_empty() {
                return Err("give at least one --asset PLATFORM=PATH".into());
            }
            let carry = values(args, "--carry")
                .map(|p| UpdatePlatform::parse(p).ok_or_else(|| format!("unknown platform {p:?}")))
                .collect::<Res<Vec<_>>>()?;
            let mut notes = UpdateNotes {
                en: flag(args, "--notes-en").unwrap_or_default().to_owned(),
                ar_eg: flag(args, "--notes-ar-eg").unwrap_or_default().to_owned(),
            };
            let out = flag(args, "--out").map_or_else(|| root.join("dist/update"), PathBuf::from);
            let commit = flag(args, "--commit").map(str::to_owned);
            let previous: Option<UpdateManifest> = match flag(args, "--previous").map(Path::new) {
                Some(p) if p.exists() => {
                    let m: UpdateManifest = serde_json::from_str(&read_text(p)?)
                        .map_err(|e| format!("previous feed {}: {e}", p.display()))?;
                    if m.channel != channel {
                        return Err(format!("the previous feed is for {}", m.channel.as_str()));
                    }
                    Some(m)
                }
                _ => None,
            };
            let carried = carry
                .iter()
                .map(|p| {
                    let m = previous.as_ref().ok_or("--carry needs a --previous feed")?;
                    if m.version != version {
                        return Err(format!("--carry: the previous feed is {}, not {version}", m.version));
                    }
                    let a = m
                        .asset(*p)
                        .ok_or_else(|| format!("--carry: no {} in the previous feed", p.as_str()))?;
                    Ok(UpdateAsset { build: Some(m.build_of(a)), ..a.clone() })
                })
                .collect::<Res<Vec<_>>>()?;
            let mut entry = None;
            if args.iter().any(|a| a == "--changes") {
                let since = previous.as_ref().and_then(|m| m.commit.as_deref());
                let found = changes::collect(root, since)?;
                eprintln!("{} change(s) since {}", found.len(), since.unwrap_or("HEAD^"));
                if notes == UpdateNotes::default() {
                    notes = changes::notes_from(&found);
                }
                entry = (!found.is_empty()).then(|| BuildChanges {
                    version: version.clone(),
                    build,
                    published_at: now_rfc3339(),
                    changes: found,
                    content_only: !carried.is_empty(),
                });
            }
            let history = changes::history(previous.as_ref(), entry, &changes::retro(root)?);
            let m = write_manifest(
                &out,
                channel,
                &version,
                build,
                commit,
                &assets,
                &carried,
                WhatsNew { notes, history },
            )?;
            Ok(format!(
                "{} {} build {} → {} ({} file(s))",
                m.channel.as_str(),
                m.version,
                m.build,
                out.join(MANIFEST_FILE).display(),
                m.assets.len()
            ))
        }
        Some("sign") => {
            let dir = args.get(1).filter(|a| !a.starts_with("--")).ok_or(USAGE)?;
            let secret = secret.ok_or_else(|| format!("{SECRET_ENV} is not set"))?;
            sign(Path::new(dir), &secret, &public_key(args)?)?;
            Ok(format!("signed {}", Path::new(dir).join(SIGNATURE_FILE).display()))
        }
        Some("changes") => {
            let n = changes::check_all(root)?;
            let mut out = format!("{n} file(s) in {}/ check out", changes::DIR);
            let found = match flag(args, "--gate") {
                Some(base) => changes::gate(root, base)?,
                None => changes::collect(root, flag(args, "--since"))?,
            };
            for c in found {
                out.push_str(&format!("\n{:?}\t{}\t{}", c.kind, c.text.ar_eg, c.text.en));
            }
            Ok(out)
        }
        Some("channels") => {
            let v = match flag(args, "--version") {
                Some(v) => v.to_owned(),
                None => workspace_version(root)?,
            };
            let parsed = Version::parse(&v).ok_or_else(|| format!("{v:?} isn't a version"))?;
            Ok(parsed.channels().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("\n"))
        }
        Some("verify") => {
            let dir = args.get(1).filter(|a| !a.starts_with("--")).ok_or(USAGE)?;
            let carried = values(args, "--carried")
                .map(|p| UpdatePlatform::parse(p).ok_or_else(|| format!("unknown platform {p:?}")))
                .collect::<Res<Vec<_>>>()?;
            let m = verify(Path::new(dir), &public_key(args)?, &carried)?;
            Ok(format!(
                "ok: {} {} build {}, {} file(s) match",
                m.channel.as_str(),
                m.version,
                m.build,
                m.assets.len()
            ))
        }
        _ => Err(USAGE.into()),
    }
}

/// Every value given after `name` (a flag that may repeat).
fn values<'a>(args: &'a [String], name: &'a str) -> impl Iterator<Item = &'a str> {
    args.windows(2).filter(move |w| w[0] == name).map(|w| w[1].as_str())
}

fn parse_asset(s: &str) -> Res<(UpdatePlatform, PathBuf)> {
    let (p, path) = s.split_once('=').ok_or_else(|| format!("--asset {s:?}: expected PLATFORM=PATH"))?;
    let platform = UpdatePlatform::parse(p).ok_or_else(|| format!("unknown platform {p:?}"))?;
    Ok((platform, PathBuf::from(path)))
}

fn read_text(path: &Path) -> Res<String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn io<T>(path: &Path, r: std::io::Result<T>) -> Res<T> {
    r.map_err(|e| format!("{}: {e}", path.display()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn workspace_version(root: &Path) -> Res<String> {
    read_text(&root.join("Cargo.toml"))?
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"').map(str::to_owned))
        .ok_or_else(|| "no version in Cargo.toml".into())
}

/// Writes the public key file and returns (public, secret), both base64.
pub fn keygen(public_out: &Path, force: bool) -> Res<(String, String)> {
    if !force && fs::read_to_string(public_out).is_ok_and(|t| key_line(&t).is_some()) {
        return Err(format!(
            "{} already holds a key; installed builds trust it. Pass --force only if that key is lost.",
            public_out.display()
        ));
    }
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| format!("no randomness: {e}"))?;
    let key = SigningKey::from_bytes(&seed);
    let public = B64.encode(key.verifying_key().as_bytes());
    let text = format!(
        "# Batna's update key (Ed25519, public half). Builds trust update feeds\n\
         # signed with its secret half, which lives only in the Mac's Keychain\n\
         # (`just update-keygen`). Replacing it strands every installed build.\n\
         {public}\n"
    );
    if let Some(dir) = public_out.parent() {
        io(dir, fs::create_dir_all(dir))?;
    }
    io(public_out, fs::write(public_out, text))?;
    Ok((public, B64.encode(seed)))
}

/// The manifest's notes and history (`crate::changes`).
#[derive(Default)]
pub struct WhatsNew {
    pub notes: UpdateNotes,
    pub history: Vec<BuildChanges>,
}

#[allow(clippy::too_many_arguments)] // each one is a part of the feed
pub fn write_manifest(
    out: &Path,
    channel: Channel,
    version: &str,
    build: u64,
    commit: Option<String>,
    files: &[(UpdatePlatform, PathBuf)],
    carried: &[UpdateAsset],
    whats_new: WhatsNew,
) -> Res<UpdateManifest> {
    let WhatsNew { notes, history } = whats_new;
    let parsed = Version::parse(version).ok_or_else(|| format!("version {version:?}"))?;
    if channel != Channel::Preview && !parsed.channels().contains(&channel) {
        return Err(format!("{version} doesn't belong on the {} channel", channel.as_str()));
    }
    if out.exists() {
        io(out, fs::remove_dir_all(out))?;
    }
    io(out, fs::create_dir_all(out))?;
    let mut assets: Vec<UpdateAsset> = Vec::new();
    for (platform, path) in files {
        if assets.iter().any(|a| a.platform == *platform) {
            return Err(format!("two files for {}", platform.as_str()));
        }
        let bytes = io(path, fs::read(path))?;
        let stem = match platform {
            UpdatePlatform::Content => "batna-content",
            UpdatePlatform::MacosArm64 | UpdatePlatform::MacosX86_64 => "Batna",
            UpdatePlatform::AndroidArm64 => "batna",
        };
        // Preview builds all share one version, so the build number keeps
        // each file name unique: replacing a live file mid-upload would hand
        // devices still holding the old feed a 404 or a hash mismatch.
        let file = match channel {
            Channel::Preview => {
                format!("{stem}-{version}-b{build}-{}.{}", platform.as_str(), platform.extension())
            }
            _ => format!("{stem}-{version}-{}.{}", platform.as_str(), platform.extension()),
        };
        let dest = out.join(&file);
        io(&dest, fs::write(&dest, &bytes))?;
        assets.push(UpdateAsset {
            platform: *platform,
            file,
            size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            version_code: (*platform == UpdatePlatform::AndroidArm64).then(|| parsed.version_code()),
            build: None,
        });
    }
    for a in carried {
        if assets.iter().any(|b| b.platform == a.platform) {
            return Err(format!("{} is both carried and given", a.platform.as_str()));
        }
        assets.push(a.clone());
    }
    // The look in the pack fits the app this feed carries: a carried app is
    // older than the feed, and its screens are what the stylesheets style.
    let ui_build = assets.iter().any(|a| a.platform == UpdatePlatform::Content).then(|| {
        let app = assets.iter().filter(|a| a.platform != UpdatePlatform::Content);
        app.map(|a| a.build.unwrap_or(build)).min().unwrap_or(build)
    });
    let m = UpdateManifest {
        format: UPDATE_FORMAT,
        channel,
        version: version.to_owned(),
        build,
        commit,
        published_at: now_rfc3339(),
        notes,
        assets,
        history,
        ui_build,
    };
    let path = out.join(MANIFEST_FILE);
    let json = serde_json::to_vec_pretty(&m).map_err(|e| e.to_string())?;
    io(&path, fs::write(&path, json))?;
    Ok(m)
}

/// Signs `update.json` and checks the signature against `public` before
/// writing it, so a feed is never signed with a key the app doesn't trust.
pub fn sign(dir: &Path, secret_b64: &str, public: &str) -> Res<()> {
    let seed: [u8; 32] = B64
        .decode(secret_b64.trim())
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| format!("{SECRET_ENV} isn't a base64 32-byte key"))?;
    let key = SigningKey::from_bytes(&seed);
    let path = dir.join(MANIFEST_FILE);
    let json = fs::read(&path).map_err(|_| format!("no {}; run `update manifest` first", path.display()))?;
    let sig = B64.encode(key.sign(&json).to_bytes());
    let trusted = parse_public_key(public).map_err(|e| e.to_string())?;
    if trusted != key.verifying_key() {
        return Err(format!(
            "this secret doesn't belong to the public key the app trusts ({RELEASE_KEY_FILE})"
        ));
    }
    verify_manifest(&json, &sig, &trusted).map_err(|e| e.to_string())?;
    let out = dir.join(SIGNATURE_FILE);
    io(&out, fs::write(&out, format!("{sig}\n")))
}

/// Checks a feed folder as the app would. A `carried` platform's file stays
/// on the release (`--carry`), so it is not in `dir`; its row must still be
/// from an older build, or it was not carried at all.
pub fn verify(dir: &Path, public: &str, carried: &[UpdatePlatform]) -> Res<UpdateManifest> {
    let key = parse_public_key(public).map_err(|e| e.to_string())?;
    let json = io(&dir.join(MANIFEST_FILE), fs::read(dir.join(MANIFEST_FILE)))?;
    let sig = read_text(&dir.join(SIGNATURE_FILE))?;
    let m = verify_manifest(&json, &sig, &key).map_err(|e| e.to_string())?;
    for p in carried {
        let a = m.asset(*p).ok_or_else(|| format!("--carried: no {} in the feed", p.as_str()))?;
        if m.build_of(a) >= m.build {
            return Err(format!("--carried: {} is from this build, not carried", a.file));
        }
    }
    for a in m.assets.iter().filter(|a| !carried.contains(&a.platform)) {
        let path = dir.join(&a.file);
        let bytes = io(&path, fs::read(&path))?;
        check_download(a, bytes.len() as u64, &sha256_hex(&bytes)).map_err(|e| format!("{}: {e}", a.file))?;
        // What a device checks next: a pack devices would refuse is broken.
        if a.platform == UpdatePlatform::Content {
            let pack = batna_core::pack::Pack::decode(&bytes).map_err(|e| format!("{}: {e}", a.file))?;
            if let Some(first) = pack.problems().first() {
                return Err(format!("{}: devices would refuse it: {first}", a.file));
            }
        }
    }
    Ok(m)
}

fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Days since 1970-01-01 → (year, month, day), proleptic Gregorian.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real (empty) content pack, told apart by `tag`: `verify` decodes
    /// and checks the content asset like a device.
    fn sample_pack(tag: &str) -> Vec<u8> {
        let index: batna_core::content::Index = serde_yaml_ng::from_str(&format!(
            "schema: batna.content/v1\nlectures:\n  - {{ id: hb.L01, title: {tag}, pages: [1, 2], chunk: 1 }}\n"
        ))
        .unwrap();
        batna_core::pack::Pack::new(index, Vec::new()).encode()
    }

    /// Seed of the test key behind `fixtures/update/` (never a real key).
    const FIXTURE_SEED: [u8; 32] = [0x42; 32];

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn keygen_manifest_sign_verify_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let pub_file = dir.path().join("update-key.pub");
        let (_, secret) = keygen(&pub_file, false).unwrap();
        assert!(keygen(&pub_file, false).is_err(), "an existing key is never replaced silently");
        let public = fs::read_to_string(&pub_file).unwrap();

        let apk = dir.path().join("batna.apk");
        fs::write(&apk, b"not really an apk").unwrap();
        let pack = dir.path().join("batna-content.pack");
        fs::write(&pack, sample_pack("first")).unwrap();
        let feed = dir.path().join("feed");
        let m = write_manifest(
            &feed,
            Channel::Alpha,
            "1.0.0-alpha.2",
            0,
            Some("abc1234".into()),
            &[(UpdatePlatform::AndroidArm64, apk), (UpdatePlatform::Content, pack)],
            &[],
            WhatsNew::default(),
        )
        .unwrap();
        assert_eq!(m.assets[0].file, "batna-1.0.0-alpha.2-android-arm64.apk");
        assert_eq!(m.assets[0].version_code, Some(10_000_102));
        assert_eq!(m.assets[1].file, "batna-content-1.0.0-alpha.2-content.pack");
        assert_eq!(m.assets[1].version_code, None);

        // Signing with a secret the app doesn't trust is refused.
        let stranger = B64.encode([1u8; 32]);
        assert!(sign(&feed, &stranger, &public).is_err());
        assert!(!feed.join(SIGNATURE_FILE).exists());

        sign(&feed, &secret, &public).unwrap();
        assert_eq!(verify(&feed, &public, &[]).unwrap(), m);

        // A signed feed whose pack devices would refuse fails verification.
        let mut broken = batna_core::pack::Pack::decode(&sample_pack("x")).unwrap();
        broken.index.lectures.push(broken.index.lectures[0].clone());
        let broken_pack = dir.path().join("broken.pack");
        fs::write(&broken_pack, broken.encode()).unwrap();
        let broken_feed = dir.path().join("broken-feed");
        write_manifest(
            &broken_feed,
            Channel::Alpha,
            "1.0.0-alpha.2",
            0,
            None,
            &[(UpdatePlatform::Content, broken_pack)],
            &[],
            WhatsNew::default(),
        )
        .unwrap();
        sign(&broken_feed, &secret, &public).unwrap();
        let err = verify(&broken_feed, &public, &[]).unwrap_err();
        assert!(err.contains("devices would refuse it") && err.contains("listed twice"), "{err}");

        // A swapped file fails verification.
        fs::write(feed.join(&m.assets[1].file), b"something else").unwrap();
        assert!(verify(&feed, &public, &[]).is_err());
    }

    #[test]
    fn versions_stay_on_their_channels() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("x");
        fs::write(&f, b"x").unwrap();
        let assets = [(UpdatePlatform::MacosArm64, f)];
        let out = dir.path().join("feed");
        let none = WhatsNew::default;
        assert!(
            write_manifest(&out, Channel::Stable, "1.0.0-alpha.1", 0, None, &assets, &[], none()).is_err()
        );
        assert!(write_manifest(&out, Channel::Beta, "1.0.0", 0, None, &assets, &[], none()).is_ok());
        let preview = write_manifest(&out, Channel::Preview, "0.1.0", 7, None, &assets, &[], none()).unwrap();
        assert_eq!(
            preview.assets[0].file, "Batna-0.1.0-b7-macos-arm64.app.zip",
            "preview names carry the build"
        );
    }

    #[test]
    fn the_command_line_reads_repeated_assets() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (dir.path().join("a.apk"), dir.path().join("c.pack"));
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();
        let out = dir.path().join("feed");
        let args = strings(&[
            "manifest",
            "--channel",
            "preview",
            "--version",
            "0.1.0",
            "--build",
            "12",
            "--asset",
            &format!("android-arm64={}", a.display()),
            "--asset",
            &format!("content={}", b.display()),
            "--out",
            &out.display().to_string(),
        ]);
        let said = run(dir.path(), &args).unwrap();
        assert!(said.starts_with("preview 0.1.0 build 12"), "{said}");
        let m: UpdateManifest = serde_json::from_slice(&fs::read(out.join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(m.assets.len(), 2);

        assert!(run(dir.path(), &strings(&["manifest", "--channel", "nightly"])).is_err());
        assert!(
            run(dir.path(), &strings(&["manifest", "--channel", "alpha", "--version", "1.0.0"])).is_err()
        );
        assert!(run(dir.path(), &strings(&["sign"])).is_err());
    }

    #[test]
    fn a_content_only_feed_carries_the_app_over() {
        let dir = tempfile::tempdir().unwrap();
        let (apk, pack) = (dir.path().join("a.apk"), dir.path().join("c.pack"));
        fs::write(&apk, b"a").unwrap();
        fs::write(&pack, sample_pack("one")).unwrap();
        let feed = |build: &str, out: &str, extra: &[&str]| {
            let mut args = strings(&[
                "manifest",
                "--channel",
                "preview",
                "--version",
                "0.1.1-alpha.0",
                "--build",
                build,
            ]);
            args.extend(strings(&["--asset", &format!("content={}", pack.display()), "--out", out]));
            args.extend(strings(extra));
            run(dir.path(), &args)
        };
        let first = dir.path().join("first").display().to_string();
        let apk_arg = format!("android-arm64={}", apk.display());
        feed("40", &first, &["--asset", &apk_arg]).unwrap();
        let previous = format!("{first}/{MANIFEST_FILE}");
        assert!(
            feed("41", &first, &["--carry", "android-arm64"]).is_err(),
            "carrying needs the previous feed"
        );

        fs::write(&pack, sample_pack("two")).unwrap();
        let second = dir.path().join("second").display().to_string();
        feed("45", &second, &["--previous", &previous, "--carry", "android-arm64"]).unwrap();
        let m: UpdateManifest =
            serde_json::from_slice(&fs::read(format!("{second}/{MANIFEST_FILE}")).unwrap()).unwrap();
        let app = m.asset(UpdatePlatform::AndroidArm64).unwrap();
        assert_eq!((app.file.as_str(), m.build_of(app)), ("batna-0.1.1-alpha.0-b40-android-arm64.apk", 40));
        assert!(!Path::new(&second).join(&app.file).exists(), "the carried file stays where it is");
        assert_eq!(
            m.asset(UpdatePlatform::Content).unwrap().file,
            "batna-content-0.1.1-alpha.0-b45-content.pack"
        );
        assert_eq!((m.build, m.ui_build), (45, Some(40)), "the look fits the app it carries");

        // The preview workflow verifies the feed it uploads: the carried APK
        // is on the release, not in the folder (Kabar preview run 355).
        let (pub_file, second) = (dir.path().join("k.pub"), Path::new(&second));
        let (_, secret) = keygen(&pub_file, false).unwrap();
        let public = fs::read_to_string(&pub_file).unwrap();
        sign(second, &secret, &public).unwrap();
        assert!(verify(second, &public, &[]).is_err(), "unnamed, the missing APK still fails");
        assert_eq!(verify(second, &public, &[UpdatePlatform::AndroidArm64]).unwrap(), m);
        assert!(
            verify(second, &public, &[UpdatePlatform::Content]).is_err(),
            "a row this build wrote is not carried"
        );
    }

    fn cli(root: &Path, args: &[&str], secret: Option<&str>) -> Res<String> {
        run_inner(root, &strings(args), secret.map(str::to_owned))
    }

    #[test]
    fn keygen_sign_and_verify_from_the_command_line() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let key = root.join("keys/update-key.pub").display().to_string();
        let secret = cli(root, &["keygen", "--public-out", &key], None).unwrap();
        assert_eq!(B64.decode(&secret).unwrap().len(), 32, "stdout is only the secret");
        let first = fs::read_to_string(&key).unwrap();
        assert!(key_line(&first).is_some());
        let again = cli(root, &["keygen", "--public-out", &key], None).unwrap_err();
        assert!(again.contains("already holds a key"), "{again}");
        let new_secret = cli(root, &["keygen", "--public-out", &key, "--force"], None).unwrap();
        assert_ne!(new_secret, secret);
        assert_ne!(fs::read_to_string(&key).unwrap(), first, "--force replaces the key");

        let apk = root.join("a.apk");
        fs::write(&apk, b"apk").unwrap();
        let feed = root.join("feed").display().to_string();
        let asset = format!("android-arm64={}", apk.display());
        let made = cli(
            root,
            &[
                "manifest",
                "--channel",
                "alpha",
                "--version",
                "1.0.0-alpha.2",
                "--asset",
                &asset,
                "--out",
                &feed,
            ],
            None,
        )
        .unwrap();
        assert!(
            made.starts_with("alpha 1.0.0-alpha.2 build 0 → ") && made.ends_with("(1 file(s))"),
            "{made}"
        );

        let sign = |secret: Option<&str>| cli(root, &["sign", &feed, "--public-key", &key], secret);
        assert_eq!(sign(None).unwrap_err(), "BATNA_UPDATE_SECRET is not set");
        assert!(sign(Some("not base64!")).unwrap_err().contains("isn't a base64 32-byte key"));
        assert!(sign(Some(&secret)).unwrap_err().contains("doesn't belong to the public key"), "old secret");
        assert_eq!(sign(Some(&new_secret)).unwrap(), format!("signed {feed}/{SIGNATURE_FILE}"));
        let empty = root.join("empty").display().to_string();
        let err = cli(root, &["sign", &empty, "--public-key", &key], Some(&new_secret)).unwrap_err();
        assert!(err.contains("run `update manifest` first"), "{err}");

        let ok = cli(root, &["verify", &feed, "--public-key", &key], None).unwrap();
        assert_eq!(ok, "ok: alpha 1.0.0-alpha.2 build 0, 1 file(s) match");
        let err = cli(root, &["verify", &feed, "--public-key", &key, "--carried", "ios"], None).unwrap_err();
        assert_eq!(err, "unknown platform \"ios\"");
        let err =
            cli(root, &["verify", &feed, "--public-key", &key, "--carried", "content"], None).unwrap_err();
        assert_eq!(err, "--carried: no content in the feed");
        assert_eq!(cli(root, &["verify"], None).unwrap_err(), USAGE);
        // Without --public-key the app's own key file is used: none here.
        assert!(cli(root, &["verify", &feed], None).unwrap_err().contains(RELEASE_KEY_FILE));
        assert_eq!(cli(root, &["publish"], None).unwrap_err(), USAGE);
    }

    #[test]
    fn manifest_refuses_a_feed_it_cannot_build() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (apk, pack) = (root.join("a.apk"), root.join("c.pack"));
        fs::write(&apk, b"a").unwrap();
        fs::write(&pack, sample_pack("x")).unwrap();
        let apk_arg = format!("android-arm64={}", apk.display());
        let pack_arg = format!("content={}", pack.display());
        let out = root.join("feed").display().to_string();
        let manifest = |extra: &[&str]| {
            let mut args = vec!["manifest", "--channel", "alpha", "--out", &out];
            args.extend(extra);
            cli(root, &args, None)
        };
        fn with<'a>(more: &[&'a str]) -> Vec<&'a str> {
            [&["--version", "1.0.0-alpha.1"][..], more].concat()
        }

        // No --version: the workspace's, from Cargo.toml.
        assert_eq!(
            manifest(&["--asset", &apk_arg]).unwrap_err(),
            format!("{}: No such file or directory (os error 2)", root.join("Cargo.toml").display())
        );
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        assert_eq!(manifest(&["--asset", &apk_arg]).unwrap_err(), "no version in Cargo.toml");
        fs::write(root.join("Cargo.toml"), "[workspace.package]\nversion = \"1.0.0-alpha.3\"\n").unwrap();
        assert!(manifest(&["--asset", &apk_arg]).unwrap().starts_with("alpha 1.0.0-alpha.3 build 0"));
        assert_eq!(cli(root, &["channels"], None).unwrap(), "alpha");

        let err = |extra: &[&str]| manifest(&with(extra)).unwrap_err();
        assert_eq!(err(&["--asset", &apk_arg, "--build", "x"]), "--build \"x\" isn't a number");
        assert_eq!(err(&[]), "give at least one --asset PLATFORM=PATH");
        assert_eq!(err(&["--asset", "a.apk"]), "--asset \"a.apk\": expected PLATFORM=PATH");
        assert_eq!(err(&["--asset", "ios=a.ipa"]), "unknown platform \"ios\"");
        assert_eq!(err(&["--asset", &apk_arg, "--carry", "ios"]), "unknown platform \"ios\"");
        assert_eq!(err(&["--asset", &apk_arg, "--asset", &apk_arg]), "two files for android-arm64");
        assert!(err(&["--asset", "content=/no/such.pack"]).starts_with("/no/such.pack: "));
        assert_eq!(cli(root, &["manifest"], None).unwrap_err(), "--channel is required");

        // --previous: same channel, and --carry needs the same version and the platform.
        let prev_dir = root.join("prev").display().to_string();
        let prev = format!("{prev_dir}/{MANIFEST_FILE}");
        cli(
            root,
            &[
                "manifest",
                "--channel",
                "alpha",
                "--version",
                "1.0.0-alpha.1",
                "--asset",
                &apk_arg,
                "--out",
                &prev_dir,
            ],
            None,
        )
        .unwrap();
        let other = cli(
            root,
            &[
                "manifest",
                "--channel",
                "beta",
                "--version",
                "1.0.0-beta.1",
                "--asset",
                &pack_arg,
                "--previous",
                &prev,
            ],
            None,
        )
        .unwrap_err();
        assert_eq!(other, "the previous feed is for alpha");
        assert_eq!(
            manifest(&[
                "--version",
                "1.0.0-alpha.2",
                "--asset",
                &pack_arg,
                "--previous",
                &prev,
                "--carry",
                "android-arm64"
            ])
            .unwrap_err(),
            "--carry: the previous feed is 1.0.0-alpha.1, not 1.0.0-alpha.2"
        );
        assert_eq!(
            err(&["--asset", &pack_arg, "--previous", &prev, "--carry", "macos-arm64"]),
            "--carry: no macos-arm64 in the previous feed"
        );
        assert_eq!(
            err(&["--asset", &apk_arg, "--previous", &prev, "--carry", "android-arm64"]),
            "android-arm64 is both carried and given"
        );
        fs::write(&prev, "{ not json").unwrap();
        assert!(
            err(&["--asset", &apk_arg, "--previous", &prev]).starts_with(&format!("previous feed {prev}: "))
        );
        // A missing previous feed is the channel's first.
        assert!(manifest(&with(&["--asset", &apk_arg, "--previous", "/no/feed.json"])).is_ok());
    }

    #[test]
    fn a_feed_built_with_changes_says_whats_new_since_the_last_one() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let git = |args: &[&str]| crate::testkit::git(root, args);
        crate::testkit::write(root, "changes/first.yaml", "- kind: new\n  ar_eg: أول حاجة\n");
        crate::testkit::commit_all(root, "First");
        let first = git(&["rev-parse", "HEAD"]).trim().to_owned();
        let apk = root.join("a.apk");
        fs::write(&apk, b"a").unwrap();
        let asset = format!("android-arm64={}", apk.display());
        let feed = |build: &str, out: &str, extra: &[&str]| {
            let mut args = vec!["manifest", "--channel", "preview", "--version", "0.1.0", "--build", build];
            args.extend(["--asset", &asset, "--out", out, "--changes"]);
            args.extend(extra);
            cli(root, &args, None)
        };
        let one = root.join("one").display().to_string();
        feed("1", &one, &["--commit", &first]).unwrap();
        let m: UpdateManifest =
            serde_json::from_str(&read_text(&Path::new(&one).join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(
            (m.notes.ar_eg.as_str(), m.history.len()),
            ("أول حاجة", 1),
            "HEAD alone when nothing is known"
        );

        crate::testkit::write(
            root,
            "changes/liver.yaml",
            "- kind: fixed\n  ar_eg: الصورة بقت أوضح\n  en: The picture is clearer\n",
        );
        crate::testkit::commit_all(root, "Fix the picture");
        let two = root.join("two").display().to_string();
        let previous = format!("{one}/{MANIFEST_FILE}");
        feed("2", &two, &["--previous", &previous, "--notes-en", "Given"]).unwrap();
        let m: UpdateManifest =
            serde_json::from_str(&read_text(&Path::new(&two).join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(m.notes.en, "Given", "given notes win over the changes");
        assert_eq!(m.history.iter().map(|b| b.build).collect::<Vec<_>>(), [2, 1]);
        assert_eq!(m.history[0].changes[0].text.ar_eg, "الصورة بقت أوضح");
        assert!(!m.history[0].content_only);

        // `update changes` checks every file and lists what HEAD added.
        let out = cli(root, &["changes"], None).unwrap();
        assert_eq!(out, "2 file(s) in changes/ check out\nFixed\tالصورة بقت أوضح\tThe picture is clearer");
        let out = cli(root, &["changes", "--gate", &first], None).unwrap();
        assert!(out.ends_with("Fixed\tالصورة بقت أوضح\tThe picture is clearer"), "{out}");
        crate::testkit::write(root, "changes/bad.yaml", "- kind: new\n  ar_eg: English\n");
        assert!(cli(root, &["changes"], None).unwrap_err().contains("bad.yaml"));
    }

    #[test]
    fn channels_follow_the_version() {
        let dir = tempfile::tempdir().unwrap();
        let channels = |v: &str| run(dir.path(), &strings(&["channels", "--version", v]));
        assert_eq!(channels("0.1.1-alpha.1").unwrap(), "alpha");
        assert_eq!(channels("0.2.0-beta.1").unwrap(), "alpha\nbeta");
        assert_eq!(channels("1.0.0").unwrap(), "alpha\nbeta\nstable");
        assert!(channels("one").is_err(), "a bad version fails instead of publishing nowhere");
    }

    #[test]
    fn dates_are_rfc3339() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_753), (2026, 10, 27));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn the_release_key_file_holds_no_key_until_keygen() {
        // Until `just update-keygen` runs on the Mac, the app trusts no feed
        // at all; once it has, this file holds a valid public key.
        let text =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(RELEASE_KEY_FILE))
                .unwrap();
        if key_line(&text).is_some() {
            parse_public_key(&text).unwrap();
        }
    }

    fn fixture_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/update")
    }

    /// The signed fixture the core and app tests read, file by file:
    /// `test-key.pub`, `update.json`, `update.json.sig`.
    fn fixture_files() -> [(&'static str, Vec<u8>); 3] {
        let key = SigningKey::from_bytes(&FIXTURE_SEED);
        let public = B64.encode(key.verifying_key().as_bytes());
        let key_file = format!(
            "# Test key for fixtures/update only (seed 0x42 × 32). Never trusted by a build.\n{public}\n"
        );
        // The app tests write these bytes as the feed's files (no .apk is tracked).
        let apk = b"fixture apk bytes".to_vec();
        let pack = b"fixture pack bytes".to_vec();
        let m = UpdateManifest {
            format: UPDATE_FORMAT,
            channel: Channel::Preview,
            version: "0.1.0".into(),
            build: 42,
            commit: Some("0123abc".into()),
            published_at: "2026-10-27T08:00:00Z".into(),
            notes: UpdateNotes {
                en: "Fixture: new L09 questions.".into(),
                ar_eg: "تجربة: أسئلة جديدة في المحاضرة التاسعة.".into(),
            },
            assets: vec![
                UpdateAsset {
                    platform: UpdatePlatform::AndroidArm64,
                    file: "batna-0.1.0-b42-android-arm64.apk".into(),
                    size: apk.len() as u64,
                    sha256: sha256_hex(&apk),
                    version_code: Some(100_999),
                    build: None,
                },
                UpdateAsset {
                    platform: UpdatePlatform::Content,
                    file: "batna-content-0.1.0-b42-content.pack".into(),
                    size: pack.len() as u64,
                    sha256: sha256_hex(&pack),
                    version_code: None,
                    build: None,
                },
            ],
            history: vec![
                BuildChanges {
                    version: "0.1.0".into(),
                    build: 42,
                    published_at: "2026-10-27T08:00:00Z".into(),
                    changes: changes::parse(
                        "- kind: new\n  ar_eg: أسئلة جديدة في المحاضرة التاسعة\n  en: New L09 questions\n\
                         - kind: improved\n  ar_eg: البنك بقى أسرع\n\
                         - kind: fixed\n  ar_eg: التايمر في الامتحان التجريبي مبقاش بيقف\n",
                    )
                    .unwrap(),
                    content_only: false,
                },
                BuildChanges {
                    version: "0.1.0".into(),
                    build: 40,
                    published_at: "2026-10-25T08:00:00Z".into(),
                    changes: changes::parse("- kind: new\n  ar_eg: أول نسخة تجريبية\n").unwrap(),
                    content_only: false,
                },
            ],
            ui_build: None,
        };
        let json = serde_json::to_vec_pretty(&m).unwrap();
        let sig = format!("{}\n", B64.encode(key.sign(&json).to_bytes()));
        [("test-key.pub", key_file.into_bytes()), (MANIFEST_FILE, json), (SIGNATURE_FILE, sig.into_bytes())]
    }

    /// Regenerates the fixture after a format change:
    /// `cargo test -p batna-tools write_update_fixture -- --ignored`.
    #[test]
    #[ignore]
    fn write_update_fixture() {
        fs::create_dir_all(fixture_dir()).unwrap();
        for (name, bytes) in fixture_files() {
            fs::write(fixture_dir().join(name), bytes).unwrap();
        }
    }

    /// The committed fixture is what the generator makes (Ed25519 signing is
    /// deterministic), and it passes the app's own checks.
    #[test]
    fn the_committed_fixture_is_current_and_verifies() {
        for (name, bytes) in fixture_files() {
            let committed = fs::read(fixture_dir().join(name)).unwrap();
            assert!(
                committed == bytes,
                "fixtures/update/{name} is stale: run write_update_fixture (see above)"
            );
        }
        let public = read_text(&fixture_dir().join("test-key.pub")).unwrap();
        let key = parse_public_key(&public).unwrap();
        let json = fs::read(fixture_dir().join(MANIFEST_FILE)).unwrap();
        let sig = read_text(&fixture_dir().join(SIGNATURE_FILE)).unwrap();
        let m = verify_manifest(&json, &sig, &key).unwrap();
        assert_eq!((m.channel, m.build, m.history.len()), (Channel::Preview, 42, 2));
    }
}
