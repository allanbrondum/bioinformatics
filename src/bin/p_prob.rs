use itertools::Itertools;
use rosalind::util::{lines_file, words};
use std::str::FromStr;

fn main() {
    let [dna, gc_prop_str] = lines_file("src/bin/p_prob_data.txt")
        .collect_array()
        .unwrap();
    let gc_prob = words(&gc_prop_str)
        .map(|word| f64::from_str(word).unwrap())
        .collect_vec();

    for gc_prob in gc_prob {
        let gc_count = gc_content(&dna);
        let prob = gc_prob.powi(gc_count as i32)
            * (1.0 - gc_prob).powi((dna.len() - gc_count) as i32)
            / 2f64.powi(dna.len() as i32);
        print!("{} ", prob.log10());
    }
}

fn gc_content(dna: &str) -> usize {
    dna.chars().filter(|&ch| ch == 'C' || ch == 'G').count()
}
