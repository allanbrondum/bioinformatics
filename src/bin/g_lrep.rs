use bioinformatics::polymers::DnaNt;
use std::cell::RefCell;

use bioinformatics::alphabet_model::CharT;
use bioinformatics::string::suffix_trie_mcc_arena;
use bioinformatics::string::suffix_trie_mcc_arena::{
    AncestorVisit, AncestorVisitor, DescendantVisit, DescendantVisitor, Node, NodeId,
    NodeReference, traverse_ancestors, traverse_descendants,
};
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use bumpalo::Bump;
use generic_array::ArrayLength;
use hashbrown::HashMap;
use std::collections::VecDeque;
use std::ptr;
use std::str::FromStr;

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
    } else if false {
        #[derive(Default)]
        struct CountVisitor(HashMap<NodeId, usize>);
        let count_visitor = CountVisitor::default();
        impl<'arena, 's> AncestorVisitor<'arena, 's, (), DnaNt> for CountVisitor {
            fn visit(
                &mut self,
                _context: (),
                visit: AncestorVisit<'arena, 's, DnaNt>,
            ) -> Option<()> {
                // println!("anc {}", visit.node);
                *self.0.entry(visit.node.node_id()).or_default() += 1;
                Some(())
            }
        }

        struct CountDescVisitor(CountVisitor);
        impl<'arena, 's> DescendantVisitor<'arena, 's, (), DnaNt> for CountDescVisitor {
            fn visit(
                &mut self,
                _context: (),
                visit: DescendantVisit<'arena, 's, DnaNt>,
            ) -> Option<()> {
                if visit.terminal.is_some() {
                    // println!("leaf {} term {}", visit.node, visit.terminal.unwrap());
                    *self.0.0.entry(visit.node.node_id()).or_default() += 1;
                    traverse_ancestors(visit.node, (), &mut self.0);
                }
                Some(())
            }
        }

        let mut count_desc_visitor = CountDescVisitor(count_visitor);
        traverse_descendants(trie.root(), (), &mut count_desc_visitor);

        // println!("{:?}", count_desc_visitor.0.0);

        struct CommonSubstrVisitor<'arena, 's> {
            node: NodeReference<'arena, 's, DnaNt>,
            depth: usize,
            k: usize,
            count_desc_visitor: CountDescVisitor,
        };
        impl<'arena, 's> DescendantVisitor<'arena, 's, usize, DnaNt> for CommonSubstrVisitor<'arena, 's> {
            fn visit(
                &mut self,
                parent_depth: usize,
                visit: DescendantVisit<'arena, 's, DnaNt>,
            ) -> Option<usize> {
                if self
                    .count_desc_visitor
                    .0
                    .0
                    .get(&visit.node.node_id())
                    .copied()
                    .unwrap_or_default()
                    < self.k
                {
                    None
                } else {
                    let depth = parent_depth + visit.chars.len();
                    if depth > self.depth {
                        self.depth = depth;
                        self.node = visit.node;
                    }
                    Some(depth)
                }
            }
        }

        let mut common_substr_visitor = CommonSubstrVisitor {
            node: trie.root(),
            depth: 0,
            k,
            count_desc_visitor,
        };
        traverse_descendants(trie.root(), 0, &mut common_substr_visitor);

        #[derive(Default)]
        struct BuildStringVisitor(AString<DnaNt>);
        let mut build_string_visitor = BuildStringVisitor::default();
        impl<'arena, 's> AncestorVisitor<'arena, 's, (), DnaNt> for BuildStringVisitor {
            fn visit(
                &mut self,
                _context: (),
                visit: AncestorVisit<'arena, 's, DnaNt>,
            ) -> Option<()> {
                self.0 = visit.chars.to_owned() + self.0.as_str();
                Some(())
            }
        }

        traverse_ancestors(common_substr_visitor.node, (), &mut build_string_visitor);

        println!("{}", build_string_visitor.0);
    } else {
        let mut leaf_count: HashMap<NodeId, usize> = HashMap::new();

        traverse_descendants(
            trie.root(),
            (),
            &mut |_context, visit: DescendantVisit<DnaNt>| {
                if visit.terminal.is_some() {
                    *leaf_count.entry(visit.node.node_id()).or_default() += 1;
                    traverse_ancestors(visit.node, (), &mut |(), visit: AncestorVisit<DnaNt>| {
                        *leaf_count.entry(visit.node.node_id()).or_default() += 1;
                        Some(())
                    });
                }
                Some(())
            },
        );

        let mut longest_k_str = AString::default();

        traverse_descendants(
            trie.root(),
            0,
            &mut |parent_length, visit: DescendantVisit<DnaNt>| {
                if leaf_count
                    .get(&visit.node.node_id())
                    .copied()
                    .unwrap_or_default()
                    < k
                {
                    None
                } else {
                    let length = parent_length + visit.chars.len();
                    if length > longest_k_str.len() {
                        longest_k_str.clear();
                        traverse_ancestors(
                            visit.node,
                            (),
                            &mut |_context, visit: AncestorVisit<DnaNt>| {
                                longest_k_str = visit.chars.to_owned() + longest_k_str.as_str();
                                Some(())
                            },
                        )
                    }
                    Some(length)
                }
            },
        );

        println!("{}", longest_k_str);
    }
}
