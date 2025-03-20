use rosalind::polymers::{DnaNt, translate_rna};

use rosalind::string_model::AStr;
use rosalind::util::fasta_polymers_file;

fn main() {
    let mut polymers: Vec<_> = fasta_polymers_file::<DnaNt>("src/bin/s_splc_data.txt").collect();

    let mut rna = polymers.remove(0).polymer;

    for polymer in &polymers {
        rna = rna.replace(&polymer.polymer, AStr::from_slice(&[]));
    }

    let protein = translate_rna(rna.into_iter().map(DnaNt::transcribe));

    println!("{}", protein);
}
