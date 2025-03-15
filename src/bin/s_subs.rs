use itertools::Itertools;
use rosalind::string::indexes_str;
use rosalind::util::lines_file;

fn main() {
    let [s, t]: [String; 2] = lines_file("src/bin/s_subs_data.txt")
        .collect_array()
        .unwrap();

    for index in indexes_str(&s, &t) {
        print!("{} ", index + 1);
    }
}

//alg substr
