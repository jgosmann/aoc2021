use std::collections::HashMap;

struct DeterministicDie {
    sides: usize,
    num_rolls: usize,
}

impl DeterministicDie {
    fn new(sides: usize) -> Self {
        Self {
            sides,
            num_rolls: 0,
        }
    }
}

impl Iterator for &mut DeterministicDie {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let roll = (self.num_rolls % self.sides) + 1;
        self.num_rolls += 1;
        Some(roll)
    }
}

fn dirac_dice(starting_positions: [usize; 2]) -> usize {
    let mut scores: [usize; 2] = [0, 0];
    let mut positions: [usize; 2] = [starting_positions[0] - 1, starting_positions[1] - 1];
    let mut die = DeterministicDie::new(100);
    let mut active_player = 0usize;
    while scores.iter().all(|&x| x < 1000) {
        let steps: usize = die.take(3).sum();
        positions[active_player] = (positions[active_player] + steps) % 10;
        scores[active_player] += positions[active_player] + 1;
        active_player = 1 - active_player
    }
    die.num_rolls * scores.iter().find(|&&x| x < 1000).unwrap()
}

struct QuantumDiracDice {
    table: HashMap<([usize; 2], [usize; 2], usize), [usize; 2]>,
}

impl QuantumDiracDice {
    fn play(starting_positions: [usize; 2]) -> [usize; 2] {
        let mut game = Self {
            table: HashMap::new(),
        };
        game._play(
            &[starting_positions[0] - 1, starting_positions[1] - 1],
            &[0, 0],
            0,
        )
    }

    fn _play(
        &mut self,
        starting_positions: &[usize; 2],
        starting_scores: &[usize; 2],
        active_player: usize,
    ) -> [usize; 2] {
        let key = (*starting_positions, *starting_scores, active_player);
        if let Some(&result) = self.table.get(&key) {
            return result;
        }

        if starting_scores[0] >= 21 {
            self.table.insert(key, [1, 0]);
            return [1, 0];
        } else if starting_scores[1] >= 21 {
            self.table.insert(key, [0, 1]);
            return [0, 1];
        }

        let mut win_counts = [0, 0];
        for i in 1..=3 {
            for j in 1..=3 {
                for k in 1..=3 {
                    let steps = i + j + k;
                    let mut new_positions = starting_positions.clone();
                    new_positions[active_player] = (new_positions[active_player] + steps) % 10;
                    let mut new_scores = starting_scores.clone();
                    new_scores[active_player] += new_positions[active_player] + 1;
                    let new_active_player = 1 - active_player;
                    let [a, b] = self._play(&new_positions, &new_scores, new_active_player);
                    win_counts[0] += a;
                    win_counts[1] += b;
                }
            }
        }

        self.table.insert(key, win_counts);
        win_counts
    }
}

fn main() {
    println!("Part 1: {}", dirac_dice([10, 1]));
    let quantum_result = QuantumDiracDice::play([10, 1]);
    println!("Part 2: {}", quantum_result.iter().max().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        assert_eq!(dirac_dice([4, 8]), 739785);
    }

    #[test]
    fn test_part2() {
        assert_eq!(
            QuantumDiracDice::play([4, 8]),
            [444356092776315, 341960390180808]
        );
    }
}
