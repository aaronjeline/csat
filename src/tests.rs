#![cfg(test)]
use proptest::char::CharValueTree;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::proptest;
use proptest::test_runner::Config;
use proptest::{prop_oneof, strategy::Strategy};

use crate::dimacs::{Atom, Clause, Dimacs};
use crate::eval;

#[cfg(test)]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]


    #[test]
    fn try_solve(dimacs in arbitrary_dimacs()) {
        use crate::dpllsolver::solve;
        if let Some(solution) = solve(&dimacs) {
            assert!(eval::eval(&dimacs, &solution));
        }
    }
}

fn arbitrary_dimacs() -> impl Strategy<Value = Dimacs> {
    arbitrary_clauses(10).prop_map(|clauses| Dimacs { vars: 10, clauses })
}

fn arbitrary_clauses(max_vars: u128) -> impl Strategy<Value = Vec<Clause>> {
    vec(arbitrary_clause(max_vars), 1..=20)
}

fn arbitrary_clause(max_var: u128) -> impl Strategy<Value = Clause> {
    vec(arbitrary_atom(max_var), 1..=20).prop_map(Clause)
}

fn arbitrary_atom(max_var: u128) -> impl Strategy<Value = Atom> {
    prop_oneof![
        (1..=max_var).prop_map(Atom::Pos),
        (1..=max_var).prop_map(Atom::Neg),
    ]
}
