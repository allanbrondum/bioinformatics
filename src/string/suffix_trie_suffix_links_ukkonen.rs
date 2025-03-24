//! Ukkonen's algorithm

use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use generic_array::{ArrayLength, GenericArray};

use crate::string;

use crate::util::print_histogram;
use bumpalo::Bump;
use hashbrown::HashSet;
use hdrhistogram::Histogram;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::Write;

use std::ops::DerefMut;
use std::path::Path;
use std::{mem, ptr};

const GRAPH_DEBUG: bool = false;

#[derive(Debug)]
pub struct SuffixTrie<'arena, 's, C: CharT> {
    pub(crate) root: &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    s: &'s AStr<C>,
}

#[derive(Debug)]
pub(crate) struct Node<'arena, 's, C, N: ArrayLength> {
    pub(crate) parent: Option<&'arena RefCell<Edge<'arena, 's, C, N>>>,
    pub(crate) children: GenericArray<Option<&'arena RefCell<Edge<'arena, 's, C, N>>>, N>,
    pub(crate) terminal: Option<Terminal>,
    // suffix: Option<&'arena RefCell<Node<'arena, 's, C, N>>>,
}

impl<'arena, 's, C, N: ArrayLength> Default for Node<'arena, 's, C, N> {
    fn default() -> Self {
        Self {
            parent: None,
            children: Default::default(),
            terminal: None,
            // suffix: None,
        }
    }
}

