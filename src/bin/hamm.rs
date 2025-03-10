use itertools::Itertools;
use rosalind::util::lines;

fn main() {
    let data = include_str!("hamm_data.txt");

    let [line1, line2] = lines(data).collect_array().unwrap();

    let dist = line1
        .chars()
        .zip(line2.chars())
        .filter(|(it1, it2)| it1 != it2)
        .count();

    println!("{}", dist);
}
