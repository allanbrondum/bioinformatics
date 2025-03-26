use crate::alphabet_model::{CharT, WithSpecial};
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::{AStr, AString};
use alloc::borrow::Cow;
use bumpalo::Bump;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BWT<'s, C: CharT> {
    transform: AString<C>,
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

    BWT { transform, s }
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

fn print_bwt<'s, C: CharT>(bwt: &BWT<'s, C>) {
    println!("{}", bwt.s);
    println!("{}", bwt.transform);
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
            bwt.transform,
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
