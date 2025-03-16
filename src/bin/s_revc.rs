use rosalind::polymers::DnaNt;
use rosalind::string_model::AString;
use rosalind::util::chars_file;

fn main() {
    let recv: AString<_> = chars_file::<DnaNt>("src/bin/s_revc_data.txt")
        .rev()
        .map(DnaNt::bonding_complement)
        .collect();

    println!("{}", recv)
}
