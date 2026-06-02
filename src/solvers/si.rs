use crate::parity_game::ParityGame;
use std::cmp::Ordering;

pub fn run_si(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    let mut strat0 = vec![None; game.get_num_nodes()];
    let mut strat1 = vec![None; game.get_num_nodes()];
    let mut w0 = Vec::new();
    let mut w1 = Vec::new();

    (w0, w1, strat0, strat1)
}