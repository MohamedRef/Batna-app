//! Lab values in lesson text and cards, with their normal range: "Na 128
//! mmol/L" reads as sodium, and the app puts the adult range beside it
//! (Mohamed, 2026-11-04). The numbers are [`crate::calc::normals`], the same
//! ones the calculator boxes use, so a range lives in one place; the few
//! units and tests the calculators don't take are added here, each with its
//! source. A course book's range wins where it gives one, then the NBME
//! Laboratory Values table (April 2026, nbme.org/laboratory-values), then a
//! named guideline.
//!
//! [`find`] is cautious: it wants the test's name, then at most a comparator
//! or a linking word, then a number and (for most tests) its unit. A change
//! ("a rise in creatinine of 0.3 mg/dL"), another fluid ("ascitic albumin",
//! "urine sodium") or a multiple ("ALT 3× ULN") is left alone.

use crate::calc::normals as n;
use crate::calc::{Normal, Place};

/// One test in one unit.
#[derive(Debug)]
pub struct Lab {
    pub id: &'static str,
    /// Spellings matched with their case (`Na`, `HCO3`).
    pub abbr: &'static [&'static str],
    /// Words matched ignoring case (`sodium`, `serum sodium`).
    pub words: &'static [&'static str],
    /// The unit as shown, then other spellings. Empty: the test has no
    /// unit (pH, INR).
    pub units: &'static [&'static str],
    /// A number written without a unit still counts when it falls in this
    /// span ("Na 128", "eGFR <30"); `None`: the unit must be written.
    pub bare: Option<(f64, f64)>,
    pub normal: Normal,
    /// Which value the range is for, when it matters ("fasting").
    pub note: &'static str,
    pub source: &'static str,
}

const NBME: &str = "NBME Laboratory Values, April 2026";

const fn r(low: f64, high: f64) -> Normal {
    Normal { low, high, female: None, below: false }
}
const fn sexes(male: (f64, f64), female: (f64, f64)) -> Normal {
    Normal { low: male.0, high: male.1, female: Some(female), below: false }
}
const fn under(high: f64) -> Normal {
    Normal { low: f64::NEG_INFINITY, high, female: None, below: true }
}

const MMOL: &[&str] = &["mmol/L", "mEq/L", "meq/l"];
const MG_DL: &[&str] = &["mg/dL"];
const G_DL: &[&str] = &["g/dL"];
const G_L: &[&str] = &["g/L"];
const UMOL: &[&str] = &["µmol/L", "μmol/L", "umol/L"];
const U_L: &[&str] = &["U/L", "IU/L"];
const MM_HG: &[&str] = &["mmHg", "mm Hg"];
const PER_MM3: &[&str] = &["/mm³", "/mm3", "/µL", "/μL", "/uL", "/cmm"];
const E9_L: &[&str] = &["×10⁹/L", "×10^9/L", "x10^9/L", "×10³/µL", "×10^3/µL", "x10^3/uL", "×10³/μL", "/nL"];

const HB: &[&str] = &["Hb", "Hgb"];
const HB_W: &[&str] = &["hemoglobin", "haemoglobin"];
const CREAT: &[&str] = &["Cr", "SCr", "sCr", "S.Cr"];
const CREAT_W: &[&str] = &["creatinine", "serum creatinine"];
const GLU_W: &[&str] = &[
    "glucose",
    "fasting glucose",
    "fasting plasma glucose",
    "fasting blood glucose",
    "blood glucose",
    "plasma glucose",
];
const BIL_W: &[&str] = &["bilirubin", "total bilirubin", "serum bilirubin"];
const ALB_W: &[&str] = &["albumin", "serum albumin"];
const CA: &[&str] = &["Ca", "Ca2+", "Ca²⁺"];
const CA_W: &[&str] = &["calcium", "serum calcium", "corrected calcium"];
const PLT: &[&str] = &["PLT", "Plt"];
const PLT_W: &[&str] = &["platelets", "platelet count", "platelet"];
const WBC: &[&str] = &["WBC", "WBCs", "TLC"];
const WBC_W: &[&str] = &["white cell count", "leucocytes", "leukocytes", "white cells"];
const TG: &[&str] = &["TG", "TGs"];
const TG_W: &[&str] = &["triglycerides", "triglyceride"];
const CHOL_W: &[&str] = &["total cholesterol", "cholesterol"];
const A1C: &[&str] = &["HbA1c", "A1C", "A1c", "HbA1C"];
const UREA_W: &[&str] = &["urea", "blood urea", "serum urea"];
const ACR: &[&str] = &["ACR", "UACR"];

