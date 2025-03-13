use crate::string::overlap_str;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

const GRAPH_DEBUG: bool = false;

pub fn sc_supstr(
    strs: impl IntoIterator<Item = String> + Clone,
    min_overlap: usize,
) -> Vec<String> {
    let mut nodes = strs
        .into_iter()
        .map(|str| Rc::new(RefCell::new(Node::new(str))))
        .collect_vec();

    let edges = nodes.iter().permutations(2).map(|edge_nodes| {
        let [node1, node2] = edge_nodes.into_iter().collect_array().expect("length 2");

        let mut node1_mut = node1.borrow_mut();
        let mut node2_mut = node2.borrow_mut();

        let overlap = overlap_str(&node1_mut.str, &node2_mut.str);
        let edge = Rc::new(RefCell::new(Edge::new(
            node1.clone(),
            node2.clone(),
            overlap,
        )));

        node1_mut.outgoing.push(edge.clone());
        node2_mut.incoming.push(edge.clone());

        edge
    });

    let mut edge_heap: BinaryHeap<_> = edges.map(HeapEntry::new).collect();

    let mut graph_number = 0;
    loop {
        if GRAPH_DEBUG {
            to_dot(
                format!("target/graph_{}.dot", graph_number),
                &nodes,
                edge_heap.iter().map(|entry| &entry.edge),
            );
        }
        graph_number += 1;

        let Some(edge) = edge_heap.pop() else {
            break;
        };

        let edge_ref = edge.edge.borrow();

        if edge_ref.deleted {
            continue;
        }

        // Merge the two nodes on `edge` if overlap is big enough

        drop(edge_ref);
        let mut edge_mut = edge.edge.borrow_mut();
        edge_mut.deleted = true;
        drop(edge_mut);

        let edge_ref = edge.edge.borrow();
        if edge_ref.overlap < min_overlap {
            continue;
        }

        let mut source_mut = edge_ref.source.borrow_mut();
        let mut target_mut = edge_ref.target.borrow_mut();
        source_mut.deleted = true;
        target_mut.deleted = true;

        assert_eq!(
            source_mut.str[source_mut.str.len() - edge_ref.overlap..],
            target_mut.str[..edge_ref.overlap]
        );
        let merged = source_mut.str.clone() + &target_mut.str[edge_ref.overlap..];
        let new_node = Rc::new(RefCell::new(Node::new(merged)));
        nodes.push(new_node.clone());
        let mut new_node_mut = new_node.borrow_mut();

        for source_out in source_mut.outgoing() {
            source_out.borrow_mut().deleted = true;
        }
        for source_inc in source_mut.incoming() {
            let mut source_inc_mut = source_inc.borrow_mut();
            if Rc::ptr_eq(&source_inc_mut.source, &edge_ref.target) {
                source_inc_mut.deleted = true;
            } else {
                source_inc_mut.target = new_node.clone();
                new_node_mut.incoming.push(source_inc.clone());
            }
        }

        for target_inc in target_mut.incoming() {
            target_inc.borrow_mut().deleted = true;
        }
        for target_out in target_mut.outgoing() {
            target_out.borrow_mut().source = new_node.clone();
            new_node_mut.outgoing.push(target_out.clone());
        }
    }

    if GRAPH_DEBUG {
        to_dot(
            "target/graph_f.dot",
            &nodes,
            edge_heap.iter().map(|entry| &entry.edge),
        );
    }

    nodes
        .iter()
        .filter(|node| !node.borrow().deleted)
        .map(|node| node.borrow().str.clone())
        .collect()
}

struct Node {
    str: String,
    incoming: Vec<Rc<RefCell<Edge>>>,
    outgoing: Vec<Rc<RefCell<Edge>>>,
    deleted: bool,
}

impl Node {
    fn new(str: String) -> Self {
        Self {
            str,
            incoming: Default::default(),
            outgoing: Default::default(),
            deleted: false,
        }
    }

