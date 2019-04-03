use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use crate::symbol::Symbol;

#[derive(Clone, Eq, Hash, Debug, PartialEq)]
pub struct Rule {
    pub head: Symbol,
    pub body: Vec<Symbol>,
}

impl Rule {
    pub fn new(head: Symbol, body: Vec<Symbol>) -> Rule {
        Rule { head, body }
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

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let body = self
            .body
            .iter()
            .map(Symbol::to_string)
            .collect::<Vec<String>>()
            .join(" ");

        write!(f, "{} → {}", self.head, body)
    }
}
