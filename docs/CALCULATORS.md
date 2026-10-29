# Calculators

Generated from `crates/core/src/calc` (run `cargo test -p batna-core -- --ignored print_reference`). A `calc` item names one of these in `calculator:` and gives its `inputs` by key (a key may drop its unit suffix: `alt` for `alt_u_l`). `just validate` computes every item, so a typo, a missing input or an `expect` that disagrees fails the check. `expect` may be the band id (`child_pugh.b`) or text that contains the result (`Child-Pugh B (8 points)`, `Stage C`).

## `child_pugh`: Child-Pugh

Source: Book Lecture 1, Table 2 (p. 14); INR bands: Pugh 1973 as used by AASLD

| input | kind | |
|---|---|---|
| `encephalopathy` | `none`, `grade_1_2`, `grade_3_4` | Encephalopathy |
| `ascites` | `absent`, `mild_moderate`, `severe` | Ascites |
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |
| `albumin_g_dl` | number, g/dL | Albumin |
| `pt_prolonged_s` | number, s | Prothrombin time, seconds prolonged; optional |
| `inr` | number | INR; optional |

## `meld`: MELD

Source: Formula: UNOS/OPTN MELD (Kamath 2001). The book (Lecture 1, p. 14) names MELD without a formula.

| input | kind | |
|---|---|---|
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |
| `inr` | number | INR |
| `creatinine_mg_dl` | number, mg/dL | Creatinine |
| `dialysis` | true / false (default false) | Dialysis twice in the past week |

## `meld_na`: MELD-Na

Source: Formula: UNOS/OPTN MELD-Na 2016 (Kim 2008). The book (Lecture 1, p. 14) names MELD-Na without a formula.

| input | kind | |
|---|---|---|
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |
| `inr` | number | INR |
| `creatinine_mg_dl` | number, mg/dL | Creatinine |
| `dialysis` | true / false (default false) | Dialysis twice in the past week |
| `sodium_mmol_l` | number, mmol/L | Sodium |

## `albi`: ALBI grade

Source: Johnson 2015; book Lecture 18, p. 363

| input | kind | |
|---|---|---|
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |
| `albumin_g_dl` | number, g/dL | Albumin |

## `fib4`: FIB-4

Source: Sterling 2006; book Lecture 6, p. 102

| input | kind | |
|---|---|---|
| `age_years` | number, years | Age |
| `ast_u_l` | number, U/L | AST |
| `alt_u_l` | number, U/L | ALT |
| `platelets_10e9_l` | number, ×10⁹/L | Platelets |

## `apri`: APRI

Source: Book Lecture 2, p. 34; Wai 2003; WHO HCV guidelines 2016

| input | kind | |
|---|---|---|
| `ast_u_l` | number, U/L | AST |
| `ast_uln_u_l` | number, U/L | AST upper limit of normal |
| `platelets_10e9_l` | number, ×10⁹/L | Platelets |

## `r_ratio`: R ratio (DILI pattern)

Source: Book Lecture 8, p. 129; EASL DILI 2019; Kullak-Ublick 2017

| input | kind | |
|---|---|---|
| `alt_u_l` | number, U/L | ALT |
| `alt_uln_u_l` | number, U/L | ALT upper limit of normal |
| `alp_u_l` | number, U/L | ALP |
| `alp_uln_u_l` | number, U/L | ALP upper limit of normal |

## `maddrey`: Maddrey discriminant function

Source: Maddrey 1978, Carithers 1989; AASLD 2019

| input | kind | |
|---|---|---|
| `pt_s` | number, s | Prothrombin time (patient) |
| `pt_control_s` | number, s | Prothrombin time (control) |
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |

## `glasgow_ah`: Glasgow alcoholic hepatitis score

Source: Forrest 2005 (Gut)

| input | kind | |
|---|---|---|
| `age_years` | number, years | Age |
| `wbc_10e9_l` | number, ×10⁹/L | White cells |
| `urea_mmol_l` | number, mmol/L | Urea |
| `inr` | number | INR or PT ratio |
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin |

## `saag`: SAAG

Source: Book Lecture 9, p. 150–151

