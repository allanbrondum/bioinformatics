//! McCreight algorithm

use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use generic_array::GenericArray;

use crate::string;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{mem, ptr};

const GRAPH_DEBUG: bool = false;

#[derive(Debug)]
pub struct SuffixTrie<C: CharT> {
    root: Node<C>,
}

#[derive(Debug)]
struct Node<C: CharT> {
    children: GenericArray<Option<Box<Edge<C>>>, C::AlphabetSize>,
    terminal: Option<Terminal>,
}

impl<C: CharT> Node<C> {
    fn new() -> Self {
        Self {
            children: Default::default(),
            terminal: None,
        }
    }
}

#[derive(Debug)]
struct Terminal {
    suffix_index: usize,
}

#[derive(Debug)]
struct Edge<C: CharT> {
    chars: AString<C>,
    target: Node<C>,
}

enum ScanReturn<'a, 'b, C: CharT> {
    FullMatch {
        upper: &'a Node<C>,
        lower: &'a Node<C>,
    },
    MaximalNonFullMatch {
        max: &'a Node<C>,
        t_unmatched: &'b AStr<C>,
    },
}

fn scan_rec<'a, 'b, C: CharT>(node: &'a Node<C>, t: &'b AStr<C>) -> ScanReturn<'a, 'b, C> {
    if let Some(ch) = t.first() {
        if let Some(edge) = &node.children[ch.index()] {
            let lcp_len = string::lcp(&t[1..], &edge.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge.chars.len()) {
                Ordering::Equal => scan_rec(&edge.target, &t[edge.chars.len()..]),
                Ordering::Less => match lcp_len.cmp(&t.len()) {
                    Ordering::Equal => ScanReturn::FullMatch {
                        upper: node,
                        lower: &edge.target,
                    },
                    Ordering::Less => ScanReturn::MaximalNonFullMatch {
                        max: node,
                        t_unmatched: &t[lcp_len..],
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
            ScanReturn::MaximalNonFullMatch {
                max: node,
                t_unmatched: t,
            }
        }
    } else {
        ScanReturn::FullMatch {
            upper: node,
            lower: node,
        }
    }
}

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<C: CharT>(trie: &SuffixTrie<C>, t: &AStr<C>) -> HashSet<usize> {
    let mut result = HashSet::new();

    let scan_ret = scan_rec(&trie.root, t);
    if let ScanReturn::FullMatch { lower, .. } = scan_ret {
        terminals_rec(lower, &mut result);
    }

    result
}

fn terminals_rec<C: CharT>(node: &Node<C>, result: &mut HashSet<usize>) {
    if let Some(terminal) = &node.terminal {
        result.insert(terminal.suffix_index);
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        terminals_rec(&edge.target, result);
    }
}

/// Builds suffix trie
pub fn build_trie<C: CharT>(s: &AStr<C>) -> SuffixTrie<C> {
    let mut trie = SuffixTrie { root: Node::new() };

    for i in 0..s.len() {
        insert_rec(i, &s[i..], &mut trie.root);
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_comp.dot", &trie);
    }

    trie
}

fn insert_rec<C: CharT>(suffix_index: usize, s: &AStr<C>, node: &mut Node<C>) {
    if let Some(ch) = s.first() {
        if let Some(edge) = &mut node.children[ch.index()] {
            let lcp_len = string::lcp(&s[1..], &edge.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge.chars.len()) {
                Ordering::Equal => {
                    insert_rec(suffix_index, &s[edge.chars.len()..], &mut edge.target)
                }
                Ordering::Less => {
                    let new_node = Node::new();
                    let new_edge = Edge {
                        chars: edge.chars[..lcp_len].to_owned(),
                        target: new_node,
                    };
                    let mut edge_remainder = mem::replace(edge, Box::new(new_edge));
                    edge_remainder.chars = edge_remainder.chars[lcp_len..].to_owned();
                    let rem_ch = edge_remainder.chars[0];
                    edge.target.children[rem_ch.index()] = Some(edge_remainder);

                    insert_rec(suffix_index, &s[lcp_len..], &mut edge.target);
                }
                Ordering::Greater => {
                    unreachable!()
                }
            }
        } else {
            let mut new_node = Node::new();
            new_node.terminal = Some(Terminal { suffix_index });
            node.children[ch.index()] = Some(Box::new(Edge {
                chars: s.to_owned(),
                target: new_node,
            }));
        }
    } else {
        node.terminal = Some(Terminal { suffix_index });
    }
}

fn to_dot<C: CharT>(filepath: impl AsRef<Path>, trie: &SuffixTrie<C>) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root);

    writeln!(file, "}}").unwrap();
}

fn node_id<C: CharT>(node: &Node<C>) -> impl Display {
    ptr::from_ref(node) as usize
}

fn to_dot_rec<C: CharT>(write: &mut impl Write, node: &Node<C>) {
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
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
            node_id(node),
            node_id(&edge.target),
            edge.chars
        )
        .unwrap();
        to_dot_rec(write, &edge.target);
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

        let trie = build_trie(s);

        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[])),
            HashSet::from([])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let trie = build_trie(s);

        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[])),
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[A, A, A])),
            HashSet::from([])
        );
        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[A, B, A])),
            HashSet::from([0, 3, 5])
        );
        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[B, A, A])),
            HashSet::from([1, 6])
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(3)) {
            let trie = build_trie(&s);
            let expected = string::indexes(&s, &t);
            let indexes = indexes_substr(&trie, &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
