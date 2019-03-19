use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;

use toml::Value;

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Debug)]
pub enum ParseError {
    File(String),
    Key(String),
    Rule(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::File(error) => write!(f, "Cannot parse file: {}", error),
            ParseError::Key(name) => write!(f, "Cannot parse key '{}'", name),
            ParseError::Rule(name) => write!(f, "Cannot parse rule '{}'", name),
        }
    }
}

impl Error for ParseError {}

pub fn parse(filename: &str) -> Result<Grammar, ParseError> {
    let value = match fs::read_to_string(filename) {
        Ok(contents) => match contents.parse::<Value>() {
            Ok(value) => value,
            Err(error) => return Err(ParseError::File(error.to_string())),
        },
        Err(error) => return Err(ParseError::File(error.to_string())),
    };

    let data: &BTreeMap<String, Value> = match value.as_table() {
        Some(value) => value,
        None => return Err(ParseError::File("Not a Table".to_owned())),
    };

    let name = from_table(&data, "name", &Value::as_str)?.to_owned();
    let description = from_table(&data, "description", &Value::as_str)
        .unwrap_or(filename)
        .to_owned();
    let start_symbol =
        Symbol::NonTerminal(from_table(&data, "start_symbol", &Value::as_str)?.to_owned());

    let mut grammar = Grammar::new(name, description, start_symbol);
    let rules = from_table(&data, "rules", &Value::as_table)?;
    let nonterminals: HashSet<&str> = rules.keys().map(|name| name.as_str()).collect();

    for (name, rules) in rules {
        let symbol = Symbol::NonTerminal(name.clone());
        let rules = match rules.as_array() {
            Some(value) => value.clone(),
            None => vec![rules.clone()],
        };

        let mut list: Vec<Rule> = Vec::new();

        for rule in rules {
            let rule = match rule.as_str() {
                Some(value) => value,
                None => return Err(ParseError::Rule(name.to_owned())),
            };

            let symbols: Vec<Symbol> = if rule.is_empty() {
                vec![Symbol::Null]
            } else {
                rule.split_whitespace()
                    .map(|name| {
                        if nonterminals.contains(name) {
                            Symbol::NonTerminal(name.to_owned())
                        } else {
                            Symbol::Terminal(name.to_owned())
                        }
                    })
                    .collect()
            };

            list.push(Rule::new(symbols));
        }

        grammar.rules.insert(symbol, list);
    }

    Ok(grammar)
}

fn from_table<'a, T: ?Sized>(
    data: &'a BTreeMap<String, Value>,
    key: &str,
    f: &Fn(&'a Value) -> Option<&T>,
) -> Result<&'a T, ParseError> {
    match data.get(key).and_then(f) {
        Some(value) => Ok(value),
        None => Err(ParseError::Key(key.to_owned())),
    }
}
