use rosalind::polymers::DnaNt;
use rosalind::util::chars_file;

fn main() {
    let recv: String = chars_file("src/bin/s_revc_data.txt")
        .map(DnaNt::from_char)
        .rev()
        .map(DnaNt::bonding_complement)
        .map(DnaNt::to_char)
        .collect();

    println!("{}", recv)
}
