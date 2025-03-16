#![feature(ascii_char)]

use itertools::Itertools;
use rosalind::string::{AsciiStr, lcs};
use rosalind::util::fasta_polymers;
use std::time::Instant;
use rosalind::string;

fn main() {
    let strs: Vec<_> = fasta_polymers("src/bin/s_lcsm_data.txt").collect_vec();

    let start = Instant::now();

    let mut lcs = strs[0].polymer.as_ascii().unwrap();
    for str in strs.iter().skip(1) {
        lcs = string::lcs(lcs, str.polymer.as_ascii().unwrap());
    }

    println!("{}", AsciiStr(lcs));

    println!("elapsed: {:?}", start.elapsed());
}

//alg longest common substr
