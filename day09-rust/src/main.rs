use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::error::Error;
use std::io::{self, BufRead};
use std::iter::{FromIterator, Iterator};

type Pos = (usize, usize);

pub struct NeighborIterator {
    pos: Pos,
    upper_bounds: Pos,
    state: u8,
}

impl NeighborIterator {
    pub fn new(pos: Pos, upper_bounds: Pos) -> Self {
        Self {
            pos,
            upper_bounds,
            state: 0,
        }
    }
}

impl Iterator for NeighborIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        while self.state < 4 {
            let next_item = match self.state {
                0 if self.pos.1 > 0 => Some((self.pos.0, self.pos.1 - 1)),
                1 if self.pos.0 > 0 => Some((self.pos.0 - 1, self.pos.1)),
                2 if self.pos.0 < self.upper_bounds.0 => Some((self.pos.0 + 1, self.pos.1)),
                3 if self.pos.1 < self.upper_bounds.1 => Some((self.pos.0, self.pos.1 + 1)),
                _ => None,
            };
            self.state += 1;
            if next_item.is_some() {
                return next_item;
            }
        }
        return None;
    }
}

fn find_low_points<Map: AsRef<[Row]>, Row: AsRef<[u8]>>(map: Map) -> Vec<u8> {
    let map = map.as_ref();
    map.iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let row = row.as_ref();
            row.iter().enumerate().filter_map(move |(j, &height)| {
                let mut iter = NeighborIterator::new((i, j), (map.len() - 1, row.len() - 1));
                if iter.all(|pos| height < map[pos.0].as_ref()[pos.1]) {
                    return Some(height);
                }
                return None;
            })
        })
        .collect()
}

fn determine_basin_area<Map: AsRef<[Row]>, Row: AsRef<[u8]>>(map: Map) -> u64 {
    let map = map.as_ref();
    let mut visited = HashSet::<Pos>::new();
    let mut basins: BinaryHeap<Reverse<u64>> = BinaryHeap::with_capacity(4);
    for (i, row) in map.iter().enumerate() {
        let row = row.as_ref();
        for (j, &height) in row.iter().enumerate() {
            if height >= 9 || visited.contains(&(i, j)) {
                continue;
            }

            visited.insert((i, j));
            let mut area = 1;

            let upper_bounds = (map.len() - 1, row.len() - 1);
            let mut to_visit = VecDeque::from_iter(NeighborIterator::new((i, j), upper_bounds));
            while let Some(visited_pos) = to_visit.pop_front() {
                if visited.contains(&visited_pos) || map[visited_pos.0].as_ref()[visited_pos.1] >= 9
                {
                    continue;
                }

                visited.insert(visited_pos);
                area += 1;

                for neighbor in NeighborIterator::new(visited_pos, upper_bounds) {
                    to_visit.push_back(neighbor);
                }
            }

            basins.push(Reverse(area));
            while basins.len() > 3 {
                basins.pop();
            }
        }
    }

    basins.into_iter().take(3).map(|x| x.0).product()
}

fn calculate_risk(low_points: &[u8]) -> u64 {
    low_points.iter().map(|&x| x as u64).sum::<u64>() + low_points.len() as u64
}

fn parse_input<R: BufRead>(reader: &mut R) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    reader
        .lines()
        .map(|line| {
            Ok(line?
                .trim()
                .chars()
                .filter_map(|x| x.to_digit(10))
                .map(|x| x as u8)
                .collect::<Vec<u8>>())
        })
        .collect()
}

fn main() {
    let stdin = io::stdin();
    let map = parse_input(&mut stdin.lock()).unwrap();
    println!("Total risk: {}", calculate_risk(&find_low_points(&map)));
    println!(
        "Product of three largest basins: {}",
        determine_basin_area(&map)
    );
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, iter::FromIterator};

    use super::*;

    static TEST_INPUT: &str = include_str!("../test.input");

    static TEST_MAP: [[u8; 10]; 5] = [
        [2, 1, 9, 9, 9, 4, 3, 2, 1, 0],
        [3, 9, 8, 7, 8, 9, 4, 9, 2, 1],
        [9, 8, 5, 6, 7, 8, 9, 8, 9, 2],
        [8, 7, 6, 7, 8, 9, 6, 7, 8, 9],
        [9, 8, 9, 9, 9, 6, 5, 6, 7, 8],
    ];

    #[test]
    fn test_neighbor_iterator_middle() {
        let iter = NeighborIterator::new((1, 1), (2, 2));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(vec![(1, 0), (1, 2), (0, 1), (2, 1)].into_iter())
        );
    }

    #[test]
    fn test_neighbor_iterator_upper_left_corner() {
        let iter = NeighborIterator::new((0, 0), (2, 2));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(vec![(0, 1), (1, 0)].into_iter())
        );
    }

    #[test]
    fn test_neighbor_iterator_lower_right_corner() {
        let iter = NeighborIterator::new((2, 2), (2, 2));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(vec![(2, 1), (1, 2)].into_iter())
        );
    }

    #[test]
    fn test_find_low_points() {
        assert_eq!(find_low_points(TEST_MAP).sort(), vec![1, 0, 5, 5].sort());
    }

    #[test]
    fn test_calculate_risk() {
        assert_eq!(calculate_risk(&[1, 0, 5, 5]), 15);
    }

    #[test]
    fn test_determine_basin_area() {
        assert_eq!(determine_basin_area(TEST_MAP), 1134);
    }

    #[test]
    fn test_parse_input() {
        let mut buf: &[u8] = TEST_INPUT.as_bytes();
        assert_eq!(parse_input(&mut buf).unwrap(), TEST_MAP);
    }
}
