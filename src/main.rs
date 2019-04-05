use std::env;
use std::fs;
use std::path::Path;

use std::process;

mod grammar;
mod lr;
mod parser;
mod rule;
mod symbol;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please specify a grammar file name");
        process::exit(1);
    }

    let filename = Path::new(&args[1]);
    let grammar = match parser::parse_file(filename) {
        Ok(grammar) => grammar,
        Err(error) => {
            eprintln!("Unable to parse file '{}': {}", filename.display(), error);
            process::exit(1);
        }
    };

    if let Err(error) = grammar.verify() {
        eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
        process::exit(1);
    }

    println!("Grammar\n{}\n", grammar);

    let automaton = lr::Automaton::new(&grammar);
    println!("Automaton\n{}", automaton);

    if let Some(output) = args.get(2).map(Path::new) {
        let contents: String = automaton.to_dot();

        if let Err(error) = fs::write(output, contents) {
            eprintln!("Unable to save to file '{}': {}", output.display(), error);
            process::exit(1);
        }
    }
}
