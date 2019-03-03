use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::process;

type Grammar = HashMap<String, Vec<String>>;

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
// TODO: Define start symbol in file, default to S.
fn read(filename: &String) -> Result<Grammar, io::Error> {
    let mut grammar = Grammar::new();

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
        let head = split.next().unwrap().trim().to_owned();

        let mut body = split
            .next()
            .unwrap()
            .split("::|")
            .map(|e| {
                e.trim()
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .to_owned()
            })
            .collect();

        grammar
            .entry(head)
            .or_insert_with(Vec::new)
            .append(&mut body);
    }

    Ok(grammar)
}

// TODO: Detect left recursion.
fn verify(grammar: &Grammar) -> Result<(), String> {
    let mut used_symbols: HashSet<&str> = HashSet::from_iter(grammar.values().flat_map(|value| {
        value
            .iter()
            .flat_map(|e| e.split_whitespace())
            .collect::<Vec<&str>>()
    }));

    used_symbols.insert("S");

    for symbol in grammar.keys() {
        if !used_symbols.contains(symbol.as_str()) {
            return Err(format!("Symbol '{}' is unreachable", symbol));
        }
    }

    Ok(())
}
