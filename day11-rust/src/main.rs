use std::error::Error;
use std::io::{self, BufRead};

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
        while self.state < 9 {
            let next_item = match self.state {
                0 if self.pos.0 > 0 && self.pos.1 > 0 => Some((self.pos.0 - 1, self.pos.1 - 1)),
                1 if self.pos.1 > 0 => Some((self.pos.0, self.pos.1 - 1)),
                2 if self.pos.0 < self.upper_bounds.0 - 1 && self.pos.1 > 0 => {
                    Some((self.pos.0 + 1, self.pos.1 - 1))
                }
                3 if self.pos.0 < self.upper_bounds.0 - 1 => Some((self.pos.0 + 1, self.pos.1)),
                4 if self.pos.0 < self.upper_bounds.0 - 1
                    && self.pos.1 < self.upper_bounds.1 - 1 =>
                {
                    Some((self.pos.0 + 1, self.pos.1 + 1))
                }
                5 if self.pos.1 < self.upper_bounds.1 - 1 => Some((self.pos.0, self.pos.1 + 1)),
                6 if self.pos.0 > 0 && self.pos.1 < self.upper_bounds.1 - 1 => {
                    Some((self.pos.0 - 1, self.pos.1 + 1))
                }
                7 if self.pos.0 > 0 => Some((self.pos.0 - 1, self.pos.1)),
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

pub struct PosSet {
    containment_bitmap: u128,
}

impl PosSet {
    fn new() -> Self {
        Self {
            containment_bitmap: 0,
        }
    }

    fn insert(&mut self, pos: &Pos) {
        if pos.0 > 9 || pos.1 > 9 {
            panic!("Out of bounds.");
        }
        self.containment_bitmap |= 1 << (pos.0 * 10 + pos.1);
    }

    fn contains(&self, pos: &Pos) -> bool {
        self.containment_bitmap & (1 << (pos.0 * 10 + pos.1)) != 0
    }

    fn len(&self) -> usize {
        self.containment_bitmap.count_ones() as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OctopusGrid {
    grid: [[u8; 10]; 10],
    step_idx: usize,
    num_flashes: usize,
    first_synchronized_flash: Option<usize>,
}

impl OctopusGrid {
    pub fn new(grid: [[u8; 10]; 10]) -> Self {
        Self {
            grid,
            step_idx: 0,
            num_flashes: 0,
            first_synchronized_flash: None,
        }
    }

    pub fn num_flashes(&self) -> usize {
        self.num_flashes
    }

    pub fn first_synchronized_flash(&self) -> Option<usize> {
        self.first_synchronized_flash
    }

    pub fn step_n(&mut self, n: usize) {
        (0..n).for_each(|_| self.step());
    }

    pub fn step_until_first_synchronized_flash(&mut self) {
        while !self.first_synchronized_flash.is_some() {
            self.step()
        }
    }

    pub fn step(&mut self) {
        self.step_idx += 1;
        self.increase_overall_energy();
        let flashes = self.flash();
        self.cool_down();
        self.num_flashes += flashes.len();
        if self.first_synchronized_flash.is_none() && flashes.len() == 100 {
            self.first_synchronized_flash = Some(self.step_idx);
        }
    }

    fn increase_overall_energy(&mut self) {
        for i in 0..self.grid.len() {
            for j in 0..self.grid[i].len() {
                self.grid[i][j] += 1;
            }
        }
    }

    fn flash(&mut self) -> PosSet {
        let mut flashes = PosSet::new();
        loop {
            let num_flashes_before_iteration = flashes.len();
            for i in 0..self.grid.len() {
                for j in 0..self.grid[i].len() {
                    if self.grid[i][j] > 9 && !flashes.contains(&(i, j)) {
                        flashes.insert(&(i, j));
                        for neighbor in
                            NeighborIterator::new((i, j), (self.grid.len(), self.grid[i].len()))
                        {
                            self.grid[neighbor.0][neighbor.1] += 1;
                        }
                    }
                }
            }
            if num_flashes_before_iteration == flashes.len() {
                break;
            }
        }
        flashes
    }

    fn cool_down(&mut self) {
        for i in 0..self.grid.len() {
            for j in 0..self.grid[i].len() {
                if self.grid[i][j] > 9 {
                    self.grid[i][j] = 0;
                }
            }
        }
    }
}

fn read_octopus_grid<R: BufRead>(reader: &mut R) -> Result<OctopusGrid, Box<dyn Error>> {
    let mut grid: [[u8; 10]; 10] = [[0; 10]; 10];
    for i in 0..10 {
        let mut buf = String::with_capacity(12);
        reader.read_line(&mut buf)?;
        for (j, digit) in buf
            .chars()
            .take(10)
            .filter_map(|c| c.to_digit(10))
            .enumerate()
        {
            grid[i][j] = digit as u8;
        }
    }
    Ok(OctopusGrid::new(grid))
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut grid = read_octopus_grid(&mut stdin.lock())?;
    grid.step_n(100);
    println!("Flashes in first 100 steps: {}", grid.num_flashes());
    grid.step_until_first_synchronized_flash();
    println!(
        "First synchronized flash: {:?}",
        grid.first_synchronized_flash()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, iter::FromIterator};

    static TEST_INPUT: &[u8] = include_bytes!("../test.input");

    fn test_grid() -> OctopusGrid {
        OctopusGrid::new([
            [5, 4, 8, 3, 1, 4, 3, 2, 2, 3],
            [2, 7, 4, 5, 8, 5, 4, 7, 1, 1],
            [5, 2, 6, 4, 5, 5, 6, 1, 7, 3],
            [6, 1, 4, 1, 3, 3, 6, 1, 4, 6],
            [6, 3, 5, 7, 3, 8, 5, 4, 7, 8],
            [4, 1, 6, 7, 5, 2, 4, 6, 4, 5],
            [2, 1, 7, 6, 8, 4, 1, 7, 2, 1],
            [6, 8, 8, 2, 8, 8, 1, 1, 3, 4],
            [4, 8, 4, 6, 8, 4, 8, 5, 5, 4],
            [5, 2, 8, 3, 7, 5, 1, 5, 2, 6],
        ])
    }

    #[test]
    fn test_neighbor_iterator_middle() {
        let iter = NeighborIterator::new((1, 1), (3, 3));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(
                vec![
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                    (1, 2),
                    (0, 2),
                    (0, 1)
                ]
                .into_iter()
            )
        );
    }

    #[test]
    fn test_neighbor_iterator_upper_left_corner() {
        let iter = NeighborIterator::new((0, 0), (3, 3));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(vec![(1, 0), (1, 1), (0, 1)].into_iter())
        );
    }

    #[test]
    fn test_neighbor_iterator_lower_right_corner() {
        let iter = NeighborIterator::new((2, 2), (3, 3));
        let neighbors = HashSet::<Pos>::from_iter(iter);
        assert_eq!(
            neighbors,
            HashSet::from_iter(vec![(1, 1), (1, 2), (2, 1)].into_iter())
        );
    }

    #[test]
    fn test_read_octopus_grid() {
        let mut buf = TEST_INPUT;
        assert_eq!(read_octopus_grid(&mut buf).unwrap(), test_grid());
    }

    #[test]
    fn test_octopus_grid() {
        let mut grid = test_grid();
        grid.step_n(100);
        assert_eq!(grid.num_flashes(), 1656);
        grid.step_until_first_synchronized_flash();
        assert_eq!(grid.first_synchronized_flash(), Some(195));
    }
}
