//! Liver, ascites, tumour and acute liver failure calculators.

use super::{Table, ins};

pub fn tables() -> Vec<Table> {
    vec![
        Table::new("child_pugh", &["child_pugh.a", "child_pugh.b", "child_pugh.c"])
            .paper(
                "Pugh 1973 / book p. 14: HE none 1, mild ascites 2, bili 2.5 → 2, albumin 3.0 → 2, PT +5 s → 2 = 9",
                ins![encephalopathy: "none", ascites: "mild", bilirubin_mg_dl: 2.5, albumin_g_dl: 3.0, pt_prolonged_s: 5.0],
                "child_pugh.b",
            )
            .score(9.0, 0.0)
            .paper(
                "all worst: HE 3–4, tense ascites, bili 4, albumin 2.5, PT +7 = 15",
                ins![encephalopathy: "grade_3_4", ascites: "tense", bilirubin_mg_dl: 4.0, albumin_g_dl: 2.5, pt_prolonged_s: 7.0],
                "child_pugh.c",
            )
            .score(15.0, 0.0)
            // 6 vs 7 points: bilirubin < 2 is 1 point, 2.0 is 2 points.
            .cut(
                "bilirubin 1.9 / 2.0 mg/dL (book: < 2 = 1 point)",
                ins![encephalopathy: "none", ascites: "absent", albumin_g_dl: 3.0, pt_prolonged_s: 1.0],
                "bilirubin_mg_dl",
                (1.9, "child_pugh.a"),
                (2.0, "child_pugh.b"),
            )
            .cut(
                "bilirubin 3.0 / 3.1 mg/dL (book: 2–3 = 2 points, > 3 = 3)",
                ins![encephalopathy: "none", ascites: "mild", albumin_g_dl: 2.8, pt_prolonged_s: 4.0],
                "bilirubin_mg_dl",
                (3.0, "child_pugh.b"),
                (3.1, "child_pugh.c"),
            )
            .cut(
                "albumin 3.6 / 3.5 g/dL (book: > 3.5 = 1 point)",
                ins![encephalopathy: "none", ascites: "absent", bilirubin_mg_dl: 1.0, pt_prolonged_s: 2.0],
                "albumin_g_dl",
                (3.6, "child_pugh.a"),
                (3.5, "child_pugh.a"),
            )
            .cut(
                "albumin 2.8 / 2.7 g/dL (book: 2.8–3.5 = 2, < 2.8 = 3)",
                ins![encephalopathy: "none", ascites: "mild", bilirubin_mg_dl: 2.0, pt_prolonged_s: 4.0],
                "albumin_g_dl",
                (2.8, "child_pugh.b"),
                (2.7, "child_pugh.c"),
            )
            .cut(
                "PT prolonged 3.9 / 4.0 s (book: < 4 = 1, 4–6 = 2)",
                ins![encephalopathy: "none", ascites: "absent", bilirubin_mg_dl: 1.0, albumin_g_dl: 3.0],
                "pt_prolonged_s",
                (3.9, "child_pugh.a"),
                (4.0, "child_pugh.b"),
            )
            .cut(
                "PT prolonged 6.0 / 6.5 s (book: > 6 = 3)",
                ins![encephalopathy: "none", ascites: "mild", bilirubin_mg_dl: 2.0, albumin_g_dl: 3.0],
                "pt_prolonged_s",
                (6.0, "child_pugh.b"),
                (6.5, "child_pugh.c"),
            )
            .cut(
                "INR 1.6 / 1.7 when PT is empty (Pugh/AASLD: < 1.7 = 1)",
                ins![encephalopathy: "none", ascites: "absent", bilirubin_mg_dl: 1.0, albumin_g_dl: 3.0],
                "inr",
                (1.6, "child_pugh.a"),
                (1.7, "child_pugh.b"),
            )
            .cut(
                "INR 2.3 / 2.4 (> 2.3 = 3)",
                ins![encephalopathy: "none", ascites: "mild", bilirubin_mg_dl: 2.0, albumin_g_dl: 3.0],
                "inr",
                (2.3, "child_pugh.b"),
                (2.4, "child_pugh.c"),
            )
            .paper(
                "PT wins over INR when both are entered: PT +2 (1 point) with INR 3 (would be 3)",
                ins![encephalopathy: "none", ascites: "absent", bilirubin_mg_dl: 1.0, albumin_g_dl: 4.0, pt_prolonged_s: 2.0, inr: 3.0],
                "child_pugh.a",
            )
            .score(5.0, 0.0),
        Table::new("meld", &["meld.lt10", "meld.10_14", "meld.15_30", "meld.gt30"])
            .paper(
                "Kamath 2001: 10 × (0.957 ln 1.2 + 0.378 ln 2 + 1.120 ln 1.5 + 0.643) = 15.34 → 15",
                ins![bilirubin_mg_dl: 2.0, inr: 1.5, creatinine_mg_dl: 1.2],
                "meld.15_30",
            )
            .score(15.0, 0.0)
            .paper(
                "OPTN: values below 1 count as 1, so normal labs give 6.43 → 6",
                ins![bilirubin_mg_dl: 0.5, inr: 0.9, creatinine_mg_dl: 0.6],
                "meld.lt10",
            )
            .score(6.0, 0.0)
            .paper(
                "OPTN: dialysis twice in a week sets creatinine to 4: 6.43 + 9.57 ln 4 = 19.7 → 20",
                ins![bilirubin_mg_dl: 1.0, inr: 1.0, creatinine_mg_dl: 1.0, dialysis: true],
                "meld.15_30",
            )
            .score(20.0, 0.0)
            .noted("Dialysis")
            .paper(
                "OPTN: creatinine above 4 is capped at 4 (same 20 as creatinine 4)",
                ins![bilirubin_mg_dl: 1.0, inr: 1.0, creatinine_mg_dl: 6.0],
                "meld.15_30",
            )
            .score(20.0, 0.0)
            .noted("capped at 4")
            .paper(
                "OPTN: the score is capped at 40",
                ins![bilirubin_mg_dl: 40.0, inr: 8.0, creatinine_mg_dl: 4.0],
                "meld.gt30",
            )
            .score(40.0, 0.0)
            .noted("Capped at 40")
            .cut(
                "bilirubin 2.2 / 2.3: 9.41 → 9 and 9.58 → 10 (AASLD evaluation from MELD 10)",
                ins![inr: 1.0, creatinine_mg_dl: 1.0],
                "bilirubin_mg_dl",
                (2.2, "meld.lt10"),
                (2.3, "meld.10_14"),
            )
            .cut(
                "bilirubin 8.4 / 8.5: 14.48 → 14 and 14.52 → 15 (book p. 189: transplant at 15)",
                ins![inr: 1.0, creatinine_mg_dl: 1.0],
                "bilirubin_mg_dl",
                (8.4, "meld.10_14"),
                (8.5, "meld.15_30"),
            )
            .cut(
                "INR 2.6 / 2.65 with creatinine 4: 30.40 → 30 and 30.61 → 31 (book: > 30 case by case)",
                ins![bilirubin_mg_dl: 1.0, creatinine_mg_dl: 4.0],
                "inr",
                (2.6, "meld.15_30"),
                (2.65, "meld.gt30"),
            ),
        Table::new("meld_na", &["meld_na.lt10", "meld_na.10_14", "meld_na.15_30", "meld_na.gt30"])
            .paper(
                "Kim 2008: MELD 15, Na 130: 15 + 1.32 × 7 − 0.033 × 15 × 7 = 20.78 → 21",
                ins![bilirubin_mg_dl: 2.0, inr: 1.5, creatinine_mg_dl: 1.2, sodium_mmol_l: 130.0],
                "meld_na.15_30",
            )
            .score(21.0, 0.0)
            .paper(
                "OPTN: sodium is bounded to 125–137; 120 counts as 125: 15 + 15.84 − 5.94 = 24.9 → 25",
                ins![bilirubin_mg_dl: 2.0, inr: 1.5, creatinine_mg_dl: 1.2, sodium_mmol_l: 120.0],
                "meld_na.15_30",
            )
            .score(25.0, 0.0)
            .noted("bounded to 125")
            .paper(
                "normal labs: MELD 6, no sodium step",
                ins![bilirubin_mg_dl: 1.0, inr: 1.0, creatinine_mg_dl: 1.0, sodium_mmol_l: 140.0],
                "meld_na.lt10",
            )
            .score(6.0, 0.0)
            .paper(
                "creatinine 4, bilirubin 10, INR 2.5, Na 125: above 30",
                ins![bilirubin_mg_dl: 10.0, inr: 2.5, creatinine_mg_dl: 4.0, sodium_mmol_l: 125.0],
                "meld_na.gt30",
            )
            .cut(
                "MELD(i) 11.48 → 11 gets no sodium step (stays 11); 11.57 → 12 does (12 + 9.24 − 2.77 = 18)",
                ins![inr: 1.0, creatinine_mg_dl: 1.0, sodium_mmol_l: 130.0],
                "bilirubin_mg_dl",
                (3.8, "meld_na.10_14"),
                (3.9, "meld_na.15_30"),
            ),
        Table::new("albi", &["albi.1", "albi.2", "albi.3"])
            .paper(
                "Johnson 2015: bilirubin 17.1 µmol/L, albumin 40 g/L: 0.66 log10 17.1 − 0.085 × 40 = −2.59",
                ins![bilirubin_mg_dl: 1.0, albumin_g_dl: 4.0],
                "albi.2",
            )
            .score(-2.586, 0.001)
            .cut(
                "albumin 4.0 / 4.02 g/dL: −2.586 (grade 2) and −2.603 (grade 1, ≤ −2.60)",
                ins![bilirubin_mg_dl: 1.0],
                "albumin_g_dl",
                (4.0, "albi.2"),
                (4.02, "albi.1"),
            )
            .cut(
                "albumin 2.97 / 2.95 g/dL with bilirubin 3: −1.396 (grade 2, ≤ −1.39) and −1.379 (grade 3)",
                ins![bilirubin_mg_dl: 3.0],
                "albumin_g_dl",
                (2.97, "albi.2"),
                (2.95, "albi.3"),
            )
            .cut(
                "as shown: albumin 4.017 g/dL is −2.6007, shown −2.60, so grade 1 like the screen says",
                ins![bilirubin_mg_dl: 1.0],
                "albumin_g_dl",
                (4.0, "albi.2"),
                (4.017, "albi.1"),
            )
            .cut(
                "as shown: albumin 2.964 g/dL is −1.3907, shown −1.39, so grade 2 (≤ −1.39)",
                ins![bilirubin_mg_dl: 3.0],
                "albumin_g_dl",
                (2.964, "albi.2"),
                (2.95, "albi.3"),
            )
            .unit(
                "bilirubin 5 mg/dL = 85.5 µmol/L, albumin 2.5 g/dL = 25 g/L: 0.66 × 1.932 − 2.125 = −0.85",
                ins![bilirubin_mg_dl: 5.0, albumin_g_dl: 2.5],
                "albi.3",
            )
            .score(-0.850, 0.001),
        Table::new("fib4", &["fib4.low", "fib4.mid", "fib4.high"])
            .paper(
                "Sterling 2006 formula: 50 × 40 / (200 × √25) = 2.00",
                ins![age_years: 50, ast_u_l: 40, alt_u_l: 25, platelets_10e9_l: 200],
                "fib4.mid",
            )
            .score(2.0, 1e-9)
            .paper(
                "60 × 80 / (90 × √49) = 7.62",
                ins![age_years: 60, ast_u_l: 80, alt_u_l: 49, platelets_10e9_l: 90],
                "fib4.high",
            )
            .score(7.619, 0.001)
            .paper(
                "age 30: 30 × 20 / (250 × 6) = 0.40, with the under-35 caveat",
                ins![age_years: 30, ast_u_l: 20, alt_u_l: 36, platelets_10e9_l: 250],
                "fib4.low",
            )
            .score(0.4, 1e-9)
            .noted("Less accurate")
            .cut(
                "1.287 / 1.30 (book p. 102: 1.3 is intermediate)",
                ins![age_years: 13, alt_u_l: 1, platelets_10e9_l: 1000],
                "ast_u_l",
                (99.0, "fib4.low"),
                (100.0, "fib4.mid"),
            )
            .cut(
                "3.25 / 3.315 (book: above 3.25 is high)",
                ins![age_years: 65, alt_u_l: 1, platelets_10e9_l: 1000],
                "ast_u_l",
                (50.0, "fib4.mid"),
                (51.0, "fib4.high"),
            )
            .cut(
                "as shown: 16 × 81 / 1000 = 1.296, shown 1.30, so intermediate",
                ins![age_years: 16, alt_u_l: 1, platelets_10e9_l: 1000],
                "ast_u_l",
                (80.0, "fib4.low"),
                (81.0, "fib4.mid"),
            )
            .cut(
                "as shown: 23 × 283 / (1000 × 2) = 3.2545, shown 3.25, so still intermediate",
                ins![age_years: 23, alt_u_l: 4, platelets_10e9_l: 1000],
                "ast_u_l",
                (283.0, "fib4.mid"),
                (284.0, "fib4.high"),
            ),
        Table::new("apri", &["apri.low", "apri.mid", "apri.high"])
            .paper(
                "Wai 2003: (82 / 40) × 100 / 100 = 2.05, WHO 2016 cirrhosis note above 2",
                ins![ast_u_l: 82, ast_uln_u_l: 40, platelets_10e9_l: 100],
                "apri.high",
            )
            .score(2.05, 1e-9)
            .noted("WHO 2016")
            .cut(
                "0.49 / 0.50 (book p. 34: 0.5 is moderate)",
                ins![ast_uln_u_l: 40, platelets_10e9_l: 250],
                "ast_u_l",
                (49.0, "apri.low"),
                (50.0, "apri.mid"),
            )
            .cut(
                "1.50 / 1.525 (book: above 1.5 is severe)",
                ins![ast_uln_u_l: 40, platelets_10e9_l: 100],
                "ast_u_l",
                (60.0, "apri.mid"),
                (61.0, "apri.high"),
            )
            .cut(
                "as shown: 20 / 40 × 100 / 101 = 0.495, shown 0.50, so moderate",
                ins![ast_u_l: 20, ast_uln_u_l: 40],
                "platelets_10e9_l",
                (102.0, "apri.low"),
                (101.0, "apri.mid"),
            )
            .cut(
                "as shown: 62 / 40 × 100 / 103 = 1.505, shown 1.50, so still moderate",
                ins![ast_u_l: 62, ast_uln_u_l: 40],
                "platelets_10e9_l",
                (103.0, "apri.mid"),
                (102.0, "apri.high"),
            ),
        Table::new("fli", &["fli.low", "fli.mid", "fli.high"])
            .paper(
                "Bedogni 2006: TG 150, BMI 30, GGT 50, waist 100: y = 1.312, FLI = 78.7",
                ins![tg_mg_dl: 150, bmi: 30, ggt_u_l: 50, waist_cm: 100],
                "fli.high",
            )
            .score(78.734, 0.001)
            .cut(
                "waist 86 / 87 cm: FLI 29.3 / 30.4 (Bedogni: < 30 rules out)",
                ins![tg_mg_dl: 100, bmi: 25, ggt_u_l: 30],
                "waist_cm",
                (86.0, "fli.low"),
                (87.0, "fli.mid"),
            )
            .cut(
                "waist 109 / 111 cm: FLI 58.4 / 60.9 (≥ 60 rules in)",
                ins![tg_mg_dl: 100, bmi: 25, ggt_u_l: 30],
                "waist_cm",
                (109.0, "fli.mid"),
                (111.0, "fli.high"),
            )
            .cut(
                "as shown: waist 109.9 cm is FLI 59.5, shown 60, so fatty liver likely",
                ins![tg_mg_dl: 100, bmi: 25, ggt_u_l: 30],
                "waist_cm",
                (109.0, "fli.mid"),
                (109.9, "fli.high"),
            ),
        Table::new("baveno_vi", &["baveno_vi.spare", "baveno_vi.screen"])
            .paper(
                "Baveno VI 2015: LSM 15 kPa and platelets 180 → no screening endoscopy",
                ins![lsm_kpa: 15.0, platelets_10e9_l: 180],
                "baveno_vi.spare",
            )
            .paper(
                "LSM 25 kPa: screen, and the CSPH note (book p. 35)",
                ins![lsm_kpa: 25.0, platelets_10e9_l: 200],
                "baveno_vi.screen",
            )
            .noted("portal hypertension")
            .cut("LSM 19.9 / 20 kPa (< 20 needed)", ins![platelets_10e9_l: 200], "lsm_kpa", (19.9, "baveno_vi.spare"), (20.0, "baveno_vi.screen"))
            .cut("platelets 151 / 150 (> 150 needed)", ins![lsm_kpa: 10.0], "platelets_10e9_l", (151.0, "baveno_vi.spare"), (150.0, "baveno_vi.screen")),
        Table::new("r_ratio", &["r_ratio.hepatocellular", "r_ratio.cholestatic", "r_ratio.mixed"])
            .paper(
                "EASL DILI 2019: ALT 10 × ULN, ALP 1 × ULN → R 10, hepatocellular",
                ins![alt_u_l: 400, alt_uln_u_l: 40, alp_u_l: 100, alp_uln_u_l: 100],
                "r_ratio.hepatocellular",
            )
            .score(10.0, 1e-9)
            .paper(
                "ALT 1 × ULN, ALP 4 × ULN → R 0.25, cholestatic",
                ins![alt_u_l: 40, alt_uln_u_l: 40, alp_u_l: 480, alp_uln_u_l: 120],
                "r_ratio.cholestatic",
            )
            .score(0.25, 1e-9)
            .cut(
                "R 5.0 / 5.1 (book p. 129: 5 is mixed; note gives CIOMS)",
                ins![alt_uln_u_l: 40, alp_u_l: 100, alp_uln_u_l: 100],
                "alt_u_l",
                (200.0, "r_ratio.mixed"),
                (204.0, "r_ratio.hepatocellular"),
            )
            .cut(
                "R 2.0 / 1.9 (book: 2 is mixed)",
                ins![alt_uln_u_l: 40, alp_u_l: 100, alp_uln_u_l: 100],
                "alt_u_l",
                (80.0, "r_ratio.mixed"),
                (76.0, "r_ratio.cholestatic"),
            )
            .paper(
                "R 5.04 is shown 5.0 and banded as shown: mixed, with the CIOMS note",
                ins![alt_u_l: 201.6, alt_uln_u_l: 40, alp_u_l: 100, alp_uln_u_l: 100],
                "r_ratio.mixed",
            )
            .noted("R = 5 hepatocellular")
            .paper(
                "R 2.0 note: CIOMS calls it cholestatic",
                ins![alt_u_l: 80, alt_uln_u_l: 40, alp_u_l: 100, alp_uln_u_l: 100],
                "r_ratio.mixed",
            )
            .noted("R = 2 cholestatic"),
        Table::new("maddrey", &["maddrey.severe", "maddrey.not_severe"])
            .paper(
                "Maddrey 1978: 4.6 × (20 − 13) + 10 = 42.2",
                ins![pt_s: 20.0, pt_control_s: 13.0, bilirubin_mg_dl: 10.0],
                "maddrey.severe",
            )
            .score(42.2, 1e-9)
            .cut(
                "DF 31.9 / 32.0 (Carithers 1989: ≥ 32 is severe)",
                ins![pt_s: 18.0, pt_control_s: 13.0],
                "bilirubin_mg_dl",
                (8.9, "maddrey.not_severe"),
                (9.0, "maddrey.severe"),
            ),
        Table::new("glasgow_ah", &["glasgow.high", "glasgow.low"])
            .paper(
                "Forrest 2005: age 55 (2), WBC 16 (2), urea 6 (2), INR 1.8 (2), bilirubin 256 µmol/L (3) = 11",
                ins![age_years: 55, wbc_10e9_l: 16.0, urea_mmol_l: 6.0, inr: 1.8, bilirubin_mg_dl: 15.0],
                "glasgow.high",
            )
            .score(11.0, 0.0)
            .cut(
                "age 49 / 50 moves 8 to 9 (≥ 9 poor survival)",
                ins![wbc_10e9_l: 15.0, urea_mmol_l: 5.0, inr: 1.5, bilirubin_mg_dl: 1.0],
                "age_years",
                (49.0, "glasgow.low"),
                (50.0, "glasgow.high"),
            )
            .cut(
                "WBC 14.9 / 15 (≥ 15 = 2)",
                ins![age_years: 50, urea_mmol_l: 5.0, inr: 1.5, bilirubin_mg_dl: 1.0],
                "wbc_10e9_l",
                (14.9, "glasgow.low"),
                (15.0, "glasgow.high"),
            )
            .cut(
                "urea 4.9 / 5.0 mmol/L (≥ 5 = 2)",
                ins![age_years: 50, wbc_10e9_l: 15.0, inr: 1.5, bilirubin_mg_dl: 1.0],
                "urea_mmol_l",
                (4.9, "glasgow.low"),
                (5.0, "glasgow.high"),
            )
            .cut(
                "INR 1.4 / 1.5 (1.5–2.0 = 2)",
                ins![age_years: 50, wbc_10e9_l: 15.0, urea_mmol_l: 5.0, bilirubin_mg_dl: 1.0],
                "inr",
                (1.4, "glasgow.low"),
                (1.5, "glasgow.high"),
            )
            .cut(
                "INR 2.0 / 2.1 (> 2.0 = 3)",
                ins![age_years: 40, wbc_10e9_l: 10.0, urea_mmol_l: 5.0, bilirubin_mg_dl: 1.0],
                "inr",
                (2.0, "glasgow.low"),
                (2.1, "glasgow.low"),
            )
            .unit(
                "bilirubin 7.3 mg/dL = 124.8 µmol/L: 1 point (< 125)",
                ins![age_years: 40, wbc_10e9_l: 10.0, urea_mmol_l: 4.0, inr: 1.0, bilirubin_mg_dl: 7.3],
                "glasgow.low",
            )
            .score(5.0, 0.0)
            .unit(
                "bilirubin 7.32 mg/dL = 125.2 µmol/L: 2 points",
                ins![age_years: 40, wbc_10e9_l: 10.0, urea_mmol_l: 4.0, inr: 1.0, bilirubin_mg_dl: 7.32],
                "glasgow.low",
            )
            .score(6.0, 0.0)
            .unit(
                "bilirubin 14.6 mg/dL = 249.7 µmol/L: 2 points (125–250)",
                ins![age_years: 40, wbc_10e9_l: 10.0, urea_mmol_l: 4.0, inr: 1.0, bilirubin_mg_dl: 14.6],
                "glasgow.low",
            )
            .score(6.0, 0.0)
            .unit(
                "bilirubin 14.65 mg/dL = 250.5 µmol/L: 3 points",
                ins![age_years: 40, wbc_10e9_l: 10.0, urea_mmol_l: 4.0, inr: 1.0, bilirubin_mg_dl: 14.65],
                "glasgow.low",
            )
            .score(7.0, 0.0),
        Table::new("saag", &["saag.high", "saag.low"])
            .paper(
                "Runyon 1992 / book p. 150: serum 3.5 − ascitic 1.0 = 2.5, portal hypertension; protein 1.0 is low",
                ins![serum_albumin_g_dl: 3.5, ascitic_albumin_g_dl: 1.0, ascitic_protein_g_dl: 1.0],
                "saag.high",
            )
            .score(2.5, 1e-9)
            .noted("cirrhosis")
            .paper(
                "serum 2.8 − ascitic 2.2 = 0.6, protein 3.5 is high (book p. 150)",
                ins![serum_albumin_g_dl: 2.8, ascitic_albumin_g_dl: 2.2, ascitic_protein_g_dl: 3.5],
                "saag.low",
            )
            .score(0.6, 1e-9)
            .noted("heart failure or malignant")
            .cut(
                "SAAG 1.0 / 1.1 (book: ≥ 1.1 portal hypertension)",
                ins![serum_albumin_g_dl: 3.0],
                "ascitic_albumin_g_dl",
                (2.0, "saag.low"),
                (1.9, "saag.high"),
            ),
        Table::new(
            "ascitic_pmn",
            &[
                "ascitic.secondary",
                "ascitic.sbp_culture_positive",
                "ascitic.sbp_culture_negative",
                "ascitic.sbp",
                "ascitic.bacterascites",
                "ascitic.polymicrobial",
                "ascitic.no_sbp",
            ],
        )
            .paper(
                "AASLD / book p. 167: PMN 300 with culture pending is SBP",
                ins![pmn_per_mm3: 300, culture: "pending"],
                "ascitic.sbp",
            )
            .paper("PMN 300, one organism: culture-positive", ins![pmn_per_mm3: 300, culture: "monomicrobial"], "ascitic.sbp_culture_positive")
            .paper("PMN 100, one organism: bacterascites", ins![pmn_per_mm3: 100, culture: "monomicrobial"], "ascitic.bacterascites")
            .paper("PMN 100, several organisms: gut puncture", ins![pmn_per_mm3: 100, culture: "polymicrobial"], "ascitic.polymicrobial")
            .paper(
                "Runyon: PMN 800 with glucose 40 mg/dL → secondary peritonitis",
                ins![pmn_per_mm3: 800, culture: "negative", glucose_mg_dl: 40],
                "ascitic.secondary",
            )
            .paper(
                "LDH above serum LDH → secondary",
                ins![pmn_per_mm3: 800, culture: "monomicrobial", ldh_above_serum: true],
                "ascitic.secondary",
            )
            .cut("PMN 249 / 250 (≥ 250 is SBP)", ins![culture: "negative"], "pmn_per_mm3", (249.0, "ascitic.no_sbp"), (250.0, "ascitic.sbp_culture_negative"))
            .cut("PMN 5000 / 5001 (> 5000 secondary)", ins![culture: "pending"], "pmn_per_mm3", (5000.0, "ascitic.sbp"), (5001.0, "ascitic.secondary"))
            .cut(
                "protein 1.5 / 1.6 g/dL (> 1.5 secondary)",
                ins![pmn_per_mm3: 400, culture: "negative"],
                "protein_g_dl",
                (1.5, "ascitic.sbp_culture_negative"),
                (1.6, "ascitic.secondary"),
            )
            .cut(
                "glucose 50 / 49 mg/dL (< 50 secondary)",
                ins![pmn_per_mm3: 400, culture: "negative"],
                "glucose_mg_dl",
                (50.0, "ascitic.sbp_culture_negative"),
                (49.0, "ascitic.secondary"),
            ),
        Table::new("sos_criteria", &["sos_criteria.met", "sos_criteria.not_met"])
            .paper(
                "McDonald 1984: jaundice and tender hepatomegaly on day 10",
                ins![jaundice: true, tender_hepatomegaly: true, days_since_transplant: 10],
                "sos_criteria.met",
            )
            .score(2.0, 0.0)
            .cut(
                "day 20 / 21 (book: within 20 days)",
                ins![weight_gain: true, ascites: true],
                "days_since_transplant",
                (20.0, "sos_criteria.met"),
                (21.0, "sos_criteria.not_met"),
            )
            .edge("one sign only", ins![ascites: true, days_since_transplant: 5], "sos_criteria.not_met")
            .edge("two signs", ins![ascites: true, jaundice: true, days_since_transplant: 5], "sos_criteria.met"),
        Table::new("cha2ds2_vasc", &["cha2ds2_vasc.low", "cha2ds2_vasc.mid", "cha2ds2_vasc.high"])
            .paper(
                "Lip 2010: woman 72 with hypertension and diabetes = age 1 + sex 1 + 1 + 1 = 4; ESC 2020: anticoagulate",
                ins![age_years: 72, sex: "female", hypertension: true, diabetes: true],
                "cha2ds2_vasc.low",
            )
            .score(4.0, 0.0)
            .noted("recommended")
            .paper("ESC 2020: woman 50, nothing else = 1 (sex only): no anticoagulation", ins![age_years: 50, sex: "female"], "cha2ds2_vasc.low")
            .score(1.0, 0.0)
            .noted("no anticoagulation")
            .paper("ESC 2020: man 50 with hypertension = 1: consider", ins![age_years: 50, sex: "male", hypertension: true], "cha2ds2_vasc.low")
            .noted("consider")
            .cut("age 64 / 65 (65–74 = 1)", ins![sex: "male"], "age_years", (64.0, "cha2ds2_vasc.low"), (65.0, "cha2ds2_vasc.low"))
            .cut("age 74 / 75 (≥ 75 = 2)", ins![sex: "male"], "age_years", (74.0, "cha2ds2_vasc.low"), (75.0, "cha2ds2_vasc.low"))
            .edge(
                "4 points: low (book p. 87: 1–4)",
                ins![age_years: 80, sex: "male", heart_failure: true, hypertension: true],
                "cha2ds2_vasc.low",
            )
            .edge(
                "5 points: between the rows",
                ins![age_years: 80, sex: "female", heart_failure: true, hypertension: true],
                "cha2ds2_vasc.mid",
            )
            .edge(
                "6 points: between the rows",
                ins![age_years: 80, sex: "female", heart_failure: true, hypertension: true, diabetes: true],
                "cha2ds2_vasc.mid",
            )
            .edge(
                "7 points: high (≥ 7)",
                ins![age_years: 80, sex: "female", heart_failure: true, hypertension: true, diabetes: true, vascular: true],
                "cha2ds2_vasc.high",
            )
            .paper(
                "all items: 9",
                ins![age_years: 80, sex: "female", heart_failure: true, hypertension: true, diabetes: true, vascular: true, stroke_tia: true],
                "cha2ds2_vasc.high",
            )
            .score(9.0, 0.0),
        Table::new("bclc", &["bclc.0", "bclc.a", "bclc.b", "bclc.c", "bclc.d"])
            .paper(
                "BCLC 2018 / book p. 362: single 1.8 cm, PS 0, Child-Pugh 5 → stage 0",
                ins![performance_status: 0, child_pugh_points: 5, nodules: 1, largest_cm: 1.8],
                "bclc.0",
            )
            .paper(
                "portal vein invasion, PS 1 → stage C",
                ins![performance_status: 1, child_pugh_points: 6, nodules: 1, largest_cm: 4.0, vascular_invasion: true],
                "bclc.c",
            )
            .paper(
                "extrahepatic spread, PS 0 → stage C",
                ins![performance_status: 0, child_pugh_points: 6, nodules: 2, largest_cm: 2.0, extrahepatic: true],
                "bclc.c",
            )
            .cut("single nodule 2.0 / 2.1 cm (very early ≤ 2)", ins![performance_status: 0, child_pugh_points: 5, nodules: 1], "largest_cm", (2.0, "bclc.0"), (2.1, "bclc.a"))
            .cut("single nodule 5.0 / 5.1 cm (early ≤ 5)", ins![performance_status: 0, child_pugh_points: 5, nodules: 1], "largest_cm", (5.0, "bclc.a"), (5.1, "bclc.b"))
            .cut("3 nodules 3.0 / 3.1 cm (early: up to 3, each ≤ 3)", ins![performance_status: 0, child_pugh_points: 5, nodules: 3], "largest_cm", (3.0, "bclc.a"), (3.1, "bclc.b"))
            .cut("3 / 4 nodules of 2 cm", ins![performance_status: 0, child_pugh_points: 5, largest_cm: 2.0], "nodules", (3.0, "bclc.a"), (4.0, "bclc.b"))
            .cut("PS 0 / 1 (A and B need PS 0)", ins![child_pugh_points: 5, nodules: 1, largest_cm: 1.5], "performance_status", (0.0, "bclc.0"), (1.0, "bclc.c"))
            .cut("PS 2 / 3 (> 2 is D)", ins![child_pugh_points: 5, nodules: 1, largest_cm: 1.5], "performance_status", (2.0, "bclc.c"), (3.0, "bclc.d"))
            .cut("Child-Pugh 7 / 8 without ascites (preserved up to B7)", ins![performance_status: 0, nodules: 1, largest_cm: 1.5], "child_pugh_points", (7.0, "bclc.0"), (8.0, "bclc.d"))
            .cut(
                "Child-Pugh 6 / 7 with ascites (B7 with ascites is not preserved)",
                ins![performance_status: 0, nodules: 1, largest_cm: 1.5, ascites: true],
                "child_pugh_points",
                (6.0, "bclc.0"),
                (7.0, "bclc.d"),
            ),
        Table::new("milan", &["milan.within", "milan.outside"])
            .paper("Mazzaferro 1996: single 4 cm, no invasion → within", ins![nodules: 1, largest_cm: 4.0, performance_status: 0], "milan.within")
            .paper("2 lesions of 2.5 cm → within", ins![nodules: 2, largest_cm: 2.5, performance_status: 0], "milan.within")
            .paper("vascular invasion → outside", ins![nodules: 1, largest_cm: 3.0, performance_status: 0, vascular_invasion: true], "milan.outside")
            .paper("metastasis → outside", ins![nodules: 1, largest_cm: 3.0, performance_status: 0, metastasis: true], "milan.outside")
            .cut("single 5.0 / 5.1 cm", ins![nodules: 1, performance_status: 0], "largest_cm", (5.0, "milan.within"), (5.1, "milan.outside"))
            .cut("3 lesions 3.0 / 3.1 cm", ins![nodules: 3, performance_status: 0], "largest_cm", (3.0, "milan.within"), (3.1, "milan.outside"))
            .cut("3 / 4 lesions of 1 cm", ins![largest_cm: 1.0, performance_status: 0], "nodules", (3.0, "milan.within"), (4.0, "milan.outside"))
            .cut("PS 1 / 2 (book p. 368 adds PS ≤ 1)", ins![nodules: 1, largest_cm: 2.0], "performance_status", (1.0, "milan.within"), (2.0, "milan.outside")),
        Table::new("okuda", &["okuda.1", "okuda.2", "okuda.3"])
            .paper(
                "Okuda 1985: albumin 2.9 and bilirubin 3.5 → 2 criteria, stage II",
                ins![albumin_g_dl: 2.9, bilirubin_mg_dl: 3.5],
                "okuda.2",
            )
            .score(2.0, 0.0)
            .cut("albumin 3.0 / 2.9 g/dL (< 3 counts)", ins![bilirubin_mg_dl: 1.0], "albumin_g_dl", (3.0, "okuda.1"), (2.9, "okuda.2"))
            .cut("bilirubin 3.0 / 3.1 mg/dL (> 3 counts)", ins![albumin_g_dl: 4.0], "bilirubin_mg_dl", (3.0, "okuda.1"), (3.1, "okuda.2"))
            .edge("2 criteria: stage II", ins![albumin_g_dl: 2.5, bilirubin_mg_dl: 4.0], "okuda.2")
            .edge("3 criteria: stage III", ins![albumin_g_dl: 2.5, bilirubin_mg_dl: 4.0, ascites: true], "okuda.3"),
        Table::new("page_b", &["page_b.low", "page_b.mid", "page_b.high"])
            .paper(
                "Papatheodoridis 2016: man 55 (6 + 6), platelets 150 (6) = 18",
                ins![age_years: 55, sex: "male", platelets_10e9_l: 150],
                "page_b.high",
            )
            .score(18.0, 0.0)
            .noted("The book prints 1 and 2 points")
            .cut("female 40, platelets 200 / 199 (100–199 = 6)", ins![age_years: 40, sex: "female"], "platelets_10e9_l", (200.0, "page_b.low"), (199.0, "page_b.mid"))
            .cut("man 49 / 50 with platelets 199: 16 / 18", ins![sex: "male", platelets_10e9_l: 199], "age_years", (49.0, "page_b.mid"), (50.0, "page_b.high"))
            .cut("woman 69 / 70, platelets 250: 8 / 10", ins![sex: "female", platelets_10e9_l: 250], "age_years", (69.0, "page_b.low"), (70.0, "page_b.mid"))
            .cut("woman 29 / 30, platelets 99: 9 / 11", ins![sex: "female", platelets_10e9_l: 99], "age_years", (29.0, "page_b.low"), (30.0, "page_b.mid"))
            .cut("platelets 100 / 99 (< 100 = 9)", ins![age_years: 16, sex: "female"], "platelets_10e9_l", (100.0, "page_b.low"), (99.0, "page_b.low")),
        Table::new("kings_college", &["kings.met", "kings.not_met"])
            .paper(
                "O'Grady 1989: paracetamol, pH 7.25 after resuscitation, > 24 h",
                ins![cause: "acetaminophen", inr: 3.0, ph: 7.25, over_24h: true],
                "kings.met",
            )
            .paper("Wilson disease presenting as ALF qualifies by itself", ins![cause: "wilson_budd_chiari", inr: 1.5], "kings.met")
            .paper(
                "non-paracetamol: INR 4, bilirubin 18, age 45, other cause = 3 minor criteria",
                ins![cause: "other", inr: 4.0, bilirubin_mg_dl: 18.0, age_years: 45],
                "kings.met",
            )
            .paper(
                "paracetamol triad: INR 7, creatinine 3.5, HE 3",
                ins![cause: "acetaminophen", inr: 7.0, creatinine_mg_dl: 3.5, he_stage: 3],
                "kings.met",
            )
            .cut("pH 7.30 / 7.29 (< 7.3)", ins![cause: "acetaminophen", inr: 3.0, over_24h: true], "ph", (7.3, "kings.not_met"), (7.29, "kings.met"))
            .cut("lactate 3.0 / 3.1 (> 3, Bernal 2002)", ins![cause: "acetaminophen", inr: 3.0], "lactate_mmol_l", (3.0, "kings.not_met"), (3.1, "kings.met"))
            .cut(
                "triad creatinine 3.4 / 3.5 (> 3.4)",
                ins![cause: "acetaminophen", inr: 7.0, he_stage: 3],
                "creatinine_mg_dl",
                (3.4, "kings.not_met"),
                (3.5, "kings.met"),
            )
            .cut("non-paracetamol INR 6.5 / 6.6 (> 6.5 alone)", ins![cause: "other"], "inr", (6.5, "kings.not_met"), (6.6, "kings.met"))
            .cut(
                "age 40 / 41 as the third minor criterion (> 40)",
                ins![cause: "other", inr: 4.0, bilirubin_mg_dl: 18.0],
                "age_years",
                (40.0, "kings.not_met"),
                (41.0, "kings.met"),
            )
            .cut(
                "jaundice to HE 7 / 8 days (> 7)",
                ins![cause: "indeterminate_or_drug", inr: 4.0],
                "jaundice_to_he_days",
                (7.0, "kings.not_met"),
                (8.0, "kings.met"),
            )
            .edge("pH 7.2 but under 24 h: not met", ins![cause: "acetaminophen", inr: 3.0, ph: 7.2], "kings.not_met")
            .edge("pH 7.2 over 24 h: met", ins![cause: "acetaminophen", inr: 3.0, ph: 7.2, over_24h: true], "kings.met"),
        Table::new("clichy", &["clichy.met", "clichy.not_met"])
            .paper("Bernuau 1986: age 25, HE 3, factor V 15%", ins![age_years: 25, he_stage: 3, factor_v_pct: 15], "clichy.met")
            .cut("factor V 20 / 19% under 30 (< 20)", ins![age_years: 25, he_stage: 3], "factor_v_pct", (20.0, "clichy.not_met"), (19.0, "clichy.met"))
            .cut("age 29 / 30 with factor V 25% (limit 20 → 30)", ins![he_stage: 4, factor_v_pct: 25], "age_years", (29.0, "clichy.not_met"), (30.0, "clichy.met"))
            .cut("HE stage 2 / 3", ins![age_years: 50, factor_v_pct: 10], "he_stage", (2.0, "clichy.not_met"), (3.0, "clichy.met")),
    ]
}
