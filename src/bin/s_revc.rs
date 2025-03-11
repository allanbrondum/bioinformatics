use rosalind::polymers::DnaNt;
use rosalind::util::chars;

fn main() {
    let data = include_str!("s_revc_data.txt");

    let recv: String = chars(data)
        .map(DnaNt::from_char)
        .rev()
        .map(DnaNt::bonding_complement)
        .map(DnaNt::to_char)
        .collect();

    println!("{}", recv)
}
