use bioinformatics::string;
use bioinformatics::util::fasta_polymers_file;
use itertools::Itertools;
use proptest::bits::BitSetLike;
use bioinformatics::alphabet_model::CharT;
use bioinformatics::polymers::DnaNt;
use bioinformatics::string_model::AStr;

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

fn border_array<C:CharT>(s: &AStr<C>) -> Vec<usize>{
    (1..=s.len())
        .scan(0, |prev_overlap, l| {
            let max_possible_overlap = *prev_overlap + 1;
            let overlap = string::overlap(&s[1.max(l - max_possible_overlap)..l], &s[0..(l - 1).min(max_possible_overlap)]);
            *prev_overlap = overlap;
            Some(overlap)
        })
        .collect_vec()
}