use crate::alphabet_model::{CharT, CharT2, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use bumpalo::Bump;
use generic_array::typenum::{Add1, Unsigned};
use generic_array::{ArrayLength, GenericArray};
use hashbrown::HashSet;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter;

type Ranks<C: CharT2> = GenericArray<usize, C::AlphabetSizeP1>;

#[derive(Debug)]
pub struct BWT<C: CharT2> {
    /// BWT transform of s
    l: AString<WithTerminator<C>>,
    /// s sorted3
    f: AString<WithTerminator<C>>,
    lf_map: Vec<usize>,
    f_char_indexes: GenericArray<usize, C::AlphabetSizeP2>,
    // l_ranks: Vec<usize>,
    l_ranks_sparse: Vec<Ranks<C>>,
    sparse_factor: usize,
    // s: Cow<'s, AStr<C>>,
}

type WithTerminator<C> = WithSpecial<C, '$', true>;

pub fn build_bwt<'s, C: CharT2>(s: &'s AStr<C>) -> BWT<C>
where
    WithTerminator<C>: CharT,
{
    build_bwt_with(s, 5)
}

pub fn build_bwt_with<'s, C: CharT2>(s: &'s AStr<C>, sparse_factor: usize) -> BWT<C>
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

    let f_char_indexes: GenericArray<_, _> = iter::once(0)
        .chain(char_count.iter().copied().scan(0, |cumulated, count| {
            *cumulated += count;
            Some(*cumulated)
        }))
        .collect();

    // let l_ranks = l
    //     .iter()
    //     .copied()
    //     .scan(Ranks::<C>::default(), |ranks, ch| {
    //         let tmp = ranks[ch.index()];
    //         ranks[ch.index()] += 1;
    //         Some(tmp)
    //     })
    //     .collect_vec();

    let l_ranks_sparse = l
        .iter()
        .copied()
        .scan(Ranks::<C>::default(), |ranks, ch| {
            let tmp = ranks.clone();
            ranks[ch.index()] += 1;
            Some(tmp)
        })
        .step_by(sparse_factor)
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
        // l_ranks,
        l_ranks_sparse,
        sparse_factor,
    }
}

impl<'s, C: CharT2 + Ord> BWT<C>
where
    WithTerminator<C>: CharT,
{
    fn l_rank_delta(&self, ch: WithTerminator<C>, from_idx: usize, to_idx: usize) -> usize {
        self.l[from_idx..to_idx]
            .iter()
            .copied()
            .filter(|&lch| lch == ch)
            .count()
    }

    fn l_rank(&self, idx: usize) -> usize {
        let ch = self.l[idx];
        self.l_ranks_sparse[idx / self.sparse_factor][ch.index()]
            + self.l_rank_delta(ch, idx / self.sparse_factor * self.sparse_factor, idx)
    }

    fn lf_map(&self, idx: usize) -> usize {
        // self.lf_map[idx]

        // self.f_char_indexes[self.l[idx].index()] + self.l_ranks[idx]

        let ch = self.l[idx];
        self.f_char_indexes[ch.index()] + self.l_rank(idx)
    }

    pub fn indexes_substr(&self, t: &AStr<C>) -> HashSet<usize> {
        let Some((&ch, mut t_rest)) = t.split_last() else {
            return Default::default();
        };

        let mut f_idxes = (self.f_char_indexes[WithTerminator::Char(ch).index()]
            ..self.f_char_indexes[WithTerminator::Char(ch).index() + 1])
            .collect_vec();

        while let Some((&ch, t_rest_tmp)) = t_rest.split_last() {
            t_rest = t_rest_tmp;
            f_idxes.iter_mut().for_each(|idx| *idx = self.lf_map(*idx));
            f_idxes.retain(|idx| self.f[*idx] == WithSpecial::Char(ch));
        }

        f_idxes
            .into_iter()
            .map(|mut idx| {
                let mut tmp = 0;
                loop {
                    idx = self.lf_map(idx);
                    if self.f[idx] == WithTerminator::Special {
                        break tmp;
                    } else {
                        tmp += 1;
                    }
                }
            })
            .collect()
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

fn bwt_reverse<C: CharT2>(bwt: &BWT<C>) -> AString<C> {
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

fn print_bwt<'s, C: CharT2>(bwt: &BWT<C>) {
    println!("{}", bwt.f);
    println!("{}", bwt.l);
    println!("{:?}", bwt.f_char_indexes);
    println!("{:?}", bwt.l_ranks_sparse);
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

        assert_eq!(bwt.indexes_substr(AStr::from_slice(&[])), HashSet::from([]));
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

        print_bwt(&bwt);

        assert_eq!(bwt.indexes_substr(AStr::from_slice(&[])), HashSet::from([]));
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
