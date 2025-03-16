use crate::alphabet_model::CharT;
use crate::string::overlap;
use crate::string_model::AString;
use itertools::Itertools;
use petgraph::Direction;
use petgraph::dot::Dot;
use petgraph::matrix_graph::{MatrixGraph, NodeIndex};
use petgraph::visit::{
    Data, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, IntoNodeReferences, NodeIndexable,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const GRAPH_DEBUG: bool = false;

pub fn scs<C: CharT>(
    strs: impl IntoIterator<Item = AString<C>> + Clone,
    min_overlap: usize,
) -> Vec<AString<C>> {
    let strs = strs.into_iter();
    let mut graph: MatrixGraph<Node<C>, Edge> = MatrixGraph::with_capacity(strs.size_hint().0);

    let node_idxs = strs
        .into_iter()
        .map(|str| graph.add_node(Node::new(str)))
        .collect_vec();

    let edges = node_idxs.into_iter().permutations(2).map(|edge_nodes| {
        let [node1_idx, node2_idx] = edge_nodes.into_iter().collect_array().expect("length 2");

        let node1 = graph.node_weight(node1_idx);
        let node2 = graph.node_weight(node2_idx);

        let overlap = overlap(&node1.str, &node2.str);
        graph.add_edge(node1_idx, node2_idx, Edge::new(overlap));

        EdgeHeapEntry::new(node1_idx, node2_idx, overlap)
    });

    let mut edge_heap: BinaryHeap<_> = edges.collect();

    let mut graph_number = 0;
    loop {
        if GRAPH_DEBUG {
            to_dot(format!("target/graph_{}.dot", graph_number), &graph);
        }
        graph_number += 1;

        let Some(edge_entry) = edge_heap.pop() else {
            break;
        };

        if !graph.has_edge(edge_entry.source_idx, edge_entry.target_idx) {
            continue;
        }

        // Merge the two nodes on `edge` if overlap is big enough

        let edge = graph.remove_edge(edge_entry.source_idx, edge_entry.target_idx);

        if edge.overlap < min_overlap {
            continue;
        }

        let new_outgoing = graph
            .edges_directed(edge_entry.target_idx, Direction::Outgoing)
            .map(|edge| (edge.0, edge.1, edge.2.clone()))
            .filter(|edge| edge.1 != edge_entry.source_idx)
            .collect_vec();
        let new_incoming = graph
            .edges_directed(edge_entry.source_idx, Direction::Incoming)
            .map(|edge| (edge.1, edge.0, edge.2.clone()))
            .filter(|edge| edge.0 != edge_entry.target_idx)
            .collect_vec();

        let source = graph.remove_node(edge_entry.source_idx);
        let target = graph.remove_node(edge_entry.target_idx);

        assert_eq!(
            source.str[source.str.len() - edge.overlap..],
            target.str[..edge.overlap]
        );
        let merged = source.str.clone() + &target.str[edge_entry.overlap..];
        let new_node = graph.add_node(Node::new(merged));

        for outgoing in new_outgoing {
            edge_heap.push(EdgeHeapEntry::new(new_node, outgoing.1, outgoing.2.overlap));
            graph.add_edge(new_node, outgoing.1, outgoing.2);
        }

        for incoming in new_incoming {
            edge_heap.push(EdgeHeapEntry::new(incoming.0, new_node, incoming.2.overlap));
            graph.add_edge(incoming.0, new_node, incoming.2);
        }
    }

    if GRAPH_DEBUG {
        to_dot("target/graph_f.dot", &graph);
    }

    graph
        .node_identifiers()
        .collect_vec()
        .into_iter()
        .map(|node_idx| graph.remove_node(node_idx).str)
        .collect()
}

struct Node<C: CharT> {
    str: AString<C>,
}

impl<C: CharT> Display for Node<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.str.fmt(f)
    }
}

impl<C: CharT> Node<C> {
    fn new(str: AString<C>) -> Self {
        Self { str }
    }
}

#[derive(Clone)]
struct Edge {
    overlap: usize,
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.overlap.fmt(f)
    }
}

impl Edge {
    fn new(overlap: usize) -> Self {
        Self { overlap }
    }
}

struct EdgeHeapEntry {
    source_idx: NodeIndex,
    target_idx: NodeIndex,
    overlap: usize,
}

impl EdgeHeapEntry {
    fn new(node1_idx: NodeIndex, node2_idx: NodeIndex, overlap: usize) -> Self {
        Self {
            source_idx: node1_idx,
            target_idx: node2_idx,
            overlap,
        }
    }
}

impl PartialOrd for EdgeHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for EdgeHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.overlap.cmp(&other.overlap)
    }
}

impl PartialEq for EdgeHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.overlap == other.overlap
    }
}

impl Eq for EdgeHeapEntry {}

fn to_dot<I: IntoNodeReferences + IntoEdgeReferences + NodeIndexable + GraphProp>(
    filepath: impl AsRef<Path>,
    graph: I,
) where
    <I as Data>::EdgeWeight: Display,
    <I as Data>::NodeWeight: Display,
{
    let mut file = File::create(filepath).unwrap();
    write!(file, "{}", Dot::new(&graph)).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ascii::ascii;

    #[test]
    fn test_sc_supstr_one() {
        assert_eq!(
            scs([ascii("uioefghabcd").to_owned()], 3),
            vec![ascii("uioefghabcd").to_owned()],
        );
    }

    #[test]
    fn test_sc_supstr_two() {
        assert_eq!(
            scs(
                [
                    ascii("uioefghabcd").to_owned(),
                    ascii("abcdefghijk").to_owned()
                ],
                3
            ),
            vec![ascii("uioefghabcdefghijk").to_owned()],
        );
    }

    #[test]
    fn test_sc_supstr_three() {
        assert_eq!(
            scs(
                [
                    ascii("uioefghabcd").to_owned(),
                    ascii("abcdefghijk").to_owned(),
                    ascii("ijklm").to_owned()
                ],
                3
            ),
            vec![ascii("uioefghabcdefghijklm").to_owned()],
        );
    }

    #[test]
    fn test_sc_supstr_dupl() {
        assert_eq!(
            scs(
                [
                    ascii("uioefghabcd").to_owned(),
                    ascii("abcdefghijk").to_owned(),
                    ascii("abcdefghijk").to_owned()
                ],
                3
            ),
            vec![ascii("uioefghabcdefghijk").to_owned()],
        );
    }

    #[test]
    fn test_sc_supstr_no_overlap() {
        assert_eq!(
            scs(
                [
                    ascii("uioefghabcd").to_owned(),
                    ascii("abcdefghijk").to_owned()
                ],
                5
            ),
            vec![
                ascii("uioefghabcd").to_owned(),
                ascii("abcdefghijk").to_owned()
            ],
        );
    }
}
