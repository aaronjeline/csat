use log::debug;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;

// Probably want this to be a more efficient type later
type Set = HashSet<u128>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Answer {
    Sat(Model),
    Unsat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    contents: Vec<bool>,
}

impl Model {
    pub fn get(&self, i: u128) -> bool {
        self.contents[i as usize - 1]
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, val) in self.contents.iter().enumerate() {
            writeln!(f, "v{} : {}", i + 1, val)?;
        }
        Ok(())
    }
}

impl From<SolverFrame> for Model {
    fn from(value: SolverFrame) -> Self {
        if value.unassigned.is_empty() {
            let mut contents = vec![];
            contents.reserve(value.vars);

            for i in 1..=value.vars {
                contents.push(value.tru.contains(&(i as u128)));
            }
            Self { contents }
        } else {
            panic!("tried to make model out of incomplete frame!");
        }
    }
}

#[derive(Debug, Clone)]
struct SolverState {
    shared_graph: Graph,
    current_path: Vec<NodeId>,
}

impl Default for SolverState {
    fn default() -> Self {
        let mut shared_graph = Graph::default();
        let root_id = shared_graph.alloc_node(Node::new_root());
        let current_path = vec![root_id];
        Self {
            shared_graph,
            current_path,
        }
    }
}

impl SolverState {
    pub fn analyze_conflict(&mut self, frame: SolverFrame, clauses: &[Clause]) -> Option<Answer> {
        if clauses
            .iter()
            .any(|clause| state_of_clause(clause, &frame) == ClauseState::Conflicting)
        {
            self.backtrack()
        } else {
            None
        }
    }

    fn backtrack(&mut self) -> Option<Answer> {
        debug!("Backtracking!");
        loop {
            // Pop the most recent path addition, if we're out, UNSAT
            if self.current_path.pop().is_none() {
                return Some(Answer::Unsat);
            }

            if self.current_path.is_empty() {
                return Some(Answer::Unsat);
            }

            // Check if we have any paths to go, otherwise backtrack more!
            if self
                .shared_graph
                .neighbors(*self.current_path.last().unwrap())
                .iter()
                .filter(|id| !self.shared_graph.get(**id).borrow().seen)
                .next()
                .is_some()
            {
                break;
            }
        }
        return None;
    }

    pub fn make_decision(&mut self, frame: &SolverFrame) -> Option<NodeId> {
        let current_node = *self.current_path.last().unwrap();
        self.shared_graph.expand_frontier(current_node, frame);
        let first_unseen_node_id = *self
            .shared_graph
            .neighbors(current_node)
            .iter()
            .filter(|id| !self.shared_graph.get(**id).borrow().seen)
            .next()?;
        let node = self.shared_graph.get(first_unseen_node_id);
        debug!("Making decision: {:?}", node);
        node.borrow_mut().mark();
        self.current_path.push(first_unseen_node_id);
        Some(first_unseen_node_id)
    }
    pub fn compute_frame(&self, size: usize) -> SolverFrame {
        let mut frame = SolverFrame::new(size);

        for id in self.current_path.iter() {
            let node = self.shared_graph.get(*id);
            if let Some(var) = node.borrow().var {
                if node.borrow().direction {
                    frame.set_true(var);
                } else {
                    frame.set_false(var);
                }
            }
        }

        frame
    }
}

#[derive(Debug, Clone, Default)]
struct Graph {
    nodes: Vec<RefCell<Node>>,
    neighbors: HashMap<NodeId, Vec<NodeId>>,
}

