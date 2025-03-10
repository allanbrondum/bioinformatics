use itertools::Itertools;
use rosalind::util::{lines, positions_str};

fn main() {
    let data = include_str!("subs_data.txt");

    let [s, t]: [&str; 2] = lines(data).collect_array().unwrap();

    for position in positions_str(s, t) {
        print!("{} ", position);
    }
}

//alg substr
