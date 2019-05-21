use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util;

#[derive(Clone, Debug)]
pub struct Grammar {
    pub name: String,
    description: String,
    symbols: Vec<Symbol>,
    pub start_symbol: Symbol,
    rules: Vec<Rule>,
    symbol_rules: HashMap<Symbol, Vec<usize>>,
    first: RefCell<HashMap<Symbol, Vec<Symbol>>>,
}

impl Grammar {
    pub fn new(
        name: String,
        description: String,
        symbols: Vec<Symbol>,
        rules: Vec<Rule>,
        start_symbol: Symbol,
    ) -> Grammar {
        let symbol_rules = rules
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut acc, (id, rule)| {
                acc.entry(rule.head.clone())
                    .or_insert_with(Vec::new)
                    .push(id);

                acc
            });

        Grammar {
            name,
            description,
            symbols,
            start_symbol,
            rules,
            symbol_rules,
            first: RefCell::new(HashMap::new()),
        }
    }

    pub fn rules(&self, symbol: &Symbol) -> Vec<&Rule> {
        self.symbol_rules[symbol]
            .iter()
            .map(|id| self.rule(*id))
            .collect()
    }

    pub fn rule(&self, id: usize) -> &Rule {
        &self.rules[id]
    }

    pub fn verify(&self) -> Result<(), Error> {
        if !self.symbol_rules.contains_key(&self.start_symbol) {
            return Err(Error::NoSymbol(self.start_symbol.clone()));
        }

        let mut nonterminals: HashSet<&Symbol> =
            self.rules.iter().flat_map(Rule::nonterminals).collect();

        nonterminals.insert(&self.start_symbol);

        for symbol in self.symbol_rules.keys() {
            if !symbol.is_builtin() && !nonterminals.contains(symbol) {
                return Err(Error::Unreachable(symbol.clone()));
            }
        }

        for (symbol, rules) in &self.symbol_rules {
            if symbol.is_builtin() || rules.is_empty() {
                continue;
            }

            if rules.iter().all(|id| self.rule(*id).first() == symbol) {
                return Err(Error::LeftRecursive(symbol.clone()));
            }
        }

        for rule in &self.rules {
            if rule.body.iter().all(Symbol::is_terminal) {
                nonterminals.remove(&rule.head);
            }
        }

        loop {
            let realizable: HashSet<&Symbol> = nonterminals
                .iter()
                .filter(|symbol| {
                    self.symbol_rules[symbol]
                        .iter()
                        .any(|id| self.rule(*id).nonterminals().is_disjoint(&nonterminals))
                })
                .cloned()
                .collect();

            if realizable.is_empty() {
                break;
            }

            for symbol in realizable {
                nonterminals.remove(symbol);
            }
        }

        if !nonterminals.is_empty() {
            let not_realizable = util::to_sorted_vec(&nonterminals);
            return Err(Error::NotRealizable(not_realizable[0].clone()));
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

        if !self.symbol_rules.contains_key(symbol) {
            return Vec::new();
        }

        let mut rules: Vec<(&Rule, usize)> = self.symbol_rules[symbol]
            .iter()
            .map(|id| (self.rule(*id), 0))
            .collect();

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

        util::to_sorted_vec(&buffer)
    }

    fn cache_first(&self, symbol: &Symbol, buffer: &HashSet<Symbol>) -> Vec<Symbol> {
        let first = util::to_sorted_vec(buffer);
        let mut cache = self.first.borrow_mut();
        cache.insert(symbol.clone(), first.clone());
        first
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rules = util::to_string(self.rules.iter(), "\n");
        write!(f, "{} ({})\n{}", self.name, self.description, rules)
    }
}

#[derive(Debug)]
pub enum Error {
    NoSymbol(Symbol),
    Unreachable(Symbol),
    LeftRecursive(Symbol),
    NotRealizable(Symbol),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::NoSymbol(symbol) => write!(f, "Symbol {} does not exist", symbol),
            Error::Unreachable(symbol) => write!(f, "Symbol {} is unreachable", symbol),
            Error::LeftRecursive(symbol) => write!(f, "Symbol {} is left recursive", symbol),
            Error::NotRealizable(symbol) => write!(f, "Symbol {} is not realizable", symbol),
        }
    }
}

impl error::Error for Error {}
