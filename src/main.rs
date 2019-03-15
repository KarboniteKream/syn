use std::collections::HashSet;
use std::env;
use std::process;

mod grammar;
mod parser;
mod rule;
mod symbol;

use crate::parser::parse;
use crate::symbol::Symbol;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Please specify a filename.");
        process::exit(1);
    }

    let filename = &args[1];
    let grammar = match parse(filename) {
        Ok(grammar) => grammar,
        Err(error) => {
            eprintln!("Unable to parse file '{}': {}", filename, error);
            process::exit(1);
        }
    };

    if let Err(error) = grammar.verify() {
        eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
        process::exit(1);
    }

    let symbol = &grammar.start_symbol;
    let first: HashSet<Symbol> = grammar.first(symbol);

    println!("{:?}", grammar);
    println!("FIRST({}) => {:?}", symbol, first);
}
