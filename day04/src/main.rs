use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::num::ParseIntError;

#[derive(Debug, PartialEq, Eq)]
pub struct BingoBoard {
    marked_in_col: [u8; 5],
    marked_in_row: [u8; 5],
    number_positions: HashMap<u8, (usize, usize)>,
}

impl BingoBoard {
    fn new(board: &[[u8; 5]; 5]) -> Self {
        Self {
            marked_in_col: [0; 5],
            marked_in_row: [0; 5],
            number_positions: board
                .iter()
                .enumerate()
                .flat_map(|(i, row)| {
                    row.iter()
                        .enumerate()
                        .map(move |(j, number)| (*number, (i, j)))
                })
                .collect(),
        }
    }

    fn has_won(&self) -> bool {
        self.marked_in_row.iter().any(|&x| x >= 5) || self.marked_in_col.iter().any(|&x| x >= 5)
    }

    fn mark(&mut self, number: u8) {
        if let Some((_, (i, j))) = self.number_positions.remove_entry(&number) {
            self.marked_in_col[j] += 1;
            self.marked_in_row[i] += 1;
        }
    }

    fn score(&self) -> u64 {
        self.number_positions.keys().map(|&x| x as u64).sum()
    }
}

fn parse_bingo_board<R: BufRead>(reader: &mut R) -> Result<BingoBoard, Box<dyn Error>> {
    let mut board = [[0; 5]; 5];
    for i in 0..5 {
        let mut buf = String::with_capacity(3 * 5);
        while buf.trim().is_empty() {
            buf.clear();
            let num_read = reader.read_line(&mut buf)?;
            if num_read <= 0 {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Premature end of bingo board.",
                )));
            }
        }
        for (j, number) in buf
            .split(' ')
            .filter(|&s| !s.trim().is_empty())
            .enumerate()
            .take(5)
        {
            board[i][j] = u8::from_str_radix(number.trim(), 10)?;
        }
    }
    return Ok(BingoBoard::new(&board));
}

#[derive(Debug)]
struct BingoSubsystem {
    random_numbers: Vec<u8>,
    boards: Vec<BingoBoard>,
}

#[derive(Debug, PartialEq, Eq)]
struct BingoResult {
    winning_score: u64,
    losing_score: u64,
}

#[derive(Debug)]
struct DepletedRandomNumbersError {}

impl std::fmt::Display for DepletedRandomNumbersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Depleted all random numbers before all board numbers were marked.")
    }
}

impl Error for DepletedRandomNumbersError {}

impl BingoSubsystem {
    fn read<R: BufRead>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
        let mut number_line = String::default();
        reader.read_line(&mut number_line)?;

        let random_numbers: Result<Vec<u8>, ParseIntError> = number_line
            .split(',')
            .map(|x| u8::from_str_radix(x.trim(), 10))
            .collect();
        let random_numbers = random_numbers?;

        let mut boards: Vec<BingoBoard> = Vec::new();
        while let Ok(board) = parse_bingo_board(reader) {
            boards.push(board);
        }

        Ok(Self {
            random_numbers,
            boards,
        })
    }

    fn play_bingo_against_squid(&mut self) -> Result<BingoResult, Box<dyn Error>> {
        let mut winning_score: Option<u64> = None;
        let mut losing_board_index: Option<usize> = None;
        for &number in &self.random_numbers {
            for board in &mut self.boards {
                board.mark(number);
            }

            if winning_score.is_none() {
                if let Some(winner) = self.boards.iter().find(|b| b.has_won()) {
                    winning_score = Some(winner.score() * number as u64);
                }
            }

            let non_won: Vec<(usize, &BingoBoard)> = self
                .boards
                .iter()
                .enumerate()
                .filter(|(_, board)| !board.has_won())
                .collect();
            if non_won.len() == 1 {
                losing_board_index = Some(non_won[0].0);
            }

            if self.boards.iter().all(|b| b.has_won()) {
                return Ok(BingoResult {
                    winning_score: winning_score.unwrap(),
                    losing_score: self.boards[losing_board_index.unwrap()].score() * number as u64,
                });
            }
        }

        return Err(Box::new(DepletedRandomNumbersError {}));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut bingo_subsystem = BingoSubsystem::read(&mut stdin.lock())?;
    let result = bingo_subsystem.play_bingo_against_squid()?;
    println!("Winning score: {}", result.winning_score);
    println!("Winning score: {}", result.losing_score);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_board() -> BingoBoard {
        BingoBoard::new(&[
            [11, 12, 13, 14, 15],
            [21, 22, 23, 24, 25],
            [31, 32, 33, 34, 35],
            [41, 42, 43, 44, 45],
            [51, 52, 53, 54, 55],
        ])
    }

    #[test]
    fn newly_created_bingo_board_has_not_won() {
        assert!(!new_test_board().has_won());
    }

    #[test]
    fn bingo_board_with_one_number_in_each_row_and_column_has_not_won() {
        let mut board = new_test_board();
        for i in 1..=4 {
            for j in 1..=4 {
                board.mark(i * 10 + j);
            }
        }
        assert!(!board.has_won());
    }

    #[test]
    fn bingo_board_with_one_row_fully_marked_has_won() {
        let mut board = new_test_board();
        for i in 1..=5 {
            board.mark(10 + i);
        }
        assert!(board.has_won());
    }

    #[test]
    fn bingo_board_with_one_column_fully_marked_has_won() {
        let mut board = new_test_board();
        for i in 1..=5 {
            board.mark(10 * i + 1);
        }
        assert!(board.has_won());
    }

    #[test]
    fn parse_bingo_board_works() {
        let mut input = "
11  12  13  14  15
21  22  23  24  25
31  32  33  34  35
41  42  43  44  45
51  52  53  54  55"
            .as_bytes();
        let board = parse_bingo_board(&mut input).unwrap();
        assert_eq!(board, new_test_board());
    }

    #[test]
    fn bingo_subsystem() -> Result<(), Box<dyn Error>> {
        let mut input = include_str!("../test.input").as_bytes();
        let mut bingo_subsystem = BingoSubsystem::read(&mut input)?;
        let result = bingo_subsystem.play_bingo_against_squid()?;
        assert_eq!(result.winning_score, 4512);
        assert_eq!(result.losing_score, 1924);
        Ok(())
    }
}
