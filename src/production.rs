use crate::symbol::Symbol;

#[derive(Debug)]
pub struct Production {
    pub symbols: Vec<Symbol>,
    pub nonterminals: Vec<Symbol>,
}

impl Production {
    pub fn new(symbols: Vec<Symbol>) -> Production {
        let nonterminals = symbols
            .iter()
            .filter(|symbol| !symbol.terminal)
            .map(|symbol| symbol.clone())
            .collect();

        Production {
            symbols: symbols,
            nonterminals: nonterminals,
        }
    }
}
