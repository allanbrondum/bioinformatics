use rosalind::model::DnaNt;
use std::collections::HashMap;

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
