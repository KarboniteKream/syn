use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    pub description: String,
    pub start_symbol: Symbol,
    pub rules: HashMap<Symbol, Vec<Rule>>,
}

#[derive(Debug)]
pub enum GrammarError<'a> {
    Completeness(&'a Symbol),
    Unreachable(&'a Symbol),
}

impl<'a> Display for GrammarError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GrammarError::Completeness(symbol) => write!(f, "Symbol '{}' is not complete", symbol),
            GrammarError::Unreachable(symbol) => write!(f, "Symbol '{}' is unreachable", symbol),
        }
    }
}

impl Grammar {
    pub fn new(name: String, description: String, start_symbol: Symbol) -> Grammar {
        Grammar {
            name: name,
            description: description,
            start_symbol: start_symbol,
            rules: HashMap::new(),
        }
    }

    pub fn verify(&self) -> Result<(), GrammarError> {
        let mut nonterminals: HashSet<&Symbol> = self
            .rules
            .values()
            .flat_map(|rules| rules.iter().flat_map(Rule::nonterminals))
            .collect();

        nonterminals.insert(&self.start_symbol);

        for symbol in self.rules.keys() {
            if !nonterminals.contains(symbol) {
                return Err(GrammarError::Unreachable(symbol));
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
                    let nonterminals: HashSet<&Symbol> = self
                        .rules
                        .get(symbol)
                        .unwrap()
                        .iter()
                        .flat_map(Rule::nonterminals)
                        .collect();
                    (
                        symbol,
                        nonterminals
                            .iter()
                            .any(|symbol| *completeness.get(symbol).unwrap()),
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
            return Err(GrammarError::Completeness(symbol));
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

        for rule in self.rules.get(symbol).unwrap() {
            for sym in &rule.symbols {
                let next: HashSet<Symbol> = self.first(sym);
                let has_epsilon = next.iter().any(Symbol::is_epsilon);
                result.extend(next);

                if !has_epsilon {
                    break;
                }
            }
        }

        return result;
    }
}
