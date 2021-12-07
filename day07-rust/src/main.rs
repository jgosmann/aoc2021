use std::{collections::HashMap, io::{BufRead, self}, error::Error, num::ParseIntError};

fn least_fuel_constant(horizontal_positions: &[u64]) -> u64 {
    let mut num_left = 0;
    let mut num_right_buckets = HashMap::<u64, u64>::with_capacity(horizontal_positions.len());
    
    for &hpos in horizontal_positions {
        num_right_buckets.entry(hpos).and_modify(|x| *x += 1).or_insert(1);
    }
    
    let mut num_current = num_right_buckets.remove(&0).unwrap_or(0);
    let mut num_right: u64 = num_right_buckets.values().sum();

    let mut fuel = num_right_buckets.iter().map(|(pos, count)| pos * count).sum();
    let mut min_fuel = fuel;

    for hpos in 1..=(*num_right_buckets.keys().max().unwrap_or(&0)) {
        num_left += num_current;
        fuel += num_left;
        
        fuel -= num_right;
        num_current = num_right_buckets.remove(&hpos).unwrap_or(0);
        num_right -= num_current;
        
        if fuel < min_fuel {
            min_fuel = fuel
        }
    }
    
    min_fuel
}

fn least_fuel_linear(horizontal_positions: &[u64]) -> u64 {
    let mut buckets = HashMap::<u64, u64>::with_capacity(horizontal_positions.len());
    
    for &hpos in horizontal_positions {
        buckets.entry(hpos).and_modify(|x| *x += 1).or_insert(1);
    }
    
    let mut min_fuel = buckets.iter().map(|(pos, count)| (pos * (pos + 1)) / 2 * count).sum();

    for hpos in 1..=(*buckets.keys().max().unwrap_or(&0)) {
        let fuel = buckets.iter().map(|(&pos, &count)| {
            let x = (pos as i64 - hpos as i64).abs() as u64;
            (x * (x + 1)) / 2 * count  // Gauß ftw
        }).sum();

        
        if fuel < min_fuel {
            min_fuel = fuel
        }
    }
    
    min_fuel
}

fn parse_input<R: BufRead>(reader: &mut R) -> Result<Vec<u64>, Box<dyn Error>> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    Ok(buf.split(',').map(|x| u64::from_str_radix(x.trim(), 10)).collect::<Result<Vec<u64>, ParseIntError>>()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = parse_input(&mut stdin.lock())?;
    println!("Part 1 (constant): {}", least_fuel_constant(&input));
    println!("Part 2 (linear): {}", least_fuel_linear(&input));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_least_fuel() {
        let test_input = vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14];
        assert_eq!(least_fuel_constant(&test_input), 37);
    }
    
    #[test]
    fn test_least_fuel2() {
        let test_input = vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14];
        assert_eq!(least_fuel_linear(&test_input), 168);
    }
    
    #[test]
    fn test_parse_input() {
        let mut input: &[u8] = "16,1,2,0,4,2,7,1,2,14\n".as_bytes();
        assert_eq!(parse_input(&mut input).unwrap(), vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14]);
    }
}
