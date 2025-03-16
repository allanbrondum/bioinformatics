use rosalind::polymers::{RnaNt, translate_rna};
use rosalind::string_model::AString;
use rosalind::util::chars_file;

fn main() {
    let codons: AString<_> = translate_rna(chars_file::<RnaNt>("src/bin/prot_data.txt"))
        .into_iter()
        .collect();

    println!("{}", codons);
}
