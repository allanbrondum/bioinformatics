use regex::Regex;

pub fn chars(s: &str) -> impl DoubleEndedIterator<Item = char> {
    s.trim().chars()
}

pub fn lines(s: &str) -> impl Iterator<Item = &str> {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
}

pub fn words(s: &str) -> impl Iterator<Item = &str> {
    s.trim().split_whitespace()
}

pub fn fasta_polymers(s: &str) -> impl Iterator<Item = String> {
    let mut res = Vec::new();

    let mut aas = String::new();
    for line in lines(s) {
        if line.starts_with(">") {
            if !aas.is_empty() {
                res.push(aas);
                aas = String::new();
            }
        } else {
            aas.push_str(line);
        }
    }

    if !aas.is_empty() {
        res.push(aas);
    }

    res.into_iter()
}

pub fn positions(s: &str, t: &str) -> impl Iterator<Item = usize> {
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
    while let Some(mtch) = regex.find_at(&s, offset) {
        offset = mtch.start() + 1;
        res.push(offset);
    }

    res.into_iter()
}
