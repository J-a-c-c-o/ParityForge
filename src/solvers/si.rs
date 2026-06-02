use crate::parity_game::ParityGame;

pub fn run_si(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    let strat0 = vec![None; game.num_nodes()];
    let strat1 = vec![None; game.num_nodes()];
    let w0 = Vec::new();
    let w1 = Vec::new();

    (w0, w1, strat0, strat1)
}