use std::fs;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
    BufReader::new(File::open(path).unwrap()).lines().map(|res|res.unwrap())
}

#[derive(Debug, Clone, Default)]
pub struct FastaEntry {
    pub description: String,
    pub polymer: String,
}

impl FastaEntry {
    fn new(description: String) -> Self {
        Self {
            description,
            polymer: String::default(),
        }
    }
}

pub fn fasta_polymers(path: impl AsRef<Path>) -> impl Iterator<Item = FastaEntry> {
    let mut res = Vec::new();

    let mut entry = None;
    for line in lines_file(path) {
        if let Some(descr) = line.strip_prefix(">") {
            if let Some(entry) = entry.take() {
                res.push(entry);
            }
            entry = Some(FastaEntry::new(descr.to_string()));
        } else if let Some(entry) = &mut entry {
            entry.polymer.push_str(&line);
        } else {
            panic!("invalid format");
        }
    }

    if let Some(entry) = entry.take() {
        res.push(entry);
    }

    res.into_iter()
}

pub fn positions<T: PartialEq>(s: &[T], t: &[T]) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = find(&s[offset..], t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn find<T: PartialEq>(s: &[T], t: &[T]) -> Option<usize> {
    'outer: for i in 0..s.len() {
        for j in 0..t.len() {
            if i + j >= s.len() || s[i + j] != t[j] {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

pub fn positions_str(s: &str, t: &str) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = s[offset..].find(t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn positions_regex(s: &str, regex: &Regex) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(mtch) = regex.find_at(s, offset) {
        offset = mtch.start() + 1;
        res.push(offset);
    }

    res.into_iter()
}
