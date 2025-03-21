use bioinformatics::util::{lines, words};
use itertools::Itertools;
use std::str::FromStr;

fn main() {
    let data = include_str!("s_lexf_data.txt");

    let [alphabet_str, length] = lines(data).collect_array().unwrap();
    let alphabet: Vec<_> = words(alphabet_str).collect();
    let length = usize::from_str(length).unwrap();

    let mut res = vec![String::new()];
    for _ in 0..length {
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
