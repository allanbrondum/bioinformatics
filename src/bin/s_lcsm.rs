#![feature(ascii_char)]

use bioinformatics::polymers::DnaNt;
use bioinformatics::string;
use bioinformatics::util::fasta_polymers_file;
use itertools::Itertools;
use std::time::Instant;

fn main() {
    let strs: Vec<_> = fasta_polymers_file::<DnaNt>("src/bin/s_lcsm_data.txt").collect_vec();

    let start = Instant::now();

    let mut lcs = strs[0].polymer.as_str();
    for str in strs.iter().skip(1) {
        lcs = string::lcs_simple(lcs, &str.polymer);
    }

    println!("{}", lcs);

    println!("elapsed: {:?}", start.elapsed());
}

//alg longest common substr
