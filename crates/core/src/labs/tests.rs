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
    let chips: Vec<bool> =
        p.iter().filter_map(|p| if let Piece::Value { chip, .. } = p { Some(*chip) } else { None }).collect();
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
