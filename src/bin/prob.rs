use itertools::Itertools;
use rosalind::util::{lines, words};
use std::str::FromStr;

fn main() {
    let data = include_str!("prob_data.txt");

    let [polymer, gc_prop_str] = lines(data).collect_array().unwrap();
    let gc_prop = words(gc_prop_str)
        .map(|word| f64::from_str(word).unwrap())
        .collect_vec();

    for gc_prop in gc_prop {
        print!("{} ", prob(polymer, gc_prop).log10());
    }
}

fn prob(dna: &str, gc_prob: f64) -> f64 {
    todo!()
}
