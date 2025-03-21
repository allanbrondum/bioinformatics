use bioinformatics::util::words;
use itertools::Itertools;
use std::collections::VecDeque;

fn main() {
    let data = include_str!("c_fibd_data.txt");

    let [n, m]: [u64; 2] = words(data)
        .map(|val| val.parse().unwrap())
        .collect_array()
        .unwrap();

    let count = fib(n, m);

    println!("{}", count)
}

fn fib(n: u64, m: u64) -> u64 {
    let mut by_age = VecDeque::from(vec![0; m as usize]);
    by_age[0] = 1;

    for _ in 1..n {
        let new = by_age.iter().skip(1).copied().sum();
        by_age.rotate_right(1);
        by_age[0] = new;
    }

    by_age.iter().copied().sum()
}
