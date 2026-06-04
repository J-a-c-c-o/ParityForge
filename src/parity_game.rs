use std::cmp::max;

pub struct ParityGame {
    nodes: usize,
    successors: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
    priorities: Vec<usize>,
    owners: Vec<usize>,
    labels: Vec<Option<String>>,
    max_priority: usize,
}

struct TarjanFrame {
    node: usize,
    neighbors: Vec<usize>,
    next_neighbor: usize,
}

#[allow(dead_code)]
impl ParityGame {
    pub fn new(nodes: usize) -> Self {
        ParityGame {
            nodes,
            successors: vec![Vec::new(); nodes],
            predecessors: vec![Vec::new(); nodes],
            priorities: vec![0; nodes],
            owners: vec![0; nodes],
            labels: vec![None; nodes],
            max_priority: 0,
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.successors[from].push(to);
        self.predecessors[to].push(from);
    }

    fn set_priority(&mut self, node: usize, priority: usize) {
        self.priorities[node] = priority;
        self.max_priority = max(self.max_priority, priority);
    }

    fn set_owner(&mut self, node: usize, owner: usize) {
        self.owners[node] = owner;
    }

    fn set_label(&mut self, node: usize, label: String) {
        self.labels[node] = Some(label);
    }

    pub fn get_edges(&self, node: usize) -> &[usize] {
        &self.successors[node]
    }

    pub fn get_successors(&self, node: usize) -> &[usize] {
        &self.successors[node]
    }

    pub fn get_predecessors(&self, node: usize) -> &[usize] {
        &self.predecessors[node]
    }

    pub fn get_priority(&self, node: usize) -> usize {
        self.priorities[node]
    }

    pub fn get_priorities(&self) -> &[usize] {
        &self.priorities
    }

    pub fn get_owner(&self, node: usize) -> usize {
        self.owners[node]
    }

    pub fn get_label(&self, node: usize) -> Option<&String> {
        self.labels[node].as_ref()
    }

    pub fn get_nodes_with_priority(&self, priority: usize) -> Vec<usize> {
        self.priorities
            .iter()
            .enumerate()
            .filter_map(|(node, &p)| if p == priority { Some(node) } else { None })
            .collect()
    }

    pub fn get_nodes(&self) -> Vec<usize> {
        (0..self.nodes).collect()
    }

    pub fn get_nodes_with_prio_eval<F>(&self, eval: F) -> Vec<usize>
    where
        F: Fn(usize) -> bool,
    {
        self.priorities
            .iter()
            .enumerate()
            .filter_map(|(node, &p)| if eval(p) { Some(node) } else { None })
            .collect()
    }

    pub fn get_max_priority(&self) -> usize {
        self.max_priority
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes
    }

    pub fn remove_bad_self_loops(&mut self) {
        for node in 0..self.nodes {
            if self.successors[node].contains(&node)
                && ((self.owners[node] == 1 && self.priorities[node] % 2 == 0)
                    || (self.owners[node] == 0 && self.priorities[node] % 2 == 1))
                && self.successors[node].len() > 1
            {
                self.successors[node].retain(|&succ| succ != node);
                self.predecessors[node].retain(|&pred| pred != node);
            }
        }
    }
}

impl Clone for ParityGame {
    fn clone(&self) -> Self {
        ParityGame {
            nodes: self.nodes,
            successors: self.successors.clone(),
            priorities: self.priorities.clone(),
            owners: self.owners.clone(),
            labels: self.labels.clone(),
            predecessors: self.predecessors.clone(),
            max_priority: self.max_priority,
        }
    }
}

impl ParityGame {
    pub fn sccs(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
    ) -> Vec<Vec<usize>> {
        self.tarjan_sccs(in_region, sigma, player)
    }

    pub fn bottom_sccs(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
    ) -> Vec<Vec<usize>> {
        self.sccs(in_region, sigma, player)
            .into_iter()
            .filter(|scc| self.is_nontrivial_scc(scc, in_region, sigma, player))
            .filter(|scc| self.is_bottom_scc(scc, in_region, sigma, player))
            .collect()
    }

    fn tarjan_sccs(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
    ) -> Vec<Vec<usize>> {
        let mut index = vec![None; self.nodes];
        let mut lowlink = vec![0; self.nodes];
        let mut on_stack = vec![false; self.nodes];
        let mut active_stack = Vec::new();
        let mut call_stack = Vec::new();
        let mut next_index = 0;
        let mut sccs = Vec::new();

        for start in 0..self.nodes {
            if !in_region[start] || index[start].is_some() {
                continue;
            }

            index[start] = Some(next_index);
            lowlink[start] = next_index;
            next_index += 1;
            active_stack.push(start);
            on_stack[start] = true;
            call_stack.push(TarjanFrame {
                node: start,
                neighbors: self.filtered_successors(in_region, sigma, player, start),
                next_neighbor: 0,
            });

            while let Some(frame) = call_stack.last_mut() {
                let node = frame.node;

                if frame.next_neighbor < frame.neighbors.len() {
                    let next = frame.neighbors[frame.next_neighbor];
                    frame.next_neighbor += 1;

                    if index[next].is_none() {
                        index[next] = Some(next_index);
                        lowlink[next] = next_index;
                        next_index += 1;
                        active_stack.push(next);
                        on_stack[next] = true;
                        call_stack.push(TarjanFrame {
                            node: next,
                            neighbors: self.filtered_successors(in_region, sigma, player, next),
                            next_neighbor: 0,
                        });
                    } else if on_stack[next] {
                        lowlink[node] = std::cmp::min(
                            lowlink[node],
                            index[next].expect("visited node should have an index"),
                        );
                    }
                } else {
                    let node_index = index[node].expect("active node should have an index");
                    if lowlink[node] == node_index {
                        let mut component = Vec::new();

                        loop {
                            let member = active_stack
                                .pop()
                                .expect("Tarjan active stack should not be empty");
                            on_stack[member] = false;
                            component.push(member);
                            if member == node {
                                break;
                            }
                        }

                        sccs.push(component);
                    }

                    call_stack.pop();

                    if let Some(parent) = call_stack.last() {
                        lowlink[parent.node] = std::cmp::min(lowlink[parent.node], lowlink[node]);
                    }
                }
            }
        }

        sccs
    }

