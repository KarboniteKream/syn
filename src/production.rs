use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Production {
    pub symbols: Vec<Symbol>,
    pub terminals: Vec<Symbol>,
}

impl Production {
    pub fn new(symbols: Vec<Symbol>) -> Production {
        let terminals = symbols
            .iter()
            .filter(|symbol| symbol.terminal)
            .map(|symbol| symbol.clone())
            .collect();

        Production {
            symbols: symbols,
            terminals: terminals,
        }
    }
}
