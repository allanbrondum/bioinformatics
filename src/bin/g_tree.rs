use std::{fs, io};
use bioinformatics::util::{lines_file, words};
use itertools::Itertools;
use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use petgraph::graph::NodeIndex;

fn main() {
    let mut lines = lines_file("src/bin/g_tree_data.txt");

    let node_count: usize = lines.next().unwrap().parse().unwrap();

    let mut graph = Graph::<usize, ()>::new();
    for i in 0..node_count {
        graph.add_node(i + 1);
    }

    for line in lines {
        let [node1, node2] = words(&line)
            .map(|word| word.parse::<usize>().unwrap())
            .collect_array()
            .unwrap();

        graph.add_edge(NodeIndex::new(node1 - 1), NodeIndex::new(node2 - 1), ());
    }


     fs::write("target/g_tree.dot", format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel])).as_bytes()).unwrap();

}
