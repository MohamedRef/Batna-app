# Batna releases

Batna is for one learner, on his own devices: the Galaxy S24 Ultra, the Galaxy
Tab S11 and the Mac. A release is a signed APK sideloaded onto both Galaxy
devices, plus a Batna.app built on the Mac. There are no store listings and no
paid developer accounts. Everything here costs $0.

**Nothing is released until everything is finished** (Mohamed's rule). The
first release will be an **alpha**. This page and its `just` recipes are
prepared ahead, so release day is routine. Until then, the only builds are
the CI *preview* APKs (§3).

Adapted from Kabar's `docs/RELEASE.md` on `main`. Batna has its own keys,
its own feed and its own package (`app.batna`). It never shares anything
with Kabar.

## 1. Stages

| Stage | Version | When |
|---|---|---|
| **Alpha** (first) | `1.0.0-alpha.N` | All chunks in SPLIT.md §7 are merged, the critic rounds are clean, and Mohamed says go. |
| **Beta** | `1.0.0-beta.N` | Later, after daily use of the alpha. Mohamed decides. |
| **Stable** | `1.0.0` | Later still. Mohamed decides. |

The version is `version` in the root `Cargo.toml`. The APK's `versionCode` is
derived from it (`scripts/android-release-apk.sh`). An alpha always sorts
before its beta, and a beta before the release, so each stage installs over
the previous one and keeps your data.

Until the first alpha, the version stays `0.1.1-alpha.0` (versionCode
101100). A version with no pre-release counts as a final release (stage
999); early previews shipped as `0.1.0` (100999), so the pre-alpha version
must sort above that, and the first alpha is `0.1.1-alpha.1` or later
(`1.0.0-alpha.1` is fine too). The versionCode may never go down: the
preview workflow refuses to publish a build below the live feed's. Only move to a bare version
(`1.0.0`) for a real stable release.

## 2. One-time setup on the Mac

| Step | Command | Notes |
|---|---|---|
| Tools | `brew install just`, Rust, `cargo install dioxus-cli --version 0.7.10`, Android SDK + NDK | Same as `docs/RUN.md` |
| Release signing key | `just android-keystore` | Writes `~/.batna/batna-release.jks`. The password goes into the Keychain (`batna-android`). **Back the file and password up.** Android only installs an update signed with the same key, so losing it means uninstalling Batna and starting over. |
| Mac signing identity | `just mac-codesign-cert` | A free self-signed "Batna Code Signing" certificate. Every Batna.app keeps the same identity, so the Keychain and the updater keep trusting it. |
| Update key | `just update-keygen` | Once, before the first alpha. Writes `crates/app/update-key.pub` (commit it) and keeps the secret in the Keychain (`batna-update`). See §5. |
| Icons | `just icons` (only after editing `assets/src/icon/*.svg`) | Needs `pip install cairosvg pillow`. The exported PNGs are committed. |

## 3. Builds

| What | Command | Output |
|---|---|---|
| Signed release APK | `just apk` | `dist/android/batna.apk` (arm64, not debuggable, R8, versionCode from the version) |
| Install on a device | `just devices`, then `just install-android <serial>` | Installs over the previous Batna signed with the same key |
| Mac app | `just bundle-mac` | `dist/mac/Batna.app`, signed "Batna Code Signing" once that exists |
| CI preview APK | Every push to `main` (`.github/workflows/batna-android-preview.yml`) | `batna-preview.apk` on the `batna-preview` prerelease. Signed with a throwaway preview key kept in the Actions cache (never Kabar's, never the release key). It updates itself from the same release (§5). |

A preview APK and a release APK carry different keys. Uninstall one before
installing the other, or keep only the release build.

## 4. Release day (only once Mohamed says go)

1. Set `version` in `Cargo.toml` (`1.0.0-alpha.1` the first time).
2. Run `just check`, then `just apk` and `just bundle-mac`.
3. Install by hand the first time. Copy `Batna.app` to Applications, and run
   `just install-android <serial>` for the phone and the tablet.
4. Walk the device checks: study a session on each device, check that
   reminders ring on one device only, and check that export and import
   round-trip.
5. Tag it, e.g. `git tag batna-v1.0.0-alpha.1`.
6. From the second alpha on, `just publish-update` sends it over the air
   instead of step 3 (§5).

Before installing anything, **Settings → Your data → Export everything** is a
cheap backup.

## 5. Updates over the air

Adapted from Kabar's design (`docs/RELEASE.md` §5 on Kabar-app `main`), with Batna's
own key, its own feeds and one extra asset: the content pack.

### The feed

Each channel is one GitHub release of this repository (`MohamedRef/Batna-app`). Downloading from a
public release needs no token and costs nothing.

| Channel | Release tag | Who follows it |
|---|---|---|
| Preview | `batna-preview` | CI preview APKs (every push to `main`). Signed with the CI preview key. |
| Alpha | `batna-alpha` | Release builds, from the first alpha on. |
| Beta / Stable | `batna-beta`, `batna-stable` | Later. A beta also reaches alpha followers; a release reaches everyone. |

A feed holds `update.json`, `update.json.sig` (Ed25519 over the exact bytes),
and the files the manifest names: the APK, the zipped Batna.app, and
`batna-content-….pack`. The types and checks are `batna_core::update`.

What a device checks before it uses anything:

1. The signature, against the public key built into the app
   (`crates/app/update-key.pub` for release builds; previews carry the CI
   preview key). Nothing in an unsigned or changed manifest is read.
2. The channel inside the manifest is the one asked for, the format is one
   this build understands, and every file name is a plain name (no paths,
   no URLs).
3. The version and build are newer than what is installed.
4. Each download's size and SHA-256 match the signed manifest. The file is
   hashed again just before it is installed.

Size caps: 256 KB for `update.json` and its signature, 400 MB for a build.

### Content packs

A new content pack is offered whenever its SHA-256 differs from the pack in
use, even when the app build itself is unchanged, and even with app updates
off. A pack from an older build than the one installed is never taken, and
the app decodes it with `batna_core::pack` before switching to it, so a pack
this build can't read is never used.

### Keys

| Key | Where it lives | Made by |
|---|---|---|
| Release update key (secret) | Mac Keychain, service `batna-update` | `just update-keygen`, once. The public half is committed as `crates/app/update-key.pub`. |
| Preview update key | GitHub Actions cache of the preview workflow | The workflow, on first run. Never the release key. |

`crates/app/update-key.pub` holds no key until `just update-keygen` runs, so
until then a release build trusts no feed at all. Back the secret up with the
Android keystore: losing it means every installed build has to be updated by
hand once.

### Publishing (prepared, not run)

Only once Mohamed says go:

1. `just apk` and `just bundle-mac` (§3).
2. `just publish-update "notes in English" "ملاحظات بالمصري"`.

`scripts/update-publish.sh` zips Batna.app with `ditto`, builds the content
pack, writes one manifest per channel the version belongs to, signs it with
the Keychain key, verifies it with `batna-tools update verify`, and uploads
the files first and the signed manifest last, so a device never reads a
manifest whose files aren't there yet.

By hand, the same steps are:

```sh
cargo run -q -p batna-tools -- update manifest --channel alpha \
  --asset android-arm64=dist/android/batna.apk \
  --asset macos-arm64=dist/mac/Batna.app.zip \
  --asset content=dist/packs/batna-content.pack --out dist/update/alpha
BATNA_UPDATE_SECRET="$(security find-generic-password -s batna-update -w)" \
  cargo run -q -p batna-tools -- update sign dist/update/alpha
cargo run -q -p batna-tools -- update verify dist/update/alpha
```

### On the device

`crates/app/src/update/` checks the feed 20 seconds after launch and then
every 6 hours, in the background. **Settings › التحديثات** shows the build
(version, CI build number, commit), where things stand, "دوّر دلوقتي", and two
per-device choices:

| Choice | Options |
|---|---|
| How it updates | **لوحده** (automatic, the default): downloads on Wi-Fi only and installs at a calm moment. **قولّي الأول** (tell me first): says so, and downloads and installs on a tap. **لأ** (off): only on "دوّر دلوقتي". |
| Channel | Alpha, beta or stable for release builds. A preview build always follows `batna-preview`. |

When something is ready, a card above the current screen says so, with the
notes from the feed. "بعدين" hides it until the next version.

| Device | How the new build goes in |
|---|---|
| Android | `android/…/UpdateInstaller.kt` checks the APK (same package, same signing key, a versionCode no lower than the installed one) and hands it to PackageInstaller. Android asks once for "Install unknown apps", then shows its own confirmation. On Android 12+, once Batna installed itself, automatic mode updates quietly while Batna is in the background, and never during a study session, bank block, case or oral round. When notifications are allowed, one says the new version is in. |
| Mac | The verified `Batna.app.zip` is unzipped with `ditto` next to the running app, its code signature is checked against the running one ("Batna Code Signing"), and the two are swapped with renames, so a failure leaves the old app in place. In automatic mode it goes in at the next launch; otherwise "ثبّتها وافتح تاني". |
| Content pack | Kept in `<data>/updates/content.pack`, and used from the next start. |

Automatic downloads wait while Android says the network is metered (mobile
data or Data Saver). Study data never moves: it stays in the data folder
across every update.

Trying it without GitHub (development builds only): build `batna-tools update
manifest` and `sign` into a folder with a test key, serve it with
`python3 -m http.server`, and run a debug build with
`BATNA_UPDATE_URL=http://127.0.0.1:8000 BATNA_UPDATE_KEY=<public key>
BATNA_UPDATE_PLATFORM=android-arm64 BATNA_BUILD=1`.
