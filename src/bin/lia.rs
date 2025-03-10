use itertools::Itertools;
use rosalind::util::words;
use std::str::FromStr;

fn main() {
    let data = include_str!("lia_data.txt");

    let [k, N] = words(data)
        .map(|word| u32::from_str(word).unwrap())
        .collect_array()
        .unwrap();

    let total = 2u32.pow(k);
    let prob: f64 = (N..=total)
        .map(|n| {
             binomial(0.25, n as i32, total as i32)
        })
        .sum();

    println!("{}", prob);
}

fn binomial(prob: f64, k: i32, n: i32) -> f64 {
    comb(k as u64, n as u64)  * prob.powi(k) * (1.0 - prob).powi(n - k)
}

fn comb(k: u64, n: u64) -> f64 {
    let k = k.min(n - k);
    (n - k + 1..=n).map(|val| val as f64).product::<f64>()
        / (1..=k).map(|val| val as f64).product::<f64>()
}

// AABB AABb AAbb AaBB AaBb Aabb aaBB aaBb aabb

//     AB    Ab    aB    ab
// AB  AABB  AABb  AaBB  AaBb
// Ab  AABb  AAbb  AaBb  Aabb
// aB  AaBB  AaBb  aaBB  aaBb
// ab  AaBb  Aabb  aaBb  aabb
