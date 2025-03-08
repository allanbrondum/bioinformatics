use itertools::Itertools;
use rosalind::util::words;

fn main() {
    let data = include_str!("iprb_data.txt");

    let [hom_dom, het, hom_rec]: [f64; 3] = words(data)
        .map(|val| val.parse().unwrap())
        .collect_array()
        .unwrap();

    let tot = hom_dom + het + hom_rec;

    let prob_phen = (hom_dom / tot) * ((hom_dom - 1.0) / (tot - 1.0))
        + (het / tot) * ((het - 1.0) / (tot - 1.0)) * (3.0 / 4.0)
        + (hom_dom / tot) * (het / (tot - 1.0))
        + (het / tot) * (hom_dom / (tot - 1.0))
        + (hom_dom / tot) * (hom_rec / (tot - 1.0))
        + (hom_rec / tot) * (hom_dom / (tot - 1.0))
        + (het / tot) * (hom_rec / (tot - 1.0)) * (1.0 / 2.0)
        + (hom_rec / tot) * (het / (tot - 1.0)) * (1.0 / 2.0);

    println!("{}", prob_phen)
}
