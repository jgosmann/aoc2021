use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Polymerizer {
    pair_insertion_rules: HashMap<[char; 2], char>,
}

impl Polymerizer {
    fn polymerize(&self, input: &str) -> String {
        let mut output = String::with_capacity(2 * input.len() - 1);
        let mut chars = input.chars();
        if let Some(c) = chars.next() {
            output.push(c);

            let mut prev = c;
            for c in chars {
                if let Some(&insertion) = self.pair_insertion_rules.get(&[prev, c]) {
                    output.push(insertion);
                }
                output.push(c);
                prev = c;
            }
        }
        output
    }
}

fn score(polymer: &str) -> usize {
    let mut counts: HashMap<char, usize> = HashMap::new();
    for c in polymer.chars() {
        counts
            .entry(c)
            .and_modify(|count| {
                *count += 1;
            })
            .or_insert(1);
    }
    counts.values().max().unwrap_or(&0) - counts.values().min().unwrap_or(&0)
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
    let mut polymer = lines.next().ok_or(ParseError)??;
    lines.next();
    let polymerizer = read_pair_insertion_rules(&mut lines)?;
    for _ in 0..10 {
        polymer = polymerizer.polymerize(&polymer);
    }
    Ok(score(&polymer))
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
    fn test_polymerization() {
        let mut pair_insertion_rules = HashMap::new();
        pair_insertion_rules.insert(['C', 'H'], 'B');
        pair_insertion_rules.insert(['C', 'B'], 'H');
        pair_insertion_rules.insert(['H', 'H'], 'N');
        pair_insertion_rules.insert(['N', 'N'], 'C');
        pair_insertion_rules.insert(['N', 'C'], 'B');
        let polymerizer = Polymerizer {
            pair_insertion_rules,
        };
        assert_eq!(polymerizer.polymerize("NNCB"), "NCNBCHB");
    }

    #[test]
    fn test_score() {
        assert_eq!(score("NBCCNBBBCBHCB"), 5);
    }

    #[test]
    fn test_process() {
        let mut input: &[u8] = include_bytes!("../../test.input");
        assert_eq!(process(&mut input).unwrap(), 1588);
    }
}
