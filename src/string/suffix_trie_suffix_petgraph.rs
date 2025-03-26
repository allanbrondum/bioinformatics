//! McCreight algorithm

use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use generic_array::ArrayLength;

use crate::string;

use bumpalo::Bump;
use hashbrown::HashSet;
use petgraph::Direction;
use petgraph::data::Build;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::EdgeRef;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::mem;
use std::ops::DerefMut;
use petgraph::graph::EdgeIndex;

const GRAPH_DEBUG: bool = false;

type Graph<'s, C> = StableDiGraph<Node, Edge<'s, C>>;

#[derive(Debug)]
pub struct SuffixTrie<'s, C: CharT> {
    graph: Graph<'s, C>,
    root: NodeIndex,
    s: &'s AStr<C>,
}

#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) terminal: Option<Terminal>,
}

impl Default for Node {
    fn default() -> Self {
        Self { terminal: None }
    }
}

#[derive(Debug)]
pub(crate) struct Terminal {
    pub(crate) suffix_index: usize,
}

#[derive(Debug)]
pub(crate) enum Edge<'s, C> {
    Tree(TreeEdge<'s, C>),
    Suffix,
}

#[derive(Debug)]
pub(crate) struct TreeEdge<'s, C> {
    pub(crate) chars: &'s AStr<C>,
}

struct ScanReturn<'s, C> {
    upper: NodeIndex,
    lower: NodeIndex,
    t_rem_matched: &'s AStr<C>,
    matched: ScanMatch<'s, C>,
}

enum ScanMatch<'s, C> {
    FullMatch,
    MaximalNonFullMatch { t_unmatched: &'s AStr<C> },
}

// fn parent(graph: &Graph<'s, C>) {

// }

fn child<'a, 's, C: PartialEq>(
    graph: &'a Graph<'s, C>,
    node_idx: NodeIndex,
    ch: C,
) -> Option<(&'a TreeEdge<'s, C>, NodeIndex, EdgeIndex)> {
    graph
        .edges_directed(node_idx, Direction::Outgoing)
        .filter_map(|edge| match edge.weight() {
            Edge::Tree(tree_edge) if tree_edge.chars[0] == ch => Some((tree_edge, edge.target(), edge.id())),
            _ => None,
        })
        .next()
}


fn suffix<'s, C: PartialEq>(graph: &Graph<'s, C>, node_idx: NodeIndex) -> Option<NodeIndex> {
    graph
        .edges_directed(node_idx, Direction::Outgoing)
        .filter_map(|edge| match edge.weight() {
            Edge::Suffix => Some(edge.target()),
            _ => None,
        })
        .next()
}

fn parent<'a, 's, C: PartialEq>(
    graph: &'a Graph<'s, C>,
    node_idx: NodeIndex,
) -> Option<(&'a TreeEdge<'s, C>, NodeIndex)> {
    graph
        .edges_directed(node_idx, Direction::Incoming)
        .filter_map(|edge| match edge.weight() {
            Edge::Tree(tree_edge) => Some((tree_edge, edge.source())), // todo!
            _ => None,
        })
        .next()
}

fn children<'s, C>(graph: &Graph<'s, C>, node_idx: NodeIndex) -> impl Iterator<Item = NodeIndex> {
    graph
        .edges_directed(node_idx, Direction::Outgoing)
        .filter_map(|edge| match edge.weight() {
            Edge::Tree(_tree_edge) => Some(edge.target()),
            _ => None,
        })
}

fn scan_rec<'s, C: CharT + Copy>(
    graph: &Graph<'s, C>,
    node_idx: NodeIndex,
    t: &'s AStr<C>,
) -> ScanReturn<'s, C> {
    if let Some(ch) = t.first() {
        if let Some((edge, edge_target, _)) = child(graph, node_idx, *ch) {
            let lcp_len = string::lcp(&t[1..], &edge.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge.chars.len()) {
                Ordering::Equal => scan_rec(graph, edge_target, &t[edge.chars.len()..]),
                Ordering::Less => match lcp_len.cmp(&t.len()) {
                    Ordering::Equal => ScanReturn {
                        upper: node_idx,
                        lower: edge_target,
                        t_rem_matched: t,
                        matched: ScanMatch::FullMatch,
                    },
                    Ordering::Less => ScanReturn {
                        upper: node_idx,
                        lower: edge_target,
                        t_rem_matched: &t[..lcp_len],
                        matched: ScanMatch::MaximalNonFullMatch {
                            t_unmatched: &t[lcp_len..],
                        },
                    },
                    Ordering::Greater => {
                        unreachable!()
                    }
                },
                Ordering::Greater => {
                    unreachable!()
                }
            }
        } else {
            ScanReturn {
                upper: node_idx,
                lower: node_idx,
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node_idx,
            t_rem_matched: AStr::empty(),
            lower: node_idx,
            matched: ScanMatch::FullMatch,
        }
    }
}

