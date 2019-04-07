use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util;

#[derive(Clone, Debug)]
pub struct Grammar {
    pub name: String,
    description: String,
    pub start_symbol: Symbol,
    all_rules: Vec<Rule>,
    pub rules: HashMap<Symbol, Vec<Rule>>,
    first: RefCell<HashMap<Symbol, Vec<Symbol>>>,
}

impl Grammar {
    pub fn new(name: String, description: String, start_symbol: Symbol) -> Grammar {
        Grammar {
            name,
            description,
            start_symbol,
            all_rules: Vec::new(),
            rules: HashMap::new(),
            first: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_rules(&mut self, symbol: &Symbol, rules: &[Rule]) {
        self.all_rules.extend(rules.to_vec());
        self.rules.insert(symbol.clone(), rules.to_vec());
    }

    pub fn verify(&self) -> Result<(), GrammarError> {
        if !self.rules.contains_key(&self.start_symbol) {
            return Err(GrammarError::NoSymbol(self.start_symbol.clone()));
        }

        let mut nonterminals: HashSet<&Symbol> =
            self.all_rules.iter().flat_map(Rule::nonterminals).collect();

        nonterminals.insert(&self.start_symbol);

        for symbol in self.rules.keys() {
            if !nonterminals.contains(symbol) {
                return Err(GrammarError::Unreachable(symbol.clone()));
            }
        }

        for (symbol, rules) in &self.rules {
            if !rules.is_empty() && rules.iter().all(|rule| rule.first() == symbol) {
                return Err(GrammarError::LeftRecursive(symbol.clone()));
            }
        }

        for rule in &self.all_rules {
            if rule.body.iter().all(Symbol::is_terminal) {
                nonterminals.remove(&rule.head);
            }
        }

        loop {
            let realizable: HashSet<&Symbol> = nonterminals
                .iter()
                .filter(|symbol| {
                    self.rules[symbol]
                        .iter()
                        .any(|rule| rule.nonterminals().is_disjoint(&nonterminals))
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
            return Err(GrammarError::NotRealizable(not_realizable[0].clone()));
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
        let rules = util::to_string(self.all_rules.iter(), "\n");
        write!(f, "{} ({})\n{}", self.name, self.description, rules)
    }
}

#[derive(Debug)]
pub enum GrammarError {
    NoSymbol(Symbol),
    Unreachable(Symbol),
    LeftRecursive(Symbol),
    NotRealizable(Symbol),
}

impl Display for GrammarError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GrammarError::NoSymbol(symbol) => write!(f, "Symbol {} does not exist", symbol),
            GrammarError::Unreachable(symbol) => write!(f, "Symbol {} is unreachable", symbol),
            GrammarError::LeftRecursive(symbol) => write!(f, "Symbol {} is left recursive", symbol),
            GrammarError::NotRealizable(symbol) => write!(f, "Symbol {} is not realizable", symbol),
        }
    }
}

impl Error for GrammarError {}
