use rosalind::polymers::{DnaNt, ProteinAa, translate_rna};
use rosalind::util::fasta_polymers;

fn main() {
    let data = include_str!("splc_data.txt");

    let mut polymers: Vec<_> = fasta_polymers(data).collect();

    let mut rna = polymers.remove(0).polymer;

    for polymer in &polymers {
        rna = rna.replace(&polymer.polymer, "");
    }

    let protein: String = translate_rna(rna.chars().map(DnaNt::from_char).map(DnaNt::transcribe))
        .into_iter()
        .map(ProteinAa::to_char)
        .collect();

    println!("{}", protein);
}
