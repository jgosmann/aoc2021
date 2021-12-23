use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl Amphipod {
    fn target(&self) -> usize {
        use Amphipod::*;
        match self {
            Amber => 0,
            Bronze => 1,
            Copper => 2,
            Desert => 3,
        }
    }

    fn energy(&self) -> usize {
        use Amphipod::*;
        match self {
            Amber => 1,
            Bronze => 10,
            Copper => 100,
            Desert => 1000,
        }
    }
}

impl Display for Amphipod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Amphipod::*;
        f.write_str(match self {
            Amber => "A",
            Bronze => "B",
            Copper => "C",
            Desert => "D",
        })
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Location {
    Free,
    Occupied(Amphipod),
}

impl Location {
    fn is_free(&self) -> bool {
        matches!(self, Location::Free)
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Location::*;
        match self {
            Free => f.write_str("."),
            Occupied(amphipod) => amphipod.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct State {
    hallway: [Location; 11],
    rooms: [Vec<Location>; 4],
}

impl State {
    fn is_target(&self) -> bool {
        self.hallway.iter().all(|location| location.is_free())
            && self.rooms.iter().enumerate().all(|(i, room)| {
                room.iter().all(|location| match location {
                    Location::Free => false,
                    Location::Occupied(amphipod) => amphipod.target() == i,
                })
            })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("#############\n#")?;
        for location in self.hallway {
            location.fmt(f)?;
        }
        f.write_str("#\n")?;

        let room_len = self.rooms.iter().map(|room| room.len()).min().unwrap_or(0);
        for i in 0..room_len {
            if i == 0 {
                f.write_str("##")?;
            } else {
                f.write_str("  ")?;
            }
            f.write_fmt(format_args!(
                "#{}#{}#{}#{}#",
                self.rooms[0][i], self.rooms[1][i], self.rooms[2][i], self.rooms[3][i]
            ))?;
            if i == 0 {
                f.write_str("##")?;
            }
            f.write_str("\n")?;
        }
        f.write_str("  #########\n")
    }
}

struct Solver {
    memo_table: HashMap<State, Option<usize>>,
}

impl Solver {
    fn new() -> Self {
        Self {
            memo_table: HashMap::new(),
        }
    }

    fn solve(&mut self, state: &State) -> Option<usize> {
        if let Some(&result) = self.memo_table.get(state) {
            return result;
        }

        if state.is_target() {
            return Some(0);
        }

        let result = NextStateIterator::new(state)
            .filter_map(|(energy, next_state)| {
                self.solve(&next_state)
                    .map(|sub_energy| energy + sub_energy)
            })
            .min();
        self.memo_table.insert(state.clone(), result);
        result
    }
}

fn solve(start_state: &State) -> Option<usize> {
    Solver::new().solve(start_state)
}

#[derive(Debug)]
struct NextStateIterator<'a> {
    state: &'a State,
    hallway_location: usize,
    room_location: usize,
    room_index: usize,
    move_into_room: bool,
    is_exhausted: bool,
}

impl<'a> NextStateIterator<'a> {
    fn new(state: &'a State) -> Self {
        Self {
            state,
            hallway_location: 0,
            room_location: 0,
            room_index: 0,
            move_into_room: false,
            is_exhausted: false,
        }
    }

    fn is_valid_move(
        &self,
        hallway_location: usize,
        room_location: usize,
        room_index: usize,
        move_into_room: bool,
    ) -> bool {
        let is_hallway_location_valid = hallway_location != 2
            && hallway_location != 4
            && hallway_location != 6
            && hallway_location != 8;
        let are_start_and_end_valid = match (
            move_into_room,
            self.state.hallway[hallway_location],
            self.state.rooms[room_location][room_index],
        ) {
            (true, Location::Occupied(amphipod), Location::Free) => {
                amphipod.target() == room_location
                    && self.state.rooms[room_location]
                        .iter()
                        .skip(room_index + 1)
                        .all(|&l| l == Location::Occupied(amphipod))
            }
            (true, Location::Free, Location::Occupied(_)) => {
                self.state.rooms[room_location].iter().any(|l| match l {
                    Location::Occupied(a) => a.target() != room_location,
                    _ => false,
                })
            }
            _ => false,
        };
        let is_path_within_room_free = self.state.rooms[room_location]
            .iter()
            .take(room_index)
            .all(|l| l.is_free());
        let mut hallway_range = if hallway_location < 2 * room_location + 2 {
            (hallway_location + 1)..(2 * room_location + 2)
        } else {
            (2 * room_location + 2 + 1)..hallway_location
        };
        is_hallway_location_valid
            && are_start_and_end_valid
            && is_path_within_room_free
            && hallway_range.all(|i| self.state.hallway[i].is_free())
    }

    fn num_move_steps(
        &self,
        hallway_location: usize,
        room_location: usize,
        room_index: usize,
    ) -> usize {
        let hallway_range = if hallway_location < 2 * room_location + 2 {
            hallway_location..(2 * room_location + 2)
        } else {
            (2 * room_location + 2)..(hallway_location)
        };
        hallway_range.count() + room_index + 1
    }

    fn increment(&mut self) {
        self.hallway_location += 1;
        if self.hallway_location >= self.state.hallway.len() {
            self.hallway_location = 0;
            self.room_location += 1;
            if self.room_location >= self.state.rooms.len() {
                self.room_location = 0;
                self.room_index += 1;
                if self.room_index >= self.state.rooms[0].len() {
                    self.room_index = 0;
                    if self.move_into_room {
                        self.is_exhausted = true;
                    } else {
                        self.move_into_room = true;
                    }
                }
            }
        }
    }

    fn increment_until_valid(&mut self) {
        while !self.is_exhausted
            && !self.is_valid_move(
                self.hallway_location,
                self.room_location,
                self.room_index,
                self.move_into_room,
            )
        {
            self.increment();
        }
    }
}

impl<'a> Iterator for NextStateIterator<'a> {
    type Item = (usize, State);

    fn next(&mut self) -> Option<Self::Item> {
        self.increment_until_valid();
        if self.is_exhausted {
            return None;
        }

        let mut next_state = self.state.clone();
        next_state.rooms[self.room_location][self.room_index] =
            self.state.hallway[self.hallway_location];
        next_state.hallway[self.hallway_location] =
            self.state.rooms[self.room_location][self.room_index];
        let energy = match (
            self.state.hallway[self.hallway_location],
            self.state.rooms[self.room_location][self.room_index],
        ) {
            (Location::Occupied(amphipod), Location::Free) => amphipod.energy(),
            (Location::Free, Location::Occupied(amphipod)) => amphipod.energy(),
            _ => unreachable!(),
        };

        let retval = Some((
            self.num_move_steps(self.hallway_location, self.room_location, self.room_index)
                * energy,
            next_state,
        ));

        self.increment();

        retval
    }
}

fn main() {
    println!(
        "Part 1: {}",
        solve(&State {
            hallway: [Location::Free; 11],
            rooms: [
                vec![
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Desert)
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Amber)
                ],
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Desert)
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Bronze)
                ],
            ]
        })
        .unwrap()
    );
    println!(
        "Part 2: {}",
        solve(&State {
            hallway: [Location::Free; 11],
            rooms: [
                vec![
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Desert)
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Amber)
                ],
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Desert)
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Bronze)
                ],
            ]
        })
        .unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let test_scenario = State {
            hallway: [Location::Free; 11],
            rooms: [
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Amber),
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Desert),
                ],
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Copper),
                ],
                vec![
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Amber),
                ],
            ],
        };
        assert_eq!(
            test_scenario.to_string(),
            "\
#############
#...........#
###B#C#B#D###
  #A#D#C#A#
  #########
"
        );
        assert_eq!(solve(&test_scenario), Some(12521));
    }

    #[test]
    fn test_part2() {
        let test_scenario = State {
            hallway: [Location::Free; 11],
            rooms: [
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Amber),
                ],
                vec![
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Desert),
                ],
                vec![
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Bronze),
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Copper),
                ],
                vec![
                    Location::Occupied(Amphipod::Desert),
                    Location::Occupied(Amphipod::Amber),
                    Location::Occupied(Amphipod::Copper),
                    Location::Occupied(Amphipod::Amber),
                ],
            ],
        };
        assert_eq!(
            test_scenario.to_string(),
            "\
#############
#...........#
###B#C#B#D###
  #D#C#B#A#
  #D#B#A#C#
  #A#D#C#A#
  #########
"
        );
        assert_eq!(solve(&test_scenario), Some(44169));
    }
}
