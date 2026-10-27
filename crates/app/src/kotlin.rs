//! Tells `dx` to build `android/` (repo root) into the APK: it copies the folder
//! into its generated Gradle project as the library module `:plugins:plugin`,
//! adds it as an app dependency, and the manifest merger adds its manifest.
//! The macro only embeds that metadata in the binary; it declares no
//! functions. The JNI calls into these classes are hand-written next to the
//! features that use them. Adapted from kabar-app main:
//! crates/app/src/platform/android.rs (`kotlin_module`).

#![allow(unused_imports)]

#[manganis::ffi("../../android")]
extern "Kotlin" {}
