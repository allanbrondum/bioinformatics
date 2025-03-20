use crate::alphabet_model::{CharT, WithSeparator};
use crate::string::suffix_trie_suffix_links::{Node, NodeId, build_trie, node_id_rc};
use crate::string_model::{AStr, AString};
use generic_array::ArrayLength;
use generic_array::typenum::{Add1, B1, Unsigned};
use hashbrown::{HashMap, HashSet};
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter;
use std::ops::Add;
use std::rc::Rc;

pub fn lcs_trie_with_separator<'s, C: CharT>(s: &'s AStr<C>, t: &AStr<C>) -> &'s AStr<C>
where
    WithSeparator<C>: CharT,
{
    let separated: AString<_> = s
        .iter()
        .copied()
        .map(WithSeparator::Char)
        .chain(iter::once(WithSeparator::Separator))
        .chain(t.iter().copied().map(WithSeparator::Char))
        .collect();

    let trie = build_trie(&separated);

    let mut node_marks = HashMap::new();
    mark_nodes_rec(&trie.root, &mut node_marks);

    let mut lcs_res = AStr::empty();
    lcs_trie_with_separator_rec(&trie.root, &node_marks, &mut lcs_res);

    lcs_res
}

fn lcs_trie_with_separator_rec<'s, C: CharT>(
    node: &Rc<RefCell<Node<'s, WithSeparator<C>, <WithSeparator<C> as CharT>::AlphabetSize>>>,
    node_marks: &HashMap<NodeId, Marks>,
    lcs_res: &mut &AStr<C>,
) where
    WithSeparator<C>: CharT,
{
    if !node_marks
        .get(&node_id_rc(node))
        .copied()
        .unwrap_or_default()
        .both()
    {
        return;
    }
    for child_edge in node
        .borrow()
        .children
        .iter()
        .filter_map(|child| child.as_ref())
    {
        lcs_trie_with_separator_rec(&child_edge.borrow().target, node_marks, lcs_res);
    }
}

#[derive(Default, Copy, Clone)]
struct Marks {
    start_before_separator: bool,
    start_after_separator: bool,
}

impl Marks {
    fn both(&self) -> bool {
        self.start_before_separator && self.start_after_separator
    }
}

fn mark_nodes_rec<'s, C: PartialEq, N: ArrayLength>(
    node: &Rc<RefCell<Node<'s, WithSeparator<C>, N>>>,
    node_marks: &mut HashMap<NodeId, Marks>,
) {
    let mut has_separator_edge = false;
    for child_edge in node
        .borrow()
        .children
        .iter()
        .filter_map(|child| child.as_ref())
    {
        let child_edge_ref = child_edge.borrow();
        if child_edge_ref.chars.contains(&WithSeparator::Separator) {
            has_separator_edge = true;
        } else {
            mark_nodes_rec(&child_edge_ref.target, node_marks);
        }
    }
    if has_separator_edge {
        mark_ancestors(node, &mut |node_id| {
            let mark = node_marks.entry(node_id).or_default();
            let changed = !mark.start_before_separator;
            mark.start_before_separator = true;
            changed
        });
    }
    if node.borrow().terminal.is_some() {
        mark_ancestors(node, &mut |node_id| {
            let mark = node_marks.entry(node_id).or_default();
            let changed = !mark.start_after_separator;
            mark.start_after_separator = true;
            changed
        });
    }
}

fn mark_ancestors<'s, C, N: ArrayLength>(
    node: &Rc<RefCell<Node<'s, C, N>>>,
    mark_node: &mut impl FnMut(NodeId) -> bool,
) {
    if mark_node(node_id_rc(node)) {
        if let Some(parent) = node.borrow().parent.as_ref() {
            mark_ancestors(&parent.borrow().source, mark_node);
        }
    }
}

pub fn lcs_trie<'a, C: CharT>(s: &AStr<C>, t: &'a AStr<C>) -> &'a AStr<C> {
    let trie = build_trie(s);

    let mut substr: &AStr<C> = AStr::empty();
    for i in 0..t.len() {
        if t.len() - i <= substr.len() {
            break;
        }

        let m = trie.index_substr_maximal(&t[i..]);
        if m.length > substr.len() {
            substr = &t[i..i + m.length];
        }
    }

    substr
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    #[test]
    fn test_lcs_trie() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(lcs_trie(s, t), AStr::from_slice(&[B, A, A, B, A]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_lcs_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(0..20)) {
            let expected = string::lcs_simple(&s, &t);
            let lcs = lcs_trie(&s, &t);
            prop_assert_eq!(lcs.len(), expected.len());
            prop_assert!(s.contains(lcs));
            prop_assert!(t.contains(lcs));
        }
    }

    #[test]
    fn test_lcs_trie_with_separator() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(
            lcs_trie_with_separator(s, t),
            AStr::from_slice(&[B, A, A, B, A])
        );
    }
}
