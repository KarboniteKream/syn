use std::collections::HashMap;

use crate::production::Production;
use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Grammar {
    pub start_symbol: Symbol,
    pub productions: HashMap<Symbol, Vec<Production>>,
}

impl Grammar {
    pub fn new(start_symbol: String) -> Grammar {
        Grammar {
            start_symbol: Symbol {
                name: start_symbol,
                terminal: false,
            },
            productions: HashMap::new(),
        }
    }
}
