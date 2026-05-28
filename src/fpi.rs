use crate::parity_game::ParityGame;

pub fn run_fpi(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<(usize, usize)>, Vec<(usize, usize)>), String> {
    let (w0, w1, strat0, strat1) = solve(game);
    Ok((w0, w1, strat0, strat1))
}

fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let highest_priority = game.get_max_priority();
    
    let mut distractions = vec![false; game.num_nodes()];
    let mut frozen = vec![None; game.num_nodes()];
    let mut strat = vec![None; game.num_nodes()];

    let mut p = 0;
    while p <= highest_priority {
        let alpha = p % 2;
        let mut changed = false;
        
        for v in game.get_nodes_with_priority(p) {
            if frozen[v].is_some() || distractions[v] {
                continue;
            }

            let (alpha_prime, strat_v) = onestep(game, v, &distractions);
            strat[v] = strat_v;
            if alpha_prime != alpha {
                distractions[v] = true;
                changed = true;
            }
        }

        if changed {
            for v in game.get_nodes_with_prio_eval(|prio| prio < p) {
                if frozen[v].is_some() {
                    continue;
                }
                if winner(game, v, &distractions) == 1 - alpha {
                    frozen[v] = Some(p);
                } else {
                    distractions[v] = false; 
                }
            }
            p = 0;
        } else {
            for v in game.get_nodes_with_prio_eval(|prio| prio < p) {
                if frozen[v] == Some(p) {
                    frozen[v] = None;
                }
            }
            p += 1;
        }
    }

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut strat0 = Vec::new();
    let mut strat1 = Vec::new();

    for v in game.get_nodes() {
        if winner(game, v, &distractions) == 0 {
            w0.push(v);
            if let Some(s) = strat[v] && game.get_owner(v) == 0 {
                strat0.push((v, s));
            }
        } else {
            w1.push(v);
            if let Some(s) = strat[v] && game.get_owner(v) == 1 {
                strat1.push((v, s));
            }
        }
    }

    (w0, w1, strat0, strat1)
}

fn onestep(game: &ParityGame, v: usize, distractions: &[bool]) -> (usize, Option<usize>) {
    let alpha = game.get_owner(v);
    for &succ in game.get_successors(v) {
        if winner(game, succ, distractions) == alpha {
            return (alpha, Some(succ));
        }
    }
    (1 - alpha, None)
}

fn winner(game: &ParityGame, v: usize, distractions: &[bool]) -> usize {
    let prio = game.get_priority(v);
    if distractions[v] {
        1 - (prio % 2)
    } else {
        prio % 2
    }
}