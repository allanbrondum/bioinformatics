use rosalind::polymers::{ProteinAa, RnaNt, translate_rna};
use rosalind::util::chars_file;

fn main() {
    let codons: String = translate_rna(chars_file("src/bin/prot_data.txt").map(RnaNt::from_char))
        .into_iter()
        .map(ProteinAa::to_char)
        .collect();

    println!("{}", codons);
}
