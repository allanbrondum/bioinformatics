//! McCreight algorithm

use crate::alphabet_model::{CharT, WithSeparator};
use crate::string_model::{AStr, AString};
use generic_array::{ArrayLength, GenericArray};

use crate::string;
use generic_array::typenum::{Add1, B1, Unsigned};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Write;
use std::ops::{Add, DerefMut};
use std::path::Path;
use std::rc::Rc;
use std::{iter, mem, ptr};

const GRAPH_DEBUG: bool = false;

#[derive(Debug)]
pub struct SuffixTrie<'s, C: CharT> {
    root: Rc<RefCell<Node<'s, C>>>,
}

#[derive(Debug)]
struct Node<'s, C: CharT> {
    parent: Option<Rc<RefCell<Edge<'s, C>>>>,
    children: GenericArray<Option<Rc<RefCell<Edge<'s, C>>>>, C::AlphabetSize>,
    terminal: Option<Terminal>,
    suffix: Option<Rc<RefCell<Node<'s, C>>>>,
}

impl<'s, C: CharT> Default for Node<'s, C> {
    fn default() -> Self {
        Self {
            parent: None,
            children: Default::default(),
            terminal: None,
            suffix: None,
        }
    }
}

impl<'s, C: CharT> Node<'s, C> {
    fn with_parent(parent: Rc<RefCell<Edge<'s, C>>>) -> Self {
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
struct Edge<'s, C: CharT> {
    chars: &'s AStr<C>,
    source: Rc<RefCell<Node<'s, C>>>,
    target: Rc<RefCell<Node<'s, C>>>,
}

struct ScanReturn<'s, C: CharT> {
    upper: Rc<RefCell<Node<'s, C>>>,
    lower: Rc<RefCell<Node<'s, C>>>,
    t_rem_matched: &'s AStr<C>,
    matched: ScanMatch<'s, C>,
}

enum ScanMatch<'s, C: CharT> {
    FullMatch,
    MaximalNonFullMatch { t_unmatched: &'s AStr<C> },
}

fn scan_rec<'s, C: CharT>(node: &Rc<RefCell<Node<'s, C>>>, t: &'s AStr<C>) -> ScanReturn<'s, C> {
    let node_ref = node.borrow();
    if let Some(ch) = t.first() {
        if let Some(edge) = &node_ref.children[ch.index()] {
            let edge_ref = edge.borrow();
            let lcp_len = string::lcp(&t[1..], &edge_ref.chars[1..]).len() + 1;

            match lcp_len.cmp(&edge_ref.chars.len()) {
                Ordering::Equal => scan_rec(&edge_ref.target, &t[edge_ref.chars.len()..]),
                Ordering::Less => match lcp_len.cmp(&t.len()) {
                    Ordering::Equal => ScanReturn {
                        upper: node.clone(),
                        lower: edge_ref.target.clone(),
                        t_rem_matched: t,
                        matched: ScanMatch::FullMatch,
                    },
                    Ordering::Less => ScanReturn {
                        upper: node.clone(),
                        lower: edge_ref.target.clone(),
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
                upper: node.clone(),
                lower: node.clone(),
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node.clone(),
            t_rem_matched: AStr::empty(),
            lower: node.clone(),
            matched: ScanMatch::FullMatch,
        }
    }
}

fn fast_scan_rec<'s, C: CharT>(
    node: &Rc<RefCell<Node<'s, C>>>,
    t: &'s AStr<C>,
) -> ScanReturn<'s, C> {
    let node_ref = node.borrow();
    if let Some(ch) = t.first() {
        if let Some(edge) = &node_ref.children[ch.index()] {
            let edge_ref = edge.borrow();
            if t.len() < edge_ref.chars.len() {
                ScanReturn {
                    upper: node.clone(),
                    t_rem_matched: t,
                    lower: edge_ref.target.clone(),
                    matched: ScanMatch::FullMatch,
                }
            } else {
                fast_scan_rec(&edge_ref.target, &t[edge_ref.chars.len()..])
            }
        } else {
            ScanReturn {
                upper: node.clone(),
                lower: node.clone(),
                t_rem_matched: AStr::empty(),
                matched: ScanMatch::MaximalNonFullMatch { t_unmatched: t },
            }
        }
    } else {
        ScanReturn {
            upper: node.clone(),
            t_rem_matched: AStr::empty(),
            lower: node.clone(),
            matched: ScanMatch::FullMatch,
        }
    }
}

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr<'s, C: CharT>(trie: &SuffixTrie<'s, C>, t: &'s AStr<C>) -> HashSet<usize> {
    let mut result = HashSet::new();

    let scan_ret = scan_rec(&trie.root, t);
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MaximalSubstrMatch {
    index: usize,
    length: usize,
    matched: Matched,
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

/// Finds indexes of given string in the string represented in the trie
pub fn indexes_substr_maximal<'s, C: CharT>(
    trie: &SuffixTrie<'s, C>,
    t: &'s AStr<C>,
) -> HashSet<MaximalSubstrMatch> {
    let mut result = HashSet::new();

    match scan_rec(&trie.root, t) {
        ScanReturn {
            lower,
            matched: ScanMatch::FullMatch,
            ..
        } => {
            terminals(&lower.borrow(), |suffix| {
                result.insert(MaximalSubstrMatch::full(suffix, t.len()));
            });
        }
        ScanReturn {
            lower,
            matched: ScanMatch::MaximalNonFullMatch { t_unmatched },
            ..
        } => {
            terminals(&lower.borrow(), |suffix| {
                result.insert(MaximalSubstrMatch::partial(
                    suffix,
                    t.len() - t_unmatched.len(),
                ));
            });
        }
    }

    result
}

fn terminals<'s, C: CharT>(node: &Node<'s, C>, mut callback: impl FnMut(usize)) {
    terminals_rec(node, &mut callback)
}