fn fast_scan_rec<'s, C: CharT + Copy>(
    graph: &Graph<'s, C>,
    node_idx: NodeIndex,
    t: &'s AStr<C>,
) -> ScanReturn<'s, C> {
    if let Some(ch) = t.first() {
        if let Some((edge, edge_target, _)) = child(graph, node_idx, *ch) {
            if t.len() < edge.chars.len() {
                ScanReturn {
                    upper: node_idx,
                    t_rem_matched: t,
                    lower: edge_target,
                    matched: ScanMatch::FullMatch,
                }
            } else {
                fast_scan_rec(graph, edge_target, &t[edge.chars.len()..])
            }
        } else {
            ScanReturn {
                upper: node_idx,
                lower: node_idx,
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node_idx,
            t_rem_matched: AStr::empty(),
            lower: node_idx,
            matched: ScanMatch::FullMatch,
        }
    }
}

impl<'s, C: CharT + Copy> SuffixTrie<'s, C> {
    /// Finds indexes of given string in the string represented in the trie
    pub fn indexes_substr(&self, t: &'s AStr<C>) -> HashSet<usize> {
        let mut result = HashSet::new();

        let scan_ret = scan_rec(&self.graph, self.root, t);
        if let ScanReturn {
            lower,
            matched: ScanMatch::FullMatch,
            ..
        } = scan_ret
        {
            terminals(&self.graph, lower, |suffix| {
                result.insert(suffix);
            });
        }

        result
    }

    /// Finds index of maximal prefixes of given string
    pub fn index_substr_maximal(&self, t: &'s AStr<C>) -> MaximalSubstrMatch {
        match scan_rec(&self.graph, self.root, t) {
            ScanReturn {
                lower,
                matched: ScanMatch::FullMatch,
                ..
            } => MaximalSubstrMatch::full(single_terminal(&self.graph, lower), t.len()),
            ScanReturn {
                lower,
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched },
                ..
            } => MaximalSubstrMatch::partial(
                single_terminal(&self.graph, lower),
                t.len() - t_unmatched.len(),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MaximalSubstrMatch {
    pub index: usize,
    pub length: usize,
    pub matched: Matched,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Matched {
    Full,
    Partial,
}

impl MaximalSubstrMatch {
    fn full(index: usize, length: usize) -> Self {
        Self {
            index,
            length,
            matched: Matched::Full,
        }
    }

    fn partial(index: usize, length: usize) -> Self {
        Self {
            index,
            length,
            matched: Matched::Partial,
        }
    }
}

pub(crate) fn terminals<'s, C>(
    graph: &Graph<'s, C>,
    node: NodeIndex,
    mut callback: impl FnMut(usize),
) {
    terminals_rec(graph, node, &mut callback)
}

fn terminals_rec<'s, C>(
    graph: &Graph<'s, C>,
    node_idx: NodeIndex,
    callback: &mut impl FnMut(usize),
) {
    let node = &graph[node_idx];
    if let Some(terminal) = &node.terminal {
        callback(terminal.suffix_index);
    }
    for child_idx in children(graph, node_idx) {
        terminals_rec(graph, child_idx, callback);
    }
}

fn single_terminal<'s, C>(graph: &Graph<'s, C>, node: NodeIndex) -> usize {
    single_terminal_rec(graph, node)
}

fn single_terminal_rec<'s, C>(graph: &Graph<'s, C>, node_idx: NodeIndex) -> usize {
    let node = &graph[node_idx];
    if let Some(terminal) = &node.terminal {
        terminal.suffix_index
    } else {
        single_terminal_rec(
            graph,
            children(graph, node_idx)
                .next()
                .expect("must have edge if not terminal"),
        )
    }
}

/// Builds suffix trie
pub fn build_trie_with_allocator<'s, C: CharT + Copy>(s: &'s AStr<C>) -> SuffixTrie<'s, C> {
    let mut graph = StableDiGraph::new();
    let root = graph.add_node(Node::default());

    append_tail_with_terminal(&mut graph, 0, root, s);

    let mut head_tail = HeadTail {
        head: root,
        tail: s,
    };

    for i in 1..s.len() {
        head_tail = insert_suffix(&mut graph, i, head_tail);
    }

    // print_histogram("head length", &head_length);

    SuffixTrie { graph, root, s }
}

struct HeadTail<'s, C> {
    head: NodeIndex,
    tail: &'s AStr<C>,
}

