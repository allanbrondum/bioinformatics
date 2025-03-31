use bioinformatics::polymers::DnaNt;
use std::cell::RefCell;

use bioinformatics::alphabet_model::CharT;
use bioinformatics::string::suffix_trie_mcc_arena::{traverse_ancestors, traverse_descendants, AncestorVisit, AncestorVisitor, DescendantVisit, DescendantVisitor, Node, NodeId};
use bioinformatics::string::suffix_trie_mcc_arena;
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use bumpalo::Bump;
use hashbrown::HashMap;
use std::collections::VecDeque;
use std::ptr;
use std::str::FromStr;
use generic_array::ArrayLength;

fn main() {
    let mut lines = lines_file("src/bin/g_lrep_data.txt");

    let dna =
        AString::<DnaNt>::from_str(&lines.next().unwrap().strip_suffix("$").unwrap()).unwrap();
    let k: usize = lines.next().unwrap().parse().unwrap();

    let bump = Bump::new();
    let trie = suffix_trie_mcc_arena::build_trie_with_allocator(&dna, &bump);

    if false {
        let mut node_leaf_count = HashMap::<_, usize>::new();

        let mut to_visit = VecDeque::new();
        to_visit.push_front(trie.root);

        while let Some(node) = to_visit.pop_front() {
            let node_ref = node.borrow();
            if node_ref.terminal.is_some() {
                let mut node_parent_iter = node;
                loop {
                    *node_leaf_count
                        .entry(ptr::from_ref(node_parent_iter))
                        .or_default() += 1;
                    if let Some(parent) = node_parent_iter.borrow().parent {
                        node_parent_iter = parent.borrow().source;
                    } else {
                        break;
                    }
                }
            }

            for child_edge in node_ref.children.iter().flat_map(|child| child.iter()) {
                to_visit.push_back(child_edge.borrow().target);
            }
        }

        struct ToVisit<'arena, 's> {
            node: &'arena RefCell<Node<'arena, 's, DnaNt, <DnaNt as CharT>::AlphabetSize>>,
            depth: usize,
        }

        let mut to_visit = VecDeque::new();
        to_visit.push_front(ToVisit {
            node: trie.root,
            depth: 0,
        });

        struct NodeAndDepth<'arena, 's> {
            node: &'arena RefCell<Node<'arena, 's, DnaNt, <DnaNt as CharT>::AlphabetSize>>,
            depth: usize,
        }

        let mut deepest_node = NodeAndDepth {
            node: trie.root,
            depth: 0,
        };

        while let Some(node) = to_visit.pop_front() {
            if node_leaf_count
                .get(&ptr::from_ref(node.node))
                .copied()
                .unwrap_or_default()
                < k
            {
                continue;
            };

            if node.depth > deepest_node.depth {
                deepest_node = NodeAndDepth {
                    node: node.node,
                    depth: node.depth,
                };
            }

            for child_edge in node
                .node
                .borrow()
                .children
                .iter()
                .flat_map(|child| child.iter())
            {
                let child_edge_ref = child_edge.borrow();
                to_visit.push_back(ToVisit {
                    node: child_edge_ref.target,
                    depth: node.depth + child_edge_ref.chars.len(),
                });
            }
        }

        let mut dna = AString::default();
        let mut node_parent_iter = deepest_node.node;
        while let Some(parent) = node_parent_iter.borrow().parent {
            let parent_ref = parent.borrow();
            dna = parent_ref.chars.to_owned() + dna.as_str();
            node_parent_iter = parent_ref.source;
        }


        println!("{} ", dna);
    } else {
        #[derive(Default)]
        struct CountVisitor<'arena, 's>(HashMap::<NodeId<'arena, 's, DnaNt, DnaNt::AlphabetSize>, usize>);
        let mut count_visitor = CountVisitor::default();
        impl<'arena, 's> AncestorVisitor<'arena, 's, (), DnaNt> for CountVisitor<'arena, 's> {
            fn visit(&mut self, context: (), visit: AncestorVisit<'arena, 's, DnaNt, DnaNt::AlphabetSize>) -> () {
                *self.0
                    .entry(visit.node)
                    .or_default() += 1;
            }
        }

        struct CountDescVisitor;
        impl DescendantVisitor<'_, '_, (), DnaNt> for CountDescVisitor {
            fn visit(&mut self, _context: (), visit: DescendantVisit<'_, '_, DnaNt>) {
                if visit.terminal.is_some() {
                    traverse_ancestors(visit.node, (), &mut count_visitor);
                }
            }
        }

        traverse_descendants(trie.root(), CountDescVisitor);
    }
}
