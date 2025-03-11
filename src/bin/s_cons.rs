use itertools::Itertools;
use rosalind::polymers::DnaNt;
use rosalind::util::fasta_polymers;

fn main() {
    let dnas = fasta_polymers("src/bin/s_cons_data.txt")
        .map(|entry| entry.polymer.chars().map(DnaNt::from_char).collect_vec())
        .collect_vec();
    let mut profile = vec![Freq::default(); dnas[0].len()];
    for dna in &dnas {
        for (freq, nt) in profile.iter_mut().zip(dna.iter()) {
            freq.observe(*nt);
        }
    }
    let cons: String = profile
        .iter()
        .map(|freq| freq.most_frequent())
        .map(DnaNt::to_char)
        .collect();

    println!("{}", cons);
    println!("A: {}", profile.iter().map(|freq| freq.a).join(" "));
    println!("C: {}", profile.iter().map(|freq| freq.c).join(" "));
    println!("G: {}", profile.iter().map(|freq| freq.g).join(" "));
    println!("T: {}", profile.iter().map(|freq| freq.t).join(" "));
}

#[derive(Debug, Clone, Default)]
struct Freq {
    a: usize,
    c: usize,
    g: usize,
    t: usize,
}

impl Freq {
    fn observe(&mut self, nt: DnaNt) {
        *match nt {
            DnaNt::A => &mut self.a,
            DnaNt::C => &mut self.c,
            DnaNt::G => &mut self.g,
            DnaNt::T => &mut self.t,
        } += 1;
    }

    fn most_frequent(&self) -> DnaNt {
        let max = self.a.max(self.c).max(self.g).max(self.t);
        if self.a == max {
            DnaNt::A
        } else if self.c == max {
            DnaNt::C
        } else if self.g == max {
            DnaNt::G
        } else if self.t == max {
            DnaNt::T
        } else {
            unreachable!()
        }
    }
}
