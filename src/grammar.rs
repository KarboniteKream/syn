use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Display, Formatter};

use regex::Regex;

use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util;

mod reader;

pub use reader::read_file;

#[derive(Clone, Debug)]
pub struct Grammar {
    pub name: String,
    description: String,
    pub symbols: Vec<Symbol>,
    tokens: Vec<(usize, Regex)>,
    start_symbol: usize,
    rules: Vec<Rule>,
    symbol_rules: HashMap<usize, Vec<usize>>,
    first: RefCell<HashMap<usize, Vec<usize>>>,
}

impl Grammar {
    pub fn new(
        name: String,
        description: String,
        symbols: Vec<Symbol>,
        tokens: Vec<(usize, Regex)>,
        rules: Vec<Rule>,
        start_symbol: usize,
    ) -> Grammar {
        let symbol_rules = rules
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut acc, (id, rule)| {
                acc.entry(rule.head).or_insert_with(Vec::new).push(id);
                acc
            });

        Grammar {
            name,
            description,
            symbols,
            tokens,
            start_symbol,
            rules,
            symbol_rules,
            first: RefCell::new(HashMap::new()),
        }
    }

    pub fn symbol(&self, id: usize) -> &Symbol {
        &self.symbols[id]
    }

    pub fn rule(&self, id: usize) -> &Rule {
        &self.rules[id]
    }

    pub fn rules(&self, symbol: usize) -> Vec<&Rule> {
        self.symbol_rules[&symbol]
            .iter()
            .map(|id| self.rule(*id))
            .collect()
    }

    pub fn verify(&self) -> Result<(), Error> {
        if !self.symbol_rules.contains_key(&self.start_symbol) {
            return Err(Error::NoSymbol(self.symbol(self.start_symbol).clone()));
        }

        let mut nonterminals: HashSet<usize> = self
            .rules
            .iter()
            .flat_map(|rule| rule.nonterminals(&self.symbols))
            .collect();
        nonterminals.insert(self.start_symbol);

        for id in self.symbol_rules.keys() {
            let symbol = self.symbol(*id);

            if !symbol.is_builtin() && !nonterminals.contains(id) {
                return Err(Error::Unreachable(symbol.clone()));
            }
        }

        for (id, rules) in &self.symbol_rules {
            let symbol = self.symbol(*id);

            if symbol.is_builtin() || rules.is_empty() {
                continue;
            }

            if rules.iter().all(|rule| self.rule(*rule).body[0] == *id) {
                return Err(Error::LeftRecursive(symbol.clone()));
            }
        }

        for rule in &self.rules {
            if rule.body.iter().all(|id| self.symbol(*id).is_terminal()) {
                nonterminals.remove(&rule.head);
            }
        }

        loop {
            let realizable: HashSet<usize> = nonterminals
                .iter()
                .filter(|symbol| {
                    self.symbol_rules[symbol].iter().any(|id| {
                        self.rule(*id)
                            .nonterminals(&self.symbols)
                            .is_disjoint(&nonterminals)
                    })
                })
                .cloned()
                .collect();

            if realizable.is_empty() {
                break;
            }

            for symbol in realizable {
                nonterminals.remove(&symbol);
            }
        }

        if !nonterminals.is_empty() {
            let not_realizable = util::to_sorted_vec(&nonterminals);
            return Err(Error::NotRealizable(
                self.symbol(*not_realizable[0]).clone(),
            ));
        }

        Ok(())
    }

    pub fn first(&self, symbol: usize) -> Vec<usize> {
        if let Some(first) = self.first.borrow().get(&symbol) {
            return first.clone();
        }

        let mut buffer = HashSet::new();

        if self.symbol(symbol).is_terminal() {
            buffer.insert(symbol);
            return self.cache_first(symbol, &buffer);
        }

        if !self.symbol_rules.contains_key(&symbol) {
            return Vec::new();
        }

        let mut rules: Vec<(&Rule, usize)> = self.symbol_rules[&symbol]
            .iter()
            .map(|id| (self.rule(*id), 0))
            .collect();
        let null = Symbol::Null.id();

        loop {
            for (rule, idx) in &mut rules {
                let mut rule_buffer = HashSet::new();

                for id in &rule.body[*idx..] {
                    *idx += 1;

                    if *id == symbol {
                        rule_buffer.remove(&null);
                        break;
                    }

                    let first: Vec<usize> = self.first(*id);
                    let has_null = first.contains(&null);
                    rule_buffer.extend(first);

                    if !has_null {
                        rule_buffer.remove(&null);
                        break;
                    }
                }

                buffer.extend(rule_buffer);
                self.cache_first(symbol, &buffer);
            }

            let all_done = rules.iter().all(|(rule, idx)| rule.body.len() == *idx);
            let has_null = buffer.contains(&null);

            if all_done || !has_null {
                break;
            }
        }

        self.cache_first(symbol, &buffer)
    }

    pub fn first_sequence(&self, symbols: &[usize]) -> Vec<usize> {
        let mut buffer = HashSet::new();
        let null = Symbol::Null.id();

        for symbol in symbols {
            let per_symbol = self.first(*symbol);
            let has_null = per_symbol.contains(&null);
            buffer.extend(per_symbol);

            if !has_null {
                buffer.remove(&null);
                break;
            }
        }

        util::to_sorted_vec(buffer)
    }

    pub fn find_symbol(&self, text: &str) -> Option<(usize, bool)> {
        let mut symbol = None;

        for (id, regex) in &self.tokens {
            let captures = match regex.captures(text) {
                Some(captures) => captures,
                None => continue,
            };

            let is_full_match = captures
                .get(captures.len() - 1)
                .map_or(false, |m| !m.as_str().is_empty());

            if is_full_match {
                return Some((*id, true));
            }

            if symbol.is_none() {
                symbol = Some((*id, false));
            }
        }

        symbol
    }

    fn cache_first(&self, symbol: usize, buffer: &HashSet<usize>) -> Vec<usize> {
        let first = util::to_sorted_vec(buffer.clone());
        let mut cache = self.first.borrow_mut();
        cache.insert(symbol, first.clone());
        first
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rules = util::as_string(self.rules.iter(), self, "\n");
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
