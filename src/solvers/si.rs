use crate::parity_game::ParityGame;

pub fn run_si(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

pub fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {

    let mut strategy = vec![0; game.num_nodes()];
    let mut halt = vec![false; game.num_nodes()];
    let mut valuations = vec![Valuation::Val(vec![0; game.get_max_priority() + 1]); game.num_nodes()];


    for i in 0..game.num_nodes() {
        strategy[i] = *game.get_successors(i).first().unwrap();
        if game.get_priority(i) % 2 == 1 {
            halt[i] = true;
        }
    }

    

    loop {
        loop {
            compute_vals(game, &strategy, &halt, &mut valuations);

            let odd_changes = switch_strategy(1, game, &mut strategy, &halt, &valuations);
            
            if !odd_changes {
                break;
            }
        }

        for val in valuations.iter_mut() {
            if *val == Valuation::Top {
                *val = Valuation::Win;
            }
        }

        let mut even_changes = switch_strategy(0, game, &mut strategy, &halt, &valuations);
        
        for (i, halted) in halt.iter_mut().enumerate() {
            if *halted && compare(None, Some(i), &valuations) {
                *halted = false;
                even_changes = true;
            }
        }

        if !even_changes {
            break;
        }
    }

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut new_strat0: Vec<Option<usize>> = vec![None; game.num_nodes()];
    let mut new_strat1: Vec<Option<usize>> = vec![None; game.num_nodes()];

    for i in 0..game.num_nodes() {
        if valuations[i] == Valuation::Win {
            w0.push(i);
            if game.get_owner(i) == 0 {
                new_strat0[i] = Some(strategy[i]);
            }
        } else {
            w1.push(i);
            if game.get_owner(i) == 1 {
                new_strat1[i] = Some(strategy[i]);
            }
        }
    }

    (w0, w1, new_strat0, new_strat1)
}

#[derive(Clone, Eq, PartialEq)]
enum Valuation {
    Top,
    Win,
    Val(Vec<usize>),
}


fn compare(a: Option<usize>, b: Option<usize>, val: &[Valuation]) -> bool {
    if a == b { return false; }
    
    let a_top = a.is_some_and( |node| val[node] == Valuation::Win || val[node] == Valuation::Top);
    let b_top = b.is_some_and( |node| val[node] == Valuation::Win || val[node] == Valuation::Top);
    
    if a_top { return false; }
    if b_top { return true; }

    let lenght = match val.iter().find(|v| matches!(v, Valuation::Val(_))) {
        Some(Valuation::Val(v)) => v.len(),
        _ => 0,
    };

    for i in (0..lenght).rev() {
        let ai = match a {
            Some(node) => match &val[node] {
                Valuation::Val(v) => v[i],
                _ => 0,
            },
            None => 0,
        };
        let bi = match b {
            Some(node) => match &val[node] {
                Valuation::Val(v) => v[i],
                _ => 0,
            },
            None => 0,
        };
        if ai == bi { continue; }
        if i % 2 == 1 {
            return ai > bi;
        } else {
            return ai < bi;
        }
    }
    false
}

fn compute_vals(
    game: &ParityGame,
    strategy: &[usize],
    halt: &[bool],
    valuations: &mut [Valuation],
) {
    let mut first_in: Vec<Option<usize>> = vec![None; game.num_nodes()];
    let mut next_in: Vec<Option<usize>> = vec![None; game.num_nodes()];
    let mut q = Vec::new();

    for i in 0..game.num_nodes() {
        if valuations[i] == Valuation::Win { 
            continue; 
        }
        let s: usize = strategy[i];
        if halt[s] {
            q.push(i);
        } else {
            next_in[i] = first_in[s];
            first_in[s] = Some(i);
            if valuations[i] != Valuation::Top {
                valuations[i] = Valuation::Top;
            }
        }
    }

    while let Some(v) = q.pop() {
        let s = strategy[v];
        let mut new_val = vec![0; game.get_max_priority() + 1];
        
        if !halt[s] && let Valuation::Val(ref s_val) = valuations[s] {
            new_val.copy_from_slice(s_val);
        }
        new_val[game.get_priority(v)] += 1;
        
        valuations[v] = Valuation::Val(new_val.clone());

        let mut from_opt = first_in[v];
        while let Some(from) = from_opt {
            q.push(from);
            from_opt = next_in[from];
        }
    }
}

fn switch_strategy(
    pl: usize,
    game: &ParityGame,
    strategy: &mut [usize],
    halt: &[bool],
    val: &[Valuation],
) -> bool {
    let mut changes = false;

    for i in 0..game.num_nodes() {
        if val[i] == Valuation::Win || game.get_owner(i) != pl {
            continue;
        }

        let mut cur_strat = strategy[i];
        let mut changed = false;

        for &to in game.get_successors(i) {
            if to == cur_strat { continue; }

            let cur_node = if halt[cur_strat] { None } else { Some(cur_strat) };
            let to_node = if halt[to] { None } else { Some(to) };

            if pl == 0 {
                if compare(cur_node, to_node, val) {
                    cur_strat = to;
                    changed = true;
                }
            } else {
                if compare(to_node, cur_node, val) {
                    cur_strat = to;
                    changed = true;
                }
            }
        }

        if changed {
            strategy[i] = cur_strat;
            changes = true;
        }
    }

    changes
}