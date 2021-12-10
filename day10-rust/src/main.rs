use std::io::{self, BufRead};

fn map_to_closing(opening: char) -> Option<char> {
    match opening {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '<' => Some('>'),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxCheckResult {
    Ok,
    Invalid(char),
    Incomplete(Vec<char>),
}

fn check_syntax(line: &str) -> SyntaxCheckResult {
    let mut stack = vec![];
    for c in line.chars() {
        if let Some(closing) = map_to_closing(c) {
            stack.push(closing);
        } else if stack.last() == Some(&c) {
            stack.pop();
        } else {
            return SyntaxCheckResult::Invalid(c);
        }
    }
    if stack.len() > 0 {
        return SyntaxCheckResult::Incomplete(stack);
    }
    return SyntaxCheckResult::Ok;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SyntaxCheckScore {
    invalid_score: u64,
    incomplete_score: u64,
}

fn score_invalid_char(c: char) -> u64 {
    match c {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => 0,
    }
}

fn score_incomplete_char(c: char) -> u64 {
    match c {
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4,
        _ => 0,
    }
}

fn syntax_score<R: BufRead>(reader: &mut R) -> SyntaxCheckScore {
    let syntax_results: Vec<SyntaxCheckResult> = reader
        .lines()
        .map(|line| check_syntax(&line.unwrap()))
        .collect();
    let invalid_score = syntax_results
        .iter()
        .map(|result| match result {
            SyntaxCheckResult::Invalid(c) => score_invalid_char(*c),
            _ => 0,
        })
        .sum();
    let mut incomplete_scores: Vec<u64> = syntax_results
        .iter()
        .filter_map(|result| match result {
            SyntaxCheckResult::Incomplete(chars) => Some(
                chars
                    .iter()
                    .rev()
                    .copied()
                    .map(score_incomplete_char)
                    .reduce(|acc, x| acc * 5 + x)
                    .unwrap_or_default(),
            ),
            _ => None,
        })
        .collect();
    incomplete_scores.sort();

    SyntaxCheckScore {
        invalid_score,
        incomplete_score: incomplete_scores[incomplete_scores.len() / 2],
    }
}

fn main() {
    let stdin = io::stdin();
    let scores = syntax_score(&mut stdin.lock());
    println!("Score of corrupt lines: {}", scores.invalid_score);
    println!("Score of incomplete lines: {}", scores.incomplete_score);
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &str = include_str!("../test.input");

    #[test]
    fn test_check_syntax_ok() {
        assert_eq!(check_syntax("<([{}])>"), SyntaxCheckResult::Ok);
    }

    #[test]
    fn test_check_syntax_invalid() {
        assert_eq!(check_syntax("{()()()>"), SyntaxCheckResult::Invalid('>'));
    }

    #[test]
    fn test_check_syntax_incomplete() {
        assert_eq!(
            check_syntax("([]{()}<"),
            SyntaxCheckResult::Incomplete(vec![')', '>'])
        );
    }

    #[test]
    fn test_syntax_score() {
        let mut buf: &[u8] = TEST_INPUT.as_bytes();
        assert_eq!(
            syntax_score(&mut buf),
            SyntaxCheckScore {
                invalid_score: 26397,
                incomplete_score: 288957,
            }
        );
    }
}
