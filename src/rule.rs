use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::grammar::Grammar;
use crate::symbol::Symbol;
use crate::util::AsString;

#[derive(Clone, Debug)]
pub struct Rule {
    pub id: usize,
    pub head: usize,
    pub body: Vec<usize>,
}

impl Rule {
    pub fn new(id: usize, head: usize, body: Vec<usize>) -> Rule {
        Rule { id, head, body }
    }

    pub fn nonterminals(&self, symbols: &[Symbol]) -> HashSet<usize> {
        self.body
            .iter()
            .filter(|id| symbols[**id].is_nonterminal())
            .cloned()
            .collect()
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head && self.body == other.body
    }
}

impl Eq for Rule {}

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
        let body: Vec<String> = self
            .body
            .iter()
            .map(|id| grammar.symbol(*id).to_string())
            .collect();

        format!("({}) {} → {}", self.id, head, body.join(" "))
    }
}
