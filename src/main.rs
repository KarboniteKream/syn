use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::process;

mod grammar;
mod production;
mod symbol;

use crate::grammar::Grammar;
use crate::production::Production;
use crate::symbol::{Symbol, SymbolType};

const START_SYMBOL: &str = "S";

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
        eprintln!("Grammar '{}' is not valid: {}", filename, error);
        process::exit(1);
    }

    let symbol = &grammar.start_symbol;
    let first: HashSet<Symbol> = grammar.first(symbol);
    println!("FIRST({}) => {:?}", symbol, first);
}

// TODO: Convert to token parser.
// TODO: Define start symbol in grammar file.
fn parse(filename: &String) -> Result<Grammar, io::Error> {
    let mut grammar = Grammar::new(START_SYMBOL);

    let contents = fs::read_to_string(filename)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
        .collect::<Vec<&str>>()
        .join("");

    let productions: Vec<&str> = contents
        .split("::;")
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    let nonterminals: HashSet<&str> = productions
        .iter()
        .map(|production| production.splitn(2, "::=").next().unwrap().trim())
        .collect();

    for production in &productions {
        let mut split = production.splitn(2, "::=");
        let head = Symbol::new(split.next().unwrap().trim(), SymbolType::NonTerminal);

        let mut body = split
            .next()
            .unwrap()
            .split("::|")
            .map(|case| {
                Production::new(
                    case.trim()
                        .split_whitespace()
                        .map(|name| {
                            Symbol::new(
                                name,
                                if nonterminals.contains(name) {
                                    SymbolType::NonTerminal
                                } else {
                                    SymbolType::Terminal
                                },
                            )
                        })
                        .collect(),
                )
            })
            .collect();

        grammar
            .productions
            .entry(head)
            .or_insert_with(Vec::new)
            .append(&mut body);
    }

    Ok(grammar)
}