| input | kind | |
|---|---|---|
| `serum_albumin_g_dl` | number, g/dL | Serum albumin |
| `ascitic_albumin_g_dl` | number, g/dL | Ascitic albumin |
| `ascitic_protein_g_dl` | number, g/dL | Ascitic total protein; optional |

## `ascitic_pmn`: Ascitic PMN / SBP

Source: Book Lecture 9, p. 167–168

| input | kind | |
|---|---|---|
| `pmn_per_mm3` | number, /mm³ | Ascitic PMN (neutrophils) |
| `culture` | `pending`, `negative`, `monomicrobial`, `polymicrobial` | Ascitic culture |
| `protein_g_dl` | number, g/dL | Ascitic total protein; optional |
| `glucose_mg_dl` | number, mg/dL | Ascitic glucose; optional |
| `ldh_above_serum` | true / false (default false) | Ascitic LDH above serum LDH |

## `bclc`: BCLC staging

Source: Book Lecture 18, p. 362 and 364–373 (BCLC 2018)

| input | kind | |
|---|---|---|
| `performance_status` | number | ECOG performance status |
| `child_pugh_points` | number | Child-Pugh points |
| `ascites` | true / false (default false) | Ascites present |
| `nodules` | number | Number of nodules |
| `largest_cm` | number, cm | Largest nodule |
| `vascular_invasion` | true / false (default false) | Macrovascular invasion |
| `extrahepatic` | true / false (default false) | Extrahepatic spread |

## `kings_college`: King's College criteria

