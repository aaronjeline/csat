use lalrpop_util::lalrpop_mod;
use proptest::arbitrary::Arbitrary;

lalrpop_mod!(pub parser);

pub fn parse(src: &str) -> Result<Dimacs, String> {
    parser::DimacsParser::new()
        .parse(src)
        .map_err(|e| format!("{:?}", e))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DimacsHeader {
    vars: u128,
    clauses: u128,
}

#[derive(Debug)]
pub struct Dimacs {
    pub vars: u128,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause(pub Vec<Atom>);

impl Clause {
    pub fn iter(&self) -> impl Iterator<Item = &Atom> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for atom in self.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", atom)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    Pos(u128),
    Neg(u128),
}

impl Atom {
    pub fn var(self) -> u128 {
        match self {
            Atom::Pos(v) => v,
            Atom::Neg(v) => v,
        }
    }

    pub fn to_satisfy(self) -> bool {
        match self {
            Atom::Pos(_) => true,
            Atom::Neg(_) => false,
        }
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Pos(v) => write!(f, "{}", v),
            Atom::Neg(v) => write!(f, "-{}", v),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Err(String),
}