    fn filtered_successors(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
        node: usize,
    ) -> Vec<usize> {
        if self.get_owner(node) == player {
            match sigma[node] {
                Some(succ) if in_region[succ] => vec![succ],
                _ => Vec::new(),
            }
        } else {
            self.get_successors(node)
                .iter()
                .copied()
                .filter(|&succ| in_region[succ])
                .collect()
        }
    }

    fn is_bottom_scc(
        &self,
        scc: &[usize],
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
    ) -> bool {
        let mut in_scc = vec![false; self.nodes];
        for &v in scc {
            in_scc[v] = true;
        }

        for &v in scc {
            for succ in self.successors_from_strategy(in_region, sigma, player, v) {
                if !in_scc[succ] {
                    return false;
                }
            }
        }

        true
    }

    fn is_nontrivial_scc(
        &self,
        scc: &[usize],
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
    ) -> bool {
        if scc.len() > 1 {
            return true;
        }

        if let Some(&v) = scc.first() {
            if self.get_owner(v) == player {
                return sigma[v] == Some(v);
            }

            return self
                .get_successors(v)
                .iter()
                .any(|&succ| succ == v && in_region[succ]);
        }

        false
    }

    fn successors_from_strategy<'a>(
        &'a self,
        in_region: &'a [bool],
        sigma: &'a [Option<usize>],
        player: usize,
        node: usize,
    ) -> Box<dyn Iterator<Item = usize> + 'a> {
        if self.get_owner(node) == player {
            match sigma[node] {
                Some(succ) if in_region[succ] => Box::new(std::iter::once(succ)),
                _ => Box::new(std::iter::empty()),
            }
        } else {
            Box::new(
                self.get_successors(node)
                    .iter()
                    .copied()
                    .filter(move |&succ| in_region[succ]),
            )
        }
    }
}

pub struct ParityGameBuilder {
    nodes: usize,
    edges: Vec<(usize, usize)>,
    priorities: Vec<(usize, usize)>,
    owners: Vec<(usize, usize)>,
    labels: Vec<(usize, String)>,
}

impl ParityGameBuilder {
    pub fn new() -> Self {
        ParityGameBuilder {
            nodes: 0,
            edges: Vec::new(),
            priorities: Vec::new(),
            owners: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) -> &mut Self {
        self.edges.push((from, to));
        self.nodes = max(self.nodes, from + 1);
        self.nodes = max(self.nodes, to + 1);
        self
    }

    pub fn set_priority(&mut self, node: usize, priority: usize) -> &mut Self {
        self.priorities.push((node, priority));
        self.nodes = max(self.nodes, node + 1);
        self
    }

    pub fn set_owner(&mut self, node: usize, owner: usize) -> &mut Self {
        self.owners.push((node, owner));
        self.nodes = max(self.nodes, node + 1);
        self
    }

    pub fn set_label(&mut self, node: usize, label: String) -> &mut Self {
        self.labels.push((node, label));
        self.nodes = max(self.nodes, node + 1);
        self
    }

    pub fn random_game(
        &mut self,
        size: usize,
        max_edges: usize,
        max_priority: usize,
        seed: Option<u64>,
    ) -> &mut Self {
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng, rng};
        use std::collections::HashSet;

        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => {
                let mut trng = rng();
                StdRng::from_rng(&mut trng)
            }
        };

        for node in 0..size {
            let requested = rng.random_range(1..=max_edges);
            let num_edges = std::cmp::min(requested, size);

            let mut succ_set: HashSet<usize> = HashSet::with_capacity(num_edges);
            while succ_set.len() < num_edges {
                let to = rng.random_range(0..size);
                succ_set.insert(to);
            }

            for &to in &succ_set {
                self.add_edge(node, to);
            }

            let priority = rng.random_range(0..=max_priority);
            self.set_priority(node, priority);

            let owner = if rng.random_range(0..100) < 50 { 0 } else { 1 };
            self.set_owner(node, owner);
        }

        let mut in_degree = vec![0usize; size];
        for (_from, to) in &self.edges {
            if *to < size {
                in_degree[*to] += 1;
            }
        }

        for target in 0..size {
            if in_degree[target] == 0 {
                let mut src = rng.random_range(0..size);
                if size > 1 {
                    while src == target {
                        src = rng.random_range(0..size);
                    }
                }
                self.add_edge(src, target);
            }
        }

        self
    }

    pub fn build(&self) -> ParityGame {
        let mut game = ParityGame::new(self.nodes);

        for (node, priority) in &self.priorities {
            game.set_priority(*node, *priority);
        }
        for (node, label) in &self.labels {
            game.set_label(*node, label.clone());
        }
        for (node, owner) in &self.owners {
            game.set_owner(*node, *owner);
        }

        for (from, to) in &self.edges {
            game.add_edge(*from, *to);
        }

        game.remove_bad_self_loops();

        game
    }
}
