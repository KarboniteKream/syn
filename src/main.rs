use std::collections::{HashMap, HashSet};
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
    let grammar = parse(filename).expect("Unable to parse grammar");

    if let Err(message) = verify(&grammar) {
        eprintln!("Grammar '{}' is not valid: {}", filename, message);
        process::exit(1);
    }

    println!("OK!");
}

// TODO: Convert to token parser.
// TODO: Define start symbol in grammar file.
fn parse(filename: &String) -> Result<Grammar, io::Error> {
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

    let terminals: HashSet<&str> = productions
        .iter()
        .map(|production| production.splitn(2, "::=").next().unwrap().trim())
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
                        terminal: terminals.contains(name),
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
    let mut used_symbols: HashSet<&Symbol> = HashSet::from_iter(
        grammar
            .productions
            .values()
            .flat_map(|value| value.iter().flatten().collect::<Vec<&Symbol>>()),
    );

    let start_symbol = Symbol {
        name: "S".to_owned(),
        terminal: true,
    };

    used_symbols.insert(&start_symbol);

    for symbol in grammar.productions.keys() {
        if !used_symbols.contains(symbol) {
            return Err(format!("Symbol '{}' is unreachable", symbol.name));
        }
    }

    let mut completeness: HashMap<&Symbol, bool> = grammar
        .productions
        .iter()
        .map(|(symbol, productions)| {
            (
                symbol,
                productions
                    .iter()
                    .any(|production| production.iter().all(|symbol| !symbol.terminal)),
            )
        })
        .collect();

    loop {
        let changes: HashMap<&Symbol, bool> = completeness
            .iter()
            .filter(|(_, &complete)| !complete)
            .map(|(&symbol, _)| {
                let used_symbols: Vec<&Symbol> = grammar
                    .productions
                    .get(symbol)
                    .unwrap()
                    .iter()
                    .flat_map(|symbols| {
                        symbols
                            .iter()
                            .filter(|symbol| symbol.terminal)
                            .collect::<Vec<&Symbol>>()
                    })
                    .collect();

                (
                    symbol,
                    used_symbols
                        .iter()
                        .any(|symbol| *completeness.get(symbol).unwrap()),
                )
            })
            .filter(|(_, complete)| *complete)
            .collect();

        if changes.is_empty() {
            break;
        }

        completeness.extend(changes);
    }

    if let Some((&symbol, _)) = completeness.iter().find(|(_, &complete)| !complete) {
        return Err(format!("Symbol '{}' is not complete", symbol));
    }

    Ok(())
}
