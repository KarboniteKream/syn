use std::collections::HashSet;

use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Rule {
    pub symbols: Vec<Symbol>,
}

impl Rule {
    pub fn new(symbols: Vec<Symbol>) -> Rule {
        Rule { symbols: symbols }
    }

    pub fn nonterminals(&self) -> HashSet<&Symbol> {
        self.symbols
            .iter()
            .filter(|symbol| !symbol.is_terminal())
            .collect()
    }
}
