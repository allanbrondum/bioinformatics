use crate::alphabet_model::CharT;
use crate::string::suffix_trie_mcc_arena;
use crate::string_model::AStr;
use alloc::borrow::Cow;
use bumpalo::Bump;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct SuffixArray<'s, C: Copy> {
    sorted_suffixes: Vec<usize>,
    s: Cow<'s, AStr<C>>,
}

pub fn build_array<'s, C: CharT>(s: Cow<'s, AStr<C>>) -> SuffixArray<'s, C> {
    let bump = Bump::new();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(s.borrow(), &bump);
    // suffix_trie_suffix_links_arena_refs::to_dot("target/build_array.dot", &trie);

    let mut sorted_suffixes = Vec::with_capacity(s.len());

    let mut to_visit = VecDeque::new();
    to_visit.push_front(trie.root);

    while let Some(node) = to_visit.pop_front() {
        let node_ref = node.borrow();
        if let Some(terminal) = node_ref.terminal.as_ref() {
            sorted_suffixes.push(terminal.suffix_index);
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

    SuffixArray { sorted_suffixes, s }
}

impl<'s, C: CharT + Ord> SuffixArray<'s, C> {
    fn ord_suffix(&self, i: usize) -> &AStr<C> {
        &self.s[self.sorted_suffixes[i]..]
    }

    pub fn index_substr_naive(&self, t: &'s AStr<C>) -> Option<usize> {
        let mut low = 0;
        let mut high = self.sorted_suffixes.len();

        while low < high {
            let middle = (low + high) / 2;
            let middle_s = self.ord_suffix(middle);

            match t.cmp(&middle_s[..t.len().min(middle_s.len())]) {
                Ordering::Equal => return Some(self.sorted_suffixes[middle]),
                Ordering::Less => high = middle,
                Ordering::Greater => low = middle + 1,
            }
        }

        None
    }

    pub fn index_substr_simple(&self, t: &'s AStr<C>) -> Option<usize> {
        let mut low = 0;
        let mut high = self.sorted_suffixes.len();
        let mut p_low = self.ord_suffix(low).lcp(t).len();
        let mut p_high = self.ord_suffix(high - 1).lcp(t).len();

        while low < high {
            let middle = (low + high) / 2;
            let middle_s = self.ord_suffix(middle);
            let p = p_low.min(p_high);
            let p_middle = p + middle_s[p..].lcp(&t[p..]).len();

            if p_middle == t.len() {
                return Some(self.sorted_suffixes[middle]);
            } else {
                match t[p_middle..].cmp(&middle_s[p_middle..middle_s.len()]) {
                    Ordering::Less => {
                        high = middle;
                        p_high = p_middle;
                    }
                    Ordering::Greater => {
                        low = middle + 1;
                        p_low = p_middle;
                    }
                    Ordering::Equal => {
                        unreachable!()
                    }
                }
            }
        }

        None
    }
}

fn print_array<'s, C: CharT>(array: &SuffixArray<'s, C>) {
    for suffix in array.sorted_suffixes.iter().copied() {
        println!("{}", &array.s[suffix..]);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    #[test]
    #[ignore]
    fn test_build_array_and_find_substr_empty() {
        let s: &AStr<Char> = AStr::from_slice(&[]);

        let array = build_array(Cow::Borrowed(s));

        assert_eq!(array.index_substr_naive(AStr::from_slice(&[])), None);
    }

    #[test]
    fn test_build_array_and_find_substr_naive_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let array = build_array(Cow::Borrowed(s));
        print_array(&array);

        assert_eq!(
            array.index_substr_naive(AStr::from_slice(&[A, A, A])),
            Some(0)
        );
        assert_eq!(array.index_substr_naive(AStr::from_slice(&[A, A])), Some(1));
        assert_eq!(array.index_substr_naive(AStr::from_slice(&[A])), Some(1));
    }

    #[test]
    fn test_build_array_and_find_substr_naive() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);
        let array = build_array(Cow::Borrowed(s));
        print_array(&array);

        assert_eq!(array.index_substr_naive(AStr::from_slice(&[A, A, A])), None);
        assert_eq!(
            array.index_substr_naive(AStr::from_slice(&[A, B, A])),
            Some(0)
        );
        assert_eq!(
            array.index_substr_naive(AStr::from_slice(&[B, A, A])),
            Some(1)
        );
        assert_eq!(
            array.index_substr_naive(AStr::from_slice(&[B, A, B, A,])),
            Some(4)
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie_naive(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let array = build_array(Cow::Borrowed(&s));
            let index = array.index_substr_naive(&t);
            if let Some(index) = index {
                prop_assert_eq!(t.as_str(), &s[index..index + t.len()]);
            } else {
                prop_assert!(!s.contains(&t));
            }

        }
    }

    #[test]
    fn test_build_array_and_find_substr_simple_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let array = build_array(Cow::Borrowed(s));
        print_array(&array);

        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[A, A, A])),
            Some(0)
        );
        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[A, A])),
            Some(1)
        );
        assert_eq!(array.index_substr_simple(AStr::from_slice(&[A])), Some(1));
    }

    #[test]
    fn test_build_array_and_find_substr_simple() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);
        let array = build_array(Cow::Borrowed(s));
        print_array(&array);

        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[A, A, A])),
            None
        );
        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[A, B, A])),
            Some(0)
        );
        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[B, A, A])),
            Some(1)
        );
        assert_eq!(
            array.index_substr_simple(AStr::from_slice(&[B, A, B, A,])),
            Some(4)
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie_simple(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let array = build_array(Cow::Borrowed(&s));
            let index = array.index_substr_simple(&t);
            if let Some(index) = index {
                prop_assert_eq!(t.as_str(), &s[index..index + t.len()]);
            } else {
                prop_assert!(!s.contains(&t));
            }

        }
    }
}
