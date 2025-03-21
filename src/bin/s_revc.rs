use bioinformatics::polymers::DnaNt;
use bioinformatics::string_model::AString;
use bioinformatics::util::chars_file;

fn main() {
    let recv: AString<_> = chars_file::<DnaNt>("src/bin/s_revc_data.txt")
        .rev()
        .map(DnaNt::bonding_complement)
        .collect();

    println!("{}", recv)
}
