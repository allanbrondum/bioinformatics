use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use crate::alphabet_model::CharT;
use crate::string_model::AString;

pub fn chars(s: &str) -> impl DoubleEndedIterator<Item = char> {
    s.trim().chars()
}

pub fn chars_file(path: impl AsRef<Path>) -> impl DoubleEndedIterator<Item = char> {
    fs::read_to_string(path).unwrap().into_chars()
}

pub fn lines(s: &str) -> impl Iterator<Item = &str> {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
}

pub fn words(s: &str) -> impl Iterator<Item = &str> {
    s.split_whitespace()
}

pub fn lines_file(path: impl AsRef<Path>) -> impl Iterator<Item = String> {
    BufReader::new(File::open(path).unwrap())
        .lines()
        .map(|res| res.unwrap())
}

#[derive(Debug, Clone, Default)]
pub struct FastaEntry<C:CharT> {
    pub description: String,
    pub polymer: AString<C>,
}

impl<C:CharT> FastaEntry<C> {
    fn new(description: String) -> Self {
        Self {
            description,
            polymer: AString::default(),
        }
    }
}

pub fn fasta_polymers<C:CharT>(path: impl AsRef<Path>) -> impl Iterator<Item = FastaEntry<C>> {
    let mut res = Vec::new();

    let mut entry = None;
    for line in lines_file(path) {
        if let Some(descr) = line.strip_prefix(">") {
            if let Some(entry) = entry.take() {
                res.push(entry);
            }
            entry = Some(FastaEntry::new(descr.to_string()));
        } else if let Some(entry) = &mut entry {
            let astring = AString::from_str(&line).unwrap();
            entry.polymer.push_str(&astring);
        } else {
            panic!("invalid format");
        }
    }

    if let Some(entry) = entry.take() {
        res.push(entry);
    }

    res.into_iter()
}