Source: Book Lecture 10, p. 187 (O'Grady 1989; Bernal 2002)

| input | kind | |
|---|---|---|
| `cause` | `acetaminophen`, `wilson_budd_chiari`, `indeterminate_or_drug`, `other` | Cause of ALF |
| `inr` | number | INR |
| `ph` | number | Arterial pH (after resuscitation); optional |
| `lactate_mmol_l` | number, mmol/L | Lactate; optional |
| `creatinine_mg_dl` | number, mg/dL | Creatinine; optional |
| `he_stage` | number | Encephalopathy stage; optional |
| `bilirubin_mg_dl` | number, mg/dL | Bilirubin; optional |
| `age_years` | number, years | Age; optional |
| `jaundice_to_he_days` | number, days | Days from jaundice to encephalopathy; optional |
| `over_24h` | true / false (default false) | More than 24 h since ingestion |

## `clichy`: Clichy criteria

Source: Book Lecture 10, p. 187 (Bernuau 1986)

| input | kind | |
|---|---|---|
| `he_stage` | number | Encephalopathy stage |
| `factor_v_pct` | number, % of normal | Factor V |
| `age_years` | number, years | Age |

## `ranson`: Ranson's criteria

Source: Ranson 1974/1979; book Lecture 15, p. 288 (mortality bands)

| input | kind | |
|---|---|---|
| `gallstone` | true / false (default false) | Gallstone pancreatitis |
| `age_years` | number, years | Age |
| `wbc_per_mm3` | number, /mm³ | White cells |
| `glucose_mg_dl` | number, mg/dL | Glucose |
| `ldh_iu_l` | number, IU/L | LDH |
| `ast_iu_l` | number, IU/L | AST |
| `hct_fall_points` | number, points | Haematocrit fall in 48 h; optional |
| `bun_rise_mg_dl` | number, mg/dL | BUN rise in 48 h; optional |
| `calcium_mg_dl` | number, mg/dL | Calcium at 48 h; optional |
| `pao2_mm_hg` | number, mmHg | PaO₂ at 48 h; optional |
| `base_deficit_meq_l` | number, mEq/L | Base deficit at 48 h; optional |
| `fluid_sequestration_l` | number, L | Fluid sequestration in 48 h; optional |

## `bisap`: BISAP

Source: Wu 2008; book Lecture 15, p. 288

| input | kind | |
|---|---|---|
| `bun_mg_dl` | number, mg/dL | BUN |
| `impaired_mental_status` | true / false (default false) | Impaired mental status |
| `sirs` | true / false (default false) | SIRS (2 or more criteria) |
| `age_years` | number, years | Age |
| `pleural_effusion` | true / false (default false) | Pleural effusion |

## `glasgow_blatchford`: Glasgow-Blatchford score

Source: Blatchford 2000; book Gastroenterology Lecture 11, p. 309

| input | kind | |
|---|---|---|
| `bun_mg_dl` | number, mg/dL | BUN |
| `sex` | `male`, `female` | Sex |
| `hb_g_dl` | number, g/dL | Haemoglobin |
| `sbp_mmhg` | number, mmHg | Systolic blood pressure |
| `hr_bpm` | number, beats/min | Heart rate |
| `melena` | true / false (default false) | Melena |
| `syncope` | true / false (default false) | Syncope |
| `hepatic_disease` | true / false (default false) | Hepatic disease |
| `cardiac_failure` | true / false (default false) | Cardiac failure |

## `anion_gap`: Anion gap

Source: Book Nephrology Lecture 1, pp. 5, 9, 14

| input | kind | |
|---|---|---|
| `na_meq_l` | number, mEq/L | Sodium |
| `cl_meq_l` | number, mEq/L | Chloride |
| `hco3_meq_l` | number, mEq/L | Bicarbonate (HCO3) |
| `albumin_g_dl` | number, g/dL | Albumin; optional |

## `delta_ratio`: Delta ratio (gap : gap)

Source: Book Nephrology Lecture 1, pp. 12, 14

| input | kind | |
|---|---|---|
| `anion_gap_meq_l` | number, mEq/L | Anion gap |
| `hco3_meq_l` | number, mEq/L | Bicarbonate (HCO3) |

## `expected_compensation`: Expected compensation

Source: Book Nephrology Lecture 1, pp. 13–14

| input | kind | |
|---|---|---|
| `disorder` | `metabolic_acidosis`, `metabolic_alkalosis`, `respiratory_acidosis_acute`, `respiratory_acidosis_chronic`, `respiratory_alkalosis_acute`, `respiratory_alkalosis_chronic` | Primary disorder |
| `hco3_meq_l` | number, mEq/L | Bicarbonate (HCO3) |
| `pco2_mm_hg` | number, mmHg | pCO2 |

## `osmolal_gap`: Osmolal gap

Source: Book Nephrology Lecture 1, p. 20

| input | kind | |
|---|---|---|
| `measured_osm` | number, mOsm/kg | Measured osmolality |
| `na_meq_l` | number, mEq/L | Sodium |
| `glucose_mg_dl` | number, mg/dL | Glucose |
| `bun_mg_dl` | number, mg/dL | BUN |

## `cockcroft_gault`: Cockcroft-Gault creatinine clearance

Source: Book Geriatrics Lecture 2, p. 30; Lecture 3, p. 42

| input | kind | |
|---|---|---|
| `age_years` | number, years | Age |
| `weight_kg` | number, kg | Weight |
| `creatinine_mg_dl` | number, mg/dL | Serum creatinine |
| `sex` | `male`, `female` | Sex |

## `acr_eular_2010_ra`: ACR/EULAR 2010 RA criteria

Source: Book Rheumatology Lecture 16, p. 246

| input | kind | |
|---|---|---|
| `large_joints` | number | Medium-large joints involved |
| `small_joints` | number | Small joints involved |
| `serology` | `negative`, `low`, `high` | RF / ACPA |
| `synovitis_6_weeks` | true / false (default false) | Synovitis 6 weeks or longer |
| `abnormal_crp_esr` | true / false (default false) | Abnormal CRP or ESR |

## `corrected_sodium`: Corrected sodium (hyperglycaemia)

Source: Book Diabetology Lecture 15, p. 189

| input | kind | |
|---|---|---|
| `na_meq_l` | number, mEq/L | Sodium |
| `glucose_mg_dl` | number, mg/dL | Glucose |

## `effective_osmolality`: Serum osmolality (HHS)

Source: Book Diabetology Lecture 15, pp. 189, 192

| input | kind | |
|---|---|---|
| `na_meq_l` | number, mEq/L | Sodium |
| `glucose_mg_dl` | number, mg/dL | Glucose |
| `urea_mmol_l` | number, mmol/L | Urea; optional |

## `incidence`: Incidence rate

Source: Research L02: measures of disease frequency, slide 25

| input | kind | |
|---|---|---|
| `new_cases` | number | New cases during the period |
| `population_at_risk` | number | Population at risk (mid-period) |
| `per` | `100`, `1000`, `10000`, `100000` | Expressed |

## `prevalence`: Prevalence

Source: Research L02: measures of disease frequency, slide 31

| input | kind | |
|---|---|---|
| `existing_cases` | number | Existing (old + new) cases |
| `population` | number | Total population (mid-point or mid-period) |
| `per` | `100`, `1000`, `10000`, `100000` | Expressed |

## `prevalence_id`: Prevalence = Incidence × Duration

Source: Research L02: measures of disease frequency, slide 37

| input | kind | |
|---|---|---|
| `incidence` | number, per 1,000 per year | Incidence |
| `duration_years` | number, years | Mean duration of disease |

## `attack_rate`: Attack rate

Source: Research L02: measures of disease frequency, slides 26–29

| input | kind | |
|---|---|---|
| `cases` | number | Cases during the outbreak |
| `at_risk` | number | Population at risk |

## `sar`: Secondary attack rate

Source: Research L02: measures of disease frequency, slides 46–49

| input | kind | |
|---|---|---|
| `secondary_cases` | number | Secondary cases (within one incubation period) |
| `contacts` | number | Contacts (household members, all) |
| `primary_cases` | number | Primary cases among them |

## `cfr`: Case fatality rate

Source: Research L02: measures of disease frequency, slide 60

| input | kind | |
|---|---|---|
| `deaths` | number | Deaths from the disease |
| `cases` | number | People with the disease |

## `pmr`: Proportionate mortality ratio

Source: Research L02: measures of disease frequency, slide 59

| input | kind | |
|---|---|---|
| `cause_deaths` | number | Deaths from the cause |
| `all_deaths` | number | Deaths from all causes |

## `imr`: Infant mortality rate

Source: Research L02: measures of disease frequency, slide 63

| input | kind | |
|---|---|---|
| `infant_deaths` | number | Deaths under one year |
| `live_births` | number | Live births, same year |

## `nmr`: Neonatal mortality rate

Source: Research L02: measures of disease frequency, slide 64

| input | kind | |
|---|---|---|
| `neonatal_deaths` | number | Deaths in the first 28 days |
| `live_births` | number | Live births, same year |

## `relative_risk`: Relative risk (cohort)

Source: Research L05: analytic studies, slides 50–53

| input | kind | |
|---|---|---|
| `a` | number | a |
| `b` | number | b |
| `c` | number | c |
| `d` | number | d |

## `odds_ratio`: Odds ratio (case-control)

Source: Research L05: analytic studies, slides 32–34

| input | kind | |
|---|---|---|
| `a` | number | a |
| `b` | number | b |
| `c` | number | c |
| `d` | number | d |

## `attributable_risk`: Attributable risk

Source: Research L05: analytic studies, slide 51

| input | kind | |
|---|---|---|
| `incidence_exposed` | number, per 1,000 | Incidence among exposed (Ie) |
| `incidence_unexposed` | number, per 1,000 | Incidence among non-exposed (Io) |

## `nnt`: NNT / NNH

Source: Research L06–L07: experimental studies (NNT = 1/ARR)

| input | kind | |
|---|---|---|
| `control_event_rate` | number, % | Event rate, control group (CER) |
| `experimental_event_rate` | number, % | Event rate, treated group (EER) |

## `diagnostic_accuracy`: Diagnostic accuracy (2×2)

Source: Research L14: measures of diagnostic accuracy

| input | kind | |
|---|---|---|
| `tp` | number | True positives (a) |
| `fp` | number | False positives (b) |
| `fn` | number | False negatives (c) |
| `tn` | number | True negatives (d) |
| `measure` | `sensitivity`, `specificity`, `ppv`, `npv`, `accuracy`, `lr_positive`, `lr_negative` | Answer with |

## `z_score`: Z-score

Source: Research L16: normal distribution and Z-score

| input | kind | |
|---|---|---|
| `value` | number | Observation (x) |
| `mean` | number | Mean (μ) |
| `sd` | number | Standard deviation (σ) |
