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

impl<C: CharT> Node<C> {
    fn root() -> Self {
        Self {
            parent: None,
            children: Default::default(),
            terminal: None,
            suffix: None,
        }
    }

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
        root: Rc::new(RefCell::new(Node::root())),
    };

    trie.root.borrow_mut().suffix = Some(trie.root.clone());

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

/// Returns (head(suffix_index), tail(suffix_index))
fn insert_suffix<C: CharT>(suffix_index: usize, prev_head_tail: HeadTail<C>) -> HeadTail<C> {
    let prev_head_ref = prev_head_tail.head.borrow();
    if let Some(parent) = prev_head_ref.parent.as_ref() {
        let parent_ref = parent.borrow();
        let s_parent_prev_head = parent_ref.source.borrow().suffix.clone().expect("suffix");

        let v_tail = parent_ref.chars.to_owned() + prev_head_tail.tail.as_str();

        todo!()
        // insert_rec(suffix_index, &s_parent_prev_head, &v_tail)
    } else {
        todo!()
        // insert_rec(suffix_index, &prev_head_tail.head, &prev_head_tail.tail)
    }
}

/// Returns head(suffix_index)
// fn insert_rec< C: CharT>(
//     suffix_index: usize,
//     node: &Rc<RefCell<Node<C>>>,
//     s: & AStr<C>,
// ) -> HeadTail< C> {
//     let mut node_mut = node.borrow_mut();
//     if let Some(ch) = s.first() {
//         if let Some(edge) = &mut node_mut.children[ch.index()] {
//             let mut edge_mut = edge.borrow_mut();
//             let lcp_len = string::lcp(&s[1..], &edge_mut.chars[1..]).len() + 1;
//
//             if lcp_len == edge_mut.chars.len() {
//                 insert_rec(suffix_index, &edge_mut.target, &s[edge_mut.chars.len()..])
//             } else if lcp_len < edge_mut.chars.len() {
//                 let new_node = Node::with_parent(node.clone());
//                 let new_edge = Edge {
//                     chars: edge_mut.chars[..lcp_len].to_owned(),
//                     source: node.clone(),
//                     target: Rc::new(RefCell::new(new_node)),
//                 };
//                 let  edge_remainder = Rc::new(RefCell::new(mem::replace(edge_mut.deref_mut(), new_edge)));
//                 let mut edge_remainder_mut = edge_remainder.borrow_mut();
//                 edge_remainder_mut.chars = edge_remainder_mut.chars[lcp_len..].to_owned();
//                 edge_remainder_mut.source = edge_mut.target.clone();
//                 edge_remainder_mut.target.borrow_mut().parent = Some(edge_mut.target.clone());
//                 let rem_ch = edge_remainder_mut.chars[0];
//                 edge_mut.target.borrow_mut().children[rem_ch.index()] = Some(edge_remainder);
//
//                 insert_rec(suffix_index, &edge_mut.target, &s[lcp_len..]);
//             } else {
//                 unreachable!()
//             }
//         } else {
//             let mut new_node = Node::with_parent(node.clone());
//             new_node.terminal = Some(Terminal { suffix_index });
//             node_mut.children[ch.index()] = Some(Box::new(Edge {
//                 chars: s.to_owned(),
//                 target: Rc::new(RefCell::new(new_node)),
//             }));
//         }
//     } else {
//         node_mut.terminal = Some(Terminal { suffix_index });
//     }
// }

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
