use std::collections::HashSet;
use crate::alphabet_model::CharT;
use generic_array::GenericArray;
use petgraph::matrix_graph::Nullable;
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
    children: GenericArray<Option<Box<Edge<C>>>, C::N>,
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

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<C: CharT>(trie: &SuffixTrie<C>, t: &[C]) -> HashSet<usize> {
    let mut result = HashSet::new();

    indexes_substr_rec(&trie.root, t, &mut result);

    result
}

fn indexes_substr_rec<C: CharT>(node: &Node<C>, t: &[C], result: &mut HashSet<usize>) {
    if let Some((ch, t_rest)) = t.split_first() {
        if let Some(edge) = &node.children[ch.index()] {
            if edge.char.index() == ch.index() {
                indexes_substr_rec(&edge.target, t_rest, result);
            }
        }
    } else {
        terminals_rec(node, result);
    }
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
pub fn build_trie<C: CharT>(s: &[C]) -> SuffixTrie<C> {
    let mut trie = SuffixTrie { root: Node::new() };

    for i in 0..s.len() {
        insert_rec(i, &s[i..], &mut trie.root);
    }

    if GRAPH_DEBUG {
        to_dot("target/trie.dot", &trie);
    }

    trie
}

fn insert_rec<C: CharT>(suffix: usize, s: &[C], node: &mut Node<C>) {
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
            insert_rec(suffix, s_rest, &mut edge.target);
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
        "    {} [label=\"{}\" shape=point];",
        ptr::from_ref(node) as usize,
        ""
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
    use std::collections::HashSet;
    use super::*;

    use crate::string::test_util::Char;
    use proptest::arbitrary::any;
    use proptest::collection::vec;
    use proptest::{prop_assert_eq, proptest};
    use std::fmt::{Debug, Display};
    use proptest::prelude::ProptestConfig;
    use crate::string;

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string::test_util::Char::*;

        let s = [A, B, A, A, B, A, B, A, A];

        let trie = build_trie(&s);

        assert_eq!(indexes_substr(&trie, &[]), HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]));
        assert_eq!(indexes_substr(&trie, &[A, A, A]), HashSet::from([]));
        assert_eq!(indexes_substr(&trie, &[A, B, A]), HashSet::from([0, 3, 5]));
        assert_eq!(indexes_substr(&trie, &[B, A, A]), HashSet::from([1, 6]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in vec(any::<Char>(), 0..20), t in vec(any::<Char>(), 3)) {
            let trie = build_trie(&s);
            let expected = string::indexes_slice(&s, &t);
            let indexes = indexes_substr(&trie, &t);
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
