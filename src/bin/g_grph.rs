use bioinformatics::util::fasta_polymers_file;
use hashbrown::HashMap;
use itertools::Itertools;
use bioinformatics::polymers::DnaNt;
use bioinformatics::string_model::AString;

fn main() {
    let polymers = fasta_polymers_file::<DnaNt>("src/bin/g_grph_data.txt");

    for pair in polymers.permutations(2) {
        let [entry1, entry2] = pair.try_into().unwrap();
        if entry1.polymer[entry1.polymer.len() - 3.. ] == entry2.polymer[..3] {
            println!("{} {}", entry1.description, entry2.description);
        }
    }



    // for (prefix, entries) in prefix_to_polymers {
    //     for (entry1, entry2) in entries.iter().tuple_combinations() {
    //         println!("{} {} {}", entry1.description, entry2.description, prefix);
    //     }
    // }
}
