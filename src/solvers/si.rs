use crate::parity_game::ParityGame;
use std::ops;

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

    let mut in_halting = vec![false; game.num_nodes()];
    for node in 0..game.num_nodes() {
        if game.get_priority(node) % 2 == 1 {
            in_halting[node] = true;
        }
    }

        
    loop {
        
        let mut sigma_changed = false;
        let mut h_changed = false;
        let mut valuations;

        loop {
            valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);
            let mut tau_changed = false;
            for node in 0..game.num_nodes() {
                if game.get_owner(node) == 1 {
                    let current_succ = strat1[node].unwrap();
                    let mut best_succ = current_succ;
                    let mut best_val = &valuations[current_succ];

                    for &succ in game.get_successors(node).iter() {
                        let succ_val = &valuations[succ];
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


        for node in 0..game.num_nodes() {
            if game.get_owner(node) == 0 {
                let current_succ = strat0[node].unwrap();
                let mut best_succ = current_succ;
                let mut best_val = &valuations[current_succ];

                for &succ in game.get_successors(node).iter() {
                    let succ_val = &valuations[succ];
                    if succ_val > best_val {
                        best_val = succ_val;
                        best_succ = succ;
                    }
                }

                if best_succ != current_succ {
                    strat0[node] = Some(best_succ);
                    sigma_changed = true;
                }
            }
        }

        for node in 0..game.num_nodes() {
            if in_halting[node] && valuations[node] > Valuation::Finite(vec![0; game.get_max_priority() + 1]) {
                in_halting[node] = false;
                h_changed = true;
            }
        }

        if !sigma_changed && !h_changed {
            break;
        }
    }
    
    let final_valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut new_strat0: Vec<Option<usize>> = vec![None; game.num_nodes()];
    let mut new_strat1: Vec<Option<usize>> = vec![None; game.num_nodes()];
    for node in 0..game.num_nodes() {
        if &final_valuations[node] == &Valuation::Infinite {
            w0.push(node);
            new_strat0[node] = strat0[node];
        } else {
            w1.push(node);
            new_strat1[node] = strat1[node];
        }
    }

    (w0, w1, new_strat0, new_strat1)
}

#[derive(Debug, Clone)]
enum Valuation {
    Infinite,
    Finite(Vec<usize>),
}

impl ops::Add for &Valuation {
    type Output = Valuation;

    fn add(self, other: &Valuation) -> Valuation {
        match (self, other) {
            (Valuation::Infinite, _) | (_, Valuation::Infinite) => Valuation::Infinite,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => {
                let summed_vec: Vec<usize> = vec1.iter().zip(vec2.iter()).map(|(a, b)| a + b).collect();
                Valuation::Finite(summed_vec)
            }
        }
    }
}


impl PartialEq for Valuation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Valuation::Infinite, Valuation::Infinite) => true,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => vec1 == vec2,
            _ => false,
        }
    }
}

impl Eq for Valuation {}

impl PartialOrd for Valuation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Valuation::Infinite, Valuation::Infinite) => Some(std::cmp::Ordering::Equal),
            (Valuation::Infinite, _) => Some(std::cmp::Ordering::Greater),
            (_, Valuation::Infinite) => Some(std::cmp::Ordering::Less),
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => Some(compare_vec(vec1, vec2)),
        }
    }
}

impl Ord for Valuation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}


fn compare_vec(vec1: &[usize], vec2: &[usize]) -> std::cmp::Ordering {
    for (priority, (a, b)) in vec1.iter().zip(vec2.iter()).enumerate().rev() {
        if a != b {
            if priority % 2 == 0 {
                return a.cmp(b);
            } else {
                return b.cmp(a);
            }
        }
    }
    std::cmp::Ordering::Equal
}

fn compute_all_valuations(game: &ParityGame, strat0: &[Option<usize>], strat1: &[Option<usize>], in_halting: &[bool]) -> Vec<Valuation> {
    let mut valuations = vec![Valuation::Finite(vec![0; game.get_max_priority() + 1]); game.num_nodes()];
    let mut visited = vec![false; game.num_nodes()];
    let mut nodes_with_no_successors = Vec::new();
    for node in 0..game.num_nodes() {
        let successor = if game.get_owner(node) == 0 {
            strat0[node]
        } else {
            strat1[node]
        };
        if successor.is_none() || in_halting[successor.unwrap()] {
            nodes_with_no_successors.push(node);
        } 
    }

    while let Some(node) = nodes_with_no_successors.pop() {
        dfs_backwards(game, node, strat0, strat1, in_halting, &mut valuations, &mut visited);
    }

    for node in 0..game.num_nodes() {
        if !visited[node] {
            valuations[node] = Valuation::Infinite;
        }
    }


    valuations
}


fn dfs_backwards(game: &ParityGame, node: usize, strat0: &[Option<usize>], strat1: &[Option<usize>], in_halting: &[bool], valuations: &mut [Valuation], visited: &mut [bool]) {

    visited[node] = true;


    match valuations[node] {
        Valuation::Infinite => {},
        Valuation::Finite(ref mut vec) => {
            let priority = game.get_priority(node);
            vec[priority] += 1;
        }
    }

    if in_halting[node] {
        return;
    }
    


    for &pred in game.get_predecessors(node) {
        if strat0[pred] != Some(node) && strat1[pred] != Some(node) {
            continue;
        } 

        let valuation = &valuations[pred] + &valuations[node];
        valuations[pred] = valuation;

        dfs_backwards(game, pred, strat0, strat1, in_halting, valuations, visited);

        
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
    fn test_dfs_backwards() {
        let game = example_game();
        let strat0 = vec![Some(4), None, Some(3), Some(2), None, Some(3), Some(2), Some(6), Some(6)];
        let strat1 = vec![None, Some(7), None, None, Some(0), None, None, None, None];
        let in_halting = vec![true, false, false, false, true, false, true, false, true];

        let valuations = compute_all_valuations(&game, &strat0, &strat1, &in_halting);
        println!("{:?}", valuations);

        panic!();

    }

    #[test]
    fn test_si() {
        let game = example_game();
        let (w0, w1, strat0, strat1) = solve(&game);
        println!("Winning set for player 0: {:?}", w0);
        println!("Winning set for player 1: {:?}", w1);
        println!("Strategy for player 0: {:?}", strat0);
        println!("Strategy for player 1: {:?}", strat1);

        panic!();
    }

    

}