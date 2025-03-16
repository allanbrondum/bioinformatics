use rosalind::polymers::DnaNt;
use rosalind::util::fasta_polymers;

fn main() {
    let dna: Vec<_> = fasta_polymers("src/bin/s_revp_data.txt")
        .next()
        .unwrap()
        .polymer
        .chars()
        .map(DnaNt::from_char)
        .collect();

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
