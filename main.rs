use std::io::{BufRead, BufReader, Write};

use crate::either::Either;
use crate::term::Term;

mod either;
mod parse;
mod term;

/// Parses exactly one term on the input string.
fn toplevel_term_parser(input: &str) -> Option<(Term, &str)> {
    let p = parse::and(Box::new(term_parser), parse::chars("$$".to_string()));

    match p(input) {
        Some(((term, _), rest)) => Some((term, rest)),
        None => None,
    }
}

/// Parses any lambda calculus term.
fn term_parser(input: &str) -> Option<(Term, &str)> {
    let p = parse::or(
        Box::new(parse::identifier),
        parse::or(Box::new(lambda_parser), Box::new(application_parser)),
    );

    match p(input) {
        Some((Either::Left(identifier), rest)) => Some((Term::Var(identifier), rest)),
        Some((Either::Right(Either::Left(lambda)), rest)) => Some((lambda, rest)),
        Some((Either::Right(Either::Right(app)), rest)) => Some((app, rest)),
        None => None,
    }
}

/// Parses a lambda term.
fn lambda_parser(input: &str) -> Option<(Term, &str)> {
    let p = parse::and(
        parse::chars("\\".to_string()),
        parse::and(
            Box::new(parse::identifier),
            parse::and(parse::chars(".".to_string()), Box::new(term_parser)),
        ),
    );
    if let Some(((_lambda, (identifier, (_dot, body))), rest)) = p(input) {
        Some((Term::Lambda(identifier, Box::new(body)), rest))
    } else {
        None
    }
}

/// Parses an application term.
fn application_parser(input: &str) -> Option<(Term, &str)> {
    let p = parse::and(
        parse::chars("(".to_string()),
        parse::and(
            Box::new(term_parser),
            parse::and(
                Box::new(term_parser),
                parse::chars(")".to_string()),
            ),
        ),
    );
    if let Some(((_p_open, (term1, (term2, _p_closed))), rest)) = p(input) {
        Some((Term::Application(Box::new(term1), Box::new(term2)), rest))
    } else {
        None
    }
}

/// Displays a the prompt `msg` to stdout, and then reads a line into `line`.
/// Returns the IO result of reading the line.
fn prompt<T: std::io::Read>(
    reader: &mut BufReader<T>,
    msg: &str,
    line: &mut String,
) -> std::io::Result<usize> {
    print!("{}", msg);
    std::io::stdout().flush()?;
    reader.read_line(line)
}

fn main() {
    let mut reader = BufReader::new(std::io::stdin());
    let mut line = String::new();
    let parser = toplevel_term_parser;

    while prompt(&mut reader, "> ", &mut line).is_ok_and(|bytes_read| bytes_read != 0) {
        match parser(&(line.to_string() + "$$")) {
            Some((term, _)) => println!("{}", term.normal_order_reduction()),
            None => eprintln!("Parse error."),
        }
        line.clear();
    }
}

// (λx.(x.x) λx.(x.x)) - How do we handle this?
// I understand that I am not supposed to try to 
// reduce it, but I don't know how to check when 
// this case occurs and prevent reduction.