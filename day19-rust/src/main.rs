use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead};
use std::iter::FromIterator;

type Pos = (i64, i64, i64);

fn dist(a: &Pos, b: &Pos) -> u64 {
    (a.0 - b.0).pow(2) as u64 + (a.1 - b.1).pow(2) as u64 + (a.2 - b.2).pow(2) as u64
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Scanner {
    beacons: HashSet<Pos>,
    dists: HashSet<u64>,
    scanner: Pos,
}

impl Scanner {
    fn new(beacons: &[Pos]) -> Self {
        Self {
            beacons: HashSet::from_iter(beacons.iter().cloned()),
            dists: beacons
                .iter()
                .enumerate()
                .flat_map(|(i, a)| beacons[i + 1..].iter().map(move |b| dist(a, b)))
                .collect(),
            scanner: (0, 0, 0),
        }
    }

    fn rotate_around_x(&self) -> Self {
        Self {
            beacons: self
                .beacons
                .iter()
                .map(|p| {
                    (
                        p.0,
                        p.2 - self.scanner.2 + self.scanner.1,
                        -p.1 + self.scanner.1 + self.scanner.2,
                    )
                })
                .collect(),
            dists: self.dists.clone(),
            scanner: self.scanner,
        }
    }

    fn rotate_around_y(&self) -> Self {
        Self {
            beacons: self
                .beacons
                .iter()
                .map(|p| {
                    (
                        p.2 - self.scanner.2 + self.scanner.0,
                        p.1,
                        -p.0 + self.scanner.0 + self.scanner.2,
                    )
                })
                .collect(),
            dists: self.dists.clone(),
            scanner: self.scanner,
        }
    }

    fn rotate_around_z(&self) -> Self {
        Self {
            beacons: self
                .beacons
                .iter()
                .map(|p| {
                    (
                        p.1 - self.scanner.1 + self.scanner.0,
                        -p.0 + self.scanner.0 + self.scanner.1,
                        p.2,
                    )
                })
                .collect(),
            dists: self.dists.clone(),
            scanner: self.scanner,
        }
    }

    fn translate(&self, delta: Pos) -> Self {
        Self {
            beacons: self
                .beacons
                .iter()
                .map(|p| (p.0 + delta.0, p.1 + delta.1, p.2 + delta.2))
                .collect(),
            dists: self.dists.clone(),
            scanner: (
                self.scanner.0 + delta.0,
                self.scanner.1 + delta.1,
                self.scanner.2 + delta.2,
            ),
        }
    }
}

struct ScannerOrientationsIterator {
    cur: Scanner,
    index: u8,
}

impl ScannerOrientationsIterator {
    fn new(scanner: Scanner) -> Self {
        Self {
            cur: scanner,
            index: 0,
        }
    }
}

impl Iterator for ScannerOrientationsIterator {
    type Item = Scanner;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 24 {
            return None;
        }
        self.cur = self.cur.rotate_around_x();
        if self.index % 4 == 0 && self.index <= 16 {
            self.cur = self.cur.rotate_around_y();
        }
        if self.index == 16 {
            self.cur = self.cur.rotate_around_z();
        }
        if self.index == 20 {
            self.cur = self.cur.rotate_around_z().rotate_around_z();
        }
        self.index += 1;
        return Some(self.cur.clone());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Map {
    beacons: HashSet<Pos>,
    dists: HashSet<u64>,
    scanners: HashSet<Pos>,
}

impl Map {
    fn new() -> Self {
        Self {
            beacons: HashSet::new(),
            dists: HashSet::new(),
            scanners: HashSet::new(),
        }
    }

    fn insert(&mut self, scanner: &Scanner) {
        self.beacons.extend(scanner.beacons.iter());
        self.dists.extend(
            self.beacons
                .iter()
                .flat_map(|a| scanner.beacons.iter().map(move |b| dist(a, b)))
                .filter(|&d| d != 0),
        );
        self.scanners.insert(scanner.scanner);
    }

    fn might_be_aligned(&self, scanner: &Scanner) -> bool {
        scanner
            .dists
            .iter()
            .filter(|d| self.dists.contains(d))
            .count()
            >= 12
    }

    fn is_aligned(&self, scanner: &Scanner) -> bool {
        scanner
            .beacons
            .iter()
            .filter(|beacon| self.beacons.contains(beacon))
            .count()
            >= 12
    }

    fn try_aligned_insert(&mut self, scanner: &Scanner) -> bool {
        if self.might_be_aligned(&scanner) {
            for scanner in ScannerOrientationsIterator::new(scanner.clone()) {
                let reference_beacon = scanner.beacons.iter().next().unwrap();
                for beacon in self.beacons.iter() {
                    let delta = (
                        beacon.0 - reference_beacon.0,
                        beacon.1 - reference_beacon.1,
                        beacon.2 - reference_beacon.2,
                    );
                    let translated_scanner = scanner.translate(delta);
                    if self.is_aligned(&translated_scanner) {
                        self.insert(&translated_scanner);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn max_scanner_manhatten_dist(&self) -> u64 {
        self.scanners
            .iter()
            .flat_map(|s0| {
                self.scanners.iter().map(move |s1| {
                    (s0.0 - s1.0).abs() as u64
                        + (s0.1 - s1.1).abs() as u64
                        + (s0.2 - s1.2).abs() as u64
                })
            })
            .max()
            .unwrap_or(0)
    }
}

fn build_map(scanners: &Vec<Scanner>) -> Map {
    let mut map = Map::new();
    let mut queue = VecDeque::from_iter(scanners.iter());
    map.insert(queue.pop_front().unwrap());
    while let Some(scanner) = queue.pop_front() {
        if !map.try_aligned_insert(scanner) {
            queue.push_back(scanner);
        }
    }
    map
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

fn parse_pos(input: &str) -> Result<Pos, Box<dyn Error>> {
    let mut coordinates = input
        .split(',')
        .map(|part| i64::from_str_radix(part.trim(), 10));
    Ok((
        coordinates.next().ok_or(ParseError)??,
        coordinates.next().ok_or(ParseError)??,
        coordinates.next().ok_or(ParseError)??,
    ))
}

fn parse_scanner<R: BufRead>(input: &mut R) -> Result<Scanner, Box<dyn Error>> {
    let is_boundary = |line: &Result<String, io::Error>| {
        line.as_ref().map_or(false, |line| line.trim().is_empty())
    };
    let beacons: Vec<Pos> = input
        .lines()
        .take_while(|line| !is_boundary(line))
        .filter_map(|line| match line {
            Ok(line) => {
                if line.is_empty() {
                    None
                } else {
                    Some(parse_pos(&line))
                }
            }
            Err(err) => Some(Err(Box::new(err))),
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    Ok(Scanner::new(&beacons))
}

fn parse_input<R: BufRead>(input: &mut R) -> Result<Vec<Scanner>, Box<dyn Error>> {
    let mut scanners = vec![];
    let mut line = String::new();
    while input.read_line(&mut line).is_ok() {
        if !line.starts_with("---") {
            return Ok(scanners);
        }
        scanners.push(parse_scanner(input)?);
        line.clear();
    }
    Ok(scanners)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let scanners = parse_input(&mut stdin.lock())?;
    let map = build_map(&scanners);
    println!("Number of beacons: {}", map.beacons.len());
    println!(
        "Max. Manhatten distance between scanners {}",
        map.max_scanner_manhatten_dist()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter::FromIterator;

    #[test]
    fn test_parsing() {
        let mut input: &[u8] = b"--- scanner 0 ---
0,2,0
4,1,1
3,3,2

--- scanner 1 ---
-1,-1,3
-5,0,4
-2,1,5";
        assert_eq!(
            parse_input(&mut input).unwrap(),
            vec![
                Scanner::new(&[(0, 2, 0), (4, 1, 1), (3, 3, 2)]),
                Scanner::new(&[(-1, -1, 3), (-5, 0, 4), (-2, 1, 5)]),
            ]
        );
    }

    #[test]
    fn test_orientations_iterator() {
        let scanner = Scanner::new(&[
            (-1, -1, 1),
            (-2, -2, 2),
            (-3, -3, 3),
            (-2, -3, 1),
            (5, 6, -4),
            (8, 0, 7),
        ]);
        let all_orientations: Vec<Scanner> = ScannerOrientationsIterator::new(scanner).collect();
        assert!(all_orientations.contains(&Scanner::new(&[
            (1, -1, 1),
            (2, -2, 2),
            (3, -3, 3),
            (2, -1, 3),
            (-5, 4, -6),
            (-8, -7, 0)
        ])));
        assert!(all_orientations.contains(&Scanner::new(&[
            (-1, -1, -1),
            (-2, -2, -2),
            (-3, -3, -3),
            (-1, -3, -2),
            (4, 6, 5),
            (-7, 0, 8)
        ])));
        assert!(all_orientations.contains(&Scanner::new(&[
            (1, 1, -1),
            (2, 2, -2),
            (3, 3, -3),
            (1, 3, -2),
            (-4, -6, 5),
            (7, 0, 8)
        ])));
        assert!(all_orientations.contains(&Scanner::new(&[
            (1, 1, 1),
            (2, 2, 2),
            (3, 3, 3),
            (3, 1, 2),
            (-6, -4, -5),
            (0, 7, -8)
        ])));
        assert!(all_orientations.contains(&Scanner::new(&[
            (-1, -1, 1),
            (-2, -2, 2),
            (-3, -3, 3),
            (-2, -3, 1),
            (5, 6, -4),
            (8, 0, 7)
        ])));
        assert!(all_orientations.contains(&Scanner::new(&[
            (-1, -1, 1),
            (-2, -2, 2),
            (-3, -3, 3),
            (-2, -3, 1),
            (5, 6, -4),
            (8, 0, 7)
        ])));
    }

    #[test]
    fn test_build_map() {
        let mut input: &[u8] = include_bytes!("../test.input");
        let scanners = parse_input(&mut input).unwrap();
        let map = build_map(&scanners);
        assert_eq!(
            map.beacons,
            HashSet::from_iter(
                [
                    (-892, 524, 684),
                    (-876, 649, 763),
                    (-838, 591, 734),
                    (-789, 900, -551),
                    (-739, -1745, 668),
                    (-706, -3180, -659),
                    (-697, -3072, -689),
                    (-689, 845, -530),
                    (-687, -1600, 576),
                    (-661, -816, -575),
                    (-654, -3158, -753),
                    (-635, -1737, 486),
                    (-631, -672, 1502),
                    (-624, -1620, 1868),
                    (-620, -3212, 371),
                    (-618, -824, -621),
                    (-612, -1695, 1788),
                    (-601, -1648, -643),
                    (-584, 868, -557),
                    (-537, -823, -458),
                    (-532, -1715, 1894),
                    (-518, -1681, -600),
                    (-499, -1607, -770),
                    (-485, -357, 347),
                    (-470, -3283, 303),
                    (-456, -621, 1527),
                    (-447, -329, 318),
                    (-430, -3130, 366),
                    (-413, -627, 1469),
                    (-345, -311, 381),
                    (-36, -1284, 1171),
                    (-27, -1108, -65),
                    (7, -33, -71),
                    (12, -2351, -103),
                    (26, -1119, 1091),
                    (346, -2985, 342),
                    (366, -3059, 397),
                    (377, -2827, 367),
                    (390, -675, -793),
                    (396, -1931, -563),
                    (404, -588, -901),
                    (408, -1815, 803),
                    (423, -701, 434),
                    (432, -2009, 850),
                    (443, 580, 662),
                    (455, 729, 728),
                    (456, -540, 1869),
                    (459, -707, 401),
                    (465, -695, 1988),
                    (474, 580, 667),
                    (496, -1584, 1900),
                    (497, -1838, -617),
                    (527, -524, 1933),
                    (528, -643, 409),
                    (534, -1912, 768),
                    (544, -627, -890),
                    (553, 345, -567),
                    (564, 392, -477),
                    (568, -2007, -577),
                    (605, -1665, 1952),
                    (612, -1593, 1893),
                    (630, 319, -379),
                    (686, -3108, -505),
                    (776, -3184, -501),
                    (846, -3110, -434),
                    (1135, -1161, 1235),
                    (1243, -1093, 1063),
                    (1660, -552, 429),
                    (1693, -557, 386),
                    (1735, -437, 1738),
                    (1749, -1800, 1813),
                    (1772, -405, 1572),
                    (1776, -675, 371),
                    (1779, -442, 1789),
                    (1780, -1548, 337),
                    (1786, -1538, 337),
                    (1847, -1591, 415),
                    (1889, -1729, 1762),
                    (1994, -1805, 1792),
                ]
                .iter()
                .cloned()
            )
        );
        assert_eq!(
            map.scanners,
            HashSet::from_iter(
                [
                    (0, 0, 0),
                    (68, -1246, -43),
                    (1105, -1205, 1229),
                    (-92, -2380, -20),
                    (-20, -1133, 1061)
                ]
                .iter()
                .cloned()
            )
        );
        assert_eq!(map.max_scanner_manhatten_dist(), 3621);
    }
}
