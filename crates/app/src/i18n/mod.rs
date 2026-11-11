//! UI text, in Egyptian Arabic. One file per feature: a feature chunk adds
//! its own `i18n/<feature>.rs` and one `pub mod <feature>;` line here, and
//! touches nothing else. Medical terms stay in English (wrap them in `bdi`
//! in the markup).

pub mod abbr;
pub mod anki;
pub mod app;
pub mod bank;
pub mod book;
pub mod calc;
pub mod case;
pub mod catchup;
pub mod cite;
pub mod cloud;
pub mod count;
pub mod courses;
pub mod data;
pub mod desk;
pub mod gopage;
pub mod hover;
pub mod keys;
pub mod labs;
pub mod learn;
pub mod lecture;
pub mod metacog;
pub mod nav;
pub mod notes;
pub mod onboard;
pub mod oral;
pub mod order;
pub mod own;
pub mod palette;
pub mod plan;
pub mod recap;
pub mod refs;
pub mod remind;
pub mod saving;
pub mod settings;
pub mod sheet;
pub mod sound;
pub mod stats;
pub mod study;
pub mod study_settings;
pub mod table;
pub mod today;
pub mod update;
pub mod verify;
pub mod views;
pub mod weak;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub mod web;

#[cfg(test)]
mod grammar_tests;
#[cfg(test)]
mod line_tests;
