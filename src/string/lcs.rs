use crate::alphabet_model::{CharT, WithSpecial};
use crate::string::suffix_trie_mcc_arena::{Node, NodeId, node_id_ptr, terminals};
use crate::string_model::{AStr, AString};

use generic_array::ArrayLength;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter;

use crate::string::suffix_trie_mcc_arena;
use bumpalo::Bump;
use hashbrown::HashMap;

type WithSeparator<C> = WithSpecial<C, '#', false>;

pub fn lcs_joined_trie<'s, C: CharT>(s: &'s AStr<C>, t: &AStr<C>) -> &'s AStr<C>
where
    WithSeparator<C>: CharT,
{
    let separated: AString<_> = s
        .iter()
        .copied()
        .map(WithSeparator::Char)
        .chain(iter::once(WithSeparator::Special))
        .chain(t.iter().copied().map(WithSeparator::Char))
        .collect();

    // let start = Instant::now();
    let alloc = Bump::new();
    // let alloc = alloc::Global;
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(&separated, &alloc);
    // println!("build trie elapsed {:?}", start.elapsed());

    // let start = Instant::now();
    let mut node_marks = HashMap::new();
    mark_nodes_rec(trie.root, s.len(), &mut node_marks);
    // println!("mark nodes elapsed {:?}", start.elapsed());

    // let start = Instant::now();
    // let mut deepest_path = PathDepth {
    //     depth: 0,
    //     lower: trie.root,
    // };
    // lcs_trie_with_separator_rec(&trie.root, 0, &node_marks, &mut deepest_path);
    let deepest_path = lcs_trie_with_separator_queue(trie.root, &node_marks);
    // println!("scan for lcs elapsed {:?}", start.elapsed());

    // let start = Instant::now();
    let mut min_suffix = usize::MAX;
    terminals(&deepest_path.lower.borrow(), |suffix| {
        min_suffix = min_suffix.min(suffix)
    });
    // println!("terminals elapsed {:?}", start.elapsed());
    &s[min_suffix..min_suffix + deepest_path.depth]
}

struct PathDepth<'arena, 's, C, N: ArrayLength> {
    depth: usize,
    lower: &'arena RefCell<Node<'arena, 's, C, N>>,
}

fn lcs_trie_with_separator_rec<'arena, 's, C, N: ArrayLength>(
    node: &'arena RefCell<Node<'arena, 's, WithSeparator<C>, N>>,
    node_depth: usize,
    node_marks: &HashMap<NodeId, Marks>,
    deepest_path: &mut PathDepth<'arena, 's, WithSeparator<C>, N>,
) where
    WithSeparator<C>: CharT,
{
    if node_depth > deepest_path.depth {
        *deepest_path = PathDepth {
            depth: node_depth,
            lower: node,
        };
    }

    for child_edge in node
        .borrow()
        .children
        .iter()
        .filter_map(|child| child.as_ref())
    {
        let child_edge_ref = child_edge.borrow();
        if node_marks
            .get(&node_id_ptr(child_edge_ref.target.as_ptr()))
            .copied()
            .unwrap_or_default()
            .both()
        {
            lcs_trie_with_separator_rec(
                child_edge_ref.target,
                node_depth + child_edge_ref.chars.len(),
                node_marks,
                deepest_path,
            );
        }
    }
}

