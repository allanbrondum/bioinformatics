//! McCreight algorithm

use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use core::fmt::Debug;
use generic_array::{ArrayLength, GenericArray};

use crate::string;
use std::cmp::Ordering;

use hashbrown::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{mem, ptr};

const GRAPH_DEBUG: bool = false;

#[derive(Debug)]
pub struct SuffixTrie<'s, C: CharT> {
    pub root: Node<'s, C, C::AlphabetSize>,
}

#[derive(Debug)]
pub struct Node<'s, C, N: ArrayLength> {
    pub children: GenericArray<Option<Box<Edge<'s, C, N>>>, N>,
    terminal: Option<Terminal>,
}

impl<'s, C, N: ArrayLength> Node<'s, C, N> {
    fn new() -> Self {
        Self {
            children: Default::default(),
            terminal: None,
        }
    }
}

#[derive(Debug)]
struct Terminal {
    str_index: usize,
}

#[derive(Debug)]
pub struct Edge<'s, C, N: ArrayLength> {
    pub chars: &'s AStr<C>,
    pub target: Node<'s, C, N>,
}

#[derive(Debug)]
struct ScanReturn<'a, 's, 't, C, N: ArrayLength> {
    #[allow(unused)]
    upper: &'a Node<'s, C, N>,
    lower: &'a Node<'s, C, N>,
    t_rem_matched: &'t AStr<C>,
    matched: ScanMatch<'t, C>,
}

#[derive(Debug)]
enum ScanMatch<'t, C> {
    FullMatch,
    MaximalNonFullMatch { t_unmatched: &'t AStr<C> },
}

fn scan_rec<'a, 's, 't, C: CharT>(
    node: &'a Node<'s, C, C::AlphabetSize>,
    t: &'t AStr<C>,
) -> ScanReturn<'a, 's, 't, C, C::AlphabetSize> {
    if let Some(ch) = t.first() {
        if let Some(edge) = &node.children[ch.index()] {
            let lcp_len = string::lcp(&t[1..], &edge.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge.chars.len()) {
                Ordering::Equal => scan_rec(&edge.target, &t[edge.chars.len()..]),
                Ordering::Less => match lcp_len.cmp(&t.len()) {
                    Ordering::Equal => ScanReturn {
                        upper: node,
                        lower: &edge.target,
                        t_rem_matched: t,
                        matched: ScanMatch::FullMatch,
                    },
                    Ordering::Less => ScanReturn {
                        upper: node,
                        lower: &edge.target,
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

impl<'s, C: CharT + Debug> SuffixTrie<'s, C> {
    /// Finds indexes of given string in the string represented in the trie
    pub fn find(&self, t: &AStr<C>) -> Option<usize> {
        let scan_ret = scan_rec(&self.root, t);
        if let ScanMatch::FullMatch = scan_ret.matched {
            if scan_ret.t_rem_matched.is_empty() {
                scan_ret.lower.terminal.as_ref().map(|term| term.str_index)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn terminals_rec<'s, C, N: ArrayLength>(node: &Node<'s, C, N>, result: &mut HashSet<usize>) {
    if let Some(terminal) = &node.terminal {
        result.insert(terminal.str_index);
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        terminals_rec(&edge.target, result);
    }
}

/// Builds suffix trie
pub fn build_trie<'s, C: CharT>(strs: impl IntoIterator<Item = &'s AStr<C>>) -> SuffixTrie<'s, C> {
    let mut trie = SuffixTrie { root: Node::new() };

    for (i, str) in strs.into_iter().enumerate() {
        insert_rec(i, str, &mut trie.root);
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_comp.dot", &trie);
    }

    trie
}

fn insert_rec<'s, C: CharT>(
    str_index: usize,
    s: &'s AStr<C>,
    node: &mut Node<'s, C, C::AlphabetSize>,
) {
    if let Some(ch) = s.first() {
        if let Some(edge) = &mut node.children[ch.index()] {
            let lcp_len = string::lcp(&s[1..], &edge.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge.chars.len()) {
                Ordering::Equal => insert_rec(str_index, &s[edge.chars.len()..], &mut edge.target),
                Ordering::Less => {
                    let new_node = Node::new();
                    let new_edge = Edge {
                        chars: &edge.chars[..lcp_len],
                        target: new_node,
                    };
                    let mut edge_remainder = mem::replace(edge, Box::new(new_edge));
                    edge_remainder.chars = &edge_remainder.chars[lcp_len..];
                    let rem_ch = edge_remainder.chars[0];
                    edge.target.children[rem_ch.index()] = Some(edge_remainder);

                    insert_rec(str_index, &s[lcp_len..], &mut edge.target);
                }
                Ordering::Greater => {
                    unreachable!()
                }
            }
        } else {
            let mut new_node = Node::new();
            new_node.terminal = Some(Terminal { str_index });
            node.children[ch.index()] = Some(Box::new(Edge {
                chars: s,
                target: new_node,
            }));
        }
    } else {
        node.terminal = Some(Terminal {
            str_index: str_index,
        });
    }
}

fn to_dot<'s, C: CharT>(filepath: impl AsRef<Path>, trie: &SuffixTrie<'s, C>) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root);

    writeln!(file, "}}").unwrap();
}

fn node_id<'s, C, N: ArrayLength>(node: &Node<'s, C, N>) -> impl Display {
    ptr::from_ref(node) as usize
}

fn to_dot_rec<'s, C: CharT>(write: &mut impl Write, node: &Node<'s, C, C::AlphabetSize>) {
    writeln!(write, "    {} [label=\"\" shape=point];", node_id(node)).unwrap();
    if let Some(terminal) = &node.terminal {
        writeln!(
            write,
            "    {} [label=\"{}\"];",
            ptr::from_ref(terminal) as usize,
            terminal.str_index
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
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;
    use proptest::collection::vec;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, proptest};

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string_model::test_util::Char::*;

        let s = [
            AStr::from_slice(&[A, B, A, A]),
            AStr::from_slice(&[B, A, B, B, A]),
        ];

        let trie = build_trie(s);

        assert_eq!(trie.find(AStr::from_slice(&[A, B, A])), None);
        assert_eq!(trie.find(AStr::from_slice(&[A, B, A, A])), Some(0));
        assert_eq!(trie.find(AStr::from_slice(&[A, B, A, A, A])), None);

        assert_eq!(trie.find(AStr::from_slice(&[B, A, B, B])), None);
        assert_eq!(trie.find(AStr::from_slice(&[B, A, B, B, A])), Some(1));
        assert_eq!(trie.find(AStr::from_slice(&[B, A, B, B, A, A])), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(strs in vec(arb_astring::<Char>(0..20), 2..10)) {
            let trie = build_trie(strs.iter().map(|str|str.as_str()));
            for str in &strs {
                prop_assert!(trie.find(&str).is_some());
            }
        }
    }
}
