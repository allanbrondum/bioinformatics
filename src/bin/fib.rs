use itertools::Itertools;
use rosalind::util::words;

fn main() {
    let data = include_str!("fib_data.txt");

    let [n, k]: [u64; 2] = words(data)
        .map(|val| val.parse().unwrap())
        .collect_array()
        .unwrap();

    let count = fib(n, k);

    println!("{}", count)
}

fn fib(n: u64, k: u64) -> u64 {
    if n <= 2 {
        return 1;
    }

    fib(n - 1, k) + k * fib(n - 2, k)
}
