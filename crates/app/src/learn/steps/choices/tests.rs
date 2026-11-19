use super::*;

fn opts(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_key_written_first_shows_on_every_letter_about_as_often() {
    let options = opts(&["Key", "Wrong one", "Wrong two", "Wrong three"]);
    let mut at = [0usize; 4];
    for k in 0..4000 {
        let order = option_order(&options, &format!("hb.L01.lesson{}/{k}", k % 40));
        at[order.iter().position(|&i| i == 0).unwrap()] += 1;
    }
    // One in four each; the old order-puzzle shuffle gave A about 12%.
    for (letter, n) in at.iter().enumerate() {
        assert!((800..1200).contains(n), "letter {letter}: {n} of 4000 ({at:?})");
    }
}

#[test]
fn two_options_are_not_always_swapped() {
    let options = opts(&["Key", "Wrong"]);
    let first = (0..400).filter(|k| option_order(&options, &format!("s{k}"))[0] == 0).count();
    assert!((120..280).contains(&first), "{first} of 400 kept the key first");
}

#[test]
fn the_same_step_always_opens_the_same_way() {
    let options = opts(&["a", "b", "c", "d", "e"]);
    let one = option_order(&options, "neph.L09.lesson2/4");
    assert_eq!(one, option_order(&options, "neph.L09.lesson2/4"));
    let mut sorted = one.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, [0, 1, 2, 3, 4], "a permutation: {one:?}");
}

#[test]
fn options_that_point_at_others_keep_the_book_order() {
    for options in [
        opts(&["Pearson correlation", "Student t-test", "Chi-square test", "None of the above"]),
        opts(&["Raised ALP", "Raised GGT", "Both A and B", "Neither"]),
    ] {
        for k in 0..20 {
            assert_eq!(option_order(&options, &format!("res.L18.lesson2/{k}")), [0, 1, 2, 3], "{options:?}");
        }
    }
}

#[test]
fn numeric_options_go_smallest_first() {
    let options = opts(&["About 65%", "Under 5%", "1/2 to 2/3", "About 10%"]);
    // Not all start with a number, so these are shuffled like any others.
    let order = option_order(&options, "endo.L04.lesson10/3");
    assert_eq!(order.len(), 4);
    let numbers = opts(&["250/mm3", "100/mm3", "500/mm3"]);
    assert_eq!(option_order(&numbers, "hb.L09.lesson3/1"), [1, 0, 2]);
}
