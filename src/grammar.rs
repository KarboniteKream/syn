use std::collections::HashMap;

use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Grammar {
    pub start_symbol: Symbol,
    pub productions: HashMap<Symbol, Vec<Vec<Symbol>>>,
}

impl Grammar {
    pub fn new(start_symbol: String) -> Grammar {
        Grammar {
            start_symbol: Symbol {
                name: start_symbol,
                terminal: true,
            },
            productions: HashMap::new(),
        }
    }
}
