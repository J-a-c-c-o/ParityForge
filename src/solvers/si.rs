use crate::parity_game::ParityGame;

pub fn run_si(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

pub fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    let mut strat0 = vec![None; game.num_nodes()];
    let mut strat1 = vec![None; game.num_nodes()];

    for node in 0..game.num_nodes() {
        let player = game.get_owner(node);
        let best = game.get_successors(node).iter().min_by_key(|&s| game.get_priority(*s)).unwrap();
        if player == 0 {
            strat0[node] = Some(*best);
        } else {
            strat1[node] = Some(*best);
        }
    }

    let mut halting_set: Vec<usize> = (0..game.num_nodes()).collect();

        
    loop {
        let mut sigma_or_h_changed = false;

        loop {
            let in_halting = build_halting_map(game.num_nodes(), &halting_set);
            let valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

            let mut tau_changed = false;
            for node in 0..game.num_nodes() {
                if game.get_owner(node) == 1 {
                    let current_succ = strat1[node].unwrap();
                    let mut best_succ = current_succ;
                    let mut best_val = valuations[current_succ];

                    for &succ in game.get_successors(node).iter() {
                        let succ_val = valuations[succ];
                        if succ_val < best_val {
                            best_val = succ_val;
                            best_succ = succ;
                        }
                    }

                    if best_succ != current_succ {
                        strat1[node] = Some(best_succ);
                        tau_changed = true;
                    }
                }
            }

            if !tau_changed {
                break;
            }
        }

        let in_halting = build_halting_map(game.num_nodes(), &halting_set);
        let valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

        for node in 0..game.num_nodes() {
            if game.get_owner(node) == 0 {
                let current_succ = strat0[node].unwrap();
                let mut best_succ = current_succ;
                let mut best_val = valuations[current_succ];

                for &succ in game.get_successors(node).iter() {
                    let succ_val = valuations[succ];
                    if succ_val > best_val {
                        best_val = succ_val;
                        best_succ = succ;
                    }
                }

                if best_succ != current_succ {
                    strat0[node] = Some(best_succ);
                    sigma_or_h_changed = true;
                }
            }
        }

        halting_set.retain(|&node| {
            if valuations[node].score >= 0 {
                sigma_or_h_changed = true;
                false 
            } else {
                true
            }
        });

        if !sigma_or_h_changed {
            break;
        }
    }
    

    let in_halting = build_halting_map(game.num_nodes(), &halting_set);
    let final_valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    for node in 0..game.num_nodes() {
        if final_valuations[node].score >= 0 {
            w0.push(node);
        } else {
            w1.push(node);
        }
    }

    (w0, w1, strat0, strat1)
}


fn build_halting_map(node_count: usize, halting_set: &[usize]) -> Vec<bool> {
    let mut in_halting = vec![false; node_count];
    for &node in halting_set {
        if node < node_count {
            in_halting[node] = true;
        }
    }
    in_halting
}