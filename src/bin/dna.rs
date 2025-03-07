use std::collections::HashMap;
use rosalind::model::DnaNt;

fn main() {
    let data = include_str!("dna_data.txt").trim();

    let nt_count: HashMap<DnaNt, usize> =
        data.chars().map(DnaNt::from_char).fold(HashMap::default(), |mut nt_count, nt| {
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
