#![allow(unused)]

use rand::Rng;

type ScoreType = i64;

const HEIGHT: usize = 5;
const WIDTH: usize = 5;
const END_TURN: usize = 5;
const CHARACTER_N: usize = 3;
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
struct AutoMoveMazeState {
    pub game_score: ScoreType,
    pub evaluated_score: ScoreType,
    points: [[ScoreType; WIDTH]; HEIGHT],
    turn: usize,
    characters: [Coord; CHARACTER_N],
}

impl AutoMoveMazeState {
    fn new() -> AutoMoveMazeState {
        let mut rng = rand::thread_rng();
        let mut points = [[0; WIDTH]; HEIGHT];
        for row in points.iter_mut() {
            for point in row.iter_mut() {
                *point = rng.gen_range(0..10);
            }
        }
        AutoMoveMazeState {
            game_score: 0,
            evaluated_score: 0,
            points,
            turn: 0,
            characters: [Coord::new(); CHARACTER_N],
        }
    }

    fn init_characters(&mut self) {
        let mut rng = rand::thread_rng();
        for character in self.characters.iter_mut() {
            character.y = rng.gen_range(0..HEIGHT);
            character.x = rng.gen_range(0..WIDTH);
        }
    }

    fn transition(&mut self) {
        let mut rng = rand::thread_rng();
        let character = &mut self.characters[rng.gen_range(0..CHARACTER_N)];
        character.y = rng.gen_range(0..HEIGHT);
        character.x = rng.gen_range(0..WIDTH);
    }

    fn set_character(&mut self, character_id: usize, y: usize, x: usize) {
        self.characters[character_id].y = y;
        self.characters[character_id].x = x;
    }

    pub fn is_done(&self) -> bool {
        self.turn == END_TURN
    }

    fn get_score(&self, is_print: bool) -> ScoreType {
        let mut state = *self;
        for character in state.characters.iter() {
            let point = &mut state.points[character.y][character.x];
            *point = 0;
        }
        while !state.is_done() {
            state.advance();
            if is_print {
                println!("{}", state);
            }
        }
        state.game_score
    }

    fn move_player(&mut self, character_id: usize) {
        let dx = [1, -1, 0, 0];
        let dy = [0, 0, 1, -1];
        let character = &mut self.characters[character_id];
        let mut best_point = -INF;
        let mut best_action_index = 0;
        for action in 0..4 {
            let ty = character.y.checked_add_signed(dy[action]).unwrap_or(HEIGHT);
            let tx = character.x.checked_add_signed(dx[action]).unwrap_or(WIDTH);
            if ty < HEIGHT && tx < WIDTH {
                let point = self.points[ty][tx];
                if best_point < point {
                    best_point = point;
                    best_action_index = action;
                }
            }
        }
        character.y = character
            .y
            .checked_add_signed(dy[best_action_index])
            .unwrap();
        character.x = character
            .x
            .checked_add_signed(dx[best_action_index])
            .unwrap();
    }

    pub fn advance(&mut self) {
        for id in 0..CHARACTER_N {
            self.move_player(id);
        }
        for character in self.characters.iter() {
            let point = &mut self.points[character.y][character.x];
            self.game_score += *point;
            *point = 0;
        }
        self.turn += 1;
    }
}

impl std::fmt::Display for AutoMoveMazeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "turn:\t{}", self.turn)?;
        writeln!(f, "score:\t{}", self.game_score)?;
        for h in 0..HEIGHT {
            for w in 0..WIDTH {
                let mut is_char = false;
                for character in self.characters.iter() {
                    if character.y == h && character.x == w {
                        is_char = true;
                        break;
                    }
                }
                let ch = if is_char {
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

fn random_action(state: &mut AutoMoveMazeState) -> AutoMoveMazeState {
    let mut rng = rand::thread_rng();
    for id in 0..CHARACTER_N {
        let y = rng.gen_range(0..HEIGHT);
        let x = rng.gen_range(0..WIDTH);
        state.set_character(id, y, x);
    }
    *state
}

fn hill_climb(state: &AutoMoveMazeState, number: usize) -> AutoMoveMazeState {
    let mut now_state = *state;
    now_state.init_characters();
    let mut best_score = now_state.get_score(false);
    for _ in 0..number {
        let mut next_state = now_state;
        next_state.transition();
        let next_score = next_state.get_score(false);
        if best_score < next_score {
            best_score = next_score;
            now_state = next_state;
        }
    }
    now_state
}

fn simulated_annealing(
    state: &AutoMoveMazeState,
    number: usize,
    start_temp: f64,
    end_tmp: f64,
) -> AutoMoveMazeState {
    let mut rng = rand::thread_rng();
    let mut now_state = *state;
    now_state.init_characters();
    let mut best_score = now_state.get_score(false);
    let mut now_score = best_score;
    let mut best_state = now_state;
    for i in 0..number {
        let mut next_state = now_state;
        next_state.transition();
        let next_score = next_state.get_score(false);
        let temp = start_temp + (end_tmp - start_temp) * (i as f64 / number as f64);
        let probability = ((next_score - now_score) as f64 / temp).exp();
        let is_force_next = probability > rng.gen_range(0.0..1.0);
        if now_score < next_score || is_force_next {
            now_score = next_score;
            now_state = next_state;
        }
        if best_score < next_score {
            best_score = next_score;
            best_state = next_state;
        }
    }
    best_state
}

pub fn play_game() {
    let mut state = AutoMoveMazeState::new();
    let state = random_action(&mut state);
    println!("{}", state);
    let score = state.get_score(true);
    println!("Score of random Action: {}", score);
}

#[cfg(test)]
mod tests {
    use super::*;
    const GAME_NUMBER: usize = 100;

    #[test]
    fn test_random_action() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let mut state = AutoMoveMazeState::new();
            let state = random_action(&mut state);
            let score = state.get_score(false);
            mean += score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Score of random Action: {}", mean);
    }

    #[test]
    fn test_hill_climb_action() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let state = AutoMoveMazeState::new();
            let state = hill_climb(&state, 10000);
            let score = state.get_score(false);
            mean += score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Score of Hill Climb Action: {}", mean);
    }

    #[test]
    fn test_simulated_annealing_action() {
        let mut mean = 0.0;
        for _ in 0..GAME_NUMBER {
            let state = AutoMoveMazeState::new();
            let state = simulated_annealing(&state, 10000, 500.0, 10.0);
            let score = state.get_score(false);
            mean += score as f64;
        }
        mean /= GAME_NUMBER as f64;
        println!("Score of Simulated Annealing Action: {}", mean);
    }
}
