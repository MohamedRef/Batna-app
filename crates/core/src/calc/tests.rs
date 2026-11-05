use super::*;

#[test]
fn normal_ranges_fit_inside_the_possible_range() {
    for spec in all() {
        for f in spec.fields {
            let Some(n) = f.normal else { continue };
            let Some((min, max)) = f.limits() else {
                panic!("{}.{}: a normal range on a field that is not a number", spec.id, f.key);
            };
            for (lo, hi) in [(n.low, n.high)].into_iter().chain(n.female) {
                assert!(lo <= hi, "{}.{}: normal range upside down", spec.id, f.key);
                assert!(!n.below || lo.is_infinite(), "{}.{}: `below` needs an open low end", spec.id, f.key);
                assert!(
                    (lo.is_infinite() || lo >= min) && (hi.is_infinite() || hi <= max),
                    "{}.{}: normal {lo}–{hi} outside the possible {min}–{max}",
                    spec.id,
                    f.key
                );
            }
        }
    }
}

#[test]
fn where_a_value_sits() {
    let hb = normals::HB_G_DL;
    assert_eq!(hb.place(12.5, Some(false)), Place::Low);
    assert_eq!(hb.place(12.5, Some(true)), Place::Normal);
    // Sex unknown: abnormal only when abnormal for both.
    assert_eq!(hb.place(12.5, None), Place::Normal);
    assert_eq!(hb.place(11.0, None), Place::Low);
    assert_eq!(hb.place(17.0, Some(true)), Place::High);
    assert_eq!(normals::LACTATE_MMOL_L.place(2.0, None), Place::Normal);
    assert_eq!(normals::LACTATE_MMOL_L.place(2.1, None), Place::High);
    assert_eq!(normals::ALBUMIN_G_DL.place(3.5, None), Place::Normal);
    // "Below 150": 150 itself is high.
    assert_eq!(normals::TRIGLYCERIDES_MG_DL.place(149.0, None), Place::Normal);
    assert_eq!(normals::TRIGLYCERIDES_MG_DL.place(150.0, None), Place::High);
    let waist = normals::WAIST_CM;
    assert_eq!(waist.place(85.0, Some(false)), Place::Normal);
    assert_eq!(waist.place(85.0, Some(true)), Place::High);
    assert_eq!(waist.place(85.0, None), Place::Normal);
    assert_eq!(waist.place(94.0, None), Place::High);
}

#[test]
fn run_refuses_impossible_numbers() {
    // Albumin 35 is g/L typed into a g/dL box.
    let i = Inputs::new()
        .choice("encephalopathy", "none")
        .choice("ascites", "absent")
        .num("bilirubin_mg_dl", 1.0)
        .num("albumin_g_dl", 35.0)
        .num("inr", 1.0);
    assert_eq!(
        compute("child_pugh", &i),
        Err(CalcError::Invalid { field: "albumin_g_dl", reason: IMPOSSIBLE })
    );
    assert!(compute("child_pugh", &i.num("albumin_g_dl", 3.5)).is_ok());
}

#[test]
fn guides_name_real_bands_once() {
    let mut bands: Vec<&str> = GUIDE_LISTS.iter().flat_map(|l| l.iter()).map(|(b, _)| *b).collect();
    let n = bands.len();
    bands.sort_unstable();
    bands.dedup();
    assert_eq!(bands.len(), n, "a band has two guides");
    for b in bands {
        let calc = b.split_once('.').map_or(b, |(c, _)| c);
        // Band ids start with their calculator's id or a short form of
        // it (`kings.met`), which BAND_WORDS also uses.
        let known = all().iter().any(|s| s.id == calc)
            || BAND_WORDS.iter().any(|(w, _)| w.split_once('.').is_some_and(|(c, _)| c == calc));
        assert!(known, "guide for unknown band {b}");
        let g = guide(b).unwrap();
        assert!(!g.next.is_empty() || !g.dd.is_empty(), "{b}: empty guide");
    }
}

