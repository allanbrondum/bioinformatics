use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use generic_array::GenericArray;

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ptr;

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
    char: C,
    target: Node<C>,
}

enum ScanReturn<'a, 'b, C: CharT> {
    FullMatch {
        node: &'a Node<C>,
    },
    MaximalNotFullMatch {
        node: &'a Node<C>,
        t_rest: &'b AStr<C>,
    },
}

fn scan_rec<'a, 'b, C: CharT>(node: &'a Node<C>, t: &'b AStr<C>) -> ScanReturn<'a, 'b, C> {
    if let Some((ch, t_rest)) = t.split_first() {
        if let Some(edge) = &node.children[ch.index()] {
            scan_rec(&edge.target, AStr::from_slice(t_rest))
        } else {
            ScanReturn::MaximalNotFullMatch { node, t_rest: t }
        }
    } else {
        ScanReturn::FullMatch { node }
    }
}

enum ScanReturnMut<'a, 'b, C: CharT> {
    FullMatch {
        node: &'a mut Node<C>,
    },
    MaximalNotFullMatch {
        node: &'a mut Node<C>,
        t_rest: &'b AStr<C>,
    },
}

fn scan_rec_mut<'a, 'b, C: CharT>(
    node: &'a mut Node<C>,
    t: &'b AStr<C>,
) -> ScanReturnMut<'a, 'b, C> {
    if let Some((ch, t_rest)) = t.split_first() {
        if node.children[ch.index()].is_some() {
            scan_rec_mut(
                &mut node.children[ch.index()].as_mut().unwrap().target,
                AStr::from_slice(t_rest),
            )
        } else {
            ScanReturnMut::MaximalNotFullMatch { node, t_rest: t }
        }
    } else {
        ScanReturnMut::FullMatch { node }
    }
}

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<C: CharT>(trie: &SuffixTrie<C>, t: &AStr<C>) -> HashSet<usize> {
    let mut result = HashSet::new();

    let scan_ret = scan_rec(&trie.root, t);
    if let ScanReturn::FullMatch { node } = scan_ret {
        terminals_rec(node, &mut result);
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
        match scan_rec_mut(&mut trie.root, &s[i..]) {
            ScanReturnMut::FullMatch { node } => {
                insert_rec(i, node, AStr::empty() );
            }
            ScanReturnMut::MaximalNotFullMatch { node, t_rest } => {
                insert_rec(i,node, t_rest, );
            }
        }
    }

    if GRAPH_DEBUG {
        to_dot("target/trie.dot", &trie);
    }

    trie
}

fn insert_rec<C: CharT>(suffix: usize,  node: &mut Node<C>, s: &AStr<C>) {
    match s.split_first() {
        None => {
            assert!(node.terminal.is_none());
            node.terminal = Some(Terminal {
                suffix_index: suffix,
            });
        }
        Some((ch, s_rest)) => {
            let edge = node.children[ch.index()].get_or_insert_with(|| {
                Box::new(Edge {
                    char: *ch,
                    target: Node::new(),
                })
            });
            insert_rec(suffix,  &mut edge.target, AStr::from_slice(s_rest));
        }
    }
}

fn to_dot<C: CharT>(filepath: impl AsRef<Path>, trie: &SuffixTrie<C>) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root);

    writeln!(file, "}}").unwrap();
}

fn to_dot_rec<C: CharT>(write: &mut impl Write, node: &Node<C>) {
    writeln!(
        write,
        "    {} [label=\"\" shape=point];",
        ptr::from_ref(node) as usize,
    )
    .unwrap();
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
            "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
            ptr::from_ref(node) as usize,
            ptr::from_ref(terminal) as usize,
            '$'
        )
        .unwrap();
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
            ptr::from_ref(node) as usize,
            ptr::from_ref(&edge.target) as usize,
            edge.char
        )
        .unwrap();
        to_dot_rec(write, &edge.target);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

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
