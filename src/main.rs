use std::fs;
use std::path::Path;
use std::process;

mod grammar;
mod lr;
mod parser;
mod rule;
mod symbol;
mod util;

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

    let automaton = lr::Automaton::new(&grammar);
    println!("\nAutomaton\n{}", automaton);

    if let Some(output) = args.value_of("output") {
        let contents: String = automaton.to_dot();

        if let Err(error) = fs::write(Path::new(output), contents) {
            eprintln!("Unable to save to file '{}': {}", output, error);
            process::exit(1);
        }
    }
}
