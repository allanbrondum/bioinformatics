use bioinformatics::polymers::DnaNt;
use bioinformatics::string::border_array::border_array;
use bioinformatics::util::fasta_polymers_file;
use itertools::Itertools;

fn main() {
    let dna = fasta_polymers_file::<DnaNt>("src/bin/s_kmp_data.txt")
        .next()
        .unwrap()
        .polymer;

    let border_array = border_array(&dna);

    println!(
        "{}",
        border_array
            .iter()
            .copied()
            .format_with(" ", |item, f| f(&item))
    );
}