impl<'arena, 's, C, N: ArrayLength> Node<'arena, 's, C, N> {
    fn with_parent(parent: &'arena RefCell<Edge<'arena, 's, C, N>>) -> Self {
        Self {
            parent: Some(parent),
            children: Default::default(),
            terminal: None,
            // suffix: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Terminal {
    pub(crate) suffix_index: usize,
}

#[derive(Debug)]
pub(crate) struct Edge<'arena, 's, C, N: ArrayLength> {
    pub(crate) chars: &'s AStr<C>,
    pub(crate) source: &'arena RefCell<Node<'arena, 's, C, N>>,
    pub(crate) target: &'arena RefCell<Node<'arena, 's, C, N>>,
}

struct ScanReturn<'arena, 's, C, N: ArrayLength> {
    upper: &'arena RefCell<Node<'arena, 's, C, N>>,
    lower: &'arena RefCell<Node<'arena, 's, C, N>>,
    t_rem_matched: &'s AStr<C>,
    matched: ScanMatch<'s, C>,
}

enum ScanMatch<'s, C> {
    FullMatch,
    MaximalNonFullMatch { t_unmatched: &'s AStr<C> },
}

fn scan_rec<'arena, 's, C: CharT + Copy>(
    node: &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    t: &'s AStr<C>,
) -> ScanReturn<'arena, 's, C, C::AlphabetSize> {
    let node_ref = node.borrow();
    if let Some(ch) = t.first() {
        if let Some(edge) = &node_ref.children[ch.index()] {
            let edge_ref = edge.borrow();
            let lcp_len = string::lcp(&t[1..], &edge_ref.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge_ref.chars.len()) {
                Ordering::Equal => scan_rec(edge_ref.target, &t[edge_ref.chars.len()..]),
                Ordering::Less => match lcp_len.cmp(&t.len()) {
                    Ordering::Equal => ScanReturn {
                        upper: node,
                        lower: edge_ref.target,
                        t_rem_matched: t,
                        matched: ScanMatch::FullMatch,
                    },
                    Ordering::Less => ScanReturn {
                        upper: node,
                        lower: edge_ref.target,
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
                upper: node,
                lower: node,
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node,
            t_rem_matched: AStr::empty(),
            lower: node,
            matched: ScanMatch::FullMatch,
        }
    }
}

fn fast_scan_rec<'arena, 's, C: CharT + Copy>(
    node: &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    t: &'s AStr<C>,
) -> ScanReturn<'arena, 's, C, C::AlphabetSize> {
    let node_ref = node.borrow();
    if let Some(ch) = t.first() {
        if let Some(edge) = &node_ref.children[ch.index()] {
            let edge_ref = edge.borrow();
            if t.len() < edge_ref.chars.len() {
                ScanReturn {
                    upper: node,
                    t_rem_matched: t,
                    lower: edge_ref.target,
                    matched: ScanMatch::FullMatch,
                }
            } else {
                fast_scan_rec(edge_ref.target, &t[edge_ref.chars.len()..])
            }
        } else {
            ScanReturn {
                upper: node,
                lower: node,
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node,
            t_rem_matched: AStr::empty(),
            lower: node,
            matched: ScanMatch::FullMatch,
        }
    }
}

impl<'arena, 's, C: CharT + Copy> SuffixTrie<'arena, 's, C> {
    /// Finds indexes of given string in the string represented in the trie
    pub fn indexes_substr(&self, t: &'s AStr<C>) -> HashSet<usize> {
        let mut result = HashSet::new();

        let scan_ret = scan_rec(self.root, t);
        if let ScanReturn {
            lower,
            matched: ScanMatch::FullMatch,
            ..
        } = scan_ret
        {
            terminals(&lower.borrow(), |suffix| {
                result.insert(suffix);
            });
        }

        result
    }
}

pub(crate) fn terminals<'arena, 's, C, N: ArrayLength>(
    node: &Node<'arena, 's, C, N>,
    mut callback: impl FnMut(usize),
) {
    terminals_rec(node, &mut callback)
}

fn terminals_rec<'arena, 's, C, N: ArrayLength>(
    node: &Node<'arena, 's, C, N>,
    callback: &mut impl FnMut(usize),
) {
    if let Some(terminal) = &node.terminal {
        callback(terminal.suffix_index);
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        terminals_rec(&edge.borrow().target.borrow(), callback);
    }
}

fn single_terminal<'arena, 's, C, N: ArrayLength>(node: &Node<'arena, 's, C, N>) -> usize {
    single_terminal_rec(node)
}

fn single_terminal_rec<'arena, 's, C, N: ArrayLength>(node: &Node<'arena, 's, C, N>) -> usize {
    if let Some(terminal) = &node.terminal {
        terminal.suffix_index
    } else {
        single_terminal_rec(
            &node
                .children
                .iter()
                .filter_map(|edge| edge.as_ref())
                .next()
                .expect("must have edge if not terminal")
                .borrow()
                .target
                .borrow(),
        )
    }
}

/// Builds suffix trie
pub fn build_trie_with_allocator<'arena, 's, C: CharT + Copy>(
    s: &'s AStr<C>,
    alloc: &'arena Bump,
) -> SuffixTrie<'arena, 's, C> {
    let trie = SuffixTrie {
        root: alloc.alloc(RefCell::new(Node::default())),
        s,
    };

    let mut suffixes = Vec::new();

    for i in 0..s.len() {
        let chars = &s[..i];

        for suffix in &mut suffixes {
            *suffix = insert_last_char(*suffix, chars, alloc);
        }

        suffixes.push(insert_last_char(&trie.root, chars, alloc));
    }

    for (suffix_index, suffix) in suffixes.iter().enumerate() {
        suffix.borrow_mut().terminal = Some(Terminal { suffix_index });
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_suffix_link.dot", &trie);
    }

    trie
}

/// Inserts single char (last in given string) relative to given node. Returns
/// resulting node.
fn insert_last_char<'arena, 's, C: CharT + Copy>(
    node: &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    chars: &'s AStr<C>,
    alloc: &'arena Bump,
) -> &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>> {
    let ch = chars.last().copied().expect("chars must be non empty");

    let node_ref = node.borrow();
    if let Some(edge) = &node_ref.children[ch.index()] {
        let edge_ref = edge.borrow();
        todo!()
    } else if node_ref.children.is_empty() && node_ref.parent.is_some() {
        // Extend this leaf
        let mut parent_edge_mut = node_ref.parent.unwrap().borrow_mut();
        parent_edge_mut.chars = &chars[chars.len() - parent_edge_mut.chars.len() - 1..];
        node
    } else {
        // Insert new leaf on edge from this node
        append_tail(node, &chars[chars.len() - 1..], alloc)
    }
}

/// Precondition: `t_rem` (or first char of) does not exist on edge from `node`
fn append_tail<'arena, 's, C: CharT + Copy>(
    node: &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    chars: &'s AStr<C>,
    alloc: &'arena Bump,
) -> &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>> {
    assert!(!chars.is_empty());

    let mut node_mut = node.borrow_mut();

    let edge = alloc.alloc(RefCell::new(Edge {
        chars,
        source: node,
        target: alloc.alloc(RefCell::new(Node::default())),
    }));
    let new_node = Node::with_parent(edge);
    let new_node_ref = alloc.alloc(RefCell::new(new_node));
    edge.borrow_mut().target = new_node_ref;
    node_mut.children[chars[0].index()] = Some(edge);

    new_node_ref
}

/// Precondition: `t_rem` exists on edge from `node`
fn insert_intermediate<'arena, 's, C: CharT + Copy>(
    node: &&'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>>,
    t_rem: &AStr<C>,
    alloc: &'arena Bump,
) -> &'arena RefCell<Node<'arena, 's, C, C::AlphabetSize>> {
    assert!(!t_rem.is_empty());
    let node_mut = node.borrow_mut();
    let edge = node_mut.children[t_rem[0].index()]
        .as_ref()
        .expect("edge must exist");
    let mut edge_mut = edge.borrow_mut();

    let new_edge = Edge {
        chars: &edge_mut.chars[..t_rem.len()],
        source: node,
        target: alloc.alloc(RefCell::new(Node::with_parent(edge))),
    };

    let edge_remainder = alloc.alloc(RefCell::new(mem::replace(edge_mut.deref_mut(), new_edge)));
    let mut edge_remainder_mut = edge_remainder.borrow_mut();
    edge_remainder_mut.chars = &edge_remainder_mut.chars[t_rem.len()..];
    edge_remainder_mut.source = edge_mut.target;
    edge_remainder_mut.target.borrow_mut().parent = Some(edge_remainder);
    let rem_ch = edge_remainder_mut.chars[0];
    drop(edge_remainder_mut);
    edge_mut.target.borrow_mut().children[rem_ch.index()] = Some(edge_remainder);

    edge_mut.target
}

struct HeadTail<'arena, 's, C, N: ArrayLength> {
    head: &'arena RefCell<Node<'arena, 's, C, N>>,
    tail: &'s AStr<C>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(usize);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

pub(crate) fn node_id<C, N: ArrayLength>(node: &Node<C, N>) -> NodeId {
    NodeId(ptr::from_ref(node) as usize)
}

pub(crate) fn node_id_ptr<C, N: ArrayLength>(node: *const Node<C, N>) -> NodeId {
    NodeId(node as usize)
}

pub(crate) fn to_dot<'arena, 's, C: CharT>(
    filepath: impl AsRef<Path>,
    trie: &SuffixTrie<'arena, 's, C>,
) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root.borrow());

    writeln!(file, "}}").unwrap();
}

fn to_dot_rec<'arena, 's, C: CharT>(
    write: &mut impl Write,
    node: &Node<'arena, 's, C, C::AlphabetSize>,
) {
    writeln!(write, "    {} [label=\"\" shape=point];", node_id(node)).unwrap();
    if let Some(terminal) = &node.terminal {
        writeln!(
            write,
            "    {} [label=\"{}\"];",
            ptr::from_ref(terminal) as usize,
            terminal.suffix_index
        )
        .unwrap();
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [label=\"$\" dir=none];",
            node_id(node),
            ptr::from_ref(terminal) as usize,
        )
        .unwrap();
    }
    // if let Some(suffix) = &node.suffix {
    //     writeln!(
    //         write,
    //         "    \"{}\" -> \"{}\" [style=dashed];",
    //         node_id(node),
    //         node_id(&suffix.borrow()),
    //     )
    //     .unwrap();
    // }
    // if let Some(parent) = &node.parent {
    //     writeln!(
    //         write,
    //         "    \"{}\" -> \"{}\" [style=dashed label=\"parent\"];",
    //         node_id(node),
    //         node_id(&parent.borrow()),
    //     )
    //         .unwrap();
    // }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        let edge_ref = edge.borrow();
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
            node_id(node),
            node_id(&edge_ref.target.borrow()),
            edge_ref.chars
        )
        .unwrap();
        to_dot_rec(write, &edge_ref.target.borrow());
    }
}

pub fn trie_stats<'arena, 's, C: CharT>(trie: &SuffixTrie<'arena, 's, C>) {
    let mut edge_len_hist =
        Histogram::<u64>::new_with_bounds(1, trie.s.len().max(2) as u64, 2).unwrap();
    let mut node_branch_depth_hist = Histogram::<u64>::new(2).unwrap();

    struct ToVisit<'arena, 's, C, N: ArrayLength> {
        node: &'arena RefCell<Node<'arena, 's, C, N>>,
        branch_depth: usize,
    }

    let mut to_visit = VecDeque::new();
    to_visit.push_front(ToVisit {
        node: &trie.root,
        branch_depth: 0,
    });

    while let Some(node) = to_visit.pop_front() {
        for child_edge in node
            .node
            .borrow()
            .children
            .iter()
            .filter_map(|child| child.as_ref())
        {
            let child_edge_ref = child_edge.borrow();
            edge_len_hist
                .record(child_edge_ref.chars.len() as u64)
                .unwrap();
            node_branch_depth_hist
                .record(node.branch_depth as u64)
                .unwrap();

            to_visit.push_back(ToVisit {
                node: child_edge_ref.target,
                branch_depth: node.branch_depth + 1,
            });
        }
    }

    println!(
        "nodes: {}, edges: {}",
        node_branch_depth_hist.len(),
        edge_len_hist.len()
    );
    print_histogram("edge length", &edge_len_hist);
    print_histogram("node branch depth", &node_branch_depth_hist);
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

        let bump = Bump::new();
        let trie = build_trie_with_allocator(s, &bump);

        assert_eq!(
            trie.indexes_substr(AStr::from_slice(&[])),
            HashSet::from([0])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let bump = Bump::new();
        let trie = build_trie_with_allocator(s, &bump);

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

        let bump = Bump::new();
        let trie = build_trie_with_allocator(s, &bump);

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

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let bump = Bump::new();
        let trie = build_trie_with_allocator(&s, &bump);
            let expected = string::indexes(&s, &t);
            let indexes = trie.indexes_substr( &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
