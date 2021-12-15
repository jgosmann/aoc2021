use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Polymerizer {
    pair_insertion_rules: HashMap<[char; 2], char>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Polymer {
    first: char,
    pair_counts: HashMap<[char; 2], usize>,
}

impl From<&str> for Polymer {
    fn from(polymer: &str) -> Self {
        let mut chars = polymer.chars();
        let first = chars.next().unwrap_or(' ');
        let mut pair_counts = HashMap::new();
        let mut prev = first;
        for c in chars {
            pair_counts
                .entry([prev, c])
                .and_modify(|count| *count += 1)
                .or_insert(1);
            prev = c;
        }
        Self { first, pair_counts }
    }
}

impl Polymer {
    fn score(&self) -> usize {
        let mut counts: HashMap<char, usize> = HashMap::new();
        counts.insert(self.first, 1);
        for (k, &v) in self.pair_counts.iter() {
            counts
                .entry(k[1])
                .and_modify(|count| {
                    *count += v;
                })
                .or_insert(v);
        }
        counts.values().max().unwrap_or(&0) - counts.values().min().unwrap_or(&0)
    }
}

impl Polymerizer {
    fn polymerize(&self, template: &Polymer) -> Polymer {
        let mut pair_counts = HashMap::new();
        for (&k, &v) in template.pair_counts.iter() {
            if let Some(&insertion) = self.pair_insertion_rules.get(&k) {
                pair_counts
                    .entry([k[0], insertion])
                    .and_modify(|count| *count += v)
                    .or_insert(v);
                pair_counts
                    .entry([insertion, k[1]])
                    .and_modify(|count| *count += v)
                    .or_insert(v);
            } else {
                pair_counts
                    .entry(k)
                    .and_modify(|count| *count += 1)
                    .or_insert(v);
            }
        }
        Polymer {
            first: template.first,
            pair_counts,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

fn read_pair_insertion_rules<I: Iterator<Item = Result<String, std::io::Error>>>(
    lines: &mut I,
) -> Result<Polymerizer, Box<dyn Error>> {
    let mut pair_insertion_rules: HashMap<[char; 2], char> = HashMap::new();
    for line in lines {
        let line = line?;
        let parts: Vec<&str> = line.split("->").map(|p| p.trim()).collect();
        let [lhs, rhs]: [&str; 2] = parts.try_into().map_err(|_| ParseError)?;
        let mut lhs_chars_iter = lhs.chars();
        pair_insertion_rules.insert(
            [
                lhs_chars_iter.next().ok_or(ParseError)?,
                lhs_chars_iter.next().ok_or(ParseError)?,
            ],
            rhs.chars().next().ok_or(ParseError)?,
        );
    }
    Ok(Polymerizer {
        pair_insertion_rules,
    })
}

fn process<R: BufRead>(reader: &mut R) -> Result<usize, Box<dyn Error>> {
    let mut lines = reader.lines();
    let mut polymer = Polymer::from(lines.next().ok_or(ParseError)??.as_str());
    lines.next();
    let polymerizer = read_pair_insertion_rules(&mut lines)?;
    for _ in 0..40 {
        polymer = polymerizer.polymerize(&polymer);
    }
    Ok(polymer.score())
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    println!("{}", process(&mut stdin.lock())?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_pair_insertion_rules() {
        let input: &[u8] = "\
            CH -> B
            HH -> N"
            .as_bytes();
        let mut pair_insertion_rules = HashMap::new();
        pair_insertion_rules.insert(['C', 'H'], 'B');
        pair_insertion_rules.insert(['H', 'H'], 'N');
        let expected = Polymerizer {
            pair_insertion_rules,
        };
        assert_eq!(
            read_pair_insertion_rules(&mut input.lines()).unwrap(),
            expected
        );
    }

    #[test]
    fn test_process() {
        let mut input: &[u8] = include_bytes!("../../test.input");
        assert_eq!(process(&mut input).unwrap(), 2188189693529);
    }
}
