use itertools::Itertools;
use rosalind::util::{lines, words};
use std::str::FromStr;

fn main() {
    let data = include_str!("lexf_data.txt");

    let [alphabet_str, length] = lines(data).collect_array().unwrap();
    let alphabet: Vec<_> = words(alphabet_str).collect();
    let length = usize::from_str(length).unwrap();

    let mut res = alphabet.iter().map(|a| a.to_string()).collect_vec();
    for _ in 1..length {
        res = res
            .into_iter()
            .cartesian_product(alphabet.iter())
            .map(|(item, letter)| item + letter)
            .collect();
    }

    for item in res {
        println!("{}", item);
    }
}
