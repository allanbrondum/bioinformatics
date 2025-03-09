use regex::Regex;
use reqwest::Url;
use rosalind::util::{fasta_polymers, lines, positions_regex};

fn main() {
    let data = include_str!("mprt_data.txt");

    let client = reqwest::blocking::Client::new();
    for prot_id in lines(data) {
        let db_id = prot_id.split("_").next().unwrap();
        let aas = prot_aas(&client, db_id);

        let regex = Regex::new("N[^P][ST][^P]").unwrap();

        let mut any_matches = false;
        for position in positions_regex(&aas, &regex) {
            if !any_matches {
                println!("{}", prot_id);
                any_matches = true;
            }
            print!("{} ", position);
        }
        if any_matches {
            println!();
        }
    }
}

fn prot_aas(client: &reqwest::blocking::Client, prot_id: &str) -> String {
    let fasta = client
        .get(
            format!("https://rest.uniprot.org/uniprotkb/{}.fasta", prot_id)
                .parse::<Url>()
                .unwrap(),
        )
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .unwrap();

    let mut polymers = fasta_polymers(&fasta);

    match (polymers.next(), polymers.next()) {
        (Some(polymer), None) => polymer,
        _ => panic!("not exactly one polymer"),
    }
}

//alg regex substr