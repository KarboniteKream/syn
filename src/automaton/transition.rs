use crate::grammar::Grammar;
use crate::util::AsString;

/// The `Transition` struct describes an arbitrary transition in the automaton.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Transition<T> {
    pub from: T,
    pub to: T,
    pub symbol: usize,
}

impl<T> Transition<T> {
    /// Constructs a new state or item transition.
    pub fn new(from: T, to: T, symbol: usize) -> Transition<T> {
        Transition { from, to, symbol }
    }
}

/// `StateTransition` describes a transition between two states in the automaton.
pub type StateTransition = Transition<usize>;

impl AsString for StateTransition {
    fn string(&self, grammar: &Grammar) -> String {
        let symbol = grammar.symbol(self.symbol);
        format!("{}, {} → {}", self.from, symbol, self.to)
    }
}

/// `ItemTransition` describes a transition between two items in the automaton.
pub type ItemTransition = Transition<(usize, usize)>;

impl AsString for ItemTransition {
    fn string(&self, grammar: &Grammar) -> String {
        let symbol = grammar.symbol(self.symbol);

        format!(
            "({}, {}), {} → ({}, {})",
            self.from.0, self.from.1, symbol, self.to.0, self.to.1
        )
    }
}
