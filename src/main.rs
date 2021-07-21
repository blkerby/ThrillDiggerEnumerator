use hashbrown::HashMap;
use itertools::Itertools;
use std::convert::TryInto;
use log::info;

const WIDTH: usize = 5;
const HEIGHT: usize = 4;
const NUM_BOMBS: usize = 4;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
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

        let mut state_info = state_map.entry(masked_state).or_insert(StateInfo {
            cnt_completions: 0,
            value: -1.0,
        });
        state_info.cnt_completions += 1;
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

fn compute_terminal_state_value(state: &State) -> f64 {
    let mut value: f64 = 0.0;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            match state[x][y] {
                Cell::Green => { value += 1.0 }
                Cell::Blue => { value += 5.0 }
                Cell::Red => { value += 20.0 }
                Cell::Empty => {}
            }
        }
    }
    value
}

fn compute_value(state: &mut State, state_map: &mut StateMap) -> f64 {
    let mut state_info = state_map.get_mut(state).unwrap();
    if state_info.value != -1.0 {
        return state_info.value;
    }

    let terminal_value = compute_terminal_state_value(state);
    let total_cnt = state_info.cnt_completions;
    let mut max_action_value = 0.0;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if state[x][y] != Cell::Empty {
                continue;
            }

            state[x][y] = Cell::Green;
            let green_cnt = match state_map.get(state) {
                Some(x) => x.cnt_completions,
                None => 0,
            };
            let green_value = if green_cnt > 0 { compute_value(state, state_map) } else { 0.0 };
            let green_prob = green_cnt as f64 / total_cnt as f64;

            state[x][y] = Cell::Blue;
            let blue_cnt = match state_map.get(state) {
                Some(x) => x.cnt_completions,
                None => 0,
            };
            let blue_value = if blue_cnt > 0 { compute_value(state, state_map) } else { 0.0 };
            let blue_prob = blue_cnt as f64 / total_cnt as f64;

            state[x][y] = Cell::Red;
            let red_cnt = match state_map.get(state) {
                Some(x) => x.cnt_completions,
                None => 0,
            };
            let red_value = if red_cnt > 0 { compute_value(state, state_map) } else { 0.0 };
            let red_prob = red_cnt as f64 / total_cnt as f64;

            let bomb_cnt = total_cnt - green_cnt - blue_cnt - red_cnt;
            let bomb_prob = bomb_cnt as f64 / total_cnt as f64;

            let action_value = green_prob * green_value + blue_prob * blue_value + red_prob * red_value + bomb_prob * terminal_value;
            if action_value > max_action_value {
                max_action_value = action_value;
            }

            state[x][y] = Cell::Empty;
        }
    }

    state_info = state_map.get_mut(state).unwrap();
    state_info.value = max_action_value;
    max_action_value
}

fn main() {
    env_logger::init();
    let mut state_map = StateMap::new();

    info!("Enumerating states");
    build_states(&mut state_map);
    info!("{} states", state_map.len());

    info!("Computing values");
    let mut state = [[Cell::Empty; HEIGHT]; WIDTH];
    let value = compute_value(&mut state, &mut state_map);
    info!("Value: {}", value);
}
