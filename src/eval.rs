use crate::{
    dimacs::{Atom, Clause, Dimacs},
    dpllsolver::Assignment,
};

pub fn eval(dimacs: &Dimacs, env: &Assignment) -> bool {
    dimacs.clauses.iter().all(eval_clause(&env))
}

fn eval_clause(env: &'_ Assignment) -> impl Fn(&Clause) -> bool + '_ {
    move |clause| clause.iter().any(eval_atom(env))
}

fn eval_atom(env: &'_ Assignment) -> impl Fn(&Atom) -> bool + '_ {
    move |atom: &Atom| match atom {
        Atom::Pos(v) => env.lookup(*v),
        Atom::Neg(v) => !env.lookup(*v),
    }
}
