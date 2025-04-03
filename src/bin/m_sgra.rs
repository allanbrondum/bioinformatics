use bioinformatics::polymers::{ProteinAa, protein_aa_with_mass, protein_aa_with_mass_closest};
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use hashbrown::HashMap;
use itertools::Itertools;
use ordered_float::NotNan;
use petgraph::dot::Dot;
use petgraph::visit::EdgeRef;
use petgraph::{EdgeDirection, Graph, algo};
use std::{fs, mem};

fn main() {
    let masses = lines_file("src/bin/m_sgra_data.txt")
        .map(|line| line.parse::<NotNan<f64>>().unwrap())
        .collect_vec();

    let mut graph: Graph<NotNan<f64>, ProteinAa> = Graph::new();

    let mass_to_nodes: HashMap<_, _> = masses
        .iter()
        .copied()
        .map(|mass| (mass, graph.add_node(mass)))
        .collect();

    for (mut m1, mut m2) in masses.iter().copied().tuple_combinations() {
        if m1 > m2 {
            mem::swap(&mut m1, &mut m2);
        }

        if let Some(aa) = protein_aa_with_mass(m2 - m1) {
            graph.add_edge(
                *mass_to_nodes.get(&m1).unwrap(),
                *mass_to_nodes.get(&m2).unwrap(),
                aa,
            );
        }
    }

    // fs::write(
    //     "target/m_sgra.dot",
    //     format!("{}", Dot::new(&graph)).as_bytes(),
    // )
    // .unwrap();

    let topo = algo::toposort(&graph, None).unwrap();

    let mut longest_path_inc: HashMap<_, AString<_>> = HashMap::new();
    for node in topo {
        let inc_path = graph
            .edges_directed(node, EdgeDirection::Incoming)
            .map(|edge| {
                let mut path = longest_path_inc
                    .get(&edge.source())
                    .cloned()
                    .unwrap_or_default();
                path.push(*edge.weight());
                path
            })
            .max_by_key(|inc_path| inc_path.len())
            .unwrap_or_default();

        longest_path_inc.insert(node, inc_path);
    }

    let longest_path = longest_path_inc
        .values()
        .max_by_key(|path| path.len())
        .unwrap();

    println!("{}", longest_path);
}
