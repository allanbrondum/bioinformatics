use crate::alphabet_model::CharT;
use crate::string;
use crate::string_model::AStr;
use itertools::Itertools;

pub fn border_array<C: CharT>(s: &AStr<C>) -> Vec<usize> {
    if s.is_empty() {
        return Default::default();
    }

    let mut arr = vec![0; s.len()];
    arr[0] = 0;

    for i in 0..s.len() - 1 {
        let mut b = arr[i];
        loop {
            if s[i + 1] == s[b] {
                arr[i + 1] = b + 1;
                break;
            } else if b == 0 {
                arr[i + 1] = 0;
                break;
            } else {
                b = arr[b - 1];
            }
        }
    }

    arr
}

pub fn border_array_simple<C: CharT>(s: &AStr<C>) -> Vec<usize> {
    (1..=s.len())
        .scan(0, |prev_overlap, l| {
            let max_possible_overlap = *prev_overlap + 1;
            let overlap = string::overlap(
                &s[1.max(l - max_possible_overlap)..l],
                &s[0..(l - 1).min(max_possible_overlap)],
            );
            *prev_overlap = overlap;
            Some(overlap)
        })
        .collect_vec()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ascii::ascii;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};

    #[test]
    fn test_border_array_simple() {
        assert_eq!(
            border_array_simple(ascii("aababbabbbabbaabaaa")),
            vec![0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 2, 3, 4, 2, 2]
        );
    }

    #[test]
    fn test_border_array() {
        assert_eq!(
            border_array(ascii("aababbabbbabbaabaaa")),
            vec![0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 2, 3, 4, 2, 2]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_border_array(s in arb_astring::<Char>(0..20)) {
            let expected = border_array_simple(&s);
            let border_array = border_array(&s);
            prop_assert_eq!(border_array, expected);
        }
    }
}
