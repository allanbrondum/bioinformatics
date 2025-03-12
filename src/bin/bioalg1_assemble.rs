use itertools::Itertools;
use rosalind::string::sc_supstr;
use rosalind::util::fasta_polymers;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn main() {
    let min_olap = 20;
    let input_path = "src/bin/bioalg1_assemble_data.txt";
    let output_path = "src/bin/bioalg1_assemble_out.txt";

    let polymers = fasta_polymers(input_path).collect_vec();

    let start = Instant::now();
    let dnas = sc_supstr(polymers.into_iter().map(|pol| pol.polymer), min_olap);
    println!("assemble elapsed: {:?}", start.elapsed());

    let mut file = File::create(output_path).unwrap();
    for (idx, dna) in dnas.iter().enumerate() {
        writeln!(file, ">{}:{}", idx, dna.len()).unwrap();
        writeln!(file, "{}", dna).unwrap();
    }
}
