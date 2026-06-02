use crate::parity_game::ParityGame;

pub fn run_fpi(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_game::ParityGameBuilder;


    fn example_game() -> ParityGame {
        let mut builder = ParityGameBuilder::new();
        let builder = builder
            .add_edge(0, 4)
            .add_edge(4, 0)
            .add_edge(7, 4)
            .add_edge(1, 7)
            .add_edge(4, 8)
            .add_edge(8, 6)
            .add_edge(7, 6)
            .add_edge(2, 1)
            .add_edge(2, 3)
            .add_edge(3, 2)
            .add_edge(3, 5)
            .add_edge(5, 3)
            .add_edge(5, 1)
            .add_edge(1, 5)
            .add_edge(6, 2)
            .set_owner(0, 0)
            .set_owner(1, 1)
            .set_owner(2, 0)
            .set_owner(3, 0)
            .set_owner(4, 1)
            .set_owner(5, 0)
            .set_owner(6, 0)
            .set_owner(7, 0)
            .set_owner(8, 0)
            .set_priority(0, 0)
            .set_priority(1, 1)
            .set_priority(2, 2)
            .set_priority(3, 3)
            .set_priority(4, 2)
            .set_priority(5, 5)
            .set_priority(6, 6)
            .set_priority(7, 7)
            .set_priority(8, 8);
        
        let game = builder.build();
        game
    }


    #[test]
    fn test_tl() {
        let game = example_game();
        let (w0, w1, strat0, strat1) = run_fpi(&game).unwrap();
        println!("Winning set for player 0: {:?}", w0);
        println!("Winning set for player 1: {:?}", w1);
        println!("Strategy for player 0: {:?}", strat0);
        println!("Strategy for player 1: {:?}", strat1);

        panic!();
    }

    

}