use bioinformatics::probability::binomial;
use bioinformatics::util::words;
use itertools::Itertools;
use std::str::FromStr;

fn main() {
    let data = include_str!("h_lia_data.txt");

    #[allow(non_snake_case)]
    let [k, N] = words(data)
        .map(|word| u32::from_str(word).unwrap())
        .collect_array()
        .unwrap();

    let total = 2u32.pow(k);
    let prob: f64 = (N..=total)
        .map(|n| binomial(0.25, n as i32, total as i32))
        .sum();

    println!("{}", prob);
}

// AABB AABb AAbb AaBB AaBb Aabb aaBB aaBb aabb

//     AB    Ab    aB    ab
// AB  AABB  AABb  AaBB  AaBb
// Ab  AABb  AAbb  AaBb  Aabb
// aB  AaBB  AaBb  aaBB  aaBb
// ab  AaBb  Aabb  aaBb  aabb
