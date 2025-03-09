use itertools::Itertools;
use rosalind::util::fasta_polymers;

fn main() {
    let data = include_str!("lcsm_data.txt");

    let mut strs:Vec<_> = fasta_polymers(data).collect_vec();

    let first = strs.remove(0);
    let mut substr = "";
    for i in 0..first.len() {
        for j in i + 1..first.len() {
            let substr_cur = &first[i..j];

            if strs.iter().all(|str| str.contains(substr_cur)) {
                if substr_cur.len() > substr.len() {
                    substr = substr_cur;
                }
            } else {
                break;
            }
        }
    }

    println!("{}", substr);
}

//alg longest common substr