use std::collections::VecDeque;
use std::error::Error;
use std::io::{self, BufRead};
use std::num::ParseIntError;

pub struct FishSchool {
    cycle: usize,
    adult_fish: [u64; 7],
    young_fish: VecDeque<u64>
}

impl FishSchool {
    fn new(initial_timers: Vec<u8>) -> Self {
        let mut fish_school = Self {
            cycle: 0,
            adult_fish: [0; 7],
            young_fish: VecDeque::from(vec![0, 0])
        };
        for timer in initial_timers {
            fish_school.adult_fish[timer as usize] += 1;
        }
        return fish_school
    }
    
    fn step(&mut self) {
        let i = self.cycle % 7;
        let num_fish_to_spawn = self.adult_fish[i];
        self.young_fish.push_back(num_fish_to_spawn);
        self.adult_fish[i] += self.young_fish.pop_front().unwrap();
        self.cycle += 1;
    }
    
    fn step_n(&mut self, n: usize) {
        for _ in 0..n {
            self.step()
        }
    }
    
    fn fish_count(&self) -> u64 {
        let num_adult_fish: u64 = self.adult_fish.iter().sum();
        let num_young_fish: u64 = self.young_fish.iter().sum();
        num_adult_fish + num_young_fish
    }
}

fn parse_input<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    Ok(buf.split(',').map(|x| u8::from_str_radix(x.trim(), 10)).collect::<Result<Vec<u8>, ParseIntError>>()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut fish_school = FishSchool::new(parse_input(&mut stdin.lock())?);
    fish_school.step_n(80);
    println!("After 80 days: {}", fish_school.fish_count());
    fish_school.step_n(256 - 80);
    println!("After 256 days: {}", fish_school.fish_count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fish_school() {
        let mut fish_school = FishSchool::new(vec![3, 4, 3, 1, 2]);
        fish_school.step_n(18);
        assert_eq!(fish_school.fish_count(), 26);
        fish_school.step_n(80 - 18);
        assert_eq!(fish_school.fish_count(), 5934);
        fish_school.step_n(256 - 80);
        assert_eq!(fish_school.fish_count(), 26984457539);
    }
    
    #[test]
    fn test_parse_input() {
        let mut input: &[u8] = "3,4,3,1,2\n".as_bytes();
        assert_eq!(parse_input(&mut input).unwrap(), vec![3, 4, 3, 1, 2]);
    }
}

