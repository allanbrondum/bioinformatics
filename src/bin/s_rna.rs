use rosalind::polymers::{DnaNt, RnaNt};
use rosalind::string_model::AString;
use rosalind::util::chars_file;

fn main() {
    let rna: AString<_> = chars_file::<DnaNt>("src/bin/s_rna_data.txt")
        .map(DnaNt::transcribe)
        .collect();

    println!("{}", rna)
}
