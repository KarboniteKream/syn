use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use crate::symbol::Symbol;

#[derive(Clone, Debug)]
pub struct Rule {
    head: Symbol,
    pub body: Vec<Symbol>,
}

impl Rule {
    pub fn new(head: Symbol, body: Vec<Symbol>) -> Rule {
        Rule { head, body }
    }

    pub fn nonterminals(&self) -> HashSet<&Symbol> {
        self.body
            .iter()
            .filter(|symbol| !symbol.is_terminal())
            .collect()
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let body = self
            .body
            .iter()
            .map(|symbol| symbol.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        write!(f, "{} → {}", self.head, body)
    }
}
