fn main() {
    let data = include_str!("fib_data.txt").trim();

    let [n, k]: [u64; 2] = data
        .split_whitespace()
        .map(|val| val.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
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
