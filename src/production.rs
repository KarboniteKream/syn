use std::collections::HashSet;

use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Production {
    pub symbols: Vec<Symbol>,
}

impl Production {
    pub fn new(symbols: Vec<Symbol>) -> Production {
        Production { symbols: symbols }
    }

    pub fn nonterminals(&self) -> HashSet<&Symbol> {
        self.symbols
            .iter()
            .filter(|symbol| !symbol.is_terminal())
            .collect()
    }
}
