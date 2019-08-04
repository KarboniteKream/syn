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
use util::AsString;

fn main() {
    let args = util::parse_args();

    let filename = args.value_of("grammar").unwrap();
    let grammar = match grammar::read_file(Path::new(filename)) {
        Ok(grammar) => grammar,
        Err(error) => {
            eprintln!("Grammar file '{}' cannot be parsed: {}", filename, error);
            process::exit(1);
        }
    };

    if let Err(error) = grammar.verify() {
        eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
        process::exit(1);
    }

    println!("GRAMMAR\n{}", grammar);
    let automaton = Automaton::new(&grammar);
    let grammar = &automaton.grammar;

    let data = match automaton.data() {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
            process::exit(1);
        }
    };

    println!("\n{}", data.string(grammar));

    if let Some(output) = args.value_of("output") {
        let contents: String = automaton.to_dot();

        if let Err(error) = fs::write(Path::new(output), contents) {
            eprintln!("Unable to save to file '{}': {}", output, error);
            process::exit(1);
        }
    }

    let filename = args.value_of("input").unwrap();
    let rules = match parser::parse_file(Path::new(filename), &grammar, &data) {
        Ok(rules) => rules,
        Err(error) => {
            eprintln!("Input file '{}' cannot be parsed: {}", filename, error);
            process::exit(1);
        }
    };

    println!("\nRULES");
    for rule in rules {
        let rule = grammar.rule(rule);
        println!("{}", rule.string(grammar));
    }
}
