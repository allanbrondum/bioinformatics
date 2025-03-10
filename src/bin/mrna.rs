use rosalind::polymers::{Codon, DnaNt, ProteinAa, all_codons, to_codon};
use std::collections::HashMap;
use std::iter;
use rosalind::util::chars;

fn main() {
    let data = include_str!("mrna_data.txt");

    let coding_count: HashMap<Codon, u64> =
        all_codons().fold(HashMap::default(), |mut map, codon| {
            *map.entry(to_codon(codon)).or_default() += 1;
            map
        });

    let count = chars(data)
        .map(ProteinAa::from_char)
        .map(|aa| coding_count[&Codon::Aa(aa)])
        .chain(iter::once(coding_count[&Codon::Stop]))
        .reduce(|it1, it2| it1 * it2 % 1_000_000)
        .unwrap();
    println!("{}", count);
}
