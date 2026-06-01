use std::cmp::max;

pub struct ParityGame {
    nodes: usize,
    successors: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
    priorities: Vec<usize>,
    owners: Vec<usize>,
    labels: Vec<Option<String>>,
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
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.successors[from].push(to);
        self.predecessors[to].push(from);
    }

    fn set_priority(&mut self, node: usize, priority: usize) {
        self.priorities[node] = priority;
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
        self.priorities.iter().enumerate().filter_map(|(node, &p)| if p == priority { Some(node) } else { None }).collect()
    }

    pub fn get_nodes(&self) -> Vec<usize> {
        (0..self.nodes).collect()
    }

    pub fn get_nodes_with_prio_eval<F>(&self, eval: F) -> Vec<usize>
    where
        F: Fn(usize) -> bool,
    {
        self.priorities.iter().enumerate().filter_map(|(node, &p)| if eval(p) { Some(node) } else { None }).collect()
    }

    pub fn get_max_priority(&self) -> usize {
        *self.priorities.iter().max().unwrap_or(&0)
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes
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
        let mut visited = vec![false; self.nodes];
        let mut order = Vec::new();

        for v in 0..self.nodes {
            if in_region[v] && !visited[v] {
                self.dfs_order(in_region, sigma, player, v, &mut visited, &mut order);
            }
        }

        let mut visited_rev = vec![false; self.nodes];
        let mut sccs = Vec::new();

        while let Some(v) = order.pop() {
            if !in_region[v] || visited_rev[v] {
                continue;
            }

            let mut component = Vec::new();
            self.dfs_collect(in_region, sigma, player, v, &mut visited_rev, &mut component);
            sccs.push(component);
        }

        sccs
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

    fn dfs_order(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
        start: usize,
        visited: &mut [bool],
        order: &mut Vec<usize>,
    ) {
        let mut stack = vec![(start, 0usize)];
        visited[start] = true;

        while let Some((node, idx)) = stack.pop() {
            let next_len = if self.get_owner(node) == player {
                if sigma[node].is_some_and(|succ| in_region[succ]) {
                    1
                } else {
                    0
                }
            } else {
                self.get_successors(node)
                    .iter()
                    .filter(|&&succ| in_region[succ])
                    .count()
            };

            if idx < next_len {
                let next = self
                    .filtered_successor_at(in_region, sigma, player, node, idx)
                    .expect("successor index should exist");
                stack.push((node, idx + 1));
                if !visited[next] {
                    visited[next] = true;
                    stack.push((next, 0));
                }
            } else {
                order.push(node);
            }
        }
    }

    fn dfs_collect(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
        start: usize,
        visited: &mut [bool],
        component: &mut Vec<usize>,
    ) {
        let mut stack = vec![start];
        visited[start] = true;

        while let Some(node) = stack.pop() {
            component.push(node);
            for &pred in self.get_predecessors(node) {
                if !in_region[pred] || visited[pred] {
                    continue;
                }
                if !self.edge_exists(in_region, sigma, player, pred, node) {
                    continue;
                }
                visited[pred] = true;
                stack.push(pred);
            }
        }
    }

    fn filtered_successor_at(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
        node: usize,
        idx: usize,
    ) -> Option<usize> {
        if self.get_owner(node) == player {
            sigma[node].filter(|&succ| in_region[succ])
        } else {
            self.get_successors(node)
                .iter()
                .copied()
                .filter(|&succ| in_region[succ])
                .nth(idx)
        }
    }

    fn edge_exists(
        &self,
        in_region: &[bool],
        sigma: &[Option<usize>],
        player: usize,
        from: usize,
        to: usize,
    ) -> bool {
        if !in_region[from] || !in_region[to] {
            return false;
        }

        if self.get_owner(from) == player {
            sigma[from] == Some(to)
        } else {
            self.get_successors(from).iter().any(|&succ| succ == to)
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

    pub fn build(self) -> ParityGame {
        let mut game = ParityGame::new(self.nodes);
        for (from, to) in self.edges {
            game.add_edge(from, to);
        }
        for (node, priority) in self.priorities {
            game.set_priority(node, priority);
        }
        for (node, label) in self.labels {
            game.set_label(node, label);
        }
        for (node, owner) in self.owners {
            game.set_owner(node, owner);
        }
        game
    }
}
