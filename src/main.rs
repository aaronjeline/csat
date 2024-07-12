mod dimacs;
mod dpllsolver;
mod eval;
mod set;
mod tests;
use std::io::Read;

fn main() {
    env_logger::init();
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let dimacs = dimacs::parse(&buf).unwrap();
    match dpllsolver::solve(&dimacs) {
        Some(model) => {
            println!("SAT");
            for (var, value) in model.iter() {
                println!("{}: {}", var, value);
            }
        }
        None => println!("UNSAT"),
    };
}
