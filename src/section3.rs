#![allow(unused)]

use std::{
    char,
    collections::BinaryHeap,
    time::{Duration, Instant},
};

use rand::Rng;

type ScoreType = i64;
type Action = usize;

const HEIGHT: usize = 30;
const WIDTH: usize = 30;
const END_TURN: u64 = 100;
const INF: ScoreType = 1_000_000_000;

#[derive(Debug, Clone, Copy)]
struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new() -> Coord {
        Coord { x: 0, y: 0 }
    }

    #[allow(unused)]
    pub fn from_point(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
struct MazeState {
    pub character: Coord,
    pub game_score: ScoreType,
    pub evaluated_score: ScoreType,
    pub first_action: Option<Action>,
    points: [[ScoreType; WIDTH]; HEIGHT],
    turn: u64,
}

impl MazeState {
    pub fn new() -> MazeState {
        let mut rng = rand::thread_rng();
        let mut character = Coord::new();
        character.y = rng.gen_range(0..HEIGHT);
        character.x = rng.gen_range(0..WIDTH);
        let mut points = [[0; WIDTH]; HEIGHT];
        for (y, points) in points.iter_mut().enumerate() {
            for (x, point) in points.iter_mut().enumerate() {
                if y == character.y && x == character.x {
                    continue;
                }
                *point = rng.gen_range(0..10);
            }
        }
        MazeState {
            character,
            game_score: 0,
            evaluated_score: 0,
            first_action: None,
            points,
            turn: 0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.turn == END_TURN
    }

    pub fn advance(&mut self, action: Action) {
        let dx = [1, -1, 0, 0];
        let dy = [0, 0, 1, -1];
        self.character.x = self.character.x.checked_add_signed(dx[action]).unwrap_or(0);
        self.character.y = self.character.y.checked_add_signed(dy[action]).unwrap_or(0);
        let point = &mut self.points[self.character.y][self.character.x];
        if 0 < *point {
            self.game_score += *point;
            *point = 0;
        }
        self.turn += 1;
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        let dx = [1, -1, 0, 0];
        let dy = [0, 0, 1, -1];
        let mut actions = vec![];
        for act in 0..4 {
            let ty = self
                .character
                .y
                .checked_add_signed(dy[act])
                .unwrap_or(HEIGHT);
            let tx = self
                .character
                .x
                .checked_add_signed(dx[act])
                .unwrap_or(WIDTH);
            if ty < HEIGHT && tx < WIDTH {
                actions.push(act);
            }
        }
        actions
    }

    pub fn evaluate_score(&mut self) {
        self.evaluated_score = self.game_score;
    }
}

impl std::fmt::Display for MazeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "turn:\t{}", self.turn)?;
        writeln!(f, "score:\t{}", self.game_score)?;
        for h in 0..HEIGHT {
            for w in 0..WIDTH {
                let ch = if self.character.y == h && self.character.x == w {
                    '@'
                } else if self.points[h][w] > 0 {
                    (self.points[h][w] as u8 + b'0') as char
                } else {
                    '.'
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl PartialEq for MazeState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score == other.evaluated_score
    }
}

impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.evaluated_score.partial_cmp(&other.evaluated_score)
    }
}

impl Eq for MazeState {}

impl Ord for MazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score.cmp(&other.evaluated_score)
    }
}

struct TimeKeeper {
    start_time: Instant,
    time_threshold: u64,
}

impl TimeKeeper {
    pub fn new(time_threshold: u64) -> TimeKeeper {
        TimeKeeper {
            start_time: Instant::now(),
            time_threshold,
        }
    }

    pub fn is_time_over(&self) -> bool {
        Duration::from_millis(self.time_threshold) <= Instant::now().duration_since(self.start_time)
    }
}

fn random_action(state: &MazeState) -> Action {
    let mut rng = rand::thread_rng();
    let legal_action = state.legal_actions();
    legal_action[rng.gen_range(0..legal_action.len())]
}

fn greedy_action(state: &MazeState) -> Action {
    let legal_actions = state.legal_actions();
    let mut best_score = -INF;
    let mut best_action = None;
    for act in legal_actions.iter() {
        let mut now_state = *state;
        now_state.advance(*act);
        now_state.evaluate_score();
        if best_score < now_state.evaluated_score {
            best_score = now_state.evaluated_score;
            best_action = Some(*act);
        }
    }
    assert_ne!(best_action, None);
    best_action.unwrap()
}

fn beam_search_action(state: &MazeState, beam_width: usize, beam_depth: u64) -> Action {
    let mut now_beam = BinaryHeap::new();
    let mut best_state = MazeState::new();
    now_beam.push(*state);
    for d in 0..beam_depth {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            let Some(now_state) = now_beam.pop() else {
                break;
            };
            let legal_actions = now_state.legal_actions();
            for act in legal_actions.iter() {
                let mut next_state = now_state;
                next_state.advance(*act);
                next_state.evaluate_score();
                if d == 0 {
                    next_state.first_action = Some(*act);
                }
                next_beam.push(next_state);
            }
        }
        now_beam = next_beam;
        best_state = *now_beam.peek().unwrap();
        if best_state.is_done() {
            break;
        }
    }
    assert_ne!(best_state.first_action, None);
    best_state.first_action.unwrap()
}

