use std::collections::HashMap;

fn main() {
    let data = include_str!("dna_data.txt");

    let nt_count: HashMap<char, usize> =
        data.chars().fold(HashMap::default(), |mut nt_count, nt| {
            *nt_count.entry(nt).or_default() += 1;
            nt_count
        });

    println!(
        "{} {} {} {}",
        nt_count.get(&'A').copied().unwrap_or_default(),
        nt_count.get(&'C').copied().unwrap_or_default(),
        nt_count.get(&'G').copied().unwrap_or_default(),
        nt_count.get(&'T').copied().unwrap_or_default()
    )
}
