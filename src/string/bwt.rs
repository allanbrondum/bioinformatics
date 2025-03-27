use crate::alphabet_model::{CharT, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use bumpalo::Bump;
use generic_array::typenum::Unsigned;
use hashbrown::HashSet;
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
    f_char_indexes: Vec<usize>,
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

    let f_char_indexes = char_count
        .iter()
        .copied()
        .scan(0, |cumulated, count| {
            let tmp = *cumulated;
            *cumulated += count;
            Some(tmp)
        })
        .collect_vec();

    let mut f_char_indexes_mut = f_char_indexes.clone();
    let lf_map = l
        .iter()
        .copied()
        .map(|ch| {
            let f_idx = f_char_indexes_mut[ch.index()];
            f_char_indexes_mut[ch.index()] += 1;
            f_idx
        })
        .collect_vec();

    BWT {
        l,
        f,
        lf_map,
        f_char_indexes,
    }
}

impl<'s, C: CharT + Ord> BWT<C> {
    // fn ord_suffix(&self, i: usize) -> &AStr<C> {
    //     &self.s[self.sorted_suffixes[i]..]
    // }

    pub fn indexes_substr(&self, t: &AStr<C>) -> HashSet<usize> {
        todo!()

        // iter::repeat(())
        //     .scan(0, |next_f_idx, _| {
        //         let tmp = *next_f_idx;
        //         *next_f_idx = bwt.lf_map[*next_f_idx];
        //         match bwt.l[tmp] {
        //             WithTerminator::Char(ch) => Some(ch),
        //             WithTerminator::Special => None,
        //         }
        //     })
        //     .collect()
    }
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
    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;
    use hashbrown::HashSet;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};
    use std::mem;

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

    #[test]
    fn test_build_trie_and_find_substr_empty() {
        let s: &AStr<Char> = AStr::from_slice(&[]);

        let bwt = build_bwt(&s);

        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let bwt = build_bwt(&s);

        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([0])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, A])),
            HashSet::from([0, 1])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A])),
            HashSet::from([0, 1, 2])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let bwt = build_bwt(&s);

        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, B, A])),
            HashSet::from([0, 3, 5])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[B, A, A])),
            HashSet::from([1, 6])
        );
    }

    // #[test]
    // fn test_find_maximal_substr() {
    //     use crate::string_model::test_util::Char::*;
    //
    //     let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);
    //
    //     let bwt = build_bwt(&s);
    //
    //     assert_eq!(
    //         bwt.indexes_substr_maximal(AStr::from_slice(&[A, B, A])),
    //         MaximalSubstrMatch::full(5, 3),
    //     );
    //     assert_eq!(
    //         bwt.index_substr_maximal(AStr::from_slice(&[B, A, A])),
    //         MaximalSubstrMatch::full(6, 3),
    //     );
    //     assert_eq!(
    //         bwt.index_substr_maximal(AStr::from_slice(&[A, A, A])),
    //         MaximalSubstrMatch::partial(7, 2),
    //     );
    // }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let bwt = build_bwt(&s);
            let expected = string::indexes(&s, &t);
            let indexes = bwt.indexes_substr( &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
