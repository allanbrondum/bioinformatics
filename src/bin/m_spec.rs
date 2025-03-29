use bioinformatics::polymers::protein_aa_with_mass_closest;
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use itertools::Itertools;
use ordered_float::NotNan;

fn main() {
    let prefix_spec = lines_file("src/bin/m_spec_data.txt")
        .map(|line| line.parse::<NotNan<f64>>().unwrap())
        .collect_vec();

    let protein: AString<_> = prefix_spec
        .iter()
        .tuple_windows::<(_, _)>()
        .map(|(m1, m2)| {
            let aa_mass = m2 - m1;
            protein_aa_with_mass_closest(aa_mass).1
        })
        .collect();

    println!("{}", protein);
}
