use std::collections::{HashMap, HashSet};

use crate::rule::Rule;
use crate::symbol::Symbol;

mod error;

use error::Error;

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    pub description: String,
    pub start_symbol: Symbol,
    pub rules: HashMap<Symbol, Vec<Rule>>,
}

impl Grammar {
    pub fn new(name: String, description: String, start_symbol: Symbol) -> Grammar {
        Grammar {
            name,
            description,
            start_symbol,
            rules: HashMap::new(),
        }
    }

    pub fn verify(&self) -> Result<(), Error> {
        let mut nonterminals: HashSet<&Symbol> = self
            .rules
            .values()
            .flat_map(|rules| rules.iter().flat_map(Rule::nonterminals))
            .collect();

        nonterminals.insert(&self.start_symbol);

        for symbol in self.rules.keys() {
            if !nonterminals.contains(symbol) {
                return Err(Error::Unreachable(symbol));
            }
        }

        // TODO: Detect left recursion.

        let mut completeness: HashMap<&Symbol, bool> = self
            .rules
            .iter()
            .map(|(symbol, rules)| {
                (
                    symbol,
                    rules
                        .iter()
                        .any(|rule| rule.symbols.iter().all(Symbol::is_terminal)),
                )
            })
            .collect();

        loop {
            let changes: HashMap<&Symbol, bool> = completeness
                .iter()
                .filter(|(_, &complete)| !complete)
                .map(|(&symbol, _)| {
                    let nonterminals: HashSet<&Symbol> = self.rules[symbol]
                        .iter()
                        .flat_map(Rule::nonterminals)
                        .collect();
                    (
                        symbol,
                        nonterminals.iter().any(|symbol| completeness[symbol]),
                    )
                })
                .filter(|(_, complete)| *complete)
                .collect();

            if changes.is_empty() {
                break;
            }

            completeness.extend(changes);
        }

        if let Some((&symbol, _)) = completeness.iter().find(|(_, &complete)| !complete) {
            return Err(Error::Completeness(symbol));
        }

        Ok(())
    }

    pub fn first(&self, symbol: &Symbol) -> HashSet<Symbol> {
        let mut result: HashSet<Symbol> = HashSet::new();

        if symbol.is_terminal() {
            result.insert(symbol.clone());
            return result;
        }

        if !self.rules.contains_key(symbol) {
            return result;
        }

        for rule in &self.rules[symbol] {
            for sym in &rule.symbols {
                let next: HashSet<Symbol> = self.first(sym);
                let has_epsilon = next.iter().any(Symbol::is_epsilon);
                result.extend(next);

                if !has_epsilon {
                    break;
                }
            }
        }

        result
    }
}