impl Graph {
    pub fn alloc_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(RefCell::new(node));
        NodeId(self.nodes.len() - 1)
    }

    pub fn neighbors(&self, id: NodeId) -> &[NodeId] {
        self.neighbors.get(&id).unwrap()
    }

    pub fn expand_frontier(&mut self, id: NodeId, frame: &SolverFrame) {
        if self.neighbors.contains_key(&id) {
            return;
        }
        let mut neighbors = vec![];
        for var in frame.unassigned.iter() {
            for direction in [true, false] {
                let node = Node::new_unexplored(*var, direction);
                let id = self.alloc_node(node);
                neighbors.push(id);
            }
        }
        self.neighbors.insert(id, neighbors);
    }

    pub fn get(&self, id: NodeId) -> &RefCell<Node> {
        &self.nodes[id.0]
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
struct NodeId(usize);

#[derive(Debug, Clone)]
struct Node {
    seen: bool,
    var: Option<u128>,
    direction: bool,
}

impl Node {
    pub fn mark(&mut self) {
        self.seen = true;
    }

    pub fn new_unexplored(var: u128, direction: bool) -> Self {
        Self {
            seen: false,
            var: Some(var),
            direction,
        }
    }

    pub fn new_root() -> Self {
        Self {
            seen: true,
            var: None,
            direction: false,
        }
    }
}

pub fn sovler(vars: usize, b: Vec<Clause>) -> Answer {
    let mut state = SolverState::default();
    loop {
        let current_frame = state.compute_frame(vars);
        match state.make_decision(&current_frame) {
            None => return Answer::Sat(current_frame.into()),
            Some(_) => {
                let new_frame = state.compute_frame(vars);
                if let Some(answer) = state.analyze_conflict(new_frame, b.as_slice()) {
                    return answer;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SolverFrame {
    vars: usize,
    tru: Set,
    fls: Set,
    unassigned: Set,
}

impl SolverFrame {
    pub fn new(vars: usize) -> Self {
        let mut unassigned = Set::new();
        for i in 1..=vars {
            unassigned.insert(i as u128);
        }
        Self {
            unassigned,
            vars,
            tru: Set::default(),
            fls: Set::default(),
        }
    }

    pub fn set_true(&mut self, var: u128) {
        self.assign(var);
        self.tru.insert(var);
    }

    pub fn set_false(&mut self, var: u128) {
        self.assign(var);
        self.fls.insert(var);
    }

    fn assign(&mut self, var: u128) {
        assert!(
            self.unassigned.remove(&var),
            "Can't assign an already assigne var"
        );
    }
}

pub type Clause = Vec<Atom>;

fn state_of_clause(clause: &Clause, f: &SolverFrame) -> ClauseState {
    if clause.iter().any(|a| a.satisfied(f)) {
        ClauseState::Satisfied
    } else {
        let assigned = clause.iter().filter(|a| a.assigned(f)).count();
        if assigned == clause.len() {
            ClauseState::Conflicting
        } else if assigned == clause.len() - 1 {
            ClauseState::Unit
        } else {
            ClauseState::Unresolved
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom {
    pub id: u128,
    pub negated: bool,
}

impl Atom {
    pub fn pos(id: u128) -> Self {
        Self { id, negated: false }
    }

    pub fn neg(id: u128) -> Self {
        Self { id, negated: true }
    }

    pub fn assigned(&self, frame: &SolverFrame) -> bool {
        !frame.unassigned.contains(&self.id)
    }

    pub fn satisfied(&self, frame: &SolverFrame) -> bool {
        if self.negated {
            frame.fls.contains(&self.id)
        } else {
            frame.tru.contains(&self.id)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseState {
    Satisfied,
    Conflicting,
    Unit,
    Unresolved,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn state() {
        let mut state = SolverFrame::new(5);
        state.set_true(1);
        state.set_false(2);
        state.set_true(4);

        let clause = vec![Atom::pos(1), Atom::pos(3), Atom::neg(4)];
        assert_eq!(state_of_clause(&clause, &state), ClauseState::Satisfied);

        let clause = vec![Atom::neg(1), Atom::pos(2)];
        assert_eq!(state_of_clause(&clause, &state), ClauseState::Conflicting);

        let clause = vec![Atom::neg(1), Atom::neg(4), Atom::pos(3)];
        assert_eq!(state_of_clause(&clause, &state), ClauseState::Unit);

        let clause = vec![Atom::neg(1), Atom::pos(3), Atom::pos(5)];
        assert_eq!(state_of_clause(&clause, &state), ClauseState::Unresolved);
    }
}
