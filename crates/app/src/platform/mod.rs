//! Everything that differs between the Mac and Android. Android code reaches
//! the framework through JNI, plus one small Kotlin service in `android/`
//! (background oral mode).

pub mod background;
#[cfg(target_os = "android")]
pub mod install;
pub mod network;
pub mod tts;
#[cfg(target_os = "android")]
pub(crate) mod tts_android;
