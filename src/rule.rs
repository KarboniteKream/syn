use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::grammar::Grammar;
use crate::symbol::Symbol;
use crate::util::AsString;

#[derive(Clone, Debug, Ord, PartialOrd)]
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
    fn eq(&self, other: &Rule) -> bool {
        self.head == other.head && self.body == other.body
    }
}

impl Eq for Rule {}

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
