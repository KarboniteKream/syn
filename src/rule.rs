use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::symbol::Symbol;
use crate::util;

#[derive(Clone, Debug, Ord, PartialOrd)]
pub struct Rule {
    pub id: usize,
    pub head: Symbol,
    pub body: Vec<Symbol>,
}

impl Rule {
    pub fn new(id: usize, head: Symbol, body: Vec<Symbol>) -> Rule {
        Rule { id, head, body }
    }

    pub fn first(&self) -> &Symbol {
        &self.body[0]
    }

    pub fn nonterminals(&self) -> HashSet<&Symbol> {
        self.body
            .iter()
            .filter(|symbol| symbol.is_nonterminal())
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

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let body = util::to_string(self.body.iter(), " ");
        write!(f, "({}) {} → {}", self.id, self.head, body)
    }
}
