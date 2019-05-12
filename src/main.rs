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
use util::to_sorted_vec;

fn main() {
    let args = util::parse_args();

    let filename = args.value_of("filename").unwrap();
    let grammar = match parser::parse_file(Path::new(filename)) {
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

    println!("Grammar\n{}", grammar);

    let automaton = Automaton::new(&grammar);
    println!("\nAutomaton\n{}", automaton);

    println!("\nACTION");
    let action_table = to_sorted_vec(&automaton.action_table());
    for ((state, symbol), action) in action_table {
        println!("{}, {} → {}", state, symbol, action);
    }

    println!("\nGOTO");
    let goto_table = to_sorted_vec(&automaton.goto_table());
    for ((from, symbol), to) in goto_table {
        println!("{}, {} → {}", from, symbol, to);
    }

    println!("\nUNIQUE");
    let unique_table = to_sorted_vec(&automaton.unique_table(&grammar));
    for ((state, symbol), rule_id) in unique_table {
        println!("{}, {} → {}", state, symbol, rule_id);
    }

    if let Some(output) = args.value_of("output") {
        let contents: String = automaton.to_dot();

        if let Err(error) = fs::write(Path::new(output), contents) {
            eprintln!("Unable to save to file '{}': {}", output, error);
            process::exit(1);
        }
    }
}
