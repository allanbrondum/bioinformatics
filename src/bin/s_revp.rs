use bioinformatics::polymers::DnaNt;
use bioinformatics::util::fasta_polymers_file;

fn main() {
    let dna = fasta_polymers_file::<DnaNt>("src/bin/s_revp_data.txt")
        .next()
        .unwrap()
        .polymer;

    for i in 0..dna.len() {
        for l in 4..=12 {
            if i + l > dna.len() {
                break;
            }

            if dna[i..i + l]
                .iter()
                .copied()
                .zip(
                    dna[i..i + l]
                        .iter()
                        .copied()
                        .rev()
                        .map(DnaNt::bonding_complement),
                )
                .all(|(it1, it2)| it1 == it2)
            {
                println!("{} {}", i + 1, l);
            }
        }
    }
}

//alg reverse complement substr
