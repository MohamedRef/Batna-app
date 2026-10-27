//! Everything that differs between the Mac and Android. Android code reaches
//! the framework through JNI; nothing here needs Kotlin.

pub mod tts;
#[cfg(target_os = "android")]
mod tts_android;
