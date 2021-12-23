use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    ops::RangeBounds,
};

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
        if let Location::Free = self {
            return true;
        }
        return false;
    }

    fn is_occupied(&self) -> bool {
        if let Location::Occupied(_) = self {
            return true;
        }
        return false;
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
    rooms: [[Location; 2]; 4],
}

impl State {
    fn is_target(&self) -> bool {
        self.hallway.iter().all(|h| h.is_free())
            && self.rooms.iter().enumerate().all(|(i, r)| {
                r.iter().all(|l| match l {
                    Location::Free => false,
                    Location::Occupied(amphipod) => amphipod.target() == i,
                })
            })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("#############\n#")?;
        for h in self.hallway {
            h.fmt(f)?;
        }
        f.write_fmt(format_args!(
            "#\n###{}#{}#{}#{}###\n  #{}#{}#{}#{}#\n  #########\n",
            self.rooms[0][0],
            self.rooms[1][0],
            self.rooms[2][0],
            self.rooms[3][0],
            self.rooms[0][1],
            self.rooms[1][1],
            self.rooms[2][1],
            self.rooms[3][1],
        ))
    }
}

fn solve(start_state: &State, seen: &mut HashMap<State, Option<usize>>) -> Option<usize> {
    if let Some(&result) = seen.get(&start_state) {
        return result;
    }

    if start_state.is_target() {
        return Some(0);
    }

    let result = NextStateIterator::new(start_state)
        .filter_map(|(energy, state)| solve(&state, seen).map(|e2| energy + e2))
        .min();
    seen.insert(start_state.clone(), result);
    result
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
        let hallway_valid = hallway_location != 2
            && hallway_location != 4
            && hallway_location != 6
            && hallway_location != 8;
        let start_end_valid = if move_into_room {
            if let Location::Occupied(amphipod) = self.state.hallway[hallway_location] {
                amphipod.target() == room_location
                    && self.state.rooms[room_location][room_index].is_free()
                    && ((room_index == 0
                        && self.state.rooms[room_location][1] == Location::Occupied(amphipod))
                        || (room_index == 1 && self.state.rooms[room_location][0].is_free()))
            } else {
                false
            }
        } else {
            if let Location::Occupied(amphipod) = self.state.rooms[room_location][room_index] {
                self.state.hallway[hallway_location].is_free()
                    && (self.state.rooms[room_location].iter().any(|l| {
                        if let Location::Occupied(a) = l {
                            a.target() != room_location
                        } else {
                            false
                        }
                    }))
            } else {
                false
            }
        };
        let room_path_free = room_index == 0 || self.state.rooms[room_location][0].is_free();
        let mut hallway_range = if hallway_location < 2 * room_location + 2 {
            (hallway_location + 1)..(2 * room_location + 2)
        } else {
            (2 * room_location + 2 + 1)..hallway_location
        };
        hallway_valid
            && start_end_valid
            && room_path_free
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
                if self.room_index >= 2 {
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
        "{}",
        solve(
            &State {
                hallway: [Location::Free; 11],
                rooms: [
                    [
                        Location::Occupied(Amphipod::Amber),
                        Location::Occupied(Amphipod::Desert)
                    ],
                    [
                        Location::Occupied(Amphipod::Copper),
                        Location::Occupied(Amphipod::Amber)
                    ],
                    [
                        Location::Occupied(Amphipod::Bronze),
                        Location::Occupied(Amphipod::Desert)
                    ],
                    [
                        Location::Occupied(Amphipod::Copper),
                        Location::Occupied(Amphipod::Bronze)
                    ],
                ]
            },
            &mut HashMap::new()
        )
        .unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_SCENARIO: State = State {
        hallway: [Location::Free; 11],
        rooms: [
            [
                Location::Occupied(Amphipod::Bronze),
                Location::Occupied(Amphipod::Amber),
            ],
            [
                Location::Occupied(Amphipod::Copper),
                Location::Occupied(Amphipod::Desert),
            ],
            [
                Location::Occupied(Amphipod::Bronze),
                Location::Occupied(Amphipod::Copper),
            ],
            [
                Location::Occupied(Amphipod::Desert),
                Location::Occupied(Amphipod::Amber),
            ],
        ],
    };

    static TEST_SCENARIO2: State = State {
        hallway: [
            Location::Free,
            Location::Free,
            Location::Free,
            Location::Free,
            Location::Free,
            Location::Occupied(Amphipod::Desert),
            Location::Free,
            Location::Free,
            Location::Free,
            Location::Free,
            Location::Free,
        ],
        rooms: [
            [Location::Free, Location::Occupied(Amphipod::Amber)],
            [
                Location::Occupied(Amphipod::Bronze),
                Location::Occupied(Amphipod::Bronze),
            ],
            [
                Location::Occupied(Amphipod::Copper),
                Location::Occupied(Amphipod::Copper),
            ],
            [
                Location::Occupied(Amphipod::Desert),
                Location::Occupied(Amphipod::Amber),
            ],
        ],
    };
    #[test]
    fn test() {
        assert_eq!(solve(&TEST_SCENARIO, &mut HashMap::new()), Some(12521));
    }
}
