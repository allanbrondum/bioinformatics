use std::collections::HashMap;
use rosalind::model::DnaNt;

fn main() {
    let data = include_str!("revc_data.txt").trim();

    let recv: String = data
        .chars()
        .map(DnaNt::from_char)
        .rev()
        .map(DnaNt::bonding_complement)
        .map(DnaNt::to_char)
        .collect();

    println!("{}", recv)
}
