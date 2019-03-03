use std::collections::{HashMap, HashSet};
use std::convert::identity;
use std::env;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::process;

mod grammar;
mod symbol;

use crate::grammar::Grammar;
use crate::symbol::Symbol;

const START_SYMBOL: &str = "S";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Please specify a filename.");
        process::exit(1);
    }

    let filename = &args[1];
    let grammar = read(filename).expect("Unable to parse grammar");

    if let Err(message) = verify(&grammar) {
        eprintln!("Grammar '{}' is not valid: {}", filename, message);
        process::exit(1);
    }

    println!("OK!");
}

// TODO: Convert to token parser.
// TODO: Define start symbol in grammar file.
fn read(filename: &String) -> Result<Grammar, io::Error> {
    let mut grammar = Grammar::new(START_SYMBOL.to_owned());

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

    for production in &productions {
        let mut split = production.splitn(2, "::=");

        let head = Symbol {
            name: split.next().unwrap().trim().to_owned(),
            terminal: true,
        };

        let mut body = split
            .next()
            .unwrap()
            .split("::|")
            .map(|case| {
                case.trim()
                    .split_whitespace()
                    .map(|name| Symbol {
                        name: name.to_owned(),
                        terminal: false,
                    })
                    .collect()
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

// TODO: Detect left recursion.
fn verify(grammar: &Grammar) -> Result<(), String> {
    let mut used_symbols: HashSet<Symbol> = HashSet::from_iter(
        grammar
            .productions
            .values()
            .flat_map(|value| value.iter().flat_map(identity).cloned().collect::<Vec<Symbol>>()),
    );

    used_symbols.insert(Symbol {
        name: "S".to_owned(),
        terminal: true,
    });

    for symbol in grammar.productions.keys() {
        if !used_symbols.contains(symbol) {
            return Err(format!("Symbol '{}' is unreachable", symbol.name));
        }
    }

    Ok(())
}
