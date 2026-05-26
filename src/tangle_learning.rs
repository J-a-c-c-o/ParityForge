use crate::parity_game::ParityGame;

use std::collections::HashSet;

pub fn run_tl(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>), String> {
    Ok(solve(game, HashSet::new()))
}

fn solve(game: &ParityGame, excluded: HashSet<usize>) -> (Vec<usize>, Vec<usize>) {
    (Vec::new(), Vec::new())
}

    

    