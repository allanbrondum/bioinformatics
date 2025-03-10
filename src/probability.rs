pub fn binomial(prob: f64, k: i32, n: i32) -> f64 {
    comb(k as u64, n as u64) * prob.powi(k) * (1.0 - prob).powi(n - k)
}

fn comb(k: u64, n: u64) -> f64 {
    let k = k.min(n - k);
    (n - k + 1..=n).map(|val| val as f64).product::<f64>()
        / (1..=k).map(|val| val as f64).product::<f64>()
}
