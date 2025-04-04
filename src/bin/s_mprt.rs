use bioinformatics::polymers::ProteinAa;
use bioinformatics::string::indexes_regex;
use bioinformatics::string_model::AString;
use bioinformatics::util::{fasta_polymers, lines};
use regex::Regex;
use reqwest::Url;

fn main() {
    let data = include_str!("s_mprt_data.txt");

    let client = reqwest::blocking::Client::new();
    let regex = Regex::new("N[^P][ST][^P]").unwrap();
    for prot_id in lines(data) {
        let db_id = prot_id.split("_").next().unwrap();
        let aas = prot_aas(&client, db_id).to_string();

        let mut any_matches = false;
        for position in indexes_regex(&aas, &regex) {
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

fn prot_aas(client: &reqwest::blocking::Client, prot_id: &str) -> AString<ProteinAa> {
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

    let mut polymers = fasta_polymers::<ProteinAa>(&fasta);

    match (polymers.next(), polymers.next()) {
        (Some(polymer), None) => polymer.polymer,
        _ => panic!("not exactly one polymer"),
    }
}

//alg regex substr
