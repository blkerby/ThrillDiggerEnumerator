use hashbrown::HashMap;
use itertools::Itertools;
use std::convert::TryInto;

const WIDTH: usize = 5;
const HEIGHT: usize = 4;
const NUM_BOMBS: usize = 4;

#[derive(Copy, Clone, Debug, Hash)]
enum Cell {
    Empty,
    Green,
    Blue,
    Red,
}

type Arrangement = [(usize, usize); NUM_BOMBS];   // Locations of the 4 bombs

type State = [[Cell; HEIGHT]; WIDTH];

struct StateInfo {
    cnt_completions: u64,
    value: f64,
}

type StateMap = HashMap<State, StateInfo>;

fn arrangement_to_state(arrangement: &Arrangement) -> State {
    let mut state: State = [[Cell::Empty; HEIGHT]; WIDTH];
    let mut counts = [[0; HEIGHT]; WIDTH];

    for (x, y) in arrangement {
        for y1 in &[*y as i32 - 1, *y as i32, *y as i32 + 1] {
            for x1 in &[*x as i32 - 1, *x as i32, *x as i32 + 1] {
                if *x1 >= 0 && *x1 < WIDTH as i32 && *y1 >= 0 && *y1 < HEIGHT as i32 {
                    counts[*x1 as usize][*y1 as usize] += 1;
                }
            }
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            state[x][y] = match counts[x][y] {
                0 => Cell::Green,
                1 | 2 => Cell::Blue,
                3 | 4 => Cell::Red,
                _ => panic!("Unexpected count")
            }
        }
    }

    for (x, y) in arrangement {
        state[*x][*y] = Cell::Empty;
    }

    state
}

fn build_arrangement(arrangement: &Arrangement, state_map: &mut StateMap) {
    let state = arrangement_to_state(arrangement);

    let mut arrangement_mask: u32 = 0;
    for (x, y) in arrangement {
        let i = y * WIDTH + x;
        arrangement_mask |= 1 << i;
    }

    for mask in 0..(1 << (WIDTH * HEIGHT - NUM_BOMBS)) {
        let mut masked_state = state.clone();
        let mut i = 0;
        let mut j = 0;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if arrangement_mask & (1 << i) == 0 {
                    if mask & (1 << j) == 0 {
                        masked_state[x][y] = Cell::Empty;
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }
}

fn build_states(state_map: &mut StateMap) {
    let mut positions: Vec<(usize, usize)> = Vec::new();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            positions.push((x, y));
        }
    }

    for combination in positions.into_iter().combinations(NUM_BOMBS) {
        let arrangement: Arrangement = combination.try_into().unwrap();
        build_arrangement(&arrangement, state_map);
    }
}

fn main() {
    // let arrangement = [(0, 0), (1, 1), (2, 3), (4, 0)];
    // println!("{:?}", arrangement);
    // println!("{:?}", arrangement_to_state(&arrangement));
    let mut state_map = StateMap::new();
    build_states(&mut state_map);
}
