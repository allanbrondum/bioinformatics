use itertools::Itertools;
use rosalind::string::scs;
use rosalind::util::fasta_polymers;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use rosalind::polymers::DnaNt;

fn main() {
    let min_olap = 20;
    let input_path = "src/bin/bioalg1_assemble_data.txt";
    let output_path = "src/bin/bioalg1_assemble_out.txt";

    let polymers = fasta_polymers::<DnaNt>(input_path).collect_vec();

    let start = Instant::now();
    let dnas = scs(polymers.into_iter().map(|pol| pol.polymer), min_olap);
    println!("assemble elapsed: {:?}", start.elapsed());

    let mut file = File::create(output_path).unwrap();
    for (idx, dna) in dnas.iter().enumerate() {
        writeln!(file, ">{}:{}", idx, dna.len()).unwrap();
        writeln!(file, "{}", dna).unwrap();
    }
}
