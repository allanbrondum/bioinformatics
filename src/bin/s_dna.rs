use rosalind::polymers::DnaNt;
use rosalind::util::chars_file;
use std::collections::HashMap;

fn main() {
    let nt_count: HashMap<DnaNt, usize> = chars_file::<DnaNt>("src/bin/s_dna_data.txt").fold(
        HashMap::default(),
        |mut nt_count, nt| {
            *nt_count.entry(nt).or_default() += 1;
            nt_count
        },
    );

    println!(
        "{} {} {} {}",
        nt_count.get(&DnaNt::A).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::C).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::G).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::T).copied().unwrap_or_default()
    )
}
