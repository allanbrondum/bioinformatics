use crate::alphabet_model::{AlphabetT, CharT};
use generic_array::GenericArray;

#[derive(Debug)]
pub struct SuffixTrie<A: AlphabetT> {
    root: Node<A>,
}

#[derive(Debug)]
struct Node<A: AlphabetT> {
    children: GenericArray<Option<Box<Edge<A>>>, A::N>,
    terminal: Option<Terminal>,
}

impl<A: AlphabetT> Node<A> {
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
struct Edge<A: AlphabetT> {
    char: A::Char,
    target: Node<A>,
}

pub fn build<A: AlphabetT>(s: &[A::Char]) -> SuffixTrie<A> {
    let mut trie = SuffixTrie {
        root: Node::new(),
    };

    for i in 0..s.len() - 1 {
        insert_rec(i, &s[i..], &mut trie.root);
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
            let mut edge = Edge {
                char: *ch,
                target: Node::new(),
            };
            insert_rec(suffix, s_rest, &mut edge.target);
            node.children[ch.index()] = Some(Box::new(edge));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::alphabet_model::{AlphabetT, CharT};
    use generic_array::typenum::U2;

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

    struct Alphabet;

    impl AlphabetT for Alphabet {
        type N = U2;
        type Char = Char;
    }
}
