//! McCreight algorithm

use crate::alphabet_model::{AlphabetT, CharT};
use generic_array::GenericArray;
use petgraph::matrix_graph::Nullable;
use std::cell::RefCell;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

const GRAPH_DEBUG: bool = true;

#[derive(Debug)]
pub struct SuffixTrie<A: AlphabetT> {
    root: Rc<RefCell<Node<A>>>,
}

#[derive(Debug)]
struct Node<A: AlphabetT> {
    children: GenericArray<Option<Box<Edge<A>>>, A::N>,
    terminal: Option<Terminal>,
    suffix: Option<Rc<RefCell<Node<A>>>>,
}

impl<A: AlphabetT> Node<A> {
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
struct Edge<A: AlphabetT> {
    char: A::Char,
    target: Rc<RefCell<Node<A>>>,
}

pub fn build<A: AlphabetT>(s: &[A::Char]) -> SuffixTrie<A> {
    let mut trie = SuffixTrie {
        root: Rc::new(RefCell::new(Node::new())),
    };

    for i in 0..s.len() - 1 {
        insert_rec(i, &s[i..], &mut trie.root.borrow_mut());
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_suffix.dot", &trie);
    }

    trie
}

fn insert_rec<A: AlphabetT>(suffix: usize, s: &[A::Char], node: &mut Node<A>) {
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
                    target: Rc::new(RefCell::new(Node::new())),
                })
            });
            insert_rec(suffix, s_rest, &mut edge.target.borrow_mut());
        }
    }
}

fn to_dot<A: AlphabetT>(filepath: impl AsRef<Path>, trie: &SuffixTrie<A>) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    to_dot_rec(&mut file, &trie.root.borrow());

    writeln!(file, "}}").unwrap();
}

fn node_id<A: AlphabetT>(node: &Node<A>) -> impl Display {
    ptr::from_ref(node) as usize
}

fn to_dot_rec<A: AlphabetT>(write: &mut impl Write, node: &Node<A>) {
    writeln!(
        write,
        "    {} [label=\"{}\" shape=point];",
        node_id(node),
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
            node_id(node),
            ptr::from_ref(terminal) as usize,
            '$'
        )
        .unwrap();
    }
    if let Some(suffix) = &node.suffix {
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [style=dashed];",
            node_id(node),
            ptr::from_ref(&suffix.borrow()) as usize,
        )
            .unwrap();

    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        writeln!(
            write,
            "    \"{}\" -> \"{}\" [label=\"{}\" dir=none];",
            node_id(node),
            node_id(&edge.target.borrow()),
            edge.char
        )
        .unwrap();
        to_dot_rec(write, &edge.target.borrow());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::alphabet_model::{AlphabetT, CharT};
    use generic_array::typenum::U2;
    use std::fmt::{Debug, Display, Formatter};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    enum Char {
        A,
        B,
    }

    impl CharT for Char {
        fn index(self) -> usize {
            match self {
                Char::A => 0,
                Char::B => 1,
            }
        }
    }

    impl Display for Char {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    #[derive(Debug)]
    struct Alphabet;

    impl AlphabetT for Alphabet {
        type N = U2;
        type Char = Char;
    }

    #[test]
    fn test_build_trie() {
        use Char::*;

        let s = [A, B, A, A, B, A, B, A, A];

        let trie = build::<Alphabet>(&s);
    }
}
