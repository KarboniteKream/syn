use std::collections::{BTreeMap, HashSet};
use std::fs;

use toml::Value;

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::{Symbol, SymbolType};

pub fn parse(filename: &str) -> Result<Grammar, String> {
    let value = match fs::read_to_string(filename) {
        Ok(contents) => match contents.parse::<Value>() {
            Ok(value) => value,
            Err(error) => return Err(error.to_string()),
        },
        Err(error) => return Err(error.to_string()),
    };

    let data: &BTreeMap<String, Value> = match value.as_table() {
        Some(value) => value,
        None => return Err(format!("Value {} is not a Table.", value)),
    };

    let name = from_table(&data, "name", &Value::as_str)?.to_owned();
    let description = from_table(&data, "description", &Value::as_str)
        .unwrap_or(filename)
        .to_owned();
    let start_symbol = Symbol::new(
        from_table(&data, "start_symbol", &Value::as_str)?.to_owned(),
        SymbolType::NonTerminal,
    );

    let mut grammar = Grammar::new(name, description, start_symbol);
    let rules = from_table(&data, "rules", &Value::as_table)?;
    let nonterminals: HashSet<&str> = rules.keys().map(|name| name.as_str()).collect();

    for (name, rules) in rules {
        let symbol = Symbol::new(name.clone(), SymbolType::NonTerminal);
        let rules = match rules.as_array() {
            Some(value) => value.clone(),
            None => vec![rules.clone()],
        };

        let mut list: Vec<Rule> = Vec::new();

        for rule in rules {
            let rule = match rule.as_str() {
                Some(value) => value,
                None => return Err(format!("Rule {} is not a String", rule)),
            };

            let symbols: Vec<Symbol> = if rule.is_empty() {
                vec![Symbol::new("".to_owned(), SymbolType::Epsilon)]
            } else {
                rule.split_whitespace()
                    .map(|name| {
                        Symbol::new(
                            name.to_owned(),
                            if nonterminals.contains(name) {
                                SymbolType::NonTerminal
                            } else {
                                SymbolType::Terminal
                            },
                        )
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
) -> Result<&'a T, String> {
    match data.get(key).and_then(f) {
        Some(value) => Ok(value),
        None => Err(format!("Unable to parse key '{}'", key)),
    }
}
