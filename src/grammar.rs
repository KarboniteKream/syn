use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{self, Display, Formatter};

use regex::Regex;

use crate::util;

mod reader;
mod rule;
mod symbol;

pub use reader::read_file;
pub use rule::Rule;
pub use symbol::Symbol;

/// The `Grammar` struct describes a grammar to parse the input file with.
#[derive(Clone, Debug)]
pub struct Grammar {
    pub name: String,
    description: String,
    pub symbols: Vec<Symbol>,
    // List of regular expressions for a specific symbol.
    tokens: Vec<(usize, Regex)>,
    start_symbol: usize,
    pub rules: Vec<Rule>,
    // List of rules for a specific symbol.
    symbol_rules: HashMap<usize, Vec<usize>>,
    first: RefCell<HashMap<usize, Vec<usize>>>,
    follow: RefCell<HashMap<usize, Vec<usize>>>,
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
            follow: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the symbol with the specified ID.
    pub fn symbol(&self, id: usize) -> &Symbol {
        &self.symbols[id]
    }

    /// Returns the rule with the specified ID.
    pub fn rule(&self, id: usize) -> &Rule {
        &self.rules[id]
    }

    /// Returns the list of rules for the specified symbol.
    pub fn rules(&self, symbol: usize) -> Vec<&Rule> {
        self.symbol_rules[&symbol]
            .iter()
            .map(|&id| self.rule(id))
            .collect()
    }

    /// Verifies if the grammar is valid.
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

        // Verify that all nonterminal symbols are reachable from at least one rule.
        for &id in self.symbol_rules.keys() {
            let symbol = self.symbol(id);

            if !symbol.is_internal() && !nonterminals.contains(&id) {
                return Err(Error::Unreachable(symbol.clone()));
            }
        }

        // Verify that no rule for a symbol is left-recursive.
        for (&head, rules) in &self.symbol_rules {
            let symbol = self.symbol(head);

            if symbol.is_internal() || rules.is_empty() {
                continue;
            }

            // At least one rule must not start with the rule's head.
            if rules.iter().all(|&rule| self.rule(rule).body[0] == head) {
                return Err(Error::LeftRecursive(symbol.clone()));
            }
        }

        // All symbols with at least one rule with only terminal symbols are realizable.
        for rule in &self.rules {
            if rule.body.iter().all(|&id| self.symbol(id).is_terminal()) {
                nonterminals.remove(&rule.head);
            }
        }

