use crate::alphabet_model::{CharT, CharT2, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use bumpalo::Bump;
use generic_array::typenum::{Add1, Unsigned};
use generic_array::{ArrayLength, GenericArray};
use hashbrown::HashSet;
use hdrhistogram::Histogram;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter;

type Ranks<C: CharT2> = GenericArray<usize, C::AlphabetSizeP1>;

#[derive(Debug)]
pub struct BWT<C: CharT2> {
    /// BWT transform of s
    l: AString<WithTerminal<C>>,
    /// s sorted3
    f: AString<WithTerminal<C>>,
    // lf_map: Vec<usize>,
    f_char_indexes: GenericArray<usize, C::AlphabetSizeP2>,
    // l_ranks: Vec<usize>,
    l_ranks_sparse: Vec<Ranks<C>>,
    suffix_array_sparse: Vec<usize>,

    rank_sparse_factor: usize,
    suffix_array_sparse_factor: usize,

    pub suffix_offset_hist: RefCell<Histogram<u64>>,
}

type WithTerminal<C> = WithSpecial<C, '$', true>;

pub fn build_bwt<'s, C: CharT2>(s: &'s AStr<C>) -> BWT<C>
where
    WithTerminal<C>: CharT,
{
    build_bwt_with(s, 5, 10)
}

pub fn build_bwt_with<'s, C: CharT2>(
    s: &'s AStr<C>,
    rank_sparse_factor: usize,
    suffix_array_sparse_factor: usize,
) -> BWT<C>
where
    WithTerminal<C>: CharT,
{
    let bump = Bump::new();
    let s_terminated: AString<_> = s
        .iter()
        .copied()
        .map(WithTerminal::Char)
        .chain(iter::once(WithTerminal::Special))
        .collect();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(&s_terminated, &bump);

    let mut l = AString::with_capacity(s_terminated.len());
    let mut suffix_array = Vec::with_capacity(s_terminated.len());

    let mut to_visit = VecDeque::new();
    to_visit.push_front(trie.root);

    while let Some(node) = to_visit.pop_front() {
        let node_ref = node.borrow();
        if let Some(terminal) = node_ref.terminal.as_ref() {
            l.push(
                s_terminated[(terminal.suffix_index + s_terminated.len() - 1) % s_terminated.len()],
            );
            suffix_array.push(terminal.suffix_index);
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

    let mut char_count = vec![0; <WithTerminal::<C> as CharT>::AlphabetSize::USIZE];
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
            ranks[ch.index()] += 1;
            Some(ranks.clone())
        })
        .step_by(rank_sparse_factor)
        .collect_vec();

    let suffix_array_sparse = suffix_array
        .into_iter()
        .step_by(suffix_array_sparse_factor)
        .collect_vec();

    BWT {
        l,
        f,
        f_char_indexes,
        l_ranks_sparse,
        suffix_array_sparse,
        rank_sparse_factor,
        suffix_array_sparse_factor,
        suffix_offset_hist: RefCell::new(Histogram::new(2).unwrap()),
    }
}

impl<'s, C: CharT2> BWT<C>
where
    WithTerminal<C>: CharT,
{
    fn l_rank_delta(&self, ch: WithTerminal<C>, from_idx: usize, to_idx: usize) -> usize {
        self.l[from_idx + 1..=to_idx]
            .iter()
            .copied()
            .filter(|&lch| lch == ch)
            .count()
    }

    fn l_rank(&self, idx: usize, ch: WithTerminal<C>) -> usize {
        let sparse_idx = idx / self.rank_sparse_factor * self.rank_sparse_factor;
        self.l_ranks_sparse[sparse_idx / self.rank_sparse_factor][ch.index()]
            + self.l_rank_delta(ch, sparse_idx, idx)
    }

    fn lf_map(&self, idx: usize) -> usize {
        let ch = self.l[idx];
        self.f_char_indexes[ch.index()] + self.l_rank(idx, ch) - 1
    }

    pub fn indexes_substr(&self, t: &AStr<C>) -> HashSet<usize> {
        let mut low = 0;
        let mut high = self.l.len() - 1;

        for ch in t.iter().copied().rev() {
            let ch_w = WithTerminal::Char(ch);
            low = self.f_char_indexes[ch_w.index()]
                + if low == 0 {
                    0
                } else {
                    self.l_rank(low - 1, ch_w)
                };
            high = self.f_char_indexes[ch_w.index()] + self.l_rank(high, ch_w) - 1;

            if high < low {
                return Default::default();
            }
        }

        (low..=high)
            .into_iter()
            .map(|mut idx| {
                let mut suffix_offset = 0;
                while idx % self.suffix_array_sparse_factor != 0 {
                    suffix_offset += 1;
                    idx = self.lf_map(idx);
                }

                self.suffix_offset_hist
                    .borrow_mut()
                    .record(suffix_offset as u64)
                    .unwrap();

                (self.suffix_array_sparse[idx / self.suffix_array_sparse_factor] + suffix_offset)
                    % self.l.len()
            })
            .collect()
    }
}

fn bwt_reverse<C: CharT2>(bwt: &BWT<C>) -> AString<C>
where
    WithTerminal<C>: CharT,
{
    iter::repeat(())
        .scan(0, |next_f_idx, _| {
            let tmp = *next_f_idx;
            *next_f_idx = bwt.lf_map(*next_f_idx);
            match bwt.l[tmp] {
                WithTerminal::Char(ch) => Some(ch),
                WithTerminal::Special => None,
            }
        })
        .collect()
}

fn print_bwt<'s, C: CharT2>(bwt: &BWT<C>) {
    println!("{}", bwt.f);
    println!("{}", bwt.l);
    println!("{:?}", bwt.f_char_indexes);
    println!("{:?}", bwt.l_ranks_sparse);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;
    use hashbrown::HashSet;
    use proptest::prelude::ProptestConfig;
    use proptest::strategy::ValueTree;
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
                WithTerminal::Char(A),
                WithTerminal::Char(B),
                WithTerminal::Char(B),
                WithTerminal::Char(A),
                WithTerminal::Special,
                WithTerminal::Char(A),
                WithTerminal::Char(A),
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

        print_bwt(&bwt);

        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A])),
            HashSet::from([0, 2, 3, 5, 7, 8])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[B])),
            HashSet::from([1, 4, 6])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[B, A, A])),
            HashSet::from([1, 6])
        );
        assert_eq!(
            bwt.indexes_substr(AStr::from_slice(&[A, B, A])),
            HashSet::from([0, 3, 5])
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

    // #[test]
    // fn test_lcs_single_trie_perf() {
    //     let mut runner = proptest::test_runner::TestRunner::default();
    //     let s = arb_astring::<Char>(10_000)
    //         .new_tree(&mut runner)
    //         .unwrap()
    //         .current();
    //     let t = arb_astring::<Char>(5)
    //         .new_tree(&mut runner)
    //         .unwrap()
    //         .current();
    //
    //     let bwt = build_bwt(&s);
    //
    //     // bwt.
    // }
}
