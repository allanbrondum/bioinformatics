use bioinformatics::polymers::{RnaNt, translate_rna};
use bioinformatics::string_model::AString;
use bioinformatics::util::chars_file;

fn main() {
    let codons: AString<_> = translate_rna(chars_file::<RnaNt>("src/bin/prot_data.txt"))
        .into_iter()
        .collect();

    println!("{}", codons);
}