/// Every test the finder knows, one row per unit.
pub static LABS: &[Lab] = &[
    // Electrolytes and acid–base.
    Lab {
        id: "sodium",
        abbr: &["Na", "Na+", "Na⁺", "SNa", "sNa"],
        words: &["sodium", "serum sodium"],
        units: MMOL,
        bare: Some((100.0, 180.0)),
        normal: n::SODIUM,
        note: "",
        source: "Nephrology book, Lecture 6, p. 127",
    },
    Lab {
        id: "potassium",
        abbr: &["K", "K+", "K⁺", "SK"],
        words: &["potassium", "serum potassium"],
        units: MMOL,
        bare: Some((1.5, 9.5)),
        normal: r(3.5, 5.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "chloride",
        abbr: &["Cl-", "Cl−", "Cl⁻"],
        words: &["chloride", "serum chloride"],
        units: MMOL,
        bare: Some((70.0, 140.0)),
        normal: n::CHLORIDE,
        note: "",
        source: NBME,
    },
    Lab {
        id: "hco3",
        abbr: &["HCO3", "HCO3-", "HCO3−", "HCO₃", "HCO₃⁻", "HCO3⁻"],
        words: &["bicarbonate", "serum bicarbonate"],
        units: MMOL,
        bare: Some((2.0, 50.0)),
        normal: n::HCO3,
        note: "",
        source: "Nephrology book, Lecture 1, p. 14–15",
    },
    Lab {
        id: "anion_gap",
        abbr: &["AG"],
        words: &["anion gap"],
        units: MMOL,
        bare: Some((0.0, 60.0)),
        normal: n::ANION_GAP,
        note: "",
        source: "Nephrology book, Lecture 1, p. 14 (12 ± 4)",
    },
    Lab {
        id: "ph",
        abbr: &["pH"],
        words: &[],
        units: &[],
        bare: Some((6.5, 8.0)),
        normal: n::PH,
        note: "",
        source: "Nephrology book, Lecture 1, p. 15",
    },
    Lab {
        id: "pco2",
        abbr: &["PaCO2", "pCO2", "PCO2", "PaCO₂", "pCO₂", "PCO₂", "Paco2"],
        words: &[],
        units: MM_HG,
        bare: Some((10.0, 120.0)),
        normal: n::PCO2_MM_HG,
        note: "",
        source: "Nephrology book, Lecture 1, p. 14–15",
    },
    Lab {
        id: "po2",
        abbr: &["PaO2", "pO2", "PO2", "PaO₂", "pO₂"],
        words: &[],
        units: MM_HG,
        bare: Some((20.0, 600.0)),
        normal: n::PAO2_MM_HG,
        note: "room air",
        source: "Nephrology book, Lecture 1, p. 15",
    },
    Lab {
        id: "osmolality",
        abbr: &["Posm", "POsm", "SOsm"],
        words: &["osmolality", "serum osmolality", "plasma osmolality"],
        units: &["mOsm/kg", "mOsmol/kg", "mOsm/L", "mOsm/kg H2O"],
        bare: Some((200.0, 400.0)),
        normal: n::OSMOLALITY,
        note: "",
        source: NBME,
    },
    Lab {
        id: "lactate",
        abbr: &[],
        words: &["lactate", "serum lactate", "lactic acid"],
        units: &["mmol/L"],
        bare: None,
        normal: n::LACTATE_MMOL_L,
        note: "",
        source: "Sepsis-3 (Singer et al., JAMA 2016)",
    },
    Lab {
        id: "magnesium",
        abbr: &["Mg", "Mg2+", "Mg²⁺"],
        words: &["magnesium", "serum magnesium"],
        units: MG_DL,
        bare: None,
        normal: r(1.5, 2.3),
        note: "",
        source: "Nephrology book, Lecture 6, p. 142",
    },
    Lab {
        id: "phosphate",
        abbr: &["PO4", "PO₄"],
        words: &["phosphate", "phosphorus", "serum phosphate"],
        units: MG_DL,
        bare: None,
        normal: r(3.0, 4.5),
        note: "",
        source: NBME,
    },
    Lab {
        id: "calcium",
        abbr: CA,
        words: CA_W,
        units: MG_DL,
        bare: None,
        normal: n::CALCIUM_MG_DL,
        note: "total",
        source: NBME,
    },
    Lab {
        id: "calcium_si",
        abbr: CA,
        words: CA_W,
        units: &["mmol/L"],
        bare: None,
        normal: r(2.1, 2.6),
        note: "total",
        source: NBME,
    },
    Lab {
        id: "uric_acid",
        abbr: &["SUA"],
        words: &["uric acid", "serum uric acid", "urate", "serum urate"],
        units: MG_DL,
        bare: None,
        normal: r(3.0, 8.2),
        note: "",
        source: NBME,
    },
    // Kidney.
    Lab {
        id: "creatinine",
        abbr: CREAT,
        words: CREAT_W,
        units: MG_DL,
        bare: Some((0.2, 15.0)),
        normal: n::CREATININE_MG_DL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "creatinine_si",
        abbr: CREAT,
        words: CREAT_W,
        units: UMOL,
        bare: None,
        normal: r(53.0, 106.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "bun",
        abbr: &["BUN"],
        words: &["urea nitrogen", "blood urea nitrogen"],
        units: MG_DL,
        bare: None,
        normal: n::BUN_MG_DL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "urea",
        abbr: &[],
        words: UREA_W,
        units: MG_DL,
        bare: None,
        normal: n::UREA_MG_DL,
        note: "",
        source: "NBME urea nitrogen 7–18 mg/dL × 2.14",
    },
    Lab {
        id: "urea_si",
        abbr: &[],
        words: UREA_W,
        units: &["mmol/L"],
        bare: None,
        normal: n::UREA_MMOL_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "egfr",
        abbr: &["eGFR", "GFR"],
        words: &[],
        units: &[
            "mL/min/1.73 m²",
            "mL/min/1.73m²",
            "mL/min/1.73 m2",
            "mL/min/1.73m2",
            "ml/min/1.73",
            "mL/min",
        ],
        bare: Some((1.0, 150.0)),
        normal: n::EGFR,
        note: "",
        source: "KDIGO 2024 CKD guideline, GFR category G1",
    },
    Lab {
        id: "acr",
        abbr: ACR,
        words: &["albumin-to-creatinine ratio", "albumin–creatinine ratio", "albumin/creatinine ratio"],
        units: &["mg/g"],
        bare: None,
        normal: n::ACR_MG_G,
        note: "",
        source: "KDIGO 2024 CKD guideline, albuminuria category A1",
    },
    Lab {
        id: "acr_si",
        abbr: ACR,
        words: &["albumin-to-creatinine ratio", "albumin–creatinine ratio", "albumin/creatinine ratio"],
        units: &["mg/mmol"],
        bare: None,
        normal: under(3.0),
        note: "",
        source: "KDIGO 2024 CKD guideline, albuminuria category A1",
    },
    // Sugar and lipids.
    Lab {
        id: "glucose",
        abbr: &["FPG", "FBG", "FBS"],
        words: GLU_W,
        units: MG_DL,
        bare: None,
        normal: r(70.0, 100.0),
        note: "fasting",
        source: NBME,
    },
    Lab {
        id: "glucose_si",
        abbr: &["FPG", "FBG", "FBS"],
        words: GLU_W,
        units: &["mmol/L"],
        bare: None,
        normal: r(3.8, 5.6),
        note: "fasting",
        source: NBME,
    },
    Lab {
        id: "hba1c",
        abbr: A1C,
        words: &[],
        units: &["%"],
        bare: None,
        normal: under(5.7),
        note: "",
        source: "Diabetes book, Lecture 1, p. 11 (A1C > 5.7%: prediabetes)",
    },
    Lab {
        id: "hba1c_si",
        abbr: A1C,
        words: &[],
        units: &["mmol/mol"],
        bare: None,
        normal: under(39.0),
        note: "",
        source: "ADA Standards of Care 2025 (5.7% = 39 mmol/mol)",
    },
    Lab {
        id: "triglycerides",
        abbr: TG,
        words: TG_W,
        units: MG_DL,
        bare: None,
        normal: n::TRIGLYCERIDES_MG_DL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "triglycerides_si",
        abbr: TG,
        words: TG_W,
        units: &["mmol/L"],
        bare: None,
        normal: under(1.7),
        note: "",
        source: NBME,
    },
    Lab {
        id: "cholesterol",
        abbr: &["TC"],
        words: CHOL_W,
        units: MG_DL,
        bare: None,
        normal: under(200.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "cholesterol_si",
        abbr: &["TC"],
        words: CHOL_W,
        units: &["mmol/L"],
        bare: None,
        normal: under(5.2),
        note: "",
        source: NBME,
    },
    Lab {
        id: "tsh",
        abbr: &["TSH"],
        words: &[],
        units: &["mIU/L", "µU/mL", "μU/mL", "uU/mL", "mU/L", "µIU/mL", "μIU/mL"],
        bare: Some((0.0, 200.0)),
        normal: r(0.4, 4.0),
        note: "",
        source: NBME,
    },
    // Liver.
    Lab {
        id: "albumin",
        abbr: &["Alb"],
        words: ALB_W,
        units: G_DL,
        bare: None,
        normal: n::ALBUMIN_G_DL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "albumin_si",
        abbr: &["Alb"],
        words: ALB_W,
        units: G_L,
        bare: None,
        normal: r(35.0, 55.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "protein",
        abbr: &[],
        words: &["total protein", "total proteins", "serum protein", "serum proteins"],
        units: G_DL,
        bare: None,
        normal: r(6.0, 7.8),
        note: "",
        source: NBME,
    },
    Lab {
        id: "bilirubin",
        abbr: &["TSB", "T.bil", "TBil"],
        words: BIL_W,
        units: MG_DL,
        bare: None,
        normal: n::BILIRUBIN_MG_DL,
        note: "total",
        source: NBME,
    },
    Lab {
        id: "bilirubin_si",
        abbr: &["TSB", "T.bil", "TBil"],
        words: BIL_W,
        units: UMOL,
        bare: None,
        normal: r(2.0, 17.0),
        note: "total",
        source: NBME,
    },
    Lab {
        id: "ast",
        abbr: &["AST", "SGOT"],
        words: &[],
        units: U_L,
        bare: None,
        normal: n::AST_U_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "alt",
        abbr: &["ALT", "SGPT"],
        words: &[],
        units: U_L,
        bare: None,
        normal: n::ALT_U_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "alp",
        abbr: &["ALP"],
        words: &["alkaline phosphatase"],
        units: U_L,
        bare: None,
        normal: n::ALP_U_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "ggt",
        abbr: &["GGT", "γGT", "GGTP"],
        words: &[],
        units: U_L,
        bare: None,
        normal: n::GGT_U_L,
        note: "",
        source: "Mayo Clinic Laboratories (adults)",
    },
    Lab {
        id: "ldh",
        abbr: &["LDH"],
        words: &["lactate dehydrogenase"],
        units: U_L,
        bare: None,
        normal: n::LDH_U_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "amylase",
        abbr: &[],
        words: &["amylase", "serum amylase"],
        units: U_L,
        bare: None,
        normal: r(25.0, 125.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "lipase",
        abbr: &[],
        words: &["lipase", "serum lipase"],
        units: U_L,
        bare: None,
        normal: r(13.0, 60.0),
        note: "",
        source: NBME,
    },
    Lab {
        id: "hvpg",
        abbr: &["HVPG"],
        words: &[],
        units: MM_HG,
        bare: Some((0.0, 40.0)),
        normal: r(1.0, 5.0),
        note: "",
        source: "Baveno VII (de Franchis et al., J Hepatol 2022)",
    },
    // Blood.
    Lab {
        id: "hb",
        abbr: HB,
        words: HB_W,
        units: G_DL,
        bare: Some((2.0, 25.0)),
        normal: n::HB_G_DL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "hb_si",
        abbr: HB,
        words: HB_W,
        units: G_L,
        bare: None,
        normal: sexes((135.0, 175.0), (120.0, 160.0)),
        note: "",
        source: NBME,
    },
    Lab {
        id: "hct",
        abbr: &["Hct", "HCT", "PCV"],
        words: &["hematocrit", "haematocrit"],
        units: &["%"],
        bare: None,
        normal: n::HCT_PCT,
        note: "",
        source: NBME,
    },
    Lab {
        id: "mcv",
        abbr: &["MCV"],
        words: &["mean corpuscular volume"],
        units: &["fL", "fl", "µm³", "μm3"],
        bare: Some((50.0, 140.0)),
        normal: n::MCV_FL,
        note: "",
        source: NBME,
    },
    Lab {
        id: "platelets",
        abbr: PLT,
        words: PLT_W,
        units: PER_MM3,
        bare: Some((1000.0, 2_000_000.0)),
        normal: r(150_000.0, 400_000.0),
        note: "",
        source: "Haematology book, Lecture 4, p. 62",
    },
    Lab {
        id: "platelets_e9",
        abbr: PLT,
        words: PLT_W,
        units: E9_L,
        bare: Some((1.0, 999.0)),
        normal: n::PLATELETS_10E9_L,
        note: "",
        source: "Haematology book, Lecture 4, p. 62",
    },
    Lab {
        id: "wbc",
        abbr: WBC,
        words: WBC_W,
        units: PER_MM3,
        bare: Some((300.0, 500_000.0)),
        normal: n::WBC_PER_MM3,
        note: "",
        source: NBME,
    },
    Lab {
        id: "wbc_e9",
        abbr: WBC,
        words: WBC_W,
        units: E9_L,
        bare: Some((0.1, 299.0)),
        normal: n::WBC_10E9_L,
        note: "",
        source: NBME,
    },
    Lab {
        id: "ferritin",
        abbr: &[],
        words: &["ferritin", "serum ferritin"],
        units: &["ng/mL", "µg/L", "μg/L", "ug/L"],
        bare: None,
        normal: sexes((20.0, 250.0), (10.0, 120.0)),
        note: "",
        source: NBME,
    },
    Lab {
        id: "esr",
        abbr: &["ESR"],
        words: &[],
        units: &["mm/h", "mm/hr", "mm/hour", "mm in the first hour", "mm/1st hour"],
        bare: None,
        normal: n::ESR_MM_H,
        note: "",
        source: NBME,
    },
    Lab {
        id: "inr",
        abbr: &["INR"],
        words: &[],
        units: &[],
        bare: Some((0.5, 15.0)),
        normal: n::INR,
        note: "",
        source: "Medscape, Lab Values, Normal Adult (2025)",
    },
    Lab {
        id: "pt",
        abbr: &["PT"],
        words: &["prothrombin time"],
        units: &["seconds", "second", "sec", "s"],
        bare: None,
        normal: n::PT_S,
        note: "",
        source: NBME,
    },
    Lab {
        id: "aptt",
        abbr: &["aPTT", "APTT", "PTT"],
        words: &[],
        units: &["seconds", "second", "sec", "s"],
        bare: None,
        normal: r(25.0, 40.0),
        note: "",
        source: NBME,
    },
    // Vital signs.
    Lab {
        id: "sbp",
        abbr: &["SBP"],
        words: &["systolic blood pressure", "systolic BP", "systolic pressure", "systolic"],
        units: MM_HG,
        bare: None,
        normal: n::SBP_MM_HG,
        note: "",
        source: "ACC/AHA 2017 (Whelton et al.)",
    },
];

/// A lab value found in a text. Offsets are byte offsets into the text.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// Where the test's name starts.
    pub name: usize,
    /// The number (and its unit): the part to mark.
    pub start: usize,
    pub end: usize,
    pub lab: &'static Lab,
    pub value: f64,
    /// Low or high when the text says so: an exact value outside the
    /// range, or a cut-off that is all low ("< 135") or all high (">300").
    pub place: Option<Place>,
    /// The number had no unit, so the chip names it.
    pub bare: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Cmp {
    Eq,
    Below,
    Above,
}

/// Comparators and linking words allowed between a name and its number.
/// Longest first where one starts another.
const LINKS: &[(&str, Option<Cmp>)] = &[
    ("<=", Some(Cmp::Below)),
    (">=", Some(Cmp::Above)),
    ("<", Some(Cmp::Below)),
    (">", Some(Cmp::Above)),
    ("≤", Some(Cmp::Below)),
    ("≥", Some(Cmp::Above)),
    ("=", None),
    (":", None),
    ("~", None),
    ("≈", None),
    ("(", None),
    ("of", None),
    ("is", None),
    ("was", None),
    ("level", None),
    ("levels", None),
    ("value", None),
    ("count", None),
    ("concentration", None),
    ("about", None),
    ("around", None),
    ("approximately", None),
    ("below", Some(Cmp::Below)),
    ("under", Some(Cmp::Below)),
    ("less than", Some(Cmp::Below)),
    ("lower than", Some(Cmp::Below)),
    ("above", Some(Cmp::Above)),
    ("over", Some(Cmp::Above)),
    ("more than", Some(Cmp::Above)),
    ("greater than", Some(Cmp::Above)),
    ("higher than", Some(Cmp::Above)),
    ("at least", Some(Cmp::Above)),
    ("exceeding", Some(Cmp::Above)),
    ("أقل من", Some(Cmp::Below)),
    ("اقل من", Some(Cmp::Below)),
    ("تحت", Some(Cmp::Below)),
    ("أكتر من", Some(Cmp::Above)),
    ("اكتر من", Some(Cmp::Above)),
    ("أكثر من", Some(Cmp::Above)),
    ("أعلى من", Some(Cmp::Above)),
    ("فوق", Some(Cmp::Above)),
    ("حوالي", None),
    ("=<", Some(Cmp::Below)),
    ("=>", Some(Cmp::Above)),
];

/// Words just before the name that make the number something else: another
/// fluid, a fraction of the total, or a change.
const NOT_SERUM: &[&str] = &[
    "urine",
    "urinary",
    "ascitic",
    "fluid",
    "pleural",
    "csf",
    "stool",
    "faecal",
    "fecal",
    "sweat",
    "direct",
    "indirect",
    "conjugated",
    "unconjugated",
    "ionized",
    "ionised",
    "free",
    "delta",
    "δ",
    "change",
    "rise",
    "increase",
    "increases",
    "increased",
    "decrease",
    "decreased",
    "drop",
    "fall",
    "reduction",
    "by",
    "raise",
    "raising",
    "lower",
    "lowering",
    "correct",
    "correction",
    "gradient",
    "spot",
    "24-h",
    "24-hour",
    "زيادة",
    "ارتفاع",
    "نزول",
    "انخفاض",
    "بمقدار",
    "تصحيح",
];

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || ('\u{0600}'..='\u{06FF}').contains(&c)
}

/// The text at `at` starts with `s`, by exact case or ignoring it.
fn starts(text: &str, at: usize, s: &str, case: bool) -> bool {
    let t = &text[at..];
    if case {
        t.starts_with(s)
    } else {
        t.len() >= s.len() && t.is_char_boundary(s.len()) && t[..s.len()].eq_ignore_ascii_case(s)
    }
}

/// The few words before `at`, lower-case, nearest first.
fn words_before(text: &str, at: usize, k: usize) -> Vec<String> {
    text[..at]
        .split(|c: char| c.is_whitespace() || ",;()[]".contains(c))
        .filter(|w| !w.is_empty())
        .rev()
        .take(k)
        .map(|w| w.trim_matches(|c: char| c == '.' || c == ':').to_lowercase())
        .collect()
}

/// A comma at `i` that groups thousands: three digits after it, then no
/// fourth ("50,000", not "5,1234" or a list "5, 6").
fn thousands(b: &[u8], i: usize) -> bool {
    b.len() >= i + 4
        && b[i + 1..i + 4].iter().all(u8::is_ascii_digit)
        && b.get(i + 4).is_none_or(|d| !d.is_ascii_digit())
}

/// A number at `at`: digits, thousands commas, a decimal point. Returns the
/// value and where it ends.
fn number(text: &str, at: usize) -> Option<(f64, usize)> {
    let b = text.as_bytes();
    let mut i = at;
    let mut digits = String::new();
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_digit() {
            digits.push(c as char);
            i += 1;
        } else if c == b',' && thousands(b, i) && !digits.is_empty() && !digits.contains('.') {
            i += 1;
        } else if c == b'.'
            && !digits.is_empty()
            && !digits.contains('.')
            && b.get(i + 1).is_some_and(u8::is_ascii_digit)
        {
            digits.push('.');
            i += 1;
        } else {
            break;
        }
    }
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok().map(|v| (v, i))
}

/// A unit spelling folded for comparing: no spaces, lower case, one micro
/// sign, superscripts as digits, `x`/`*` as `×`.
fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != '^')
        .map(|c| match c {
            'μ' => 'µ',
            '⁰' => '0',
            '¹' => '1',
            '²' => '2',
            '³' => '3',
            '⁹' => '9',
            'x' | 'X' | '*' => '×',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// The unit right after a number, if `units` holds it: the byte end of the
/// longest spelling that matches.
fn unit_at(text: &str, at: usize, units: &[&str]) -> Option<usize> {
    let rest = &text[at..];
    let mut best: Option<usize> = None;
    for u in units {
        let want = fold(u);
        // Walk the text, folding as we go, until the folded text is as long.
        let mut got = String::new();
        let mut end = None;
        for (i, c) in rest.char_indices() {
            if got.len() >= want.len() {
                end = Some(i);
                break;
            }
            // A space may sit between the number and its unit, or inside one
            // ("mm Hg", "× 10⁹/L"), but not a line break.
            if c == '\n' {
                break;
            }
            got.push_str(&fold(&c.to_string()));
            if !want.starts_with(&got) {
                break;
            }
            end = Some(i + c.len_utf8());
        }
        let Some(e) = end else { continue };
        if got != want {
            continue;
        }
        // The unit ends there: "mg/dL" but not "mg/dLx", "s" but not "sec…".
        if rest[e..].chars().next().is_some_and(|c| c.is_alphanumeric()) {
            continue;
        }
        if best.is_none_or(|b| e > b) {
            best = Some(e);
        }
    }
    best.map(|e| at + e)
}

fn skip_spaces(text: &str, mut at: usize) -> usize {
    while let Some(c) = text[at..].chars().next() {
        if c == ' ' || c == '\u{a0}' || c == '\u{2009}' || c == '\u{200A}' {
            at += c.len_utf8();
        } else {
            break;
        }
    }
    at
}

/// The name of a test at `at` (with its case where it is an abbreviation):
/// the labs it may be and where the name ends.
fn name_at(text: &str, at: usize) -> Option<(usize, Vec<&'static Lab>)> {
    let mut best: Option<(usize, Vec<&'static Lab>)> = None;
    for lab in LABS {
        let spellings = lab.abbr.iter().map(|s| (s, true)).chain(lab.words.iter().map(|s| (s, false)));
        for (s, case) in spellings {
            if !starts(text, at, s, case) {
                continue;
            }
            let end = at + s.len();
            // The name ends there: "Na" but not "NaCl", "Hb" but not "HbA1c".
            let next = text[end..].chars().next();
            if next.is_some_and(|c| {
                c.is_alphanumeric()
                    || (c == '-'
                        && s.ends_with(char::is_alphabetic)
                        && text[end + 1..].starts_with(char::is_alphabetic))
            }) {
                continue;
            }
            match &mut best {
                Some((e, labs)) if *e == end => {
                    if !labs.iter().any(|l| std::ptr::eq(*l, lab)) {
                        labs.push(lab);
                    }
                }
                Some((e, _)) if *e > end => {}
                _ => best = Some((end, vec![lab])),
            }
        }
    }
    best
}

/// Lab values in a text, in order.
pub fn find(text: &str) -> Vec<Hit> {
    let mut out = Vec::new();
    let mut at = 0;
    let mut prev: Option<char> = None;
    while at < text.len() {
        let c = text[at..].chars().next().unwrap_or(' ');
        let starts_word = !prev.is_some_and(|p| is_word(p) || p == '.' || p == '-' || p == '/');
        prev = Some(c);
        if starts_word
            && c.is_alphabetic()
            && let Some(hit) = hit_at(text, at)
        {
            at = hit.end;
            prev = text[..at].chars().next_back();
            out.push(hit);
            continue;
        }
        at += c.len_utf8();
    }
    out
}

fn hit_at(text: &str, at: usize) -> Option<Hit> {
    let (mut i, labs) = name_at(text, at)?;
    let before = words_before(text, at, 3);
    if before.iter().any(|w| NOT_SERUM.contains(&w.as_str())) {
        return None;
    }
    // A closing bracket or a written sign after the name: "(Na+)", "K⁺".
    while let Some(c) = text[i..].chars().next() {
        if ")⁺⁻".contains(c) {
            i += c.len_utf8();
        } else {
            break;
        }
    }
    // Up to three links: "of <", "level is about".
    let mut cmp = Cmp::Eq;
    for _ in 0..4 {
        i = skip_spaces(text, i);
        if text[i..].starts_with(|c: char| c.is_ascii_digit()) {
            break;
        }
        let link = LINKS.iter().filter(|(w, _)| {
            let case = !w.is_ascii() || !w.chars().all(|c| c.is_ascii_alphabetic() || c == ' ');
            starts(text, i, w, case)
                && (!w.ends_with(|c: char| c.is_alphabetic()) || !text[i + w.len()..].starts_with(is_word))
        });
        let (w, c) = link.max_by_key(|(w, _)| w.len())?;
        if let Some(c) = c {
            cmp = *c;
        }
        i += w.len();
    }
    i = skip_spaces(text, i);
    let start = i;
    let (value, mut end) = number(text, i)?;
    // A range: "7–8", "7-8", "7 to 8".
    let mut high_end = value;
    {
        let j = skip_spaces(text, end);
        let dash = ["–", "-", "—", "to "].iter().find(|d| text[j..].starts_with(**d));
        if let Some(d) = dash {
            let k = skip_spaces(text, j + d.len());
            if let Some((v2, e2)) = number(text, k)
                && v2 > value
            {
                high_end = v2;
                end = e2;
            }
        }
    }
    let u = skip_spaces(text, end);
    let tail = &text[u..];
    // A multiple ("3× ULN", "2 times"), a fold or a percentage of something.
    let multiple = tail.starts_with(['×', 'x', 'X', '*'])
        && !fold(&tail.chars().take(8).collect::<String>()).starts_with("×10");
    if multiple || tail.starts_with("times") || tail.starts_with("-fold") || tail.starts_with("fold") {
        return None;
    }
    let mut found: Option<(&'static Lab, usize, bool)> = None;
    for lab in &labs {
        if let Some(e) = unit_at(text, end, lab.units) {
            found = Some((lab, e, false));
            break;
        }
    }
    if found.is_none() {
        // No known unit: a bare number, if nothing unit-like follows.
        let unitish = tail.starts_with(|c: char| c.is_ascii_alphabetic() || "%/×µμ".contains(c))
            && !tail.starts_with(|c: char| c.is_ascii_alphabetic())
            || tail.split(|c: char| !c.is_ascii_alphanumeric() && c != '/').next().is_some_and(|w| {
                w.contains('/')
                    || ["mg", "g", "mmol", "mEq", "U", "IU", "mL", "ml", "kg", "mm", "fL"].contains(&w)
            });
        if !unitish {
            found = labs
                .iter()
                .find(|l| l.bare.is_some_and(|(lo, hi)| value >= lo && high_end <= hi))
                .map(|l| (*l, end, !l.units.is_empty()));
        }
    }
    let (lab, end, bare) = found?;
    let (low, high) = lab.normal.bounds(None);
    let place = match cmp {
        Cmp::Eq => {
            let a = lab.normal.place(value, None);
            let b = lab.normal.place(high_end, None);
            (a == b && a != Place::Normal).then_some(a)
        }
        Cmp::Below => (value <= low).then_some(Place::Low),
        Cmp::Above => (value >= high && high.is_finite()).then_some(Place::High),
    };
    Some(Hit { name: at, start, end, lab, value, place, bare })
}

/// A number as the chip shows it: no ".0", thousands with commas.
fn show(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() >= 10_000.0 {
        let s = format!("{}", v as i64);
        let mut out = String::new();
        for (k, c) in s.chars().enumerate() {
            if k > 0 && (s.len() - k) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        out
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn span(low: f64, high: f64, below: bool) -> String {
    match (low.is_finite(), high.is_finite()) {
        (true, true) => format!("{}–{}", show(low), show(high)),
        (false, true) if below => format!("<{}", show(high)),
        (false, true) => format!("≤{}", show(high)),
        (true, false) => format!("≥{}", show(low)),
        _ => String::new(),
    }
}

impl Lab {
    /// The unit as shown ("mmol/L"); empty for pH and INR.
    pub fn unit(&self) -> &'static str {
        self.units.first().copied().unwrap_or("")
    }

    /// The range in a few characters: "135–145", "♂ 13.5–17.5 · ♀ 12–16",
    /// "<150", "≥90".
    pub fn range(&self) -> String {
        let n = &self.normal;
        let men = span(n.low, n.high, n.below);
        match n.female {
            Some((fl, fh)) => format!("♂ {men} · ♀ {}", span(fl, fh, n.below)),
            None => men,
        }
    }

    /// The whole line for a tap or hover: the range, its unit, what it is
    /// for, and the source.
    pub fn peek(&self) -> String {
        let unit = if self.unit().is_empty() { String::new() } else { format!(" {}", self.unit()) };
        let note = if self.note.is_empty() { String::new() } else { format!(" ({})", self.note) };
        format!("Adult normal{note}: {}{unit}. Source: {}. Labs differ a little.", self.range(), self.source)
    }
}

/// A text cut into plain pieces and lab values, for drawing. Each test's
/// chip is shown once per text: `chip` is false on its later values.
#[derive(Debug, Clone)]
pub enum Piece<'a> {
    Text(&'a str),
    Value { text: &'a str, hit: Hit, chip: bool },
}

pub fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let mut at = 0;
    for hit in find(text) {
        if hit.start > at {
            out.push(Piece::Text(&text[at..hit.start]));
        }
        let test = base(hit.lab.id);
        let chip = !seen.contains(&test);
        seen.push(test);
        out.push(Piece::Value { text: &text[hit.start..hit.end], hit, chip });
        at = hit.end;
    }
    if at < text.len() {
        out.push(Piece::Text(&text[at..]));
    }
    out
}

/// The lab with this id.
pub fn get(id: &str) -> Option<&'static Lab> {
    LABS.iter().find(|l| l.id == id)
}

/// The test an id is for, whatever the unit: `hb_si` → `hb`.
pub fn base(id: &str) -> &str {
    id.strip_suffix("_si").or_else(|| id.strip_suffix("_e9")).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Hit {
        let h = find(text);
        assert_eq!(h.len(), 1, "{text}: {h:?}");
        h[0]
    }

    fn none(text: &str) {
        let h = find(text);
        assert!(h.is_empty(), "{text}: {h:?}");
    }

    #[test]
    fn finds_values_with_units() {
        let h = one("Serum Na 128 mmol/L with confusion.");
        assert_eq!((h.lab.id, h.value, h.place), ("sodium", 128.0, Some(Place::Low)));
        assert_eq!(&"Serum Na 128 mmol/L with"[h.start..h.end], "128 mmol/L");
        assert_eq!(find("Hb 9 g/dL and MCV 70 fL").len(), 2);
        assert_eq!(one("bilirubin >300 µmol/L").place, Some(Place::High));
        assert_eq!(one("bilirubin >300 µmol/L").lab.id, "bilirubin_si");
        assert_eq!(one("creatinine of 4.2 mg/dL").place, Some(Place::High));
        assert_eq!(one("platelets <50 × 10⁹/L").lab.id, "platelets_e9");
        assert_eq!(one("platelets 50,000/mm³").value, 50_000.0);
        assert_eq!(one("HVPG ≥10 mmHg").place, Some(Place::High));
        assert_eq!(one("HbA1c 6.5%").lab.id, "hba1c");
        assert_eq!(one("fasting plasma glucose ≥126 mg/dL").place, Some(Place::High));
        assert_eq!(one("FPG ≥7.0 mmol/L").lab.id, "glucose_si");
    }

    #[test]
    fn bare_numbers_count_where_the_unit_is_clear() {
        let h = one("الـ Na أقل من 135 يبقى hyponatraemia");
        assert_eq!((h.lab.id, h.place, h.bare), ("sodium", Some(Place::Low), true));
        assert_eq!(one("eGFR <30").place, Some(Place::Low));
        assert_eq!(one("pH 7.1").place, Some(Place::Low));
        assert!(!one("pH 7.1").bare, "pH has no unit to add");
        assert_eq!(one("INR >1.5").place, Some(Place::High));
        assert_eq!(one("Hb <7").lab.id, "hb");
        let case = "fatigue, vomiting, Hb 5.8, WBC 2.6 and creatinine 7.3. Anti-dsDNA";
        let ids: Vec<_> = find(case).iter().map(|h| (h.lab.id, h.place)).collect();
        assert_eq!(
            ids,
            [("hb", Some(Place::Low)), ("wbc_e9", Some(Place::Low)), ("creatinine", Some(Place::High))]
        );
        // A threshold that isn't all low or all high gets no colour.
        assert_eq!(one("transfuse if Hb >7 g/dL").place, None);
        assert_eq!(one("Hb 7–8 g/dL").place, Some(Place::Low));
    }

    #[test]
    fn leaves_other_numbers_alone() {
        none("a rise in creatinine of 0.3 mg/dL within 48 h");
        none("ascitic albumin 1.1 g/dL");
        none("urine sodium <20 mmol/L");
        none("ALT 3× ULN");
        none("ALP > 2× normal");
        none("vitamin K 10 mg IV");
        none("HbA1c is the best test"); // no number
        none("NaCl 0.9%");
        none("K 10 mg"); // a unit that isn't potassium's
        none("SBP 250 PMN/mm³"); // spontaneous bacterial peritonitis
        none("PT prolonged by 4 seconds");
    }

    #[test]
    fn chips_show_ranges_once() {
        let labs = |id| LABS.iter().find(|l| l.id == id).unwrap();
        assert_eq!(labs("sodium").range(), "135–145");
        assert_eq!(labs("hb").range(), "♂ 13.5–17.5 · ♀ 12–16");
        assert_eq!(labs("triglycerides").range(), "<150");
        assert_eq!(labs("egfr").range(), "≥90");
        assert_eq!(labs("lactate").range(), "≤2");
        assert_eq!(labs("platelets").range(), "150,000–400,000");
        assert!(labs("sodium").peek().contains("Nephrology book"));
        let p = pieces("Hb 9 g/dL, later Hb 7 g/dL; Na 120 mmol/L.");
        let chips: Vec<bool> = p
            .iter()
            .filter_map(|p| if let Piece::Value { chip, .. } = p { Some(*chip) } else { None })
            .collect();
        assert_eq!(chips, [true, false, true]);
    }

    /// Every lab value in the content, for reading by eye:
    /// `cargo test -p batna-core labs::tests::dump -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn dump() {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "yaml") {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../content")), &mut files);
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            for line in text.lines() {
                for h in find(line) {
                    let from = line.floor_char_boundary(h.name);
                    println!("{}\t{:?}\t{}", h.lab.id, h.place, &line[from..h.end]);
                }
            }
        }
    }

    #[test]
    fn every_lab_has_a_source_and_a_sane_range() {
        for l in LABS {
            assert!(!l.source.is_empty(), "{}", l.id);
            let (lo, hi) = l.normal.bounds(None);
            assert!(lo < hi, "{}", l.id);
        }
    }
}
