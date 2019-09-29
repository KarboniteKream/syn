use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use regex::{self, Regex};
use toml::{map::Map, Value};

use crate::automaton::Action;
use crate::grammar::{Grammar, Matcher};

use super::rule::Rule;
use super::symbol::Symbol;

/// Read the specified file, and constructs the grammar.
pub fn read_file(filename: &Path) -> Result<Grammar, Error> {
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => match contents.parse::<Value>() {
            Ok(value) => value,
            Err(error) => return Err(Error::File(error.to_string())),
        },
        Err(error) => return Err(Error::File(error.to_string())),
    };

    let data: &Map<String, Value> = match contents.as_table() {
        Some(data) => data,
        None => return Err(Error::File("Not a Table".to_owned())),
    };

    let name = from_table(data, "name", &Value::as_str)?.to_owned();
    let description = from_table(data, "description", &Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|_| {
            let path = filename.canonicalize().unwrap();
            path.into_os_string().into_string().unwrap()
        });

    let definitions = from_table(data, "rules", &Value::as_table)?;
    // All L-values are considered nonterminal symbols.
    let nonterminals: HashSet<&str> = definitions.keys().map(String::as_str).collect();

    if definitions.is_empty() {
        return Err(Error::File("No rules defined".to_owned()));
    }

    let mut symbols = Symbol::internal();
    let start_symbol = symbols.len();

    symbols.push(Symbol::NonTerminal(
        start_symbol,
        from_table(data, "start_symbol", &Value::as_str)
            .unwrap_or_else(|_| definitions.keys().next().unwrap())
            .to_owned(),
    ));

    let mut rules = vec![Rule::new(
        0,
        Symbol::Start.id(),
        vec![Symbol::End.id(), start_symbol, Symbol::End.id()],
        vec![Symbol::Null.id()],
    )];

    let mut names: HashMap<String, usize> = symbols
        .iter()
        .skip(Symbol::internal().len())
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
            let rule = Rule::new(rules.len(), head, body, Vec::new());

            if rules.contains(&rule) {
                continue;
            }

            rules.push(rule);
        }
    }

    let mut matchers = Vec::new();

    // Generate regular expressions for all terminal symbols.
    for symbol in &symbols {
        let (id, name) = match symbol {
            Symbol::Terminal(id, name) => (*id, name),
            _ => continue,
        };

        matchers.push((id, Matcher::Text(name.to_owned())));
    }

    let definitions = from_table(data, "tokens", &Value::as_table)
        .map(Map::clone)
        .unwrap_or_default();

    for (name, pattern) in &definitions {
        let symbol = match names.get(name) {
            Some(&symbol) => symbol,
            None => continue,
        };

        // Replace the existing regular expression.
        match matchers.iter().position(|&(id, _)| id == symbol) {
            Some(idx) => matchers.remove(idx),
            None => continue,
        };

        let matcher = create_matcher(name, pattern)?;
        matchers.push((symbol, matcher));
    }

    let definitions = from_table(data, "ignore", &Value::as_table)
        .map(Map::clone)
        .unwrap_or_default();

    // All ignored tokens correspond to ϵ symbols.
    for (name, pattern) in &definitions {
        let matcher = create_matcher(name, pattern)?;
        matchers.push((Symbol::Null.id(), matcher));
    }

    let mut actions = HashMap::new();

    let definitions = from_table(data, "actions", &Value::as_table)
        .map(Map::clone)
        .unwrap_or_default();

    for (name, action) in &definitions {
        let symbol = match names.get(name) {
            Some(&symbol) => symbol,
            None => continue,
        };

        let action = match action.as_str() {
            Some(action) => action,
            None => return Err(Error::Action(name.to_owned())),
        };

        let action = match action {
            "shift" => Action::Shift(0),
            "reduce" => Action::Reduce(0),
            _ => return Err(Error::Action(name.to_owned())),
        };

        actions.insert(symbol, action);
    }

    Ok(Grammar::new(
        name,
        description,
        symbols,
        matchers,
        start_symbol,
        rules,
        actions,
    ))
}

/// Returns a value from a TOML table.
fn from_table<'a, T: ?Sized>(
    data: &'a Map<String, Value>,
    key: &str,
    f: &dyn Fn(&'a Value) -> Option<&T>,
) -> Result<&'a T, Error> {
    match data.get(key).and_then(f) {
        Some(value) => Ok(value),
        None => Err(Error::Key(key.to_owned())),
    }
}

/// Returns the symbol ID for the specified symbol name.
fn get_symbol(
    name: &str,
    is_terminal: bool,
    names: &mut HashMap<String, usize>,
    symbols: &mut Vec<Symbol>,
) -> usize {
    if let Some(&id) = names.get(name) {
        return id;
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

/// Creates a `Matcher` from a specified pattern.
fn create_matcher(name: &str, pattern: &Value) -> Result<Matcher, Error> {
    // If the pattern is a single string, create a regex matcher.
    if let Some(pattern) = pattern.as_str() {
        let pattern = format!("^{}$", pattern);

        return match Regex::new(&pattern) {
            Ok(regex) => Ok(Matcher::Regex(regex)),
            Err(_) => Err(Error::Regex(pattern)),
        };
    }

    // If the pattern is an array of strings, create a group matcher.
    let patterns = match pattern.as_array() {
        Some(patterns) => patterns,
        None => return Err(Error::Token(name.to_owned())),
    };

    let mut group = Vec::new();

    for pattern in patterns {
        let pattern = match pattern.as_str() {
            Some(pattern) => pattern.to_owned(),
            None => return Err(Error::Token(name.to_owned())),
        };

        group.push(pattern);
    }

    Ok(Matcher::Group(group))
}

#[derive(Debug)]
pub enum Error {
    Action(String),
    File(String),
    Key(String),
    Regex(String),
    Rule(String),
    Token(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Action(name) => write!(f, "Cannot parse action for {}", name),
            Self::File(error) => write!(f, "Cannot read file {}", error),
            Self::Key(name) => write!(f, "Cannot parse key '{}'", name),
            Self::Regex(pattern) => write!(f, "Cannot parse expression /{}/", pattern),
            Self::Rule(name) => write!(f, "Cannot parse rule {}", name),
            Self::Token(name) => write!(f, "Cannot parse token '{}'", name),
        }
    }
}

impl error::Error for Error {}
