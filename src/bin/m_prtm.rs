use bioinformatics::polymers::{ProteinAa, protein_aa_mass};
use bioinformatics::util::chars_file;

fn main() {
    let mass: f64 = chars_file::<ProteinAa>("src/bin/m_prtm_data.txt")
        .map(protein_aa_mass)
        .sum();

    println!("{}", mass)
}
