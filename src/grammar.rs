use std::collections::{HashMap, HashSet};
use std::error::Error;
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

impl Grammar {
    pub fn new(name: String, description: String, start_symbol: Symbol) -> Grammar {
        Grammar {
            name,
            description,
            start_symbol,
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
                return Err(GrammarError::Unreachable(symbol.clone()));
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
            return Err(GrammarError::Incomplete(symbol.clone()));
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

        let mut rules: Vec<(&Rule, usize)> =
            self.rules[symbol].iter().map(|rule| (rule, 0)).collect();

        loop {
            for (rule, idx) in &mut rules {
                let mut per_rule: HashSet<Symbol> = HashSet::new();

                for sym in &rule.symbols[*idx..] {
                    *idx += 1;

                    if sym == symbol {
                        per_rule.remove(&Symbol::Null);
                        break;
                    }

                    let per_symbol: HashSet<Symbol> = self.first(sym);
                    let has_null = per_symbol.iter().any(|symbol| *symbol == Symbol::Null);
                    per_rule.extend(per_symbol);

                    if !has_null {
                        per_rule.remove(&Symbol::Null);
                        break;
                    }
                }

                result.extend(per_rule);
            }

            let all_done = rules.iter().all(|(rule, idx)| rule.symbols.len() == *idx);
            let has_null = result.iter().any(|symbol| *symbol == Symbol::Null);

            if all_done || !has_null {
                break;
            }
        }

        result
    }
}

#[derive(Debug)]
pub enum GrammarError {
    Incomplete(Symbol),
    Unreachable(Symbol),
}

impl Display for GrammarError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GrammarError::Incomplete(symbol) => write!(f, "Symbol '{}' is not complete", symbol),
            GrammarError::Unreachable(symbol) => write!(f, "Symbol '{}' is unreachable", symbol),
        }
    }
}

impl Error for GrammarError {}