fn terminals_rec<'s, C: CharT>(node: &Node<'s, C>, mut callback: &mut impl FnMut(usize)) {
    if let Some(terminal) = &node.terminal {
        callback(terminal.suffix_index);
    }
    for edge in node.children.iter().filter_map(|edge| edge.as_ref()) {
        terminals_rec(&edge.borrow().target.borrow(), callback);
    }
}

/// Builds suffix trie
pub fn build_trie<'s, C: CharT>(s: &'s AStr<C>) -> SuffixTrie<'s, C> {
    let trie = SuffixTrie {
        root: Rc::new(RefCell::new(Node::default())),
    };

    insert_tail(0, &trie.root, s);

    let mut head_tail = HeadTail {
        head: trie.root.clone(),
        tail: s,
    };

    for i in 1..s.len() {
        head_tail = insert_suffix(i, head_tail);

        if GRAPH_DEBUG {
            to_dot(format!("target/trie_suffix_link_{}.dot", i), &trie);
        }
    }

    if GRAPH_DEBUG {
        to_dot("target/trie_suffix_link.dot", &trie);
    }

    trie
}

struct HeadTail<'s, C: CharT> {
    head: Rc<RefCell<Node<'s, C>>>,
    tail: &'s AStr<C>,
}

fn insert_suffix<C: CharT>(suffix_index: usize, prev_head_tail: HeadTail<C>) -> HeadTail<C> {
    let parent_edge = prev_head_tail.head.borrow().parent.clone();
    let (to_suffix_base_node, to_suffix_str, is_head) = if let Some(parent_edge) = parent_edge {
        let parent_edge_ref = parent_edge.borrow();
        let parent_ref = parent_edge_ref.source.borrow();

        let (to_s_prev_head_base_node, to_s_prev_head_str) = if parent_ref.parent.is_some() {
            (
                parent_ref.suffix.as_ref().expect("suffix"),
                parent_edge_ref.chars,
            )
        } else {
            (&parent_edge_ref.source, &parent_edge_ref.chars[1..])
        };

        let ScanReturn {
            upper,
            t_rem_matched: rem_matched,
            matched: ScanMatch::FullMatch,
            ..
        } = fast_scan_rec(to_s_prev_head_base_node, to_s_prev_head_str)
        else {
            panic!("should be full match");
        };
        let rem_matched = rem_matched.to_owned();
        drop(parent_ref);
        drop(parent_edge_ref);

        let (s_prev_head, is_head) = if rem_matched.is_empty() {
            (upper, false)
        } else {
            (insert_intermediate(&upper, &rem_matched), true)
        };

        prev_head_tail.head.borrow_mut().suffix = Some(s_prev_head.clone());

        (s_prev_head, prev_head_tail.tail, is_head)
    } else {
        (
            prev_head_tail.head.clone(),
            &prev_head_tail.tail[1..],
            false,
        )
    };

    let (head, tail) = if is_head {
        (to_suffix_base_node, to_suffix_str)
    } else {
        let (upper, to_head_str, tail) = match scan_rec(&to_suffix_base_node, to_suffix_str) {
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
            insert_intermediate(&upper, to_head_str)
        };

        (head, tail)
    };

    insert_tail(suffix_index, &head, tail);

    HeadTail { head, tail }
}

