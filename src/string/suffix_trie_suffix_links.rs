//! McCreight algorithm

use crate::alphabet_model::CharT;
use generic_array::GenericArray;
use petgraph::matrix_graph::Nullable;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use crate::string_model::AStr;

const GRAPH_DEBUG: bool = false;

#[derive(Debug)]
pub struct SuffixTrie<C: CharT> {
    root: Rc<RefCell<Node<C>>>,
}

#[derive(Debug)]
struct Node<C: CharT> {
    children: GenericArray<Option<Box<Edge<C>>>, C::AlphabetSize>,
    terminal: Option<Terminal>,
    suffix: Option<Rc<RefCell<Node<C>>>>,
}

impl<C: CharT> Node<C> {
    fn new() -> Self {
        Self {
            children: Default::default(),
            terminal: None,
            suffix: None,
        }
    }
}

#[derive(Debug)]
struct Terminal {
    suffix_index: usize,
}


#[derive(Debug)]
struct Edge<C: CharT> {
    chars: Vec<C>,
    target: Rc<RefCell<Node<C>>>,
}

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<C: CharT>(trie: &SuffixTrie<C>, t: &AStr<C>) -> HashSet<usize> {
    let mut result = HashSet::new();

    indexes_substr_rec(&trie.root.borrow(), t, &mut result);

    result
}

fn indexes_substr_rec<C: CharT>(node: &Node<C>, t: &AStr<C>, result: &mut HashSet<usize>) {
    if let Some(ch) = t.first() {
        if let Some(edge) = &node.children[ch.index()] {
            if t.len() <= edge.chars.len() {
                if t[1..] == edge.chars[1..t.len()] {
                    terminals_rec(&edge.target.borrow(), result);
                }
            } else {
                if t[1..edge.chars.len()] == edge.chars[1..] {
                    indexes_substr_rec(&edge.target.borrow(), AStr::from_slice(&t[edge.chars.len()..]), result);
                }
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
        terminals_rec(&edge.target.borrow(), result);
    }
}

/// Builds suffix trie
pub fn build_trie<C: CharT>(s: &AStr<C>) -> SuffixTrie<C> {
    assert_ne!(s.len(), 0);

    let mut trie = SuffixTrie {
        root: Rc::new(RefCell::new(Node::new())),
    };

    let mut root_mut = trie.root.borrow_mut();
    // root_mut.suffix = Some(trie.root.clone());
    let mut node_0 = Node::new();
    node_0.terminal = Some(Terminal {
        suffix_index: 0,
    });
    root_mut.children[s[0].index()] = Some(Box::new(Edge {
        chars: s.to_vec(),
        target: Rc::new(RefCell::new(node_0)),
    }));
    drop(root_mut);

    for i in 1..s.len() {
        insert_rec(i, AStr::from_slice(&s[i..]), &mut trie.root.borrow_mut());
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_suffix.dot", &trie);
    }

    trie
}

fn insert_rec<C: CharT>(suffix: usize, s: &AStr<C>, node: &mut Node<C>) {
    todo!()
    // if let Some(ch) = t.first() {
    //
    //
    // }
    // match s.split_first() {
    //     None => {
    //         assert!(node.terminal.is_none());
    //         node.terminal = Some(Terminal {
    //             suffix_index: suffix,
    //         });
    //     }
    //     Some((ch, s_rest)) => {
    //         let edge = node.children[ch.index()].get_or_insert_with(|| {
    //             Box::new(Edge {
    //                 char: *ch,
    //                 target: Rc::new(RefCell::new(Node::new())),
    //             })
    //         });
    //         insert_rec(suffix, s_rest, &mut edge.target.borrow_mut());
    //     }
    // }
}

fn to_dot<C: CharT>(filepath: impl AsRef<Path>, trie: &SuffixTrie<C>) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root.borrow());

    writeln!(file, "}}").unwrap();
}

fn node_id<C: CharT>(node: &Node<C>) -> impl Display {
    ptr::from_ref(node) as usize
}

fn to_dot_rec<C: CharT>(write: &mut impl Write, node: &Node<C>) {
    todo!()
    // writeln!(
    //     write,
    //     "    {} [label=\"{}\" shape=point];",
    //     node_id(node),
    //     ""
    // )
    // .unwrap();
    // if let Some(terminal) = &node.terminal {
    //     writeln!(
    //         write,
    //         "    {} [label=\"{}\"];",
    //         ptr::from_ref(terminal) as usize,
    //         terminal.suffix_index
    //     )
    //     .unwrap();
    //     writeln!(
    //         write,
    //         "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
    //         node_id(node),
    //         ptr::from_ref(terminal) as usize,
    //         '$'
    //     )
    //     .unwrap();
    // }
    // if let Some(suffix) = &node.suffix {
    //     writeln!(
    //         write,
    //         "    \"{}\" -> \"{}\" [style=dashed];",
    //         node_id(node),
    //         ptr::from_ref(&suffix.borrow()) as usize,
    //     )
    //     .unwrap();
    // }
    // for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
    //     writeln!(
    //         write,
    //         "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
    //         node_id(node),
    //         node_id(&edge.target.borrow()),
    //         edge.char
    //     )
    //     .unwrap();
    //     to_dot_rec(write, &edge.target.borrow());
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alphabet_model::CharT;
    use crate::string;
    use crate::string::test_util::Char;
    use generic_array::typenum::U2;
    use proptest::arbitrary::any;
    use proptest::collection::vec;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};
    use proptest_derive::Arbitrary;
    use std::fmt::{Debug, Display, Formatter};

    #[test]
    fn test_build_trie_and_find_substr() {
        use crate::string::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let trie = build_trie(&s);

        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[])),
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(indexes_substr(&trie, AStr::from_slice(&[A, A, A])), HashSet::from([]));
        assert_eq!(indexes_substr(&trie, AStr::from_slice(&[A, B, A])), HashSet::from([0, 3, 5]));
        assert_eq!(indexes_substr(&trie, AStr::from_slice(&[B, A, A])), HashSet::from([1, 6]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_trie(s in vec(any::<Char>(), 0..20), t in vec(any::<Char>(), 3)) {
            let trie = build_trie(AStr::from_slice(&s));
            let expected = string::indexes(AStr::from_slice(&s), AStr::from_slice(&t));
            let indexes = indexes_substr(&trie, AStr::from_slice(&t));
            prop_assert_eq!(indexes, HashSet::from_iter(expected.into_iter()));
        }
    }
}
