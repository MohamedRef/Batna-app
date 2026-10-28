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

## 2. One-time setup on the Mac

| Step | Command | Notes |
|---|---|---|
| Tools | `brew install just`, Rust, `cargo install dioxus-cli --version 0.7.10`, Android SDK + NDK | Same as `docs/RUN.md` |
| Release signing key | `just android-keystore` | Writes `~/.batna/batna-release.jks`. The password goes into the Keychain (`batna-android`). **Back the file and password up.** Android only installs an update signed with the same key, so losing it means uninstalling Batna and starting over. |
| Mac signing identity | `just mac-codesign-cert` | A free self-signed "Batna Code Signing" certificate. Every Batna.app keeps the same identity, so the Keychain and the updater keep trusting it. |
| Icons | `just icons` (only after editing `assets/src/icon/*.svg`) | Needs `pip install cairosvg pillow`. The exported PNGs are committed. |

## 3. Builds

| What | Command | Output |
|---|---|---|
| Signed release APK | `just apk` | `dist/android/batna.apk` (arm64, not debuggable, R8, versionCode from the version) |
| Install on a device | `just devices`, then `just install-android <serial>` | Installs over the previous Batna signed with the same key |
| Mac app | `just bundle-mac` | `dist/mac/Batna.app`, signed "Batna Code Signing" once that exists |
| CI preview APK (coming in a later chunk 22 PR) | Every push to `batna` | Signed with a throwaway preview key kept in the Actions cache (never Kabar's, never the release key). Published to the `batna-preview` prerelease |

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
5. Tag it, e.g. `git tag batna-v1.0.0-alpha.1`. Every later update reaches
   the devices over the air.

Before installing anything, **Settings → Your data → Export everything** is a
cheap backup.

## 5. Updates over the air

The over-the-air updater (`crates/app/src/update/`) arrives in the next
chunk 22 PR. It works like Kabar's: a signed `update.json` (Ed25519, Batna's
own key) on a GitHub release per channel, a SHA-256 check on each download,
and three settings (automatic, tell me first, off). The channel is
`batna-preview` now and `batna-alpha` later. Each step of that design is
described here once it lands.