#[test]
fn registry_ids_are_unique_and_known() {
    let mut ids: Vec<_> = all().iter().map(|s| s.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all().len());
    assert!(spec("child_pugh").is_some());
    assert!(matches!(compute("nope", &Inputs::new()), Err(CalcError::Unknown(_))));
}

#[test]
fn every_field_key_is_unique_per_calculator() {
    for s in all() {
        let mut keys: Vec<_> = s.fields.iter().map(|f| f.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), s.fields.len(), "{}", s.id);
    }
}

#[test]
fn empty_inputs_fail_cleanly() {
    for s in all() {
        if s.fields.iter().any(|f| !f.optional) {
            assert!((s.compute)(&Inputs::new()).is_err(), "{}", s.id);
        }
    }
}

#[test]
fn expect_accepts_band_ids_and_written_results() {
    let i = Inputs::new()
        .choice("encephalopathy", "none")
        .choice("ascites", "mild")
        .num("bilirubin_mg_dl", 2.5)
        .num("albumin_g_dl", 3.0)
        .num("pt_prolonged_s", 3.0);
    let o = compute("child_pugh", &i).unwrap();
    assert!(expect_matches(&o, "child_pugh.b"));
    assert!(expect_matches(&o, "Child-Pugh B (8 points)"));
    assert!(!expect_matches(&o, "Child-Pugh B (9 points)"));
    assert!(!expect_matches(&o, "child_pugh.c"));
    let albi = compute("albi", &Inputs::new().num("bilirubin_mg_dl", 1.0).num("albumin_g_dl", 4.0)).unwrap();
    assert!(expect_matches(&albi, "ALBI −2.59, grade 2"));
    let bclc = compute(
        "bclc",
        &Inputs::new()
            .num("performance_status", 1.0)
            .num("child_pugh_points", 5.0)
            .num("nodules", 1.0)
            .num("largest_cm", 3.0),
    )
    .unwrap();
    assert!(expect_matches(&bclc, "Stage C"));
    assert!(!expect_matches(&bclc, "Stage B"));
}

// Issue #42: a number equal to the score must not carry a wrong band.
#[test]
fn expect_naming_another_band_fails() {
    let glasgow = |o: &Outcome| (o.band, o.score);
    let i = Inputs::new()
        .num("age_years", 55.0)
        .num("wbc_10e9_l", 16.0)
        .num("urea_mmol_l", 6.0)
        .num("inr", 1.6)
        .num("bilirubin_mg_dl", 5.0);
    let o = compute("glasgow_ah", &i).unwrap();
    assert_eq!(glasgow(&o), ("glasgow.high", Some(9.0)));
    assert!(expect_matches(&o, "9 or more"));
    assert!(expect_matches(&o, "GAHS 9"));
    assert!(!expect_matches(&o, "Below 9"));
    assert!(!expect_matches(&o, "glasgow.low"));

    let cp = Inputs::new()
        .choice("encephalopathy", "none")
        .choice("ascites", "mild")
        .num("bilirubin_mg_dl", 2.5)
        .num("albumin_g_dl", 2.5)
        .num("pt_prolonged_s", 3.0);
    let o = compute("child_pugh", &cp).unwrap();
    assert_eq!((o.band, o.score), ("child_pugh.b", Some(9.0)));
    assert!(expect_matches(&o, "Child-Pugh B (9 points)"));
    assert!(!expect_matches(&o, "Child-Pugh C (9 points)"));
    assert!(!expect_matches(&o, "Class C, 9 points"));

    let r = compute(
        "r_ratio",
        &Inputs::new()
            .num("alt_u_l", 200.0)
            .num("alt_uln_u_l", 40.0)
            .num("alp_u_l", 120.0)
            .num("alp_uln_u_l", 120.0),
    )
    .unwrap();
    // The book's bands (p. 129): exactly 5 is mixed, not the CIOMS
    // hepatocellular.
    assert_eq!(r.band, "r_ratio.mixed");
    assert!(expect_matches(&r, "R = 5, mixed"));
    assert!(!expect_matches(&r, "R = 5, hepatocellular"));
}

