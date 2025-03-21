use bioinformatics::polymers::DnaNt;
use bioinformatics::string_model::AString;
use bioinformatics::util::chars_file;

fn main() {
    let rna: AString<_> = chars_file::<DnaNt>("src/bin/s_rna_data.txt")
        .map(DnaNt::transcribe)
        .collect();

    println!("{}", rna)
}
