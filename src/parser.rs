use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use toml::{map::Map, Value};

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;

pub fn parse_file(filename: &Path) -> Result<Grammar, Error> {
    let value = match fs::read_to_string(filename) {
        Ok(contents) => match contents.parse::<Value>() {
            Ok(value) => value,
            Err(error) => return Err(Error::File(error.to_string())),
        },
        Err(error) => return Err(Error::File(error.to_string())),
    };

    let data: &Map<String, Value> = match value.as_table() {
        Some(value) => value,
        None => return Err(Error::File("Not a Table".to_owned())),
    };

    let name = from_table(&data, "name", &Value::as_str)?.to_owned();
    let description = from_table(&data, "description", &Value::as_str)
        .map(&str::to_owned)
        .unwrap_or_else(|_| {
            let path = filename.canonicalize().unwrap();
            path.into_os_string().into_string().unwrap()
        });

    let definitions = from_table(&data, "rules", &Value::as_table)?;
    let nonterminals: HashSet<&str> = definitions.keys().map(String::as_str).collect();

    if definitions.is_empty() {
        return Err(Error::File("No rules defined".to_owned()));
    }

    let mut symbols = Symbol::builtin();
    let start_symbol = symbols.len();

    symbols.push(Symbol::NonTerminal(
        start_symbol,
        from_table(&data, "start_symbol", &Value::as_str)
            .unwrap_or_else(|_| definitions.keys().next().unwrap())
            .to_owned(),
    ));

    let mut rules = vec![Rule::new(
        0,
        Symbol::Start.id(),
        vec![Symbol::End.id(), start_symbol, Symbol::End.id()],
    )];

    let mut names: HashMap<String, usize> = symbols
        .iter()
        .map(|symbol| (symbol.name(), symbol.id()))
        .collect();

    for (name, definitions) in definitions {
        let definitions = match definitions.as_array() {
            Some(value) => value.clone(),
            None => vec![definitions.clone()],
        };

        for definition in definitions {
            let definition = match definition.as_str() {
                Some(value) => value,
                None => return Err(Error::Rule(name.to_owned())),
            };

            let body = if definition.is_empty() {
                vec![Symbol::Null.id()]
            } else {
                definition
                    .split_whitespace()
                    .map(|name| {
                        let is_terminal = !nonterminals.contains(name);
                        get_symbol(name, is_terminal, &mut names, &mut symbols)
                    })
                    .collect()
            };

            let head = get_symbol(name, false, &mut names, &mut symbols);
            rules.push(Rule::new(rules.len(), head, body));
        }
    }

    Ok(Grammar::new(
        name,
        description,
        symbols,
        rules,
        start_symbol,
    ))
}

fn from_table<'a, T: ?Sized>(
    data: &'a Map<String, Value>,
    key: &str,
    f: &Fn(&'a Value) -> Option<&T>,
) -> Result<&'a T, Error> {
    match data.get(key).and_then(f) {
        Some(value) => Ok(value),
        None => Err(Error::Key(key.to_owned())),
    }
}

fn get_symbol(
    name: &str,
    is_terminal: bool,
    names: &mut HashMap<String, usize>,
    symbols: &mut Vec<Symbol>,
) -> usize {
    if let Some(id) = names.get(name) {
        return *id;
    }

    let id = symbols.len();
    let name = name.to_owned();
    names.insert(name.clone(), id);

    symbols.push(if is_terminal {
        Symbol::Terminal(id, name)
    } else {
        Symbol::NonTerminal(id, name)
    });

    id
}

#[derive(Debug)]
pub enum Error {
    File(String),
    Key(String),
    Rule(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::File(error) => write!(f, "Cannot parse file: {}", error),
            Error::Key(name) => write!(f, "Cannot parse key '{}'", name),
            Error::Rule(name) => write!(f, "Cannot parse rule {}", name),
        }
    }
}

impl error::Error for Error {}
