use crate::parity_game::ParityGame;

pub fn run_fpj(
    game: &ParityGame,
) -> Result<
    (
        Vec<usize>,
        Vec<usize>,
        Vec<Option<usize>>,
        Vec<Option<usize>>,
    ),
    String,
> {
    Ok(solve(game))
}

fn solve(
    game: &ParityGame,
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    let highest_priority = game.get_max_priority();

    let mut distractions = vec![false; game.num_nodes()];
    let mut justifications = vec![false; game.num_nodes()];
    let mut strat = vec![None; game.num_nodes()];

    let mut p = 0;
    while p <= highest_priority {
        let alpha = p % 2;

        let mut changed = false;
        let mut to_unjustify = Vec::new();
        for v in game.get_nodes_with_priority(p) {
            if justifications[v] || distractions[v] {
                continue;
            }

            let (alpha_prime, strat_v) = onestep(game, v, &distractions);

            if alpha_prime != alpha {
                distractions[v] = true;
                changed = true;
                to_unjustify.push(v);
            }

            strat[v] = strat_v;
            justifications[v] = true;
        }

        if changed {
            for v in to_unjustify {
                unjustify(game, v, &mut justifications, &mut distractions, &strat);
            }
            p = 0;
        } else {
            p += 1;
        }
    }

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut strat0 = vec![None; game.num_nodes()];
    let mut strat1 = vec![None; game.num_nodes()];

    for v in game.get_nodes() {
        if winner(game, v, &distractions) == 0 {
            w0.push(v);
            if game.get_owner(v) == 0 {
                strat0[v] = strat[v];
            }
        } else {
            w1.push(v);
            if game.get_owner(v) == 1 {
                strat1[v] = strat[v];
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

fn unjustify(
    game: &ParityGame,
    v: usize,
    justifications: &mut [bool],
    distractions: &mut [bool],
    strat: &[Option<usize>],
) {
    for &pred in game.get_predecessors(v) {
        if !justifications[pred] {
            continue;
        }

        let pred_owner = game.get_owner(pred);
        let pred_winner = winner(game, pred, distractions);

        let depends_on_v = if pred_owner == pred_winner {
            strat[pred] == Some(v)
        } else {
            true
        };

        if depends_on_v {
            justifications[pred] = false;
            distractions[pred] = false;

            unjustify(game, pred, justifications, distractions, strat);
        }
    }
}
