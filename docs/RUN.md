# Run Batna

## Mac (desktop)
1. Install Rust (`curl https://sh.rustup.rs -sSf | sh`) and `just` (`brew install just`).
2. In the repo, switch to the Batna branch: `git fetch origin batna && git switch batna`.
3. `just run` opens the app. `BATNA_WINDOW_SIZE=412x900 just run` previews the phone layout. `BATNA_OPEN_SCREEN=study just run` opens a screen directly. `BATNA_OPEN_SETTINGS=look just run` opens Settings on a section (`study`, `look`, `tablet`, `mac`, `data`, `updates`, `about`; empty for the list). Theme and text size are kept per device in `device-prefs.txt` in the data folder (Settings › Your data shows where).
   - Progress screen previews: `BATNA_OPEN_SCREEN=stats BATNA_STATS_TAB=plan BATNA_PLAN_EXAM=2027-01-20 BATNA_STATS_DEMO=1 just run` opens the exam planner with a preset exam day and demo numbers (labelled as demo on screen). `BATNA_STATS_TAB` is `progress`, `mistakes` or `plan`.

## S24 Ultra and Tab S11 (Android)
1. One-time setup: install the Dioxus CLI 0.7.10 (`cargo install dioxus-cli --version 0.7.10 --locked`), Java 17, and Android Studio's SDK plus NDK r27c. Then run `rustup target add aarch64-linux-android`.
2. Turn on USB debugging on the phone or tablet (Settings › About phone › Software information › tap Build number 7 times, then Developer options › USB debugging), and connect it with a cable.
3. `just android` builds and installs a debug build. The app id is `app.batna`, so it sits next to Kabar without touching it.

### Kotlin glue (`android/`)
`android/` at the repo root is a small Gradle library module with the Kotlin that Rust calls through JNI (screen-off oral mode and vibration now; reminders, export and updates later). Nothing to configure: the `#[manganis::ffi("../../android")]` marker in `crates/app/src/platform/background.rs` makes `dx` 0.7.10 copy the folder into its generated Gradle project, add it as an app dependency, and merge `android/src/main/AndroidManifest.xml` (permissions and receivers the glue needs) into the app's manifest.

`scripts/android-check-apk.sh <apk>` fails if the Kotlin classes or their permissions are missing from an APK; CI runs it on every change to `android/`. When you add a Kotlin class that Rust looks up by name, add it to `CLASSES` in that script. `adb logcat` shows Kotlin errors on the device.

Release builds (`just apk`, `just install-android`, `just bundle-mac`), their one-time keys, the CI preview APK and updates over the air are described in [RELEASE.md](RELEASE.md). Nothing is released before Mohamed says go, so for now the debug build, or the CI preview APK on the `batna-preview` prerelease, is the way to try Batna.

## Study in Anki now
One file, `/mnt/project-files/medstudy/anki/batna.apkg`, holds every written lecture as a subdeck of `Batna`. Open it with AnkiDroid on the phone or tablet, or with Anki on the Mac. Importing a newer `batna.apkg` updates the cards in place and keeps your progress. `just anki` builds it into `dist/anki/`; details in [ANKI.md](ANKI.md).

## Your data and your content checks
- Reviews, your ✓ / ✎ / ✗ checks and review settings live in one SQLite file on each device: `~/Library/Application Support/app.Batna.Batna/batna.db` on the Mac, and the app's private storage on Android. Settings › مذاكرتك shows the exact path. `BATNA_DATA_DIR=/some/folder just run` uses a separate profile (handy for previews).
- Settings › صدّر مراجعاتك للمحتوى writes `batna-verdicts-YYYY-MM-DD.yaml` (format `batna.verdicts/v1`) to `~/Downloads` on the Mac, or to `Android/data/app.batna/files` on the phone and tablet (open it with the Files app or over USB). The same text is shown on screen, so you can copy it. Send it in the chat: `batna-tools` turns it into a fix list per lecture and adds your ✓ to `content/approved.txt`.
- Settings also sets the desired retention (85, 90 or 95%) and new items a day (15, 30, 45 or 60). After 400 reviews the schedule is fitted to your own memory, and it is refitted once a week.
