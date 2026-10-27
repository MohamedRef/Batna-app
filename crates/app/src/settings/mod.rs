//! Settings. Chunk 1 owns the settings sheet; `menu` (the Mac menu bar) is
//! chunk 23's.

#[cfg(all(feature = "desktop", not(target_os = "android")))]
pub mod menu;
