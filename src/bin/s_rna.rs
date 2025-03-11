use rosalind::polymers::{DnaNt, RnaNt};
use rosalind::util::chars;

fn main() {
    let data = include_str!("s_rna_data.txt");

    let rna: String = chars(data)
        .map(DnaNt::from_char)
        .map(DnaNt::transcribe)
        .map(RnaNt::to_char)
        .collect();

    println!("{}", rna)
}
