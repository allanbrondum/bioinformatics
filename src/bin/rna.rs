fn main() {
    let data = include_str!("rna_data.txt");

    let rna: String = data
        .chars()
        .map(|nt| match nt {
            'T' => 'U',
            _ => nt,
        })
        .collect();

    println!("{}", rna)
}
