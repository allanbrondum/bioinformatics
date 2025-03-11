use rosalind::polymers::{DnaNt, RnaNt};
use rosalind::util::chars_file;

fn main() {


    let rna: String = chars_file("src/bin/s_rna_data.txt")
        .map(DnaNt::from_char)
        .map(DnaNt::transcribe)
        .map(RnaNt::to_char)
        .collect();

    println!("{}", rna)
}
