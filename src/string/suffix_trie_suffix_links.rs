//! McCreight algorithm

use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use generic_array::GenericArray;

use crate::string;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;
use std::path::Path;
use std::rc::Rc;
use std::{mem, ptr};

const GRAPH_DEBUG: bool = true;

#[derive(Debug)]
pub struct SuffixTrie<C: CharT> {
    root: Rc<RefCell<Node<C>>>,
}

#[derive(Debug)]
struct Node<C: CharT> {
    parent: Option<Rc<RefCell<Edge<C>>>>,
    children: GenericArray<Option<Rc<RefCell<Edge<C>>>>, C::AlphabetSize>,
    terminal: Option<Terminal>,
    suffix: Option<Rc<RefCell<Node<C>>>>,
}

impl<C: CharT> Default for Node<C> {
    fn default() -> Self {
        Self {
            parent: None,
            children: Default::default(),
            terminal: None,
            suffix: None,
        }
    }
}

impl<C: CharT> Node<C> {
    fn with_parent(parent: Rc<RefCell<Edge<C>>>) -> Self {
        Self {
            parent: Some(parent),
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
    chars: AString<C>,
    source: Rc<RefCell<Node<C>>>,
    target: Rc<RefCell<Node<C>>>,
}

enum ScanReturn<'b, C: CharT> {
    FullMatch {
        upper: Rc<RefCell<Node<C>>>,
        t_rem_matched: &'b AStr<C>,
        lower: Rc<RefCell<Node<C>>>,
    },
    MaximalNonFullMatch {
        max: Rc<RefCell<Node<C>>>,
        t_rem_matched: &'b AStr<C>,
        t_unmatched: &'b AStr<C>,
    },
}

// todo add fastscan
fn scan_rec<'b, C: CharT>(node: &Rc<RefCell<Node<C>>>, t: &'b AStr<C>) -> ScanReturn<'b, C> {
    let node_ref = node.borrow();
    if let Some(ch) = t.first() {
        if let Some(edge) = &node_ref.children[ch.index()] {
            let edge_ref = edge.borrow();
            let lcp_len = string::lcp(&t[1..], &edge_ref.chars[1..]).len() + 1;

            if lcp_len == edge_ref.chars.len() {
                scan_rec(&edge_ref.target, &t[edge_ref.chars.len()..])
            } else if lcp_len < edge_ref.chars.len() {
                if lcp_len == t.len() {
                    ScanReturn::FullMatch {
                        upper: node.clone(),
                        t_rem_matched: t,
                        lower: edge_ref.target.clone(),
                    }
                } else if lcp_len < t.len() {
                    ScanReturn::MaximalNonFullMatch {
                        max: node.clone(),
                        t_rem_matched: &t[..lcp_len],
                        t_unmatched: &t[lcp_len..],
                    }
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        } else {
            ScanReturn::MaximalNonFullMatch {
                max: node.clone(),
                t_rem_matched: AStr::empty(),
                t_unmatched: t,
            }
        }
    } else {
        ScanReturn::FullMatch {
            upper: node.clone(),
            t_rem_matched: AStr::empty(),
            lower: node.clone(),
        }
    }
}

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<C: CharT>(trie: &SuffixTrie<C>, t: &AStr<C>) -> HashSet<usize> {
    let mut result = HashSet::new();

    let scan_ret = scan_rec(&trie.root, t);
    if let ScanReturn::FullMatch { lower, .. } = scan_ret {
        terminals_rec(&lower.borrow(), &mut result);
    }

    result
}

fn terminals_rec<C: CharT>(node: &Node<C>, result: &mut HashSet<usize>) {
    if let Some(terminal) = &node.terminal {
        result.insert(terminal.suffix_index);
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        terminals_rec(&edge.borrow().target.borrow(), result);
    }
}

/// Builds suffix trie
pub fn build_trie<C: CharT>(s: &AStr<C>) -> SuffixTrie<C> {
    let mut trie = SuffixTrie {
        root: Rc::new(RefCell::new(Node::default())),
    };

    let mut head_tail = HeadTail {
        head: trie.root.clone(),
        tail: s.to_owned(),
    };

    for i in 0..s.len() {
        head_tail = insert_suffix(i, head_tail);
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_suffix_link.dot", &trie);
    }

    trie
}

struct HeadTail<C: CharT> {
    head: Rc<RefCell<Node<C>>>,
    tail: AString<C>,
}

fn insert_suffix<C: CharT>(suffix_index: usize, prev_head_tail: HeadTail<C>) -> HeadTail<C> {
    let mut prev_head_mut_guard = prev_head_tail.head.borrow_mut();
    let mut prev_head_mut = prev_head_mut_guard.deref_mut();
    let (to_suffix_base_node, to_suffix_str) = if let Some(parent_edge) = prev_head_mut.parent.as_ref() {
        let parent_edge_ref = parent_edge.borrow();
        let parent_ref = parent_edge_ref.source.borrow();

        let (to_s_prev_head_base_node, to_s_prev_head_str) = if parent_ref.parent.is_some() {
            (
                parent_ref.suffix.as_ref().expect("suffix"),
                parent_edge_ref.chars.as_str(),
            )
        } else {
            (&parent_edge_ref.source, &parent_edge_ref.chars[1..])
        };

        let ScanReturn::FullMatch {
            upper,
            t_rem_matched: rem_matched,
            lower: _lower,
        } = scan_rec(to_s_prev_head_base_node, to_s_prev_head_str)
        else {
            panic!("should be full match");
        };

        let s_prev_head = if rem_matched.len() == 0 {
            upper
        } else {
            insert_intermediate(&upper, rem_matched)
        };

        prev_head_mut.suffix = Some(s_prev_head.clone());

        (s_prev_head, prev_head_tail.tail.as_str())
    } else {
        (prev_head_tail.head.clone(), &prev_head_tail.tail[1..])
    };

    let (upper, to_head_str, tail) = match scan_rec(&to_suffix_base_node, to_suffix_str) {
        ScanReturn::FullMatch {
            upper,
            t_rem_matched,
            lower: _lower,
        } => (upper, t_rem_matched, AStr::empty()),
        ScanReturn::MaximalNonFullMatch {
            max,
            t_rem_matched,
            t_unmatched,
        } => (max, t_rem_matched, t_unmatched),
    };

    let head = if to_head_str.len() == 0 {
        upper
    } else {
        insert_intermediate(&upper, to_head_str)
    };

    insert_tail(suffix_index, &head, tail);

    HeadTail {
        head,
        tail: tail.to_owned(),
    }
}

/// Precondition: `t_rem` does not exists on edge from `node`
fn insert_tail<C: CharT>(suffix_index: usize, node: &Rc<RefCell<Node<C>>>, t_rem: &AStr<C>) {
    let mut node_mut = node.borrow_mut();
    if t_rem.is_empty() {
        node.borrow_mut().terminal = Some(Terminal { suffix_index });
    } else {
        let edge = Rc::new(RefCell::new(Edge {
            chars: t_rem.to_owned(),
            source: node.clone(),
            target: Rc::new(RefCell::new(Node::default())),
        }));
        let mut new_node = Node::with_parent(edge.clone());
        new_node.terminal = Some(Terminal { suffix_index });
        edge.borrow_mut().target = Rc::new(RefCell::new(new_node));
        node_mut.children[t_rem[0].index()] = Some(edge);
    }
}

/// Precondition: `t_rem` exists on edge from `node`
fn insert_intermediate<C: CharT>(
    node: &Rc<RefCell<Node<C>>>,
    t_rem: &AStr<C>,
) -> Rc<RefCell<Node<C>>> {
    assert!(!t_rem.is_empty());
    let node_mut = node.borrow_mut();
    let edge = node_mut.children[t_rem[0].index()]
        .as_ref()
        .expect("edge must exist");
    let mut edge_mut = edge.borrow_mut();

    let new_edge = Edge {
        chars: edge_mut.chars[..t_rem.len()].to_owned(),
        source: node.clone(),
        target: Rc::new(RefCell::new(Node::with_parent(edge.clone()))),
    };

    let edge_remainder = Rc::new(RefCell::new(mem::replace(edge_mut.deref_mut(), new_edge)));
    let mut edge_remainder_mut = edge_remainder.borrow_mut();
    edge_remainder_mut.chars = edge_remainder_mut.chars[t_rem.len()..].to_owned();
    edge_remainder_mut.source = edge_mut.target.clone();
    edge_remainder_mut.target.borrow_mut().parent = Some(edge_remainder.clone());
    let rem_ch = edge_remainder_mut.chars[0];
    drop(edge_remainder_mut);
    edge_mut.target.borrow_mut().children[rem_ch.index()] = Some(edge_remainder);

    edge_mut.target.clone()
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
            node_id(&suffix.borrow()),
        )
        .unwrap();
    }
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
