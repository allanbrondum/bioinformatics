use itertools::Itertools;
use rosalind::util::words;
use std::collections::HashMap;
use std::str::FromStr;

fn main() {
    let data = include_str!("lia_data.txt");

    let [k, N] = words(data)
        .map(|word| usize::from_str(word).unwrap())
        .collect_array()
        .unwrap();

    let prob: f64 = (0..=N).map(|n| prob_n(k, n)).sum();

    println!("{}", prob);
}

// AaBb
fn prob_n(k: usize, n: usize) -> f64 {
    let mut data: HashMap<(&'static str, usize), f64> = HashMap::default();
    data.insert(("AaBb", 1), 1.0);

    for i in 0..k {
        let mut new_data: HashMap<(&'static str, usize), f64> = HashMap::default();

        for entry in data {
            match entry.0.0 {
                // genotype @ "AABB" => "",
                // genotype @ "AABb" => todo!(),
                // genotype @ "AAbb" => todo!(),
                // genotype @ "AaBB" => todo!(),
                // genotype @ "AaBb" => todo!(),
                // genotype @ "Aabb" => todo!(),
                // genotype @ "aaBB" => todo!(),
                // genotype @ "aaBb" => todo!(),
                // genotype @ "aabb" => todo!(),
                _ => unreachable!(),
            }
        }

        data = new_data;
    }

    todo!()
}

// AABB AABb AAbb AaBB AaBb Aabb aaBB aaBb aabb

//     AB    Ab    aB    ab
// AB  AABB  AABb  AaBB  AaBb
// Ab  AABb  AAbb  AaBb  Aabb
// aB  AaBB  AaBb  aaBB  aaBb
// ab  AaBb  Aabb  aaBb  aabb

