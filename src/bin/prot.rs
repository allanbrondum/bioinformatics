use itertools::Itertools;
use rosalind::model::{to_codon, ProteinAa, Codon, RnaNt};

fn main() {
    let data = include_str!("prot_data.txt").trim();

    let codons: String = data
        .chars()
        .map(RnaNt::from_char)
        .chunks(3)
        .into_iter()
        .map(|codon| to_codon(codon.collect_array().unwrap()))
        .take_while(|&codon | codon != Codon::Stop)
        .map(|codon| match codon {
            Codon::Start => todo!(),
            Codon::Stop => unreachable!(),
            Codon::Aa(aa) => aa,
        })
        .map(ProteinAa::to_char)
        .collect();

    println!("{}", codons);
}
