use std::{
    collections::HashSet,
    error::Error,
    fmt::Display,
    io::{self, BufRead},
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SeaCucumbers {
    dimensions: (usize, usize),
    eastwards: HashSet<(usize, usize)>,
    southwards: HashSet<(usize, usize)>,
}

impl SeaCucumbers {
    fn new(
        dimensions: (usize, usize),
        eastwards: HashSet<(usize, usize)>,
        southwards: HashSet<(usize, usize)>,
    ) -> Self {
        Self {
            dimensions,
            eastwards,
            southwards,
        }
    }

    fn is_free(&self, pos: &(usize, usize)) -> bool {
        !self.eastwards.contains(pos) && !self.southwards.contains(pos)
    }

    fn step(&mut self) -> bool {
        let has_moved_eastwards = self.move_eastwards();
        let has_moved_southwards = self.move_southwards();
        has_moved_eastwards || has_moved_southwards
    }

    fn step_until_settled(&mut self) -> usize {
        (1..).skip_while(|_| self.step()).next().unwrap()
    }

    fn move_eastwards(&mut self) -> bool {
        let mut has_moved_any = false;
        self.eastwards = self
            .eastwards
            .iter()
            .copied()
            .map(|(x, y)| {
                let next_pos = ((x + 1) % self.dimensions.0, y);
                if self.is_free(&next_pos) {
                    has_moved_any = true;
                    next_pos
                } else {
                    (x, y)
                }
            })
            .collect();
        has_moved_any
    }

    fn move_southwards(&mut self) -> bool {
        let mut has_moved_any = false;
        self.southwards = self
            .southwards
            .iter()
            .copied()
            .map(|(x, y)| {
                let next_pos = (x, (y + 1) % self.dimensions.1);
                if self.is_free(&next_pos) {
                    has_moved_any = true;
                    next_pos
                } else {
                    (x, y)
                }
            })
            .collect();
        has_moved_any
    }
}

impl Display for SeaCucumbers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if self.eastwards.contains(&(x, y)) {
                    f.write_str(">")?;
                } else if self.southwards.contains(&(x, y)) {
                    f.write_str("v")?;
                } else {
                    f.write_str(".")?;
                }
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

fn read_sea_cucumbers<R: BufRead>(reader: &mut R) -> Result<SeaCucumbers, Box<dyn Error>> {
    let mut eastwards = HashSet::new();
    let mut southwards = HashSet::new();
    let mut dimensions = (0, 0);
    for (y, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        for (x, c) in line.chars().enumerate() {
            match c {
                '.' => (),
                '>' => {
                    eastwards.insert((x, y));
                }
                'v' => {
                    southwards.insert((x, y));
                }
                _ => Err(ParseError)?,
            }
        }
        dimensions = (line.len(), y + 1);
    }

    Ok(SeaCucumbers::new(dimensions, eastwards, southwards))
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut sea_cucumbers = read_sea_cucumbers(&mut stdin.lock())?;
    println!("{}", sea_cucumbers.step_until_settled());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &[u8] = include_bytes!("../test.input");

    #[test]
    fn test_part1() {
        let mut buf: &[u8] = TEST_INPUT;
        let mut sea_cucumbers = read_sea_cucumbers(&mut buf).unwrap();
        assert_eq!(sea_cucumbers.step_until_settled(), 58);
    }
}
