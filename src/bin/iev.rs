use itertools::Itertools;
use rosalind::util::words;
use std::str::FromStr;

fn main() {
    let data = include_str!("iev_data.txt");

    let [AA_AA, AA_Aa, AA_aa, Aa_Aa, Aa_aa, aa_aa] = words(data)
        .map(|word| f64::from_str(word).unwrap())
        .collect_array()
        .unwrap();

    let prob_dom = 2.0 * ((AA_AA + AA_Aa + AA_aa) + (Aa_Aa) * (3.0 / 4.0) + (Aa_aa) * (2.0 / 4.0));

    println!("{}", prob_dom);
}