fn lcs_trie_with_separator_queue<'arena, 's, C, N: ArrayLength>(
    root: &'arena RefCell<Node<'arena, 's, WithSeparator<C>, N>>,
    node_marks: &HashMap<NodeId, Marks>,
) -> PathDepth<'arena, 's, WithSeparator<C>, N>
where
    WithSeparator<C>: CharT,
{
    struct ToVisit<'arena, 's, C, N: ArrayLength> {
        node: &'arena RefCell<Node<'arena, 's, WithSeparator<C>, N>>,
        depth: usize,
    }

    let mut to_visit = VecDeque::new();
    to_visit.push_front(ToVisit {
        node: root,
        depth: 0,
    });

    let mut deepest_path = PathDepth {
        depth: 0,
        lower: root,
    };

    while let Some(node) = to_visit.pop_front() {
        if node.depth > deepest_path.depth {
            deepest_path = PathDepth {
                depth: node.depth,
                lower: node.node,
            };
        }

        for child_edge in node
            .node
            .borrow()
            .children
            .iter()
            .filter_map(|child| child.as_ref())
        {
            let child_edge_ref = child_edge.borrow();
            if node_marks
                .get(&node_id_ptr(child_edge_ref.target.as_ptr()))
                .copied()
                .unwrap_or_default()
                .both()
            {
                to_visit.push_back(ToVisit {
                    node: child_edge_ref.target,
                    depth: node.depth + child_edge_ref.chars.len(),
                });
            }
        }
    }

    deepest_path
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

fn mark_nodes_rec<'arena, 's, C: PartialEq, N: ArrayLength>(
    node: &'arena RefCell<Node<'arena, 's, WithSeparator<C>, N>>,
    separator_index: usize,
    node_marks: &mut HashMap<NodeId, Marks>,
) {
    for child_edge in node
        .borrow()
        .children
        .iter()
        .filter_map(|child| child.as_ref())
    {
        let child_edge_ref = child_edge.borrow();
        mark_nodes_rec(child_edge_ref.target, separator_index, node_marks);
    }

    if let Some(terminal) = node.borrow().terminal.as_ref() {
        #[allow(clippy::comparison_chain)]
        if terminal.suffix_index < separator_index {
            mark_ancestors(node, &mut |node_id| {
                let mark = node_marks.entry(node_id).or_default();
                let changed = !mark.start_before_separator;
                mark.start_before_separator = true;
                changed
            });
        } else if terminal.suffix_index > separator_index {
            mark_ancestors(node, &mut |node_id| {
                let mark = node_marks.entry(node_id).or_default();
                let changed = !mark.start_after_separator;
                mark.start_after_separator = true;
                changed
            });
        }
    }
}

fn mark_ancestors<'arena, 's, C, N: ArrayLength>(
    node: &'arena RefCell<Node<'arena, 's, C, N>>,
    mark_node: &mut impl FnMut(NodeId) -> bool,
) {
    if mark_node(node_id_ptr(node.as_ptr())) {
        if let Some(parent) = node.borrow().parent.as_ref() {
            mark_ancestors(parent.borrow().source, mark_node);
        }
    }
}

pub fn lcs_single_trie<'a, C: CharT>(s: &AStr<C>, t: &'a AStr<C>) -> &'a AStr<C> {
    // let start = Instant::now();
    let bump = Bump::new();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(s, &bump);
    // println!("build trie elapsed {:?}", start.elapsed());

    // let start = Instant::now();
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
    // println!("probe elapsed {:?}", start.elapsed());

    substr
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::strategy::Strategy;
    use proptest::strategy::ValueTree;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    #[test]
    fn test_lcs_single_trie() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(lcs_single_trie(s, t), AStr::from_slice(&[B, A, A, B, A]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_lcs_single_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(0..20)) {
            let expected = string::lcs_simple(&s, &t);
            let lcs = lcs_single_trie(&s, &t);
            prop_assert_eq!(lcs.len(), expected.len());
            prop_assert!(s.contains(lcs));
            prop_assert!(t.contains(lcs));
        }
    }

    #[test]
    fn test_lcs_joined_trie() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(lcs_joined_trie(s, t), AStr::from_slice(&[B, A, A, B, A]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_lcs_joined_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(0..20)) {
            let expected = string::lcs_simple(&s, &t);
            let lcs = lcs_joined_trie(&s, &t);
            prop_assert_eq!(lcs.len(), expected.len());
            prop_assert!(s.contains(lcs));
            prop_assert!(t.contains(lcs));
        }
    }

    #[test]
    fn test_lcs_joined_trie_perf() {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(10_000)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let t = arb_astring::<Char>(10_000)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        let _ = lcs_joined_trie(&s, &t);
    }

    #[test]
    fn test_lcs_single_trie_perf() {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(10_000)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let t = arb_astring::<Char>(10_000)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        let _ = lcs_single_trie(&s, &t);
    }
}
