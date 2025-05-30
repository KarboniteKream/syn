use std::fs;
use std::path::Path;
use std::process;

mod automaton;
mod grammar;
mod lexer;
mod parser;
mod util;

use automaton::Automaton;
use util::AsString;

fn main() {
    let args = util::parse_args();

    let filename = args.get_one::<String>("grammar").unwrap();
    let mut grammar = match grammar::read_file(Path::new(filename)) {
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

    let filename = args.get_one::<String>("input").unwrap();
    let tokens = match lexer::get_tokens(Path::new(filename), &grammar) {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("Input file '{}' cannot be parsed: {}", filename, error);
            process::exit(1);
        }
    };

    if tokens.is_empty() {
        process::exit(0);
    }

    let rules = match args.get_one::<String>("parser").unwrap().as_str() {
        "ll" => parser::parse_ll(&tokens, &grammar),
        "lr" => {
            let automaton = Automaton::new(&grammar, 0);

            let data = match automaton.data() {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("Grammar '{}' is not valid: {}", grammar.name, error);
                    process::exit(1);
                }
            };

            if let Some(output) = args.get_one::<String>("output") {
                let contents = automaton.to_dot();

                if let Err(error) = fs::write(Path::new(output), contents) {
                    eprintln!("Unable to save to file '{}': {}", output, error);
                    process::exit(1);
                }
            }

            parser::parse_lr(&tokens, &grammar, &data)
        }
        "lllr" => parser::parse_lllr(&tokens, &mut grammar),
        _ => Err(parser::Error::Internal),
    };

    if let Err(error) = rules {
        eprintln!("Input file '{}' cannot be parsed: {}", filename, error);
        process::exit(1);
    }

    for rule in rules.unwrap().into_iter().skip(1) {
        let rule = grammar.rule(rule);
        println!("{}", rule.string(&grammar));
    }
}