    fn incoming(&self) -> impl Iterator<Item = &Rc<RefCell<Edge>>> {
        self.incoming.iter().filter(|edge| {
            let edge_ref = edge.borrow();
            !edge_ref.deleted
        })
    }

    fn outgoing(&self) -> impl Iterator<Item = &Rc<RefCell<Edge>>> {
        self.outgoing.iter().filter(|edge| {
            let edge_ref = edge.borrow();
            !edge_ref.deleted
        })
    }
}

struct Edge {
    source: Rc<RefCell<Node>>,
    target: Rc<RefCell<Node>>,
    overlap: usize,
    deleted: bool,
}

impl Edge {
    fn new(source: Rc<RefCell<Node>>, target: Rc<RefCell<Node>>, overlap: usize) -> Self {
        Self {
            source,
            target,
            overlap,
            deleted: false,
        }
    }
}

struct HeapEntry {
    edge: Rc<RefCell<Edge>>,
    overlap: usize,
}

impl HeapEntry {
    fn new(edge: Rc<RefCell<Edge>>) -> Self {
        let overlap = edge.borrow().overlap;

        Self { edge, overlap }
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.overlap.cmp(&other.overlap)
    }
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.overlap == other.overlap
    }
}

impl Eq for HeapEntry {}

fn to_dot<'a>(
    filepath: impl AsRef<Path>,
    nodes: impl IntoIterator<Item = &'a Rc<RefCell<Node>>>,
    edges: impl IntoIterator<Item = &'a Rc<RefCell<Edge>>>,
) {
    let mut file = File::create(filepath).unwrap();
    writeln!(file, "digraph G {{").unwrap();

    for node in nodes.into_iter() {
        let node_ref = node.borrow();
        if node_ref.deleted {
            continue;
        }
        writeln!(
            file,
            "    {} [label=\"{}\"];",
            Rc::as_ptr(node) as usize,
            &node_ref.str
        )
        .unwrap();
    }

    for edge in edges.into_iter() {
        let edge_ref = edge.borrow();
        if edge_ref.deleted {
            continue;
        }
        writeln!(
            file,
            "    \"{}\" -> \"{}\" [label=\"{}\"];",
            Rc::as_ptr(&edge_ref.source) as usize,
            Rc::as_ptr(&edge_ref.target) as usize,
            edge_ref.overlap
        )
        .unwrap();
    }

    writeln!(file, "}}").unwrap();
}

#[cfg(test)]
mod test {
    use crate::string::sc_supstr;

    #[test]
    fn test_sc_supstr_one() {
        assert_eq!(
            sc_supstr(["uioefghabcd".to_string()], 3),
            vec!["uioefghabcd".to_string()],
        );
    }

    #[test]
    fn test_sc_supstr_two() {
        assert_eq!(
            sc_supstr(["uioefghabcd".to_string(), "abcdefghijk".to_string()], 3),
            vec!["uioefghabcdefghijk".to_string()],
        );
    }

    #[test]
    fn test_sc_supstr_three() {
        assert_eq!(
            sc_supstr(
                [
                    "uioefghabcd".to_string(),
                    "abcdefghijk".to_string(),
                    "ijklm".to_string()
                ],
                3
            ),
            vec!["uioefghabcdefghijklm".to_string()],
        );
    }

    #[test]
    fn test_sc_supstr_dupl() {
        assert_eq!(
            sc_supstr(
                [
                    "uioefghabcd".to_string(),
                    "abcdefghijk".to_string(),
                    "abcdefghijk".to_string()
                ],
                3
            ),
            vec!["uioefghabcdefghijk".to_string()],
        );
    }

    #[test]
    fn test_sc_supstr_no_overlap() {
        assert_eq!(
            sc_supstr(["uioefghabcd".to_string(), "abcdefghijk".to_string()], 5),
            vec!["uioefghabcd".to_string(), "abcdefghijk".to_string()],
        );
    }
}
