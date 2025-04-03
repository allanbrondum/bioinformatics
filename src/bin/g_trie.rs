use bioinformatics::polymers::DnaNt;
use bioinformatics::string::trie_compact;
use bioinformatics::string::trie_compact::Node;
use bioinformatics::string_model::AString;
use bioinformatics::util::{fasta_polymers_file, lines_file};
use generic_array::ArrayLength;
use itertools::Itertools;
use std::collections::VecDeque;
use std::str::FromStr;

fn main() {
    let polymers = lines_file("src/bin/g_trie_data.txt")
        .map(|line| AString::<DnaNt>::from_str(&line).unwrap())
        .collect_vec();

    let trie = trie_compact::build_trie(polymers.iter().map(|dna| dna.as_str()));

    let mut node_seq = 1;

    struct ToVisit<'a, 's, C, N: ArrayLength> {
        node: &'a Node<'s, C, N>,
        node_id: usize,
    }

    let mut to_visit = VecDeque::new();
    to_visit.push_front(ToVisit {
        node: &trie.root,
        node_id: node_seq,
    });
    node_seq += 1;

    while let Some(node) = to_visit.pop_front() {
        for child_edge in node.node.children.iter().flat_map(|child| child.iter()) {
            let mut prev_node_id = node.node_id;
            for ch in child_edge.chars {
                println!("{} {} {}", prev_node_id, node_seq, ch);
                prev_node_id = node_seq;
                node_seq += 1;
            }

            to_visit.push_back(ToVisit {
                node: &child_edge.target,
                node_id: prev_node_id,
            });
        }
    }
}
