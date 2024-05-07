use crate::solver::{Atom, Clause};
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::i128;
use nom::character::complete::u128;
use nom::combinator::map;
use nom::combinator::verify;
use nom::multi::many1;
use nom::multi::separated_list0;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DimacsHeader {
    vars: u128,
    clauses: u128,
}

pub struct Dimacs {
    pub vars: u128,
    pub clauses: Vec<Clause>,
}

pub fn dimacs_file(src: &str) -> Result<Dimacs, ParseError> {
    match dimacs(src) {
        Ok((_, d)) => Ok(d),
        Err(e) => Err(ParseError::Err(e.to_string())),
    }
}

#[derive(Debug)]
pub enum ParseError {
    Err(String),
}

fn dimacs(src: &str) -> IResult<&str, Dimacs> {
    let (rest, header) = header(src)?;
    let vars = header.vars;
    map(clauses(header), move |clauses| Dimacs { vars, clauses })(rest)
}

fn header(src: &str) -> IResult<&str, DimacsHeader> {
    map(
        tuple((
            tag("p"),
            whitespace,
            tag("cnf"),
            whitespace,
            u128,
            whitespace,
            u128,
            newline,
        )),
        |(_, _, _, _, vars, _, clauses, _)| DimacsHeader { vars, clauses },
    )(src)
}

fn clauses(header: DimacsHeader) -> impl Fn(&str) -> IResult<&str, Vec<Clause>> {
    println!("{:?}", header);
    move |src: &str| {
        verify(
            separated_list0(newline, clause(header.vars)),
            |clauses: &Vec<Clause>| {
                for clause in clauses {
                    println!("{:?}", clause);
                }
                true
            }, //clauses.len() == header.clauses as usize,
        )(src)
    }
}

fn clause(max: u128) -> impl Fn(&str) -> IResult<&str, Clause> {
    move |src: &str| separated_list0(whitespace, atom(max))(src)
}

fn atom(max: u128) -> impl Fn(&str) -> IResult<&str, Atom> {
    move |src: &str| {
        let (rest, i) = i128(src)?;
        let negated = i < 0;
        let id = i.abs() as u128;
        if id <= max {
            Ok((rest, Atom { negated, id }))
        } else {
            Err(nom::Err::Failure(nom::error::make_error(
                src,
                nom::error::ErrorKind::Fail,
            )))
        }
    }
}

fn newline(src: &str) -> IResult<&str, &str> {
    tag("\n")(src)
}

fn whitespace(src: &str) -> IResult<&str, ()> {
    map(take_while1(|c| c == ' '), |_| ())(src)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn headers() {
        let src = "p cnf 12 39\n";
        let (_, h) = header(src).unwrap();
        assert_eq!(
            h,
            DimacsHeader {
                vars: 12,
                clauses: 39
            }
        );
    }

    #[test]
    fn one_clause() {
        let src = "3 4 -1 5";
        let (_, clause) = clause(6)(src).unwrap();
        let expected = vec![Atom::pos(3), Atom::pos(4), Atom::neg(1), Atom::pos(5)];
        assert_eq!(expected, clause);
    }

    #[test]
    fn two_clauses() {
        let src = "3 4 -1 5\n1 2 3 -4\n";
        let header = DimacsHeader {
            vars: 5,
            clauses: 2,
        };
        let (_, clauses) = clauses(header)(src).unwrap();
        let expected = vec![Atom::pos(3), Atom::pos(4), Atom::neg(1), Atom::pos(5)];
        assert_eq!(&expected, &clauses[0]);
    }
}
