use rand::Rng;
use rosalind::polymers::DnaNt;
use rosalind::string_model::AString;
use std::fs::File;
use std::io::Write;

fn main() {
    let length = 10000;
    let output_path = "src/bin/bioalg1_seq_sim_data.txt";
    let gc = 0.4;

    let mut file = File::create(output_path).unwrap();
    writeln!(file, ">generated dna").unwrap();
    let mut rng = rand::rng();
    let data: AString<_> = (0..length)
        .map(|_| {
            #[allow(clippy::collapsible_else_if)]
            if rng.random::<f64>() < gc {
                if rng.random::<bool>() {
                    DnaNt::C
                } else {
                    DnaNt::G
                }
            } else {
                if rng.random::<bool>() {
                    DnaNt::A
                } else {
                    DnaNt::T
                }
            }
        })
        .collect();
    writeln!(file, "{}", data).unwrap();
}
