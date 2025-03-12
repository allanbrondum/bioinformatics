use itertools::Itertools;
use rosalind::string::positions_str;
use rosalind::util::lines_file;

fn main() {
    let [s, t]: [String; 2] = lines_file("src/bin/s_subs_data.txt")
        .collect_array()
        .unwrap();

    for position in positions_str(&s, &t) {
        print!("{} ", position);
    }
}

//alg substr
