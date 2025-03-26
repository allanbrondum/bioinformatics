use crate::alphabet_model::{CharT, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use alloc::borrow::Cow;
use bumpalo::Bump;
use generic_array::typenum::Unsigned;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::iter;

#[derive(Debug)]
pub struct BWT<'s, C: CharT> {
    l: AString<C>,
    s: Cow<'s, AStr<C>>,
}

type WithTerminator<C> = WithSpecial<C, '$', true>;

pub fn build_bwt<'s, C: CharT>(s: Cow<'s, AStr<C>>) -> BWT<'s, C> {
    let bump = Bump::new();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(s.borrow(), &bump);

    let mut transform = AString::with_capacity(s.len());

    let mut to_visit = VecDeque::new();
    to_visit.push_front(trie.root);

    while let Some(node) = to_visit.pop_front() {
        let node_ref = node.borrow();
        if let Some(terminal) = node_ref.terminal.as_ref() {
            transform.push(s[(terminal.suffix_index + s.len() - 1) % s.len()]);
        }

        for child_edge in node_ref
            .children
            .iter()
            .rev()
            .filter_map(|child| child.as_ref())
        {
            let child_edge_ref = RefCell::borrow(child_edge);
            to_visit.push_front(child_edge_ref.target);
        }
    }

    BWT { l: transform, s }
}

impl<'s, C: CharT + Ord> BWT<'s, C> {
    // fn ord_suffix(&self, i: usize) -> &AStr<C> {
    //     &self.s[self.sorted_suffixes[i]..]
    // }

    // pub fn index_substr_naive(&self, t: &'s AStr<C>) -> Option<usize> {
    //     let mut low = 0;
    //     let mut high = self.sorted_suffixes.len();
    //
    //     while low < high {
    //         let middle = (low + high) / 2;
    //         let middle_s = self.ord_suffix(middle);
    //
    //         match t.cmp(&middle_s[..t.len().min(middle_s.len())]) {
    //             Ordering::Equal => return Some(self.sorted_suffixes[middle]),
    //             Ordering::Less => high = middle,
    //             Ordering::Greater => low = middle + 1,
    //         }
    //     }
    //
    //     None
    // }
}

fn bwt_reverse<C: CharT>(l: &AStr<WithTerminator<C>>) -> AString<WithTerminator<C>>
where
    WithTerminator<C>: CharT,
{
    let f = {
        let mut tmp = l.to_owned();
        tmp.sort();
        tmp
    };

    println!("{} (l)", l);
    println!("{} (f)", f);

    let mut char_count = vec![0; <WithTerminator::<C> as CharT>::AlphabetSize::USIZE];
    for char in l {
        char_count[char.index()] += 1;
    }

    println!("{:?} (char_count)", char_count);

    let mut f_char_indexes = char_count
        .iter()
        .copied()
        .scan(0, |cumulated, count| {
            let tmp = *cumulated;
            *cumulated += count;
            Some(tmp)
        })
        .collect_vec();

    println!("{:?} (f_char_indexes)", f_char_indexes);

    let lf_map = l
        .iter()
        .copied()
        .map(|ch| {
            let f_idx = f_char_indexes[ch.index()];
            f_char_indexes[ch.index()] += 1;
            f_idx
        })
        .collect_vec();

    println!("{:?} (lf_map)", lf_map);

    let s_rev: AString<_> = iter::repeat_n((), l.len())
        .scan(0, |next_f_idx, _| {
            let tmp = *next_f_idx;
            *next_f_idx = lf_map[*next_f_idx];
            Some(f[tmp])
        })
        .collect();

    s_rev.into_iter().rev().collect()
}

fn print_bwt<'s, C: CharT>(bwt: &BWT<'s, C>) {
    println!("{}", bwt.s);
    println!("{}", bwt.l);
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    #[test]
    fn test_build_bwt() {
        use crate::string_model::test_util::Char::*;

        let s: &AStr<WithTerminator<Char>> = AStr::from_slice(&[
            WithTerminator::Char(A),
            WithTerminator::Char(B),
            WithTerminator::Char(A),
            WithTerminator::Char(A),
            WithTerminator::Char(B),
            WithTerminator::Char(A),
            WithTerminator::Special,
        ]);

        let bwt = build_bwt(Cow::Borrowed(s));

        assert_eq!(
            bwt.l,
            AStr::from_slice(&[
                WithTerminator::Char(A),
                WithTerminator::Char(B),
                WithTerminator::Char(B),
                WithTerminator::Char(A),
                WithTerminator::Special,
                WithTerminator::Char(A),
                WithTerminator::Char(A),
            ])
        );
    }

    #[test]
    fn test_reverse_bwt() {
        use crate::string_model::test_util::Char::*;

        let s: &AStr<WithTerminator<Char>> = AStr::from_slice(&[
            WithTerminator::Char(A),
            WithTerminator::Char(B),
            WithTerminator::Char(A),
            WithTerminator::Char(A),
            WithTerminator::Char(B),
            WithTerminator::Char(A),
            WithTerminator::Special,
        ]);

        let bwt = build_bwt(Cow::Borrowed(s));

        assert_eq!(bwt_reverse(&bwt.l), s);
    }

    // #[test]
    // #[ignore]
    // fn test_build_array_and_find_substr_empty() {
    //     let s: &AStr<Char> = AStr::from_slice(&[]);
    //
    //     let array = build_array(Cow::Borrowed(s));
    //
    //     assert_eq!(array.index_substr_naive(AStr::from_slice(&[])), None);
    // }
    //
    // #[test]
    // fn test_build_array_and_find_substr_naive_repetition() {
    //     use crate::string_model::test_util::Char::*;
    //
    //     let s = AStr::from_slice(&[A, A, A]);
    //
    //     let array = build_array(Cow::Borrowed(s));
    //     print_array(&array);
    //
    //     assert_eq!(
    //         array.index_substr_naive(AStr::from_slice(&[A, A, A])),
    //         Some(0)
    //     );
    //     assert_eq!(array.index_substr_naive(AStr::from_slice(&[A, A])), Some(1));
    //     assert_eq!(array.index_substr_naive(AStr::from_slice(&[A])), Some(1));
    // }
    //
    // #[test]
    // fn test_build_array_and_find_substr_naive() {
    //     use crate::string_model::test_util::Char::*;
    //
    //     let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);
    //     let array = build_array(Cow::Borrowed(s));
    //     print_array(&array);
    //
    //     assert_eq!(array.index_substr_naive(AStr::from_slice(&[A, A, A])), None);
    //     assert_eq!(
    //         array.index_substr_naive(AStr::from_slice(&[A, B, A])),
    //         Some(0)
    //     );
    //     assert_eq!(
    //         array.index_substr_naive(AStr::from_slice(&[B, A, A])),
    //         Some(1)
    //     );
    //     assert_eq!(
    //         array.index_substr_naive(AStr::from_slice(&[B, A, B, A,])),
    //         Some(4)
    //     );
    // }
    //
    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(2000))]
    //
    //     #[test]
    //     fn prop_test_trie_naive(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
    //         let array = build_array(Cow::Borrowed(&s));
    //         let index = array.index_substr_naive(&t);
    //         if let Some(index) = index {
    //             prop_assert_eq!(t.as_str(), &s[index..index + t.len()]);
    //         } else {
    //             prop_assert!(!s.contains(&t));
    //         }
    //
    //     }
    // }
}
