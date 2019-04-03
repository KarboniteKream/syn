use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Grammar {
    pub name: String,
    description: String,
    pub start_symbol: Symbol,
    pub rules: HashMap<Symbol, Vec<Rule>>,
    first: RefCell<HashMap<Symbol, Vec<Symbol>>>,
}

impl Grammar {
    pub fn new(name: String, description: String, start_symbol: Symbol) -> Grammar {
        Grammar {
            name,
            description,
            start_symbol,
            rules: HashMap::new(),
            first: RefCell::new(HashMap::new()),
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

        for (symbol, rules) in &self.rules {
            if !rules.is_empty() && rules.iter().all(|rule| rule.body[0] == *symbol) {
                return Err(GrammarError::LeftRecursive(symbol.clone()));
            }
        }

        let mut realizable: HashMap<&Symbol, bool> = self
            .rules
            .iter()
            .map(|(symbol, rules)| {
                (
                    symbol,
                    rules
                        .iter()
                        .any(|rule| rule.body.iter().all(Symbol::is_terminal)),
                )
            })
            .collect();

        loop {
            let changes: HashMap<&Symbol, bool> = realizable
                .iter()
                .filter(|(_, &realizable)| !realizable)
                .map(|(&symbol, _)| {
                    let nonterminals: HashSet<&Symbol> = self.rules[symbol]
                        .iter()
                        .flat_map(Rule::nonterminals)
                        .collect();
                    (symbol, nonterminals.iter().any(|symbol| realizable[symbol]))
                })
                .filter(|(_, realizable)| *realizable)
                .collect();

            if changes.is_empty() {
                break;
            }

            realizable.extend(changes);
        }

        if let Some((&symbol, _)) = realizable.iter().find(|(_, &realizable)| !realizable) {
            return Err(GrammarError::NotRealizable(symbol.clone()));
        }

        Ok(())
    }

    pub fn first(&self, symbol: &Symbol) -> Vec<Symbol> {
        if let Some(first) = self.first.borrow().get(symbol) {
            return first.clone();
        }

        let mut buffer = HashSet::new();

        if symbol.is_terminal() {
            buffer.insert(symbol.clone());
            return self.cache_first(symbol, &buffer);
        }

        if !self.rules.contains_key(symbol) {
            return Vec::new();
        }

        let mut rules: Vec<(&Rule, usize)> =
            self.rules[symbol].iter().map(|rule| (rule, 0)).collect();

        // Sort rules by type of first symbol to avoid some cycles.
        rules.sort_unstable_by(|a, b| b.0.body[0].is_terminal().cmp(&a.0.body[0].is_terminal()));

        loop {
            for (rule, idx) in &mut rules {
                let mut rule_buffer = HashSet::new();

                for sym in &rule.body[*idx..] {
                    *idx += 1;

                    if sym == symbol {
                        rule_buffer.remove(&Symbol::Null);
                        break;
                    }

                    let first: Vec<Symbol> = self.first(sym);
                    let has_null = first.iter().any(|symbol| *symbol == Symbol::Null);
                    rule_buffer.extend(first);

                    if !has_null {
                        rule_buffer.remove(&Symbol::Null);
                        break;
                    }
                }

                buffer.extend(rule_buffer);
                self.cache_first(symbol, &buffer);
            }

            let all_done = rules.iter().all(|(rule, idx)| rule.body.len() == *idx);
            let has_null = buffer.iter().any(|symbol| *symbol == Symbol::Null);

            if all_done || !has_null {
                break;
            }
        }

        self.cache_first(symbol, &buffer)
    }

    pub fn first_sequence(&self, symbols: &[Symbol]) -> Vec<Symbol> {
        let mut buffer = HashSet::new();

        for symbol in symbols {
            let per_symbol = self.first(symbol);
            let has_null = per_symbol.iter().any(|symbol| *symbol == Symbol::Null);
            buffer.extend(per_symbol);

            if !has_null {
                buffer.remove(&Symbol::Null);
                break;
            }
        }

        let mut first: Vec<Symbol> = buffer.into_iter().collect();
        first.sort_unstable();
        first
    }

    fn cache_first(&self, symbol: &Symbol, buffer: &HashSet<Symbol>) -> Vec<Symbol> {
        let mut first: Vec<Symbol> = buffer.iter().cloned().collect();
        first.sort_unstable();

        let mut cache = self.first.borrow_mut();
        cache.insert(symbol.clone(), first.clone());

        first
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut rules: Vec<String> = self.rules[&self.start_symbol]
            .iter()
            .map(Rule::to_string)
            .collect();

        rules.extend(
            self.rules
                .iter()
                .filter(|(symbol, _)| **symbol != self.start_symbol)
                .flat_map(|(_, rules)| rules.iter().map(Rule::to_string).collect::<Vec<String>>())
                .collect::<Vec<String>>(),
        );

        write!(
            f,
            "{} ({})\n{}",
            self.name,
            self.description,
            rules.join("\n")
        )
    }
}

#[derive(Debug)]
pub enum GrammarError {
    Unreachable(Symbol),
    LeftRecursive(Symbol),
    NotRealizable(Symbol),
}

impl Display for GrammarError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GrammarError::Unreachable(symbol) => write!(f, "Symbol {} is unreachable", symbol),
            GrammarError::LeftRecursive(symbol) => write!(f, "Symbol {} is left recursive", symbol),
            GrammarError::NotRealizable(symbol) => write!(f, "Symbol {} is not realizable", symbol),
        }
    }
}

impl Error for GrammarError {}
