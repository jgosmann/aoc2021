use std::io::{self, BufRead, Read};

fn filter_by_criteria(mut lines: Vec<String>, invert: bool) -> String {
    let mut i: usize = 0;
    while lines.len() > 1 {
        let ones_count: usize = lines.iter().map(|line| match line.as_bytes()[i] {
            b'1' => 1,
            _ => 0,
        }).sum();

        if (ones_count >= lines.len() - ones_count) != invert {
            lines = lines.into_iter().filter(|line| line.as_bytes()[i] == b'1').collect();
        } else {
            lines = lines.into_iter().filter(|line| line.as_bytes()[i] == b'0').collect();
        }
        i += 1;
    }

    return lines[0].clone();
}

fn main() {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().map(Result::unwrap).collect();
    let oxygen = u64::from_str_radix(&filter_by_criteria(lines.clone(), false), 2).unwrap();
    let co2 = u64::from_str_radix(&filter_by_criteria(lines, true), 2).unwrap();

    println!("oxygen: {}", oxygen);
    println!("co2: {}", co2);
    println!("life support rating: {}", oxygen * co2);
}
