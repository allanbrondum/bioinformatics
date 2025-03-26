use crate::alphabet_model::{CharT, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use alloc::borrow::Cow;
use bumpalo::Bump;
use generic_array::typenum::Unsigned;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter;

#[derive(Debug)]
pub struct BWT<C> {
    /// BWT transform of s
    l: AString<WithTerminator<C>>,
    /// s sorted
    f: AString<WithTerminator<C>>,
    lf_map: Vec<usize>,
    // s: Cow<'s, AStr<C>>,
}

type WithTerminator<C> = WithSpecial<C, '$', true>;

pub fn build_bwt<'s, C: CharT>(s: &'s AStr<C>) -> BWT<C>
where
    WithTerminator<C>: CharT,
{
    let bump = Bump::new();
    let s_terminated: AString<_> = s
        .iter()
        .copied()
        .map(WithTerminator::Char)
        .chain(iter::once(WithTerminator::Special))
        .collect();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(&s_terminated, &bump);

    let mut l = AString::with_capacity(s_terminated.len());

    let mut to_visit = VecDeque::new();
    to_visit.push_front(trie.root);

    while let Some(node) = to_visit.pop_front() {
        let node_ref = node.borrow();
        if let Some(terminal) = node_ref.terminal.as_ref() {
            l.push(
                s_terminated[(terminal.suffix_index + s_terminated.len() - 1) % s_terminated.len()],
            );
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

    let f = {
        let mut tmp = s_terminated.to_owned();
        tmp.sort();
        tmp
    };

    let mut char_count = vec![0; <WithTerminator::<C> as CharT>::AlphabetSize::USIZE];
    for char in l.iter().copied() {
        char_count[char.index()] += 1;
    }

    let mut f_char_indexes = char_count
        .iter()
        .copied()
        .scan(0, |cumulated, count| {
            let tmp = *cumulated;
            *cumulated += count;
            Some(tmp)
        })
        .collect_vec();

    let lf_map = l
        .iter()
        .copied()
        .map(|ch| {
            let f_idx = f_char_indexes[ch.index()];
            f_char_indexes[ch.index()] += 1;
            f_idx
        })
        .collect_vec();

    // println!("{:?} (lf_map)", lf_map);

    BWT { l, f, lf_map }
}

impl<'s, C: CharT + Ord> BWT<C> {
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

// fn bwt_reverse<C: CharT>(bwt: &BWT<C>) -> AString<C>
// {
//     let s_rev: AString<_> = iter::repeat_n((), bwt.l.len())
//         .scan(0, |next_f_idx, _| {
//             let tmp = *next_f_idx;
//             *next_f_idx = bwt.lf_map[*next_f_idx];
//             Some(bwt.f[tmp])
//         })
//         .collect();
//
//     s_rev
//         .into_iter()
//         .rev()
//         .filter_map(|ch| match ch {
//             WithTerminator::Char(ch) => Some(ch),
//             WithTerminator::Special => None,
//         })
//         .collect()
// }

fn bwt_reverse<C: CharT>(bwt: &BWT<C>) -> AString<C> {
    iter::repeat(())
        .scan(0, |next_f_idx, _| {
            let tmp = *next_f_idx;
            *next_f_idx = bwt.lf_map[*next_f_idx];
            match bwt.l[tmp] {
                WithTerminator::Char(ch) => Some(ch),
                WithTerminator::Special => None,
            }
        })
        .collect()
}

fn print_bwt<'s, C: CharT>(bwt: &BWT<C>) {
    println!("{}", bwt.f);
    println!("{}", bwt.l);
    println!("{:?}", bwt.lf_map);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    use crate::string_model::test_util::Char;

    #[test]
    fn test_build_bwt() {
        use crate::string_model::test_util::Char::*;

        let s: &AStr<Char> = AStr::from_slice(&[A, B, A, A, B, A]);

        let bwt = build_bwt(&s);

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

        let s: &AStr<Char> = AStr::from_slice(&[A, B, A, A, B, A]);

        let bwt = build_bwt(&s);

        assert_eq!(bwt_reverse(&bwt), s);
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
