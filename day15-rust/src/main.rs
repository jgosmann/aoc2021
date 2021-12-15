use std::{
    borrow::Borrow,
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    error::Error,
    io::{self, BufRead},
};

type Pos = (usize, usize);

fn read_input<R: BufRead>(reader: &mut R) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    reader
        .lines()
        .map(|line| {
            Ok(line?
                .chars()
                .filter_map(|c| c.to_digit(10).map(|d| d as u8))
                .collect::<Vec<u8>>())
        })
        .collect()
}

pub struct NeighborIterator<P: Borrow<Pos>> {
    pos: P,
    upper_bounds: P,
    state: u8,
}

impl<P: Borrow<Pos>> NeighborIterator<P> {
    pub fn new(pos: P, upper_bounds: P) -> Self {
        Self {
            pos,
            upper_bounds,
            state: 0,
        }
    }
}

impl<P: Borrow<Pos>> Iterator for NeighborIterator<P> {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        while self.state < 4 {
            let pos = self.pos.borrow();
            let next_item = match self.state {
                0 if pos.1 > 0 => Some((pos.0, pos.1 - 1)),
                1 if pos.0 > 0 => Some((pos.0 - 1, pos.1)),
                2 if pos.0 < self.upper_bounds.borrow().0 => Some((pos.0 + 1, pos.1)),
                3 if pos.1 < self.upper_bounds.borrow().1 => Some((pos.0, pos.1 + 1)),
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

fn dijkstra_on_grid(grid: &Vec<Vec<u8>>) -> Vec<Pos> {
    let upper_bounds = (grid.len() - 1, grid[0].len() - 1);

    let mut prev_node = HashMap::new();
    let mut distances: Vec<Vec<u64>> = grid.iter().map(|r| vec![u64::MAX; r.len()]).collect();
    distances[0][0] = 0;

    let mut touched = HashSet::new();
    let mut heap = BinaryHeap::new();
    heap.push(Reverse((0u64, (0usize, 0usize))));

    while let Some(Reverse((dist, pos))) = heap.pop() {
        for neighbor in NeighborIterator::new(pos, upper_bounds) {
            let dist_to_neighbor = dist + grid[neighbor.0][neighbor.1] as u64;
            if dist_to_neighbor < distances[neighbor.0][neighbor.1] {
                distances[neighbor.0][neighbor.1] = dist_to_neighbor;
                prev_node.insert(neighbor, pos);
                if !touched.contains(&neighbor) {
                    heap.push(Reverse((dist_to_neighbor, neighbor)));
                    touched.insert(neighbor);
                }
            }
        }
    }

    let mut path = vec![upper_bounds];
    while path.last() != Some(&(0, 0)) {
        if let Some(&prev) = prev_node.get(path.last().unwrap()) {
            path.push(prev);
        } else {
            break;
        }
    }
    path
}

fn risk(grid: &Vec<Vec<u8>>, path: &[Pos]) -> u64 {
    path.iter().map(|&(i, j)| grid[i][j] as u64).sum::<u64>() - grid[0][0] as u64
}

fn replicate(grid: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let replicated_cols: Vec<Vec<u8>> = grid
        .iter()
        .map(|row| {
            (0..5)
                .flat_map(|i| row.iter().map(move |&x| (x + i - 1) % 9 + 1))
                .collect()
        })
        .collect();
    (0..5)
        .flat_map(|i| {
            replicated_cols
                .iter()
                .map(move |row| row.iter().map(|&x| (x + i - 1) % 9 + 1).collect())
        })
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let grid = read_input(&mut stdin.lock())?;
    let path = dijkstra_on_grid(&grid);
    println!("Risk initial grid: {}", risk(&grid, &path));
    let full_grid = replicate(&grid);
    let path = dijkstra_on_grid(&full_grid);
    println!("Risk full grid: {}", risk(&full_grid, &path));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

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
    fn test_part1() {
        let mut buf: &[u8] = include_bytes!("../test.input");
        let grid = read_input(&mut buf).unwrap();
        let path = dijkstra_on_grid(&grid);
        assert_eq!(risk(&grid, &path), 40);
    }

    #[test]
    fn test_part2() {
        let mut buf: &[u8] = include_bytes!("../test.input");
        let grid = read_input(&mut buf).unwrap();
        let full_grid = replicate(&grid);
        let path = dijkstra_on_grid(&full_grid);
        assert_eq!(risk(&full_grid, &path), 315);
    }
}
