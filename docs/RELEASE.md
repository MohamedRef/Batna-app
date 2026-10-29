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
| CI preview APK | Every push to `main` (`.github/workflows/batna-android-preview.yml`) | `batna-preview.apk` on the `batna-preview` prerelease. Signed with a throwaway preview key kept in the Actions cache (never Kabar's, never the release key). A run on Mondays and Thursdays keeps the keys from expiring, and a run fails rather than silently making new keys once previews are out. If it ever says a key is missing, the only way on is Actions › Android preview APK › Run workflow with **new_keys** ticked, then uninstall the old preview once. It updates itself from the same release (§5). |

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

A build download that stops (the line drops, Batna is closed) keeps its
`<file>.part`; the next try, a few seconds later or at the next check or tap,
asks only for the rest with an HTTP `Range` request (GitHub's release CDN
serves ranges). The whole file is still hashed at the end, and a resumed
file that doesn't match is fetched once more from the start. Content packs
are a few MB and download whole.

### Content packs

A new content pack is offered whenever its SHA-256 differs from the pack in
use, even when the app build itself is unchanged, and even with app updates
off. A pack from an older build than the one installed is never taken, and
the app decodes it with `batna_core::pack` before switching to it, so a pack
this build can't read is never used.

A pack also carries the app's stylesheets (`Pack::look`, every `.css` file
under `crates/app`), so a change to the look needs no install either. The
manifest's `ui_build` names the app build they were made with, and only a
running build with exactly that number uses them (`crates/app/src/look.rs`);
any other build takes the lectures and keeps its own look. Strings, icons,
layout and logic are code and still come with an APK.

**Content-only feeds.** When everything merged since the build on the
preview feed is under `content/index.yaml`, `content/hb/`, `content/lessons/`,
`changes/`, `docs/`, a top-level `.md` or a `crates/app/**/*.css` file, CI
builds no APK: `batna-tools update manifest --carry android-arm64` keeps the
feed's APK entry as it is (same file, its own `build`), and only the pack is
new; `batna-tools validate` runs before the pack is built, and
`update verify --carried android-arm64` checks the feed without the APK file,
which stays on the release. Apps compare an asset's own `build`, so nothing is offered for install;
the pack is downloaded in the background and switched to live once Mohamed
is between sessions (the calm moments a quiet install waits for). The sheet
doesn't open by itself for it: a quiet card on Today names the first change
and opens "What's new" (titled "تحديث محتوى") on a tap. Settings and the
sheet show the app build and the content apart ("بناء 40 · المحتوى من 45").
Open screens and the review catalog follow the new pack. A pack whose
file fails its checks, won't decode, or carries a stylesheet that reaches
outside the app (`@import`, a remote `url(…)`, script) is refused and its
sha256 remembered, so it isn't fetched again; a pack that is opened three
times in a row without Batna getting past its first screen goes the same way
and the built-in content comes back (`crates/app/src/update/mod.rs`). The glossary is built into the app, so a glossary change is
a full build.

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
first two lines of what's new (below) and "إيه الجديد؟". "بعدين" hides it
until the next version.

**"A new Batna is out" notification.** On Android a JobScheduler job
(`UpdateCheckJob` in `android/…/UpdateNotice.kt`) runs about every four hours,
even while Batna is closed and across reboots. It loads Batna's native
library and calls Rust's `update::background_check` over JNI, so the feed is
checked with the same signature and hash checks, and in automatic mode on
Wi-Fi the build is downloaded too. The app's own six-hourly checks send the
notification as well, when Batna is in the background. There is one per
version and step (out, then downloaded and ready), with the first three lines
of what's new and a **نزّلها** or **ثبّتها** button. A tap opens the "what's
new" sheet and the button starts the download or the install. A tap while
Batna is already open is picked up within two seconds (the tap's activity is
translucent, so the page may never hide), and a tap left over a minute is
dropped. The job marks a notification as sent only after Android showed it
(`UpdateCheckJob.shown`), so one Android refused goes out at the next run
(Batna-app#165). Nothing is sent while Batna is in front
(the card covers that), for a version whose card was closed, in "لأ" mode, with
Settings › Updates' switch off, or when Android's notification switch is off.
The Mac gets the same notification once UNUserNotificationCenter lands
(Batna-app#50).

**What's new.** Every pull request adds or extends
`changes/<topic>.yaml` (CI fails one that adds no line: `batna-tools update
changes --gate HEAD^1` compares the merge commit with its own base, and a
comment or whitespace edit doesn't count), one line per change, in Egyptian Arabic (`en`
optional), each `new`, `improved` or `fixed`:

```yaml
- kind: fixed
  ar_eg: المؤقت في الامتحان التجريبي مبقاش بيقف
  en: The mock timer no longer stops
```

A file holding `[]` says there's nothing to tell (CI, refactors, docs). When
a feed is built (CI for `batna-preview`, `just publish-update` for releases),
`batna-tools update manifest --previous <current update.json> --changes`
gathers the files added since the commit the feed carries now, one pull
request at a time, plus the lines a pull request appended to an existing
file (a fragment that is moved or renamed keeps its old lines old). A commit with no file (pushed straight to main, or merged before the
check) is looked up by title in `changes/retro/titles.yaml`, which also
rewrites the English-only lines older feeds carry; failing that it falls
back to its title in English, and titles starting with CI, docs, refactor,
test and the like are skipped. That build's entry goes on top of the feed's `history` (40
builds, inside the signed `update.json`, so nothing extra to download or
trust), and the notes older apps read are its first three lines.
`batna-tools update changes
[--since <commit>]` previews what the next build will say.

The app keeps every history it verifies (`updates/history.json`) and shows
it in one sheet, laid out like Morphe Manager's update dialog (ideas only,
none of its code): the installed build → the new one, how many builds that
is and a count per kind, then a timeline with the newest build open and a
card per kind ("اعرض الكل" past six lines). On a phone it fills the screen under Batna's icon; GitHub (the channel's releases page) and the download or install button stay pinned at the bottom. It opens from the banner's "إيه
الجديد؟", from the notification, and from Settings › التحديثات (also "سجل
التغييرات": every build so far); downloading and installing happen in it
too. At the first launch after an update it opens once by itself, listing
everything since the build that ran before.

| Device | How the new build goes in |
|---|---|
| Android | `android/…/UpdateInstaller.kt` checks the APK (same package, same signing key, a versionCode no lower than the installed one) and hands it to PackageInstaller. Android asks once for "Install unknown apps", then shows its own confirmation. On Android 12+, once Batna installed itself, automatic mode updates quietly while Batna is in the background, and never during a study session, bank block, case or oral round. When notifications are allowed, one says the new version is in. |
| Mac | The verified `Batna.app.zip` is unzipped with `ditto` next to the running app, its code signature is checked against the running one ("Batna Code Signing"), and the two are swapped with renames, so a failure leaves the old app in place. In automatic mode it goes in at the next launch; otherwise "ثبّتها وافتح تاني". |
| Content pack | Kept in `<data>/updates/content.pack`, and switched to live once he is between sessions (else at the next start). No install. |

Automatic downloads wait while Android says the network is metered (mobile
data or Data Saver). Study data never moves: it stays in the data folder
across every update.

Trying it without GitHub (development builds only): build `batna-tools update
manifest` and `sign` into a folder with a test key, serve it with
`python3 -m http.server`, and run a debug build with
`BATNA_UPDATE_URL=http://127.0.0.1:8000 BATNA_UPDATE_KEY=<public key>
BATNA_UPDATE_PLATFORM=android-arm64 BATNA_BUILD=1`.
