use rosalind::model::{DnaNt, RnaNt};

fn main() {
    let data = include_str!("rna_data.txt").trim();

    let rna: String = data
        .chars()
        .map(DnaNt::from_char)
        .map(DnaNt::transcribe)
        .map(RnaNt::to_char)
        .collect();

    println!("{}", rna)
}
