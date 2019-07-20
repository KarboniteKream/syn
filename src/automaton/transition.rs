use crate::grammar::Grammar;
use crate::util::AsString;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Transition<T> {
    pub from: T,
    pub to: T,
    pub symbol: usize,
}

impl<T> Transition<T> {
    pub fn new(from: T, to: T, symbol: usize) -> Transition<T> {
        Transition { from, to, symbol }
    }
}

pub type StateTransition = Transition<usize>;

impl AsString for StateTransition {
    fn string(&self, grammar: &Grammar) -> String {
        let symbol = grammar.symbol(self.symbol);
        format!("{} → {} {}", self.from, self.to, symbol)
    }
}

pub type ItemTransition = Transition<(usize, usize)>;

impl AsString for ItemTransition {
    fn string(&self, grammar: &Grammar) -> String {
        let symbol = grammar.symbol(self.symbol);

        format!(
            "({}, {}) → ({}, {}) {}",
            self.from.0, self.from.1, self.to.0, self.to.1, symbol
        )
    }
}
