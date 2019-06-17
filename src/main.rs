use std::fs;
use std::path::Path;
use std::process;

mod automaton;
mod grammar;
mod parser;
mod rule;
mod symbol;
mod util;

use automaton::Automaton;

fn main() {
    let args = util::parse_args();

    let filename = args.value_of("filename").unwrap();
    let grammar = match parser::parse_file(Path::new(filename)) {
        Ok(grammar) => grammar,
        Err(error) => {
            eprintln!("File '{}' cannot be parsed: {}", filename, error);
            process::exit(1);
        }
    };

    if let Err(error) = grammar.verify() {
        eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
        process::exit(1);
    }

    println!("GRAMMAR\n{}", grammar);
    let automaton = Automaton::new(grammar);
    println!("\nAUTOMATON\n{}", automaton);
    let grammar = &automaton.grammar;

    let action_table = match automaton.action_table() {
        Ok(action_table) => util::to_sorted_vec(action_table),
        Err(error) => {
            eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
            process::exit(1);
        }
    };
    println!("\nACTION");
    for ((state, symbol), action) in action_table {
        println!("{}, {} → {}", state, grammar.symbol(symbol), action);
    }

    let goto_table = util::to_sorted_vec(automaton.goto_table());
    println!("\nGOTO");
    for ((from, symbol), to) in goto_table {
        println!("{}, {} → {}", from, grammar.symbol(symbol), to);
    }

    let unique_table = util::to_sorted_vec(automaton.unique_table());
    println!("\nUNIQUE");
    for ((state, symbol), item) in unique_table {
        println!("{}, {} → {}", state, grammar.symbol(symbol), item);
    }

    let parse_table = util::to_sorted_vec(automaton.parse_table());
    println!("\nPARSE");
    for ((from_item, state), to_item) in parse_table {
        println!("{}, {} → {}", from_item, state, to_item);
    }

    if let Some(output) = args.value_of("output") {
        let contents: String = automaton.to_dot();

        if let Err(error) = fs::write(Path::new(output), contents) {
            eprintln!("Unable to save to file '{}': {}", output, error);
            process::exit(1);
        }
    }
}
