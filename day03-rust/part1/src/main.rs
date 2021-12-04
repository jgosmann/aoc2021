use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut bit_counts: Vec<i64> = vec![];
    for line in stdin.lock().lines().map(Result::unwrap){
        for (i, c) in line.chars().enumerate() {
            if i >= bit_counts.len() {
                bit_counts.push(0);
            }
            bit_counts[i] += match c {
                '0' => -1,
                '1' => 1,
                _ => 0,
            }
        }
    }


    let mut gamma_rate: u64 = 0;
    for &count in &bit_counts {
        gamma_rate = gamma_rate << 1;
        gamma_rate |= (count > 0) as u64;
    }

    let epsilon_rate = !(u64::MAX << bit_counts.len()) ^ gamma_rate;

    println!("gamma rate: {}", gamma_rate);
    println!("epsilon rate: {}", epsilon_rate);
    println!("power: {}", gamma_rate * epsilon_rate);
}