fn insert_suffix<'s, C: CharT + Copy>(
    graph: &mut Graph<'s, C>,
    suffix_index: usize,
    prev_head_tail: HeadTail<'s, C>,
) -> HeadTail<'s, C> {
    let (to_suffix_base_node, to_suffix_str, is_head) =
        if let Some((parent_edge, parent_idx)) = parent(graph, prev_head_tail.head) {
            let (to_s_prev_head_base_node, to_s_prev_head_str) =
                if parent(graph, parent_idx).is_some() {
                    (
                        suffix(graph, parent_idx).expect("suffix"),
                        parent_edge.chars,
                    )
                } else {
                    (parent_idx, &parent_edge.chars[1..])
                };

            let ScanReturn {
                upper,
                t_rem_matched: rem_matched,
                matched: ScanMatch::FullMatch,
                ..
            } = fast_scan_rec(graph, to_s_prev_head_base_node, to_s_prev_head_str)
            else {
                panic!("should be full match");
            };
            let rem_matched = rem_matched.to_owned();

            let (s_prev_head, is_head) = if rem_matched.is_empty() {
                (upper, false)
            } else {
                (insert_intermediate(graph, upper, &rem_matched), true)
            };

            graph.add_edge(prev_head_tail.head, s_prev_head, Edge::Suffix);

            (s_prev_head, prev_head_tail.tail, is_head)
        } else {
            (prev_head_tail.head, &prev_head_tail.tail[1..], false)
        };

    let (head, tail) = if is_head {
        (to_suffix_base_node, to_suffix_str)
    } else {
        let (upper, to_head_str, tail) = match scan_rec(graph, to_suffix_base_node, to_suffix_str) {
            ScanReturn {
                upper,
                t_rem_matched,
                matched: ScanMatch::FullMatch,
                ..
            } => (upper, t_rem_matched, AStr::empty()),
            ScanReturn {
                upper,
                t_rem_matched,
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched },
                ..
            } => (upper, t_rem_matched, t_unmatched),
        };

        let head = if to_head_str.is_empty() {
            upper
        } else {
            insert_intermediate(graph, upper, to_head_str)
        };

        (head, tail)
    };

    append_tail_with_terminal(graph, suffix_index, head, tail);

    HeadTail { head, tail }
}

/// Precondition: `t_rem` (or first char of) does not exist on edge from `node`
fn append_tail_with_terminal<'s, C: CharT + Copy>(
    graph: &mut Graph<'s, C>,
    suffix_index: usize,
    node_idx: NodeIndex,
    t_rem: &'s AStr<C>,
) {
    let node_mut = &mut graph[node_idx];
    if t_rem.is_empty() {
        node_mut.terminal = Some(Terminal { suffix_index });
    } else {
        let mut new_node = Node::default();
        new_node.terminal = Some(Terminal { suffix_index });
        let new_node_idx = graph.add_node(new_node);
        let edge = Edge::Tree (TreeEdge{ chars: t_rem });
        graph.add_edge(node_idx, new_node_idx, edge);
    }
}

/// Precondition: `t_rem` exists on edge from `node`
fn insert_intermediate<'s, C: CharT + Copy>(
    graph: &mut Graph<'s, C>,
    node_idx: NodeIndex,
    t_rem: &AStr<C>,
) -> NodeIndex {
    assert!(!t_rem.is_empty());
    let (edge, edge_target, edge_idx) = child(graph, node_idx, t_rem[0])
        .expect("edge must exist");

    let new_edge = Edge::Tree(TreeEdge {
        chars: &edge.chars[..t_rem.len()],
    });
    let new_node = Node::default();
    let edge_remainder = Edge::Tree( TreeEdge {
        chars: &edge.chars[t_rem.len()..],
    });

    graph.remove_edge(edge_idx);
    let new_node_idx = graph.add_node(new_node);
    graph.add_edge(node_idx, new_node_idx, new_edge);
    graph.add_edge(new_node_idx, edge_target, edge_remainder);

    new_node_idx
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(NodeIndex);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.index(), f)
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

        let trie = build_trie_with_allocator(s);

        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let trie = build_trie_with_allocator(s);

        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([0])
        );
        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[A, A])),
            HashSet::from([0, 1])
        );
        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[A])),
            HashSet::from([0, 1, 2])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let trie = build_trie_with_allocator(s);

        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[A, A, A])),
            HashSet::from([])
        );
        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[A, B, A])),
            HashSet::from([0, 3, 5])
        );
        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[B, A, A])),
            HashSet::from([1, 6])
        );
    }

    #[test]
    fn test_find_maximal_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let trie = build_trie_with_allocator(s);

        assert_eq!(
            trie.index_substr_maximal(AStr::from_slice(&[A, B, A])),
            MaximalSubstrMatch::full(5, 3),
        );
        assert_eq!(
            trie.index_substr_maximal(AStr::from_slice(&[B, A, A])),
            MaximalSubstrMatch::full(6, 3),
        );
        assert_eq!(
            trie.index_substr_maximal(AStr::from_slice(&[A, A, A])),
            MaximalSubstrMatch::partial(7, 2),
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let trie = build_trie_with_allocator(&s);
            let expected = string::indexes(&s, &t);
            let indexes = trie.indexes_substr( &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
