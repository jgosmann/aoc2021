use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Write};
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::{collections::BTreeMap, iter::FromIterator};

type Pos = (usize, usize);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Origami {
    by_x: BTreeMap<Pos, ()>,
    by_y: BTreeMap<Pos, ()>,
}

impl Origami {
    pub fn new(marks: &[Pos]) -> Self {
        Self {
            by_x: BTreeMap::from_iter(marks.iter().copied().map(|m| (m, ()))),
            by_y: BTreeMap::from_iter(marks.into_iter().map(|m| ((m.1, m.0), ()))),
        }
    }

    pub fn num_marks(&self) -> usize {
        self.by_x.len()
    }

    pub fn fold_x(&mut self, x: usize) {
        let folded_positions: Vec<Pos> = self
            .by_x
            .range((x, 0)..)
            .map(|(pos, _)| pos)
            .copied()
            .collect();
        for pos in folded_positions {
            self.move_mark(pos, (x - (pos.0 - x), pos.1));
        }
    }

    pub fn fold_y(&mut self, y: usize) {
        let folded_positions: Vec<Pos> = self
            .by_y
            .range((y, 0)..)
            .map(|(pos, _)| pos)
            .copied()
            .collect();
        for pos in folded_positions {
            self.move_mark((pos.1, pos.0), (pos.1, y - (pos.0 - y)));
        }
    }

    fn move_mark(&mut self, from: Pos, to: Pos) {
        self.by_x.remove(&from);
        self.by_x.insert(to, ());
        self.by_y.remove(&(from.1, from.0));
        self.by_y.insert((to.1, to.0), ());
    }
}

impl Display for Origami {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut carret = (0, 0);
        for pos in self.by_y.keys() {
            while carret.0 < pos.0 {
                f.write_char('\n')?;
                carret.0 += 1;
                carret.1 = 0;
            }
            while carret.1 < pos.1 {
                f.write_char(' ')?;
                carret.1 += 1;
            }
            f.write_char('#')?;
            carret.1 += 1;
        }
        f.write_char('\n')
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

fn read_input<R: BufRead>(reader: &mut R) -> Result<Origami, Box<dyn Error>> {
    let positions: Result<Vec<Pos>, Box<dyn Error>> = reader
        .lines()
        .take_while(|line| line.as_ref().map(|l| !l.trim().is_empty()).unwrap_or(false))
        .map(|line| {
            let parts: Result<Vec<usize>, ParseIntError> = line?
                .split(',')
                .map(|x| usize::from_str_radix(x, 10))
                .collect();
            let pos: [usize; 2] = parts?.try_into().map_err(|_| ParseError)?;
            Ok((pos[0], pos[1]))
        })
        .collect();
    Ok(Origami::new(&positions?))
}

fn process_fold(origami: &mut Origami, line: &str) -> Result<(), Box<dyn Error>> {
    let line = line.trim();
    if let Some(fold_def) = line.strip_prefix("fold along ") {
        let parts: Vec<&str> = fold_def.split('=').collect();
        let parts: [&str; 2] = parts.try_into().map_err(|_| ParseError)?;
        match parts {
            ["x", x] => origami.fold_x(usize::from_str_radix(x, 10)?),
            ["y", y] => origami.fold_y(usize::from_str_radix(y, 10)?),
            _ => return Err(Box::new(ParseError)),
        }
    } else {
        return Err(Box::new(ParseError));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut origami = read_input(&mut reader)?;
    let mut lines = reader.lines();

    if let Some(line) = lines.next() {
        process_fold(&mut origami, &line?)?;
        println!("Dots after first folding: {}", origami.num_marks());
    }

    for line in lines {
        process_fold(&mut origami, &line?)?;
    }

    println!("{}", origami);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_TRANSPARENCY_MARKS: [Pos; 18] = [
        (6, 10),
        (0, 14),
        (9, 10),
        (0, 3),
        (10, 4),
        (4, 11),
        (6, 0),
        (6, 12),
        (4, 1),
        (0, 13),
        (10, 12),
        (3, 4),
        (3, 0),
        (8, 4),
        (1, 10),
        (2, 14),
        (8, 10),
        (9, 0),
    ];

    static TEST_INPUT: &[u8] = include_bytes!("../test.input");

    #[test]
    fn test_origami() {
        let mut origami = Origami::new(&TEST_TRANSPARENCY_MARKS);
        origami.fold_y(7);
        assert_eq!(origami.num_marks(), 17);
        origami.fold_x(5);
        assert_eq!(origami.num_marks(), 16);
    }

    #[test]
    fn test_read_input() {
        let mut test_input = TEST_INPUT;
        let origami = read_input(&mut test_input).unwrap();
        assert_eq!(origami, Origami::new(&TEST_TRANSPARENCY_MARKS));
    }

    #[test]
    fn test_process_fold() {
        let mut test_input = TEST_INPUT;
        let mut origami = read_input(&mut test_input).unwrap();
        for line in test_input.lines() {
            process_fold(&mut origami, &line.unwrap()).unwrap();
        }
        assert_eq!(
            origami.to_string(),
            "\
#####
#   #
#   #
#   #
#####
"
        )
    }
}
