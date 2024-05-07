mod dimacs;
mod set;
mod solver;
use std::io::Read;

fn main() {
    env_logger::init();
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let dimacs = dimacs::dimacs_file(&buf).unwrap();
    let clauses = dimacs
        .clauses
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect();
    match solver::sovler(dimacs.vars as usize, clauses) {
        solver::Answer::Sat(model) => {
            println!("SAT");
            println!("{model}");
        }
        solver::Answer::Unsat => println!("UNSAT"),
    };
}
