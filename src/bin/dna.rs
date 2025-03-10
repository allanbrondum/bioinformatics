use rosalind::polymers::DnaNt;
use rosalind::util::chars;
use std::collections::HashMap;

fn main() {
    let data = include_str!("dna_data.txt");

    let nt_count: HashMap<DnaNt, usize> =
        chars(data)
            .map(DnaNt::from_char)
            .fold(HashMap::default(), |mut nt_count, nt| {
                *nt_count.entry(nt).or_default() += 1;
                nt_count
            });

    println!(
        "{} {} {} {}",
        nt_count.get(&DnaNt::A).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::C).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::G).copied().unwrap_or_default(),
        nt_count.get(&DnaNt::T).copied().unwrap_or_default()
    )
}
