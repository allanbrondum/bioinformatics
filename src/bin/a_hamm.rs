use itertools::Itertools;
use rosalind::util::lines_file;

fn main() {
    let [line1, line2] = lines_file("src/bin/a_hamm_data.txt")
        .collect_array()
        .unwrap();

    let dist = line1
        .chars()
        .zip(line2.chars())
        .filter(|(it1, it2)| it1 != it2)
        .count();

    println!("{}", dist);
}
