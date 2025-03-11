use rand::Rng;
use std::fs::File;
use std::io::Write;

fn main() {
    let length = 10000;
    let output_path = "src/bin/bioalg1_seq_sim_data.txt";
    let gc = 0.4;

    let mut file = File::create(output_path).unwrap();
    writeln!(file, ">dna").unwrap();
    let mut rng = rand::rng();
    for _ in 0..length {
        let ch = if rng.random::<f64>() < gc {
            if rng.random::<bool>() { b'C' } else { b'G'}
        } else {
            if rng.random::<bool>() { b'A' } else { b'T' }
        };

        file.write_all(&[ch]).unwrap();
    }
    writeln!(file).unwrap();

}