/// Precondition: `t_rem` does not exists on edge from `node`
fn insert_tail<'s, C: CharT>(
    suffix_index: usize,
    node: &Rc<RefCell<Node<'s, C>>>,
    t_rem: &'s AStr<C>,
) {
    let mut node_mut = node.borrow_mut();
    if t_rem.is_empty() {
        node_mut.terminal = Some(Terminal { suffix_index });
    } else {
        let edge = Rc::new(RefCell::new(Edge {
            chars: t_rem,
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
fn insert_intermediate<'s, C: CharT>(
    node: &Rc<RefCell<Node<'s, C>>>,
    t_rem: &AStr<C>,
) -> Rc<RefCell<Node<'s, C>>> {
    assert!(!t_rem.is_empty());
    let node_mut = node.borrow_mut();
    let edge = node_mut.children[t_rem[0].index()]
        .as_ref()
        .expect("edge must exist");
    let mut edge_mut = edge.borrow_mut();

    let new_edge = Edge {
        chars: &edge_mut.chars[..t_rem.len()],
        source: node.clone(),
        target: Rc::new(RefCell::new(Node::with_parent(edge.clone()))),
    };

    let edge_remainder = Rc::new(RefCell::new(mem::replace(edge_mut.deref_mut(), new_edge)));
    let mut edge_remainder_mut = edge_remainder.borrow_mut();
    edge_remainder_mut.chars = &edge_remainder_mut.chars[t_rem.len()..];
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
            "    \"{}\" -> \"{}\" [label=\"$\" dir=none];",
            node_id(node),
            ptr::from_ref(terminal) as usize,
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

pub fn lcs_trie_with_separator<C: CharT>(s: &AStr<C>, t: &AStr<C>) -> AString<C>
where
    C::AlphabetSize: Unsigned + Add<B1>,
    Add1<C::AlphabetSize>: Debug + ArrayLength,
{
    let separated: AString<_> = s
        .iter()
        .copied()
        .map(WithSeparator::Char)
        .chain(iter::once(WithSeparator::Separator))
        .chain(t.iter().copied().map(WithSeparator::Char))
        .collect();

    let trie = build_trie(&separated);

    todo!()
}

pub fn lcs_trie<'a, C: CharT>(s: &AStr<C>, t: &'a AStr<C>) -> &'a AStr<C> {
    let trie = build_trie(s);

    let mut substr: &AStr<C> = AStr::empty();
    for i in 0..t.len() {
        if t.len() - i <= substr.len() {
            break;
        }

        if let Some(m) = indexes_substr_maximal(&trie, &t[i..]).into_iter().next() {
            if m.length > substr.len() {
                substr = &t[i..i + m.length];
            }
        }
    }

    substr
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::string;
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;

    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, prop_assert_eq, proptest};

    #[test]
    fn test_build_trie_and_find_substr_empty() {
        let s: &AStr<Char> = AStr::from_slice(&[]);

        let trie = build_trie(s);

        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[])),
            HashSet::from([0])
        );
    }

    #[test]
    fn test_build_trie_and_find_substr_repetition() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, A, A]);

        let trie = build_trie(s);

        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[A, A, A])),
            HashSet::from([0])
        );
        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[A, A])),
            HashSet::from([0, 1])
        );
        assert_eq!(
            indexes_substr(&trie, AStr::from_slice(&[A])),
            HashSet::from([0, 1, 2])
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

    #[test]
    fn test_find_maximal_substr() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[A, B, A, A, B, A, B, A, A]);

        let trie = build_trie(s);

        assert_eq!(
            indexes_substr_maximal(&trie, AStr::from_slice(&[A, B, A])),
            HashSet::from([
                MaximalSubstrMatch::full(0, 3),
                MaximalSubstrMatch::full(3, 3),
                MaximalSubstrMatch::full(5, 3)
            ])
        );
        assert_eq!(
            indexes_substr_maximal(&trie, AStr::from_slice(&[B, A, A])),
            HashSet::from([
                MaximalSubstrMatch::full(1, 3),
                MaximalSubstrMatch::full(6, 3)
            ])
        );
        assert_eq!(
            indexes_substr_maximal(&trie, AStr::from_slice(&[A, A, A])),
            HashSet::from([
                MaximalSubstrMatch::partial(2, 2),
                MaximalSubstrMatch::partial(7, 2)
            ])
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

    #[test]
    fn test_lcs_trie() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(lcs_trie(s, t), AStr::from_slice(&[B, A, A, B, A]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_lcs_trie(s in arb_astring::<Char>(0..20), t in arb_astring::<Char>(0..20)) {
            let expected = string::lcs(&s, &t);
            let lcs = lcs_trie(&s, &t);
            prop_assert_eq!(lcs.len(), expected.len());
            prop_assert!(s.contains(lcs));
            prop_assert!(t.contains(lcs));
        }
    }

    #[test]
    fn test_lcs_trie_with_separator() {
        use crate::string_model::test_util::Char::*;

        let s = AStr::from_slice(&[B, A, B, A, A, B, A, B, A, A]);
        let t = AStr::from_slice(&[B, B, A, A, B, A, A, A, A, B]);

        assert_eq!(
            lcs_trie_with_separator(s, t),
            AStr::from_slice(&[B, A, A, B, A])
        );
    }
}

