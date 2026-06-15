use crate::parity_game::ParityGame;

pub fn verify_solution(
    game: &ParityGame,
    w0: &[usize],
    w1: &[usize],
    strat0: &[Option<usize>],
    strat1: &[Option<usize>],
) -> Result<(), String> {
    let node_count = game.num_nodes();
    
    let mut in_w0 = vec![false; node_count];
    let mut in_w1 = vec![false; node_count];

    for &v in w0 { in_w0[v] = true; }
    for &v in w1 {
        if in_w0[v] { return Err(format!("Node {} is in both W0 and W1", v)); }
        in_w1[v] = true; 
    }
    for v in 0..node_count {
        if !in_w0[v] && !in_w1[v] { return Err(format!("Node {} is in neither W0 nor W1", v)); }
    }

    validate_player_witness(game, &in_w0, strat0, 0)?;
    validate_player_witness(game, &in_w1, strat1, 1)?;

    check_parity_condition(game, &in_w0, strat0, 0)?;
    check_parity_condition(game, &in_w1, strat1, 1)?;

    Ok(())
}

fn check_parity_condition(
    game: &ParityGame,
    in_region: &[bool],
    strategy: &[Option<usize>],
    player: usize,
) -> Result<(), String> {
    let sccs = game.tarjan_sccs_restricted(in_region, strategy, player);

    for scc in sccs {
        let is_nontrivial = scc.len() > 1 || {
            let u = scc[0];
            if game.get_owner(u) == player {
                strategy[u] == Some(u)
            } else {
                game.get_successors(u).contains(&u)
            }
        };
        if !is_nontrivial { continue; }

        let max_prio = scc.iter().map(|&v| game.get_priority(v)).max().unwrap_or(0);
        if max_prio % 2 != player {
            return Err(format!(
                "Bottom SCC {:?} in W{} has max priority {} (wrong parity)",
                scc, player, max_prio
            ));
        }
    }
    Ok(())
}

fn validate_player_witness(
    game: &ParityGame,
    in_region: &[bool],
    strategy: &[Option<usize>],
    player: usize,
) -> Result<(), String> {
    for v in 0..game.num_nodes() {
        if !in_region[v] { continue; }

        if game.get_owner(v) == player {
            let succ = strategy[v].ok_or_else(|| {
                format!("Winning player {} has no strategy at node {}", player, v)
            })?;
            
            if !game.get_successors(v).contains(&succ) {
                return Err(format!("Invalid strategy edge ({}, {})", v, succ));
            }
            if !in_region[succ] {
                return Err(format!("Strategy edge ({}, {}) leaves winning region W{}", v, succ, player));
            }
        } else {
            for &succ in game.get_successors(v) {
                if !in_region[succ] {
                    return Err(format!("Opponent {} can escape region W{} via edge to {}", v, player, succ));
                }
            }
        }
    }
    Ok(())
}