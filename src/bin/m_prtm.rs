use rosalind::polymers::{protein_aa_mass, ProteinAa};
use rosalind::util::chars_file;

fn main() {

    let mass: f64 = chars_file("src/bin/m_prtm_data.txt")
        .map(ProteinAa::from_char)
        .map(protein_aa_mass)
        .sum();

    println!("{}", mass)
}
