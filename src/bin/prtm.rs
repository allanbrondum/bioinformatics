use rosalind::polymers::{ProteinAa, protein_aa_mass};
use rosalind::util::chars;

fn main() {
    let data = include_str!("prtm_data.txt");

    let mass: f64 = chars(data)
        .map(ProteinAa::from_char)
        .map(protein_aa_mass)
        .sum();

    println!("{}", mass)
}
