use crate::alphabet_model::CharT;
use crate::string::suffix_trie_suffix_links_arena_refs;
use crate::string_model::AStr;
use alloc::borrow::Cow;
use bumpalo::Bump;
use std::collections::VecDeque;
use hashbrown::HashSet;
use std::borrow::Borrow;
use std::cell::RefCell;

#[derive(Debug)]
pub struct SuffixArray<'s, C: CharT> {
    sorted_suffixes: Vec<usize>,
    s: Cow<'s, AStr<C>>,
}

pub fn build_array<'s, C: CharT>(s: Cow<'s, AStr<C>>) -> SuffixArray<'s, C> {
    let bump = Bump::new();
    let trie = suffix_trie_suffix_links_arena_refs::build_trie_with_allocator(s.borrow(), &bump);

    let mut sorted_suffixes = Vec::with_capacity(s.len());

    let mut to_visit = VecDeque::new();
    to_visit.push_front(trie.root);

    while let Some(node) = to_visit.pop_front() {
        let node_ref = node.borrow();
        if let Some(terminal) = node_ref.terminal.as_ref() {
            sorted_suffixes.push(terminal.suffix_index);
        }

        for child_edge in node_ref.children.iter().filter_map(|child| child.as_ref()) {
            let child_edge_ref = RefCell::borrow(child_edge);
            to_visit.push_front(child_edge_ref.target);
        }
    }

    SuffixArray { sorted_suffixes, s }
}

impl<'s, C: CharT> SuffixArray<'s, C>{
    pub fn indexes_substr(&self, t: &'s AStr<C>) -> HashSet<usize> {
        let mut result = HashSet::new();



        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};

    #[test]
    fn test_build_trie_and_find_substr_empty() {
        let s: &AStr<Char> = AStr::from_slice(&[]);

        let array = build_array(Cow::Borrowed(s));

        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let array = build_array(Cow::Borrowed(s));

        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([0])
        );
        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[A, A])),
            HashSet::from([0, 1])
        );
        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[A])),
            HashSet::from([0, 1, 2])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let array = build_array(Cow::Borrowed(s));

        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([])
        );
        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[A, B, A])),
            HashSet::from([0, 3, 5])
        );
        assert_eq!(
            array.indexes_substr(AStr::from_slice(&[B, A, A])),
            HashSet::from([1, 6])
        );
    }


    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let array = build_array(Cow::Owned(s));
            let expected = string::indexes(&s, &t);
            let indexes = array.indexes_substr( &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
