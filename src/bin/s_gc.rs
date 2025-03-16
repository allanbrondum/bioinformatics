use itertools::Itertools;
use rosalind::polymers::DnaNt;
use rosalind::string_model::AStr;
use rosalind::util::{FastaEntry, fasta_polymers};

fn main() {
    let entries = fasta_polymers("src/bin/s_gc_data.txt")
        .map(|entry| EntryWithGc {
            gc: gc_content(&entry.polymer),
            entry,
        })
        .collect_vec();

    let max_entry = entries
        .iter()
        .max_by(|entry1, entry2| entry1.gc.partial_cmp(&entry2.gc).unwrap())
        .unwrap();

    println!("{}", max_entry.entry.description);
    println!("{}", max_entry.gc * 100.0);
}

struct EntryWithGc {
    entry: FastaEntry<DnaNt>,
    gc: f64,
}

fn gc_content(dna: &AStr<DnaNt>) -> f64 {
    dna.iter().copied().filter(|&ch| ch == DnaNt::C || ch == DnaNt::G).count() as f64 / dna.len() as f64
}