fn beam_search_with_time_threshold_action(
    state: &MazeState,
    beam_width: usize,
    time_threshold: u64,
) -> Action {
    let time_keeper = TimeKeeper::new(time_threshold);
    let mut now_beam = BinaryHeap::new();
    let mut best_state = MazeState::new();
    now_beam.push(*state);
    for d in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if time_keeper.is_time_over() {
                return best_state.first_action.unwrap();
            }
            let Some(now_state) = now_beam.pop() else {
                break;
            };
            let legal_actions = now_state.legal_actions();
            for act in legal_actions.iter() {
                let mut next_state = now_state;
                next_state.advance(*act);
                next_state.evaluate_score();
                if d == 0 {
                    next_state.first_action = Some(*act);
                }
                next_beam.push(next_state);
            }
        }
        now_beam = next_beam;
        best_state = *now_beam.peek().unwrap();
        if best_state.is_done() {
            break;
        }
    }
    assert_ne!(best_state.first_action, None);
    best_state.first_action.unwrap()
}

fn chokudai_search_action(
    state: &MazeState,
    beam_width: usize,
    beam_depth: usize,
    beam_number: usize,
) -> Option<Action> {
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(*state);
    for _ in 0..beam_number {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if beam[t].is_empty() {
                    break;
                }
                let Some(now_state) = beam[t].peek().cloned() else {
                    break;
                };
                if now_state.is_done() {
                    break;
                }
                beam[t].pop();
                let legal_actions = now_state.legal_actions();
                for act in legal_actions.iter() {
                    let mut next_state = now_state;
                    next_state.advance(*act);
                    next_state.evaluate_score();
                    if t == 0 {
                        next_state.first_action = Some(*act);
                    }
                    beam[t + 1].push(next_state);
                }
            }
        }
    }
    for t in (0..=beam_depth).rev() {
        if !beam[t].is_empty() {
            return beam[t].peek()?.first_action;
        }
    }
    None
}

fn chokudai_search_with_time_threshold_action(
    state: &MazeState,
    beam_width: usize,
    beam_depth: usize,
    time_threshold: u64,
) -> Option<Action> {
    let time_keeper = TimeKeeper::new(time_threshold);
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(*state);
    loop {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if beam[t].is_empty() {
                    break;
                }
                let Some(now_state) = beam[t].peek().cloned() else {
                    break;
                };
                if now_state.is_done() {
                    break;
                }
                beam[t].pop();
                let legal_actions = now_state.legal_actions();
                for act in legal_actions.iter() {
                    let mut next_state = now_state;
                    next_state.advance(*act);
                    next_state.evaluate_score();
                    if t == 0 {
                        next_state.first_action = Some(*act);
                    }
                    beam[t + 1].push(next_state);
                }
            }
        }
        if time_keeper.is_time_over() {
            break;
        }
    }
    for t in (0..=beam_depth).rev() {
        if !beam[t].is_empty() {
            return beam[t].peek()?.first_action;
        }
    }
    None
}

pub fn play_game() {
    let mut state = MazeState::new();
    println!("{}", state);
    while !state.is_done() {
        state.advance(
            chokudai_search_with_time_threshold_action(&state, 5, END_TURN as usize, 10).unwrap(),
        );
    }
    println!("{}", state)
}

#[cfg(test)]
mod test {
    use super::*;
    const GAME_NUMBER: usize = 100;

    #[test]
    fn test_random_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(random_action(&state))
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Random Score:\t{}", mean)
    }

    #[test]
    fn test_greedy_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(greedy_action(&state))
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Greedy Score:\t{}", mean)
    }

    #[test]
    fn test_beam_search_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(beam_search_action(&state, 2, END_TURN))
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Beam Search Score:\t{}", mean)
    }

    #[test]
    fn test_beam_search_with_time_threshold_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(beam_search_with_time_threshold_action(&state, 5, 10))
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Beam Search 10ms Score:\t{}", mean)
    }

    #[test]
    fn test_chokudai_search_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(chokudai_search_action(&state, 1, END_TURN as usize, 2).unwrap())
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Chokudai Search Score:\t{}", mean)
    }

    #[test]
    fn test_chokudai_search_1ms_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(
                    chokudai_search_with_time_threshold_action(&state, 5, END_TURN as usize, 1)
                        .unwrap(),
                )
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Beam Search 1ms Score:\t{}", mean)
    }

    #[test]
    #[ignore]
    fn test_chokudai_search_10ms_score() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = MazeState::new();
            while !state.is_done() {
                state.advance(
                    chokudai_search_with_time_threshold_action(&state, 5, END_TURN as usize, 10)
                        .unwrap(),
                )
            }
            mean += state.game_score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Beam Search 10ms Score:\t{}", mean)
    }
}
