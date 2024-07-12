use crate::dimacs::Dimacs;
use crate::dimacs::{Atom, Clause};
use log::{debug, info};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone, Debug)]
enum ControlFlow {
    Done(Assignment),
    Backtrack,
}

pub fn solve(dimacs: &Dimacs) -> Option<Assignment> {
    let mut kb = build_kb(&dimacs);

    match explore(&mut kb) {
        ControlFlow::Done(assignment) => Some(assignment),
        ControlFlow::Backtrack => None,
    }
}

fn explore(kb: &mut KnowledgeBase) -> ControlFlow {
    if !unit_propagation(kb) {
        // Branch is unsatisfiable
        return ControlFlow::Backtrack;
    }

    let next = choose_literal(kb);
    if let Some(ref x) = next {
        info!("Exploring: {x}")
    }

    match choose_literal(&kb) {
        // If we have no more unassigned literals, we have a model
        None => ControlFlow::Done(kb.assignment.clone()),
        Some(var) => {
            info!("Trying `true`");
            // First we explore the true branch
            kb.mutate(var, true);
            match explore(kb) {
                // If the true branch is satisfiable, we are done
                ControlFlow::Done(assignment) => {
                    return ControlFlow::Done(assignment);
                }
                // If the true branch is unsatisfiable, we backtrack
                // Exploring false may still be satisfiable
                ControlFlow::Backtrack => {
                    info!("Back tracked to {var}, trying `false` ");
                    // Then we explore the false branch
                    kb.mutate(var, false);
                    match explore(kb) {
                        // If the false branch is satisfiable, we are done
                        ControlFlow::Done(assignment) => return ControlFlow::Done(assignment),
                        // If the false branch is unsatisfiable, we backtrack
                        ControlFlow::Backtrack => {
                            info!("{var} is unsatisfiable, backtracking.");
                            kb.unassign(var);
                            return ControlFlow::Backtrack;
                        }
                    }
                }
            }
        }
    }
}

fn choose_literal(kb: &KnowledgeBase<'_>) -> Option<u128> {
    kb.assignment.unassigned.iter().next().copied()
}

fn build_kb<'a>(dimacs: &'a Dimacs) -> KnowledgeBase<'a> {
    let clauses = dimacs.clauses.as_slice();
    let assignment = Assignment::new(dimacs.vars);
    KnowledgeBase {
        clauses,
        assignment,
    }
}

// Probably want this to be a more efficient type later
type Set = HashSet<u128>;

struct KnowledgeBase<'a> {
    clauses: &'a [Clause],
    assignment: Assignment,
}

fn unit_propagation(kb: &mut KnowledgeBase<'_>) -> bool {
    info!("Unit Propagation");
    debug!("KB : {}", kb.assignment);
    loop {
        let mut ever_changed = false;
        let mut changed = false;
        debug!("KB clauses: {}", kb.clauses.len());
        for clause in kb.clauses.iter() {
            let state = state(clause, kb);
            debug!("Clause: {clause}, State: {state}");
            match state {
                ClauseState::Falsified => return false,
                ClauseState::Satisfied => (),
                ClauseState::Unit(var, val) => {
                    ever_changed = true;
                    changed = true;
                    kb.mutate(var, val);
                }
                ClauseState::Unresolved => (),
            }
        }
        if !changed {
            if ever_changed {
                info!("KB post unit prop: {}", kb.assignment);
            } else {
                info!("unit propagation caused no new assignments");
            }
            return true;
        }
    }
}

impl<'a> KnowledgeBase<'a> {
    pub fn satisfied(&self, a: Atom) -> Option<bool> {
        if let Some(value) = self.assignment.assignment.get(&a.var()) {
            match a {
                Atom::Pos(_) => Some(*value),
                Atom::Neg(_) => Some(!value),
            }
        } else {
            None
        }
    }

    pub fn mutate(&mut self, x: u128, v: bool) {
        self.assignment.assignment.insert(x, v);
        self.assignment.unassigned.remove(&x);
    }

    pub fn unassign(&mut self, x: u128) {
        self.assignment.assignment.remove(&x);
        self.assignment.unassigned.insert(x);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClauseState {
    Falsified,
    Satisfied,
    Unit(u128, bool),
    Unresolved,
}

// Display impl for clause state
impl std::fmt::Display for ClauseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClauseState::Falsified => write!(f, "Falsified"),
            ClauseState::Satisfied => write!(f, "Satisfied"),
            ClauseState::Unit(var, val) => write!(f, "Unit({}, {})", var, val),
            ClauseState::Unresolved => write!(f, "Unresolved"),
        }
    }
}

fn state(c: &Clause, kb: &KnowledgeBase) -> ClauseState {
    let mut unassigned = vec![];
    for atom in c.iter() {
        match kb.satisfied(*atom) {
            Some(true) => return ClauseState::Satisfied,
            Some(false) => (),
            None => unassigned.push((atom.var(), atom.to_satisfy())),
        }
    }
    match unassigned.as_slice() {
        [] => ClauseState::Falsified,
        [(var, val)] => ClauseState::Unit(*var, *val),
        _ => ClauseState::Unresolved,
    }
}

#[derive(Debug, Clone)]
pub struct Assignment {
    assignment: HashMap<u128, bool>,
    unassigned: Set,
}

// Display impl for assignment. Only pretty print the assigned variables
impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        let mut vars = self.assignment.iter().collect::<Vec<_>>();
        vars.sort_by_key(|(k, _)| *k);
        for (var, val) in vars {
            writeln!(f, "{}: {}", var, val)?;
        }
        Ok(())
    }
}

impl Assignment {
    pub fn new(num_vars: u128) -> Self {
        let mut unassigned = HashSet::new();
        for i in 1..=num_vars {
            unassigned.insert(i);
        }
        Self {
            assignment: HashMap::new(),
            unassigned,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (u128, bool)> + '_ {
        self.assignment.iter().map(|(&k, &v)| (k, v))
    }

    pub fn lookup(&self, x: u128) -> bool {
        *self.assignment.get(&x).unwrap()
    }
}
