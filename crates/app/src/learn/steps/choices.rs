//! The order a lesson question's options are shown in (check steps and
//! mini-case parts), stable per step.
//!
//! Uniform, like the bank and case steps (`batna_core::exam::option_order`):
//! numeric options smallest first, options that point at other options
//! ("None of the above", "Both A and B") in the book's order, the rest
//! shuffled. Lesson keys are almost always written first (97% of checks),
//! and the order-puzzle shuffle these steps used before keeps most items off
//! their own place, so the key showed as option A about one time in nine
//! instead of one in four (#1397).

use batna_core::exam::{Rng, option_order as exam_order};

/// `order[shown] = written option index`, the same every time for `seed`.
pub(super) fn option_order(options: &[String], seed: &str) -> Vec<usize> {
    // FNV-1a: small and stable across platforms and versions.
    let hash =
        seed.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3));
    exam_order(options, &mut Rng::new(hash))
}

#[cfg(test)]
mod tests;
