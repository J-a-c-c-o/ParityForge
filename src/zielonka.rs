use crate::parity_game::ParityGame;
use crate::utils::attract;

use std::collections::HashSet;

pub fn run_zielonka(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<(usize, usize)>, Vec<(usize, usize)>), String> {
    let (w0, w1, strat0, strat1) = solve(game, HashSet::new());
    Ok((w0, w1, strat0, strat1))
}


fn solve(game: &ParityGame, excluded: HashSet<usize>) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if game.num_nodes() == excluded.len() {
        return (vec![], vec![], vec![], vec![]);
    }

    let max_priority = game
        .get_nodes()
        .into_iter()
        .filter(|v| !excluded.contains(v))
        .map(|v| game.get_priority(v))
        .max()
        .unwrap();

    let player = max_priority % 2;

    let max_nodes: Vec<usize> = game
        .get_nodes_with_priority(max_priority)
        .into_iter()
        .filter(|v| !excluded.contains(v))
        .collect();

    let (a, strat_a) = attract(game, &excluded, &max_nodes, player);

    let mut excluded_a = excluded.clone();
    excluded_a.extend(a.iter().copied());

    let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, excluded_a);

    let opponent_region = if player == 0 { &w1 } else { &w0 };
    let opponent_strategy = if player == 0 { &strat_w1 } else { &strat_w0 };

    let (b, mut strat_b) = attract(game, &excluded, opponent_region, 1 - player);

    strat_b.extend(opponent_strategy.iter().copied());
    

    let b_set: HashSet<_> = b.iter().copied().collect();
    let opp_set: HashSet<_> = opponent_region.iter().copied().collect();

    if b_set == opp_set {
        if player == 0 {
            w0.extend(a);
            strat_w0.extend(strat_a.iter().copied());
            strat_w0.extend(pick(game, &max_nodes, &w0));
        } else {
            w1.extend(a);
            strat_w1.extend(strat_a.iter().copied());
            strat_w1.extend(pick(game, &max_nodes, &w1));
        }


        (w0, w1, strat_w0, strat_w1)
    } else {
        let mut excluded_b = excluded.clone();
        excluded_b.extend(b.iter().copied());

        let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, excluded_b);

        if player == 0 {
            w1.extend(b);
            strat_w1.extend(strat_b.iter().copied());
        } else {
            w0.extend(b);
            strat_w0.extend(strat_b.iter().copied());
        }

        (w0, w1, strat_w0, strat_w1)
    }
}


fn pick(game: &ParityGame, max_nodes: &[usize], winning_region: &[usize]) -> Vec<(usize, usize)> {
    let mut strategies = Vec::new();
    for &node in max_nodes {
        for &successor in game.get_successors(node) {
            if winning_region.contains(&successor) {
                strategies.push((node, successor));
                break;
            }
        }
    }
    strategies
}