        loop {
            // Find symbols with at least one rule where
            // all the nonterminal symbols are realizable.
            let realizable: HashSet<usize> = nonterminals
                .iter()
                .filter(|symbol| {
                    self.symbol_rules[symbol].iter().any(|&id| {
                        self.rule(id)
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

    /// Returns the FIRST set of the specified symbol.
    pub fn first(&self, symbol: usize) -> Vec<usize> {
        if let Some(first) = self.first.borrow().get(&symbol) {
            return first.clone();
        }

        let mut first = HashSet::new();
        let null = Symbol::Null.id();

        // If the symbol is a terminal, it's the only member of the FIRST set.
        if self.symbol(symbol).is_terminal() {
            first.insert(symbol);
            return self.cache_first(symbol, &first);
        }

        if !self.symbol_rules.contains_key(&symbol) {
            return Vec::new();
        }

        // Find FIRST set of all rules of the specified symbol (s).
        // Start with the first symbol of each rule (h).
        let mut rules: Vec<(&Rule, usize)> = self.symbol_rules[&symbol]
            .iter()
            .map(|&id| (self.rule(id), 0))
            .collect();

        loop {
            for (rule, idx) in &mut rules {
                let mut rule_first = HashSet::new();

                for &id in rule.tail(*idx) {
                    *idx += 1;

                    // If the nonterminal symbol is the rule head itself,
                    // skip it until we can find the partial FIRST set.
                    if id == symbol {
                        rule_first.remove(&null);
                        break;
                    }

                    let first = self.first(id);
                    let has_null = first.contains(&null);
                    rule_first.extend(first);

                    // If FIRST(h) does not contain ϵ, remove it from FIRST(s).
                    if !has_null {
                        rule_first.remove(&null);
                        *idx = rule.body.len();
                        break;
                    }
                }

                // Cache current result to prevent an infinite loop.
                first.extend(rule_first);
                self.cache_first(symbol, &first);
            }

            // If all symbols have been checked, or FIRST(s) does not contain ϵ, we're done.
            let all_done = rules.iter().all(|&(rule, idx)| rule.body.len() == idx);
            let has_null = first.contains(&null);

            if all_done || !has_null {
                break;
            }
        }

        self.cache_first(symbol, &first)
    }

    /// Returns the FOLLOW set of the specified nonterminal symbol.
    pub fn follow(&self, symbol: usize) -> Vec<usize> {
        if let Some(follow) = self.follow.borrow().get(&symbol) {
            return follow.clone();
        }

        let mut follow: HashMap<usize, HashSet<usize>> = self
            .symbols
            .iter()
            .map(|symbol| (symbol.id(), HashSet::new()))
            .collect();

        loop {
            let mut done = true;

            for rule in &self.rules {
                for idx in 0..rule.body.len() {
                    let id = rule.body[idx];

                    if self.symbols[id].is_terminal() {
                        continue;
                    }

                    let mut first: HashSet<usize> = self
                        .first_sequence(rule.tail(idx + 1))
                        .into_iter()
                        .collect();

                    let from = follow[&rule.head].clone();

                    if first.is_empty() {
                        first = from;
                    } else if first.remove(&Symbol::Null.id()) {
                        first.extend(from);
                    }

                    let to = follow.get_mut(&id).unwrap();
                    done &= first.difference(&to).count() == 0;
                    to.extend(first);
                }
            }

            if done {
                break;
            }
        }

        let follow: HashMap<usize, Vec<usize>> = follow
            .into_iter()
            .map(|(symbol, follow)| (symbol, util::to_sorted_vec(follow)))
            .collect();

        self.follow.replace(follow.clone());
        follow[&symbol].clone()
    }

    /// Returns the FIRST set of a sequence of symbols.
    pub fn first_sequence(&self, symbols: &[usize]) -> Vec<usize> {
        let mut result = HashSet::new();
        let null = Symbol::Null.id();

        for &symbol in symbols {
            let per_symbol = self.first(symbol);
            let has_null = per_symbol.contains(&null);
            result.extend(per_symbol);

            if !has_null {
                result.remove(&null);
                break;
            }
        }

        util::to_sorted_vec(result)
    }

    /// Returns the FSTFLW set of a sequence of symbols.
    pub fn first_follow(&self, symbols: &[usize], follow: usize) -> Vec<usize> {
        let mut result = HashSet::new();
        result.extend(self.first_sequence(&symbols));

        if result.is_empty() || result.remove(&Symbol::Null.id()) {
            for symbol in self.follow(follow) {
                result.extend(self.first(symbol));
            }
        }

        util::to_sorted_vec(result)
    }

    /// Wraps a sequence of symbols in a new rule and returns its ID.
    pub fn wrap_symbols(&mut self, symbols: &[usize], follow: &[usize]) -> usize {
        let head = self.symbol(symbols[0]);

        let id = head.id();
        let mut name = head.name();

        loop {
            name += "'";

            // Check if the new symbol already exists.
            let rules = self
                .symbols
                .iter()
                .find(|symbol| symbol.name() == name)
                .map(|symbol| self.rules(symbol.id()));

            if rules.is_none() {
                break;
            }

            // If a rule with the same body already exists, return its ID.
            if let Some(rule) = rules.unwrap().iter().find(|rule| rule.body == symbols) {
                return rule.id;
            }
        }

        // Create the wrapper symbol and rule.
        let symbol_id = self.symbols.len();
        let symbol = Symbol::NonTerminal(symbol_id, name);

        let rule_id = self.rules.len();
        let mut rule = Rule::new(rule_id, symbol_id, vec![Symbol::End.id()], follow.to_vec());
        rule.body.extend_from_slice(symbols);

        self.symbols.push(symbol);
        self.rules.push(rule);
        self.symbol_rules.insert(symbol_id, vec![rule_id]);

        // Copy FIRST and FOLLOW sets.
        let mut first = self.first.borrow_mut();
        if let Some(set) = first.get(&id).cloned() {
            first.insert(symbol_id, set);
        }

        let mut follow = self.follow.borrow_mut();
        if let Some(set) = follow.get(&id).cloned() {
            follow.insert(symbol_id, set);
        }

        rule_id
    }

    /// Returns a symbol matching the specified text. The second return value
    /// indicates whether the symbol is a full or a partial match.
    ///
    /// This method returns the first full match if it exists,
    /// otherwise it returns the first partial match.
    pub fn find_symbol(&self, text: &str) -> Option<(usize, bool)> {
        let mut symbol = None;

        for (id, regex) in &self.tokens {
            let captures = match regex.captures(text) {
                Some(captures) => captures,
                None => continue,
            };

            // If the last capture group is None or empty, the match is partial.
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

    /// Caches the FIRST set for the specified symbol.
    fn cache_first(&self, symbol: usize, first: &HashSet<usize>) -> Vec<usize> {
        let first = util::to_sorted_vec(first.clone());
        let mut cache = self.first.borrow_mut();
        cache.insert(symbol, first.clone());
        first
    }
}

impl Display for Grammar {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rules = util::as_string(&self.rules, self, "\n");
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
            Self::NoSymbol(symbol) => write!(f, "Symbol {} does not exist", symbol),
            Self::Unreachable(symbol) => write!(f, "Symbol {} is unreachable", symbol),
            Self::LeftRecursive(symbol) => write!(f, "Symbol {} is left recursive", symbol),
            Self::NotRealizable(symbol) => write!(f, "Symbol {} is not realizable", symbol),
        }
    }
}

impl error::Error for Error {}
