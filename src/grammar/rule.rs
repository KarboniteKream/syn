use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::grammar::{Grammar, Position, Symbol};
use crate::util::AsString;

/// The `Rule` struct describes a grammar rule.
///
/// To ensure the relation between `Eq` and `Ord`, the fields
/// in the struct _must_ be unique for a specific `id`.
#[derive(Clone, Debug, Eq)]
pub struct Rule {
    pub id: usize,
    pub head: usize,
    pub body: Vec<usize>,
    pub follow: Vec<usize>,
}

impl Rule {
    /// Constructs a new grammar rule.
    pub fn new(id: usize, head: usize, body: Vec<usize>, follow: Vec<usize>) -> Rule {
        Rule {
            id,
            head,
            body,
            follow,
        }
    }

    /// Returns the set of nonterminal symbols in the rule body.
    pub fn nonterminals(&self, symbols: &[Symbol]) -> HashSet<usize> {
        self.body
            .iter()
            .filter(|&&id| symbols[id].is_nonterminal())
            .cloned()
            .collect()
    }

    /// Returns the tail from the specified index.
    pub fn tail(&self, idx: usize) -> &[usize] {
        &self.body[idx..]
    }

    /// Returns the symbol positions from the specified index.
    pub fn positions(&self, idx: usize) -> Vec<(usize, Position)> {
        self.body
            .iter()
            .enumerate()
            .skip(idx)
            .map(|(idx, &symbol)| (symbol, (self.id, idx)))
            .collect()
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head && self.body == other.body
    }
}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.head.hash(state);
        self.body.hash(state);
    }
}

impl AsString for Rule {
    fn string(&self, grammar: &Grammar) -> String {
        let head = grammar.symbol(self.head);

        let body = self
            .body
            .iter()
            .map(|&id| grammar.symbol(id).to_string())
            .collect::<Vec<String>>()
            .join(" ");

        format!("({}) {} → {}", self.id, head, body)
    }
}
