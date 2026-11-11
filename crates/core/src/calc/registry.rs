//! The calculator registry: every calculator, in screen order.

use super::*;

/// Every calculator, in the order the Calculators screen lists them.
pub fn all() -> &'static [CalcSpec] {
    REGISTRY
}

pub fn spec(id: &str) -> Option<&'static CalcSpec> {
    REGISTRY.iter().find(|s| s.id == id)
}

pub fn compute(id: &str, inputs: &Inputs) -> Result<Outcome, CalcError> {
    let spec = spec(id).ok_or_else(|| CalcError::Unknown(id.to_owned()))?;
    spec.run(inputs)
}

impl CalcSpec {
    /// Computes the result after refusing a number outside its field's
    /// `min`..`max` ([`IMPOSSIBLE`]), so a slip like serum albumin 12 g/dL
    /// never reaches a formula. Every caller goes through this, not
    /// `compute` directly.
    pub fn run(&self, inputs: &Inputs) -> Result<Outcome, CalcError> {
        for f in self.fields {
            if let Some((min, max)) = f.limits()
                && let Some(v) = inputs.opt_num(f.key)?
                && !(min..=max).contains(&v)
            {
                return Err(CalcError::Invalid { field: f.key, reason: IMPOSSIBLE });
            }
        }
        (self.compute)(inputs)
    }
}

// A slice, not an array, so adding one doesn't touch a count that every
// other branch adding one also changes.
pub(super) static REGISTRY: &[CalcSpec] = &[
    child_pugh::SPEC,
    meld::MELD,
    meld::MELD_NA,
    albi::SPEC,
    fib4::SPEC,
    apri::SPEC,
    fli::SPEC,
    baveno::SPEC,
    r_ratio::SPEC,
    maddrey::SPEC,
    glasgow::SPEC,
    saag::SPEC,
    ascitic::SPEC,
    sos::SPEC,
    cha2ds2_vasc::SPEC,
    bclc::SPEC,
    milan::SPEC,
    okuda::SPEC,
    page_b::SPEC,
    kings::SPEC,
    clichy::SPEC,
    ranson::SPEC,
    bisap::SPEC,
    ctsi::SPEC,
    atlanta::SPEC,
    gbs::SPEC,
    lgib::SHOCK_INDEX,
    lgib::OAKLAND,
    uc_activity::SPEC,
    kyoto::SPEC,
    rome_ibs::SPEC,
    renal::ANION_GAP,
    renal::DELTA_RATIO,
    renal::COMPENSATION,
    renal::OSMOLAL_GAP,
    aki::KDIGO,
    aki::FENA,
    aki::FEUREA,
    bun_cr::SPEC,
    ckd::EGFR,
    ckd::STAGE,
    aa_gradient::SPEC,
    free_water::SPEC,
    plasmic::SPEC,
    cockcroft::SPEC,
    geri::KATZ,
    geri::LAWTON,
    geri::GDS,
    geri::MNA_SF,
    geri::CFS,
    geri::CAM,
    geri::SARC_F,
    geri::MMSE,
    geri::ABI,
    ra2010::SPEC,
    spa::CASPAR,
    spa::ASAS_AXIAL,
    spa::ASAS_PERIPHERAL,
    spa::MODIFIED_NEW_YORK,
    sle::SLE_1997,
    sle::SLE_2019,
    aav::GPA_2022,
    aav::MPA_2022,
    diabetes::CORRECTED_NA,
    diabetes::OSMOLALITY,
    dka::SPEC,
    insulin::PREMIXED,
    bmi::SPEC,
    ipss_mf::SPEC,
    rpi::SPEC,
    cll::RAI,
    cll::BINET,
    lymphoma::ANN_ARBOR,
    lymphoma::IPI,
    mpn::PV,
    mpn::ET,
    mpn::PMF,
    research::INCIDENCE,
    research::PREVALENCE,
    research::PREVALENCE_ID,
    research::ATTACK_RATE,
    research::SAR,
    research::CFR,
    research::PMR,
    research::IMR,
    research::NMR,
    research::RELATIVE_RISK,
    research::ODDS_RATIO,
    research::ATTRIBUTABLE_RISK,
    research::NNT,
    research::DIAGNOSTIC,
    research::Z_SCORE,
    frequency::DISEASE_DURATION,
    frequency::DEATH_RATE,
    frequency::PERINATAL,
    frequency::MMR,
    sample_size::SURVEY,
    sample_size::TWO_PROPORTIONS,
    diagnostic::PREDICTIVE,
    diagnostic::POST_TEST,
    significance::CHI_SQUARE,
    significance::T_TEST,
];