// Issue #252: naming the right band is a right typed answer.
#[test]
fn typed_band_names_are_accepted() {
    let i = Inputs::new()
        .choice("encephalopathy", "grade_3_4")
        .choice("ascites", "severe")
        .num("bilirubin_mg_dl", 5.0)
        .num("albumin_g_dl", 2.0)
        .num("pt_prolonged_s", 8.0);
    let o = compute("child_pugh", &i).unwrap();
    assert_eq!((o.band, o.score), ("child_pugh.c", Some(15.0)));
    for t in ["Child-Pugh C", "class c", "Child-Pugh class C", "C (15)", "15"] {
        assert!(answer_matches(&o, t), "{t}");
    }
    for t in ["Child-Pugh B", "class b", "14", "", "C"] {
        assert!(!answer_matches(&o, t), "{t}");
    }
    // The validator stays strict: content shows the number.
    assert!(!expect_matches(&o, "Child-Pugh C"));
}

#[test]
fn longer_band_words_win() {
    assert_eq!(named_bands("Criteria not met", "kings.met"), ["kings.not_met"]);
    assert_eq!(named_bands("Criteria met: INR > 6.5", "kings.met"), ["kings.met"]);
    assert_eq!(named_bands("DF 40, not severe", "maddrey.severe"), ["maddrey.not_severe"]);
    assert_eq!(named_bands("Grade 2", "child_pugh.b"), Vec::<&str>::new());
    assert_eq!(named_bands("metabolic", "clichy.met"), Vec::<&str>::new());
}

#[test]
fn keys_may_drop_their_unit_suffix() {
    let spec = spec("r_ratio").unwrap();
    let raw: BTreeMap<String, CalcValue> =
        [("alt", 120.0), ("alt_uln", 40.0), ("alp", 600.0), ("alp_uln_u_l", 120.0)]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), CalcValue::Number(v)))
            .collect();
    let i = Inputs::for_spec(spec, &raw);
    assert!(unknown_inputs(spec, &i).is_empty());
    let o = (spec.compute)(&i).unwrap();
    assert!(expect_matches(&o, "R = 0.6, cholestatic"));
}

#[test]
fn numbers_match_at_the_written_precision() {
    let i = Inputs::new()
        .num("age_years", 45.0)
        .num("ast_u_l", 30.0)
        .num("alt_u_l", 36.0)
        .num("platelets_10e9_l", 250.0);
    let o = compute("fib4", &i).unwrap();
    assert!(expect_matches(&o, "FIB-4 0.9"));
    assert!(expect_matches(&o, "FIB-4 0.90"));
    assert!(!expect_matches(&o, "FIB-4 1.9"));
}

#[test]
fn unknown_inputs_are_reported() {
    let i = Inputs::new().num("bilirubin", 2.0).num("albumin_g_dl", 3.0);
    assert_eq!(unknown_inputs(spec("albi").unwrap(), &i), vec!["bilirubin"]);
}

#[test]
fn registry_matches_the_content_schema_list() {
    let mut ids: Vec<_> = all().iter().map(|s| s.id).collect();
    let mut listed = crate::content::CALCULATORS.to_vec();
    ids.sort_unstable();
    listed.sort_unstable();
    assert_eq!(ids, listed);
}

#[test]
fn the_reference_doc_is_current() {
    let doc = include_str!("../../../../docs/CALCULATORS.md");
    assert!(doc == reference_markdown(), "docs/CALCULATORS.md is stale: run the print_reference test");
}

#[test]
#[ignore = "prints docs/CALCULATORS.md"]
fn print_reference() {
    print!("{}", reference_markdown());
}

#[test]
fn inputs_parse_from_yaml_shapes() {
    let i = Inputs::new().choice("x", "2.5").choice("f", "yes");
    assert_eq!(i.get_num("x"), Ok(2.5));
    assert_eq!(i.get_flag("f"), Ok(true));
    assert!(Inputs::new().num("x", f64::NAN).get_num("x").is_err());
    assert!(Inputs::new().num("x", 0.0).get_pos("x").is_err());
}
