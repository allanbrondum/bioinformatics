use crate::alphabet_model::CharT;
use crate::string_model::AString;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

pub fn chars<C: CharT>(s: &str) -> impl DoubleEndedIterator<Item = C> {
    s.trim()
        .chars()
        .map(|ch| C::from_char(ch).expect("invalid char"))
}

pub fn chars_file<C: CharT>(path: impl AsRef<Path>) -> impl DoubleEndedIterator<Item = C> {
    fs::read_to_string(path)
        .unwrap()
        .into_chars()
        .filter(|ch|!ch.is_whitespace())
        .map(|ch| C::from_char(ch).ok_or_else(||format!("invalid char {}", ch)).unwrap())
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
pub struct FastaEntry<C: CharT> {
    pub description: String,
    pub polymer: AString<C>,
}

impl<C: CharT> FastaEntry<C> {
    fn new(description: String) -> Self {
        Self {
            description,
            polymer: AString::default(),
        }
    }
}

pub fn fasta_polymers_file<C: CharT>(path: impl AsRef<Path>) -> impl Iterator<Item = FastaEntry<C>> {
    fasta_polymers_lines(lines_file(path))
}

pub fn fasta_polymers<C: CharT>(data: &str) -> impl Iterator<Item = FastaEntry<C>> {
    fasta_polymers_lines(lines(data))
}


fn fasta_polymers_lines<C: CharT, S: AsRef<str>>(lines: impl Iterator<Item = S>) -> impl Iterator<Item = FastaEntry<C>> {
    let mut res = Vec::new();

    let mut entry = None;
    for line in lines {
        let line = line.as_ref();
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
