use std::collections::HashSet;
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

    if definitions.is_empty() {
        return Err(Error::File("No rules defined".to_owned()));
    }

    let start_symbol = Symbol::NonTerminal(
        from_table(&data, "start_symbol", &Value::as_str)
            .unwrap_or_else(|_| definitions.keys().next().unwrap())
            .to_owned(),
    );

    let nonterminals: HashSet<&str> = definitions.keys().map(String::as_str).collect();

    let mut symbols = vec![Symbol::Start, Symbol::End, Symbol::Null];
    let mut rules = vec![Rule::new(
        0,
        Symbol::Start,
        vec![Symbol::End, start_symbol.clone(), Symbol::End],
    )];

    for (name, definitions) in definitions {
        let definitions = match definitions.as_array() {
            Some(value) => value.clone(),
            None => vec![definitions.clone()],
        };

        let symbol = Symbol::NonTerminal(name.clone());
        symbols.push(symbol.clone());

        for definition in definitions {
            let definition = match definition.as_str() {
                Some(value) => value,
                None => return Err(Error::Rule(name.to_owned())),
            };

            let body = if definition.is_empty() {
                vec![Symbol::Null]
            } else {
                definition
                    .split_whitespace()
                    .map(|name| {
                        if nonterminals.contains(name) {
                            Symbol::NonTerminal(name.to_owned())
                        } else {
                            Symbol::Terminal(name.to_owned())
                        }
                    })
                    .collect()
            };

            rules.push(Rule::new(rules.len(), symbol.clone(), body));
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
