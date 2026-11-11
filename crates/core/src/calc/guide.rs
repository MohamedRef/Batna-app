//! The next step and differential for each result band.

use super::*;

/// What to do with a result and what else it could be: the next step and
/// the differential diagnosis for one band. Kept apart from [`Outcome`] so a
/// calculator without a guide still builds; each module lists its own as
/// `GUIDES` and [`GUIDE_LISTS`] collects them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    /// Next steps for this result, most important first.
    pub next: &'static [&'static str],
    /// The differential diagnosis (or causes to look for) for this result.
    pub dd: &'static [&'static str],
}

// One line per module that has guides, so branches adding them don't touch
// the same line.
#[rustfmt::skip]
pub(super) static GUIDE_LISTS: &[&[(&str, Guide)]] = &[
    geri::GUIDES,
    spa::GUIDES,
    sle::GUIDES,
    aav::GUIDES,
    page_b::GUIDES,
    milan::GUIDES,
    okuda::GUIDES,
    ctsi::GUIDES,
    atlanta::GUIDES,
    baveno::GUIDES,
    fli::GUIDES,
    sos::GUIDES,
    cha2ds2_vasc::GUIDES,
    rpi::GUIDES,
    cll::GUIDES,
    lymphoma::GUIDES,
    mpn::GUIDES,
    lgib::GUIDES,
    uc_activity::GUIDES,
    kyoto::GUIDES,
    rome_ibs::GUIDES,
    dka::GUIDES,
    insulin::GUIDES,
    bmi::GUIDES,
    bun_cr::GUIDES,
    ckd::GUIDES,
    aa_gradient::GUIDES,
    free_water::GUIDES,
    plasmic::GUIDES,
    child_pugh::GUIDES,
    meld::GUIDES,
    albi::GUIDES,
    fib4::GUIDES,
    apri::GUIDES,
    r_ratio::GUIDES,
    maddrey::GUIDES,
    glasgow::GUIDES,
    saag::GUIDES,
    ascitic::GUIDES,
    bclc::GUIDES,
    kings::GUIDES,
    clichy::GUIDES,
    ranson::GUIDES,
    bisap::GUIDES,
    gbs::GUIDES,
    renal::GUIDES,
    aki::GUIDES,
    cockcroft::GUIDES,
    ra2010::GUIDES,
    diabetes::GUIDES,
    ipss_mf::GUIDES,
    research::GUIDES,
    sample_size::GUIDES,
    significance::GUIDES,
];

/// The next step and differential for a band id (`cam.positive`), if
/// written.
pub fn guide(band: &str) -> Option<&'static Guide> {
    GUIDE_LISTS.iter().flat_map(|l| l.iter()).find(|(b, _)| *b == band).map(|(_, g)| g)
}
