use bioinformatics::util::{lines_file, words};
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use std::collections::VecDeque;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

fn main() {
    let mut lines = lines_file("src/bin/g_tree_data.txt");

    let node_count: usize = lines.next().unwrap().parse().unwrap();

    if false {
        let mut nodes: HashSet<_> = (1..=node_count).collect();
        let mut edges: HashMap<_, Vec<_>> = HashMap::new();

        for line in lines {
            let [node1, node2] = words(&line)
                .map(|word| word.parse::<usize>().unwrap())
                .collect_array()
                .unwrap();

            edges.entry(node1).or_default().push(node2);
            edges.entry(node2).or_default().push(node1);
        }

        let mut component_count = 0;
        let mut remaining = nodes.clone();

        while let Some(node_start) = remaining.iter().next() {
            // println!("node_start {}", node_start);
            component_count += 1;

            let mut to_visit = VecDeque::new();
            to_visit.push_front(*node_start);

            while let Some(node) = to_visit.pop_front() {
                remaining.remove(&node);

                for &target in edges.get(&node).iter().flat_map(|v| v.iter()) {
                    // println!("target {}", target);
                    if remaining.contains(&target) {
                        // println!("target2 {}", target);
                        to_visit.push_front(target);
                    }
                }
            }
        }

        println!("{}", component_count - 1);
    } else {

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

        let component_count = petgraph::algo::connected_components(&graph);

        println!("{}", component_count - 1);

         // fs::write("target/g_tree.dot", format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel])).as_bytes()).unwrap();
    }
}
