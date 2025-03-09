use itertools::Itertools;
use rosalind::model::{Codon, ProteinAa, RnaNt, to_codon, translate_rna};
use rosalind::util::chars;

fn main() {
    let data = include_str!("prot_data.txt");

    let codons: String = translate_rna(chars(data).map(RnaNt::from_char))
        .into_iter().map(ProteinAa::to_char)
        .collect();

    println!("{}", codons);
}
