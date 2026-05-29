use crate::parity_game::ParityGame;
use crate::utils::attract;

pub fn run_zielonka(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    let excluded = vec![false; game.num_nodes()];
    Ok(solve(game, &excluded))
}

pub fn zielonka_solve(game: &ParityGame, excluded: &[bool]) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
     solve(game, excluded)
}


fn solve(game: &ParityGame, excluded: &[bool]) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    if excluded.iter().all(|&is_excluded| is_excluded) {
        let empty = vec![None; game.num_nodes()];
        return (vec![], vec![], empty.clone(), empty);
    }

    let max_priority = game
        .get_nodes()
        .into_iter()
        .filter(|v| !excluded[*v])
        .map(|v| game.get_priority(v))
        .max()
        .unwrap();

    let player = max_priority % 2;

    let max_nodes: Vec<usize> = game
        .get_nodes_with_priority(max_priority)
        .into_iter()
        .filter(|v| !excluded[*v])
        .collect();

    let (a, strat_a) = attract(game, &excluded, &max_nodes, player);

    let mut excluded_a = excluded.to_vec();
    for node in &a {
        excluded_a[*node] = true;
    }

    let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, &excluded_a);

    let opponent_region = if player == 0 { &w1 } else { &w0 };
    let opponent_strategy = if player == 0 { &strat_w1 } else { &strat_w0 };

    let (b, mut strat_b) = attract(game, excluded, opponent_region, 1 - player);

    merge_strategy(&mut strat_b, opponent_strategy);
    
    let mut b_sorted = b.clone();
    b_sorted.sort_unstable();
    let mut opp_sorted = opponent_region.clone();
    opp_sorted.sort_unstable();

    if b_sorted == opp_sorted {
        if player == 0 {
            w0.extend(a);
            merge_strategy(&mut strat_w0, &strat_a);
            let pick_strat = pick(game, &max_nodes, &w0);
            merge_strategy(&mut strat_w0, &pick_strat);
        } else {
            w1.extend(a);
            merge_strategy(&mut strat_w1, &strat_a);
            let pick_strat = pick(game, &max_nodes, &w1);
            merge_strategy(&mut strat_w1, &pick_strat);
        }


        (w0, w1, strat_w0, strat_w1)
    } else {
        let mut excluded_b = excluded.to_vec();
        for node in &b {
            excluded_b[*node] = true;
        }

        let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, &excluded_b);

        if player == 0 {
            w1.extend(b);
            merge_strategy(&mut strat_w1, &strat_b);
        } else {
            w0.extend(b);
            merge_strategy(&mut strat_w0, &strat_b);
        }

        (w0, w1, strat_w0, strat_w1)
    }
}


fn pick(game: &ParityGame, max_nodes: &[usize], winning_region: &[usize]) -> Vec<Option<usize>> {
    let mut strategies = vec![None; game.num_nodes()];
    for &node in max_nodes {
        for &successor in game.get_successors(node) {
            if winning_region.contains(&successor) {
                strategies[node] = Some(successor);
                break;
            }
        }
    }
    strategies
}

fn merge_strategy(target: &mut [Option<usize>], source: &[Option<usize>]) {
    for (idx, entry) in source.iter().enumerate() {
        if target[idx].is_none() {
            target[idx] = *entry;
        }
    }
}