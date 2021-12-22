#[macro_use]
extern crate lazy_static;

use std::io::{self, BufRead};
use std::{error::Error, fmt::Display};

use regex::Regex;

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Cuboid {
    x_range: (isize, isize),
    y_range: (isize, isize),
    z_range: (isize, isize),
}

impl Cuboid {
    fn subtract(&self, other: &Cuboid) -> Vec<Cuboid> {
        let x_cuts = [
            (self.x_range.0, self.x_range.1.min(other.x_range.0)),
            (
                other.x_range.0.clamp(self.x_range.0, self.x_range.1),
                other.x_range.1.clamp(self.x_range.0, self.x_range.1),
            ),
            (self.x_range.0.max(other.x_range.1), self.x_range.1),
        ];
        let y_cuts = [
            (self.y_range.0, self.y_range.1.min(other.y_range.0)),
            (
                other.y_range.0.clamp(self.y_range.0, self.y_range.1),
                other.y_range.1.clamp(self.y_range.0, self.y_range.1),
            ),
            (self.y_range.0.max(other.y_range.1), self.y_range.1),
        ];
        let z_cuts = [
            (self.z_range.0, self.z_range.1.min(other.z_range.0)),
            (
                other.z_range.0.clamp(self.z_range.0, self.z_range.1),
                other.z_range.1.clamp(self.z_range.0, self.z_range.1),
            ),
            (self.z_range.0.max(other.z_range.1), self.z_range.1),
        ];

        let mut cuboids = Vec::with_capacity(6);
        if z_cuts[0].1 - z_cuts[0].0 > 0 {
            cuboids.push(Cuboid {
                x_range: self.x_range,
                y_range: self.y_range,
                z_range: z_cuts[0],
            });
        }
        if z_cuts[2].1 - z_cuts[2].0 > 0 {
            cuboids.push(Cuboid {
                x_range: self.x_range,
                y_range: self.y_range,
                z_range: z_cuts[2],
            });
        }

        if z_cuts[1].1 - z_cuts[1].0 > 0 {
            if y_cuts[0].1 - y_cuts[0].0 > 0 {
                cuboids.push(Cuboid {
                    x_range: self.x_range,
                    y_range: y_cuts[0],
                    z_range: z_cuts[1],
                });
            }
            if y_cuts[2].1 - y_cuts[2].0 > 0 {
                cuboids.push(Cuboid {
                    x_range: self.x_range,
                    y_range: y_cuts[2],
                    z_range: z_cuts[1],
                });
            }

            if y_cuts[1].1 - y_cuts[1].0 > 0 {
                if x_cuts[0].1 - x_cuts[0].0 > 0 {
                    cuboids.push(Cuboid {
                        x_range: x_cuts[0],
                        y_range: y_cuts[1],
                        z_range: z_cuts[1],
                    });
                }
                if x_cuts[2].1 - x_cuts[2].0 > 0 {
                    cuboids.push(Cuboid {
                        x_range: x_cuts[2],
                        y_range: y_cuts[1],
                        z_range: z_cuts[1],
                    });
                }
            }
        }

        cuboids
    }

    fn num_cubes(&self) -> usize {
        (self.x_range.1 - self.x_range.0) as usize
            * (self.y_range.1 - self.y_range.0) as usize
            * (self.z_range.1 - self.z_range.0) as usize
    }
}

struct RebootStep {
    is_on: bool,
    cuboid: Cuboid,
}

impl RebootStep {
    fn try_parse(input: &str) -> Result<Self, Box<dyn Error>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(on|off)\s+x=(-?\d+)..(-?\d+),y=(-?\d+)..(-?\d+),z=(-?\d+)..(-?\d+)$"
            )
            .unwrap();
        }

        let captures = RE.captures(input).ok_or(ParseError)?;
        let is_on = match captures.get(1).ok_or(ParseError)?.as_str() {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(ParseError),
        }?;

        Ok(Self {
            is_on,
            cuboid: Cuboid {
                x_range: (
                    isize::from_str_radix(captures.get(2).ok_or(ParseError)?.as_str(), 10)?,
                    isize::from_str_radix(captures.get(3).ok_or(ParseError)?.as_str(), 10)? + 1,
                ),
                y_range: (
                    isize::from_str_radix(captures.get(4).ok_or(ParseError)?.as_str(), 10)?,
                    isize::from_str_radix(captures.get(5).ok_or(ParseError)?.as_str(), 10)? + 1,
                ),
                z_range: (
                    isize::from_str_radix(captures.get(6).ok_or(ParseError)?.as_str(), 10)?,
                    isize::from_str_radix(captures.get(7).ok_or(ParseError)?.as_str(), 10)? + 1,
                ),
            },
        })
    }
}

struct Reactor {
    on_cubes: Vec<Cuboid>,
}

impl Reactor {
    fn new() -> Self {
        Self { on_cubes: vec![] }
    }

    fn apply(&mut self, reboot_step: &RebootStep) {
        let mut new_on_cubes = Vec::with_capacity(self.on_cubes.len() * 6);
        for cuboid in self.on_cubes.iter() {
            new_on_cubes.extend(cuboid.subtract(&reboot_step.cuboid));
        }
        if reboot_step.is_on {
            new_on_cubes.push(reboot_step.cuboid.clone());
        }
        self.on_cubes = new_on_cubes;
    }

    fn count_on(&self) -> usize {
        self.on_cubes.iter().map(|c| c.num_cubes()).sum()
    }
}

fn reboot_reactor<R: BufRead>(reader: &mut R) -> Result<(usize, usize), Box<dyn Error>> {
    let mut reactor_part1 = Reactor::new();
    let mut reactor_part2 = Reactor::new();
    for step in reader.lines().map(|line| RebootStep::try_parse(&line?)) {
        let step = step?;
        if -50 <= step.cuboid.x_range.0
            && step.cuboid.x_range.1 <= 51
            && -50 <= step.cuboid.y_range.0
            && step.cuboid.y_range.1 <= 51
            && -50 <= step.cuboid.z_range.0
            && step.cuboid.z_range.1 <= 51
        {
            reactor_part1.apply(&step);
        }
        reactor_part2.apply(&step)
    }
    Ok((reactor_part1.count_on(), reactor_part2.count_on()))
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let (part1, part2) = reboot_reactor(&mut stdin.lock())?;
    println!("Part 1: {}", part1);
    println!("Part 2: {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let mut buf: &[u8] = include_bytes!("../test.input");
        assert_eq!(reboot_reactor(&mut buf).unwrap().0, 590784);
    }

    #[test]
    fn test_part2() {
        let mut buf: &[u8] = include_bytes!("../test2.input");
        assert_eq!(
            reboot_reactor(&mut buf).unwrap(),
            (474140, 2758514936282235)
        );
    }
}
