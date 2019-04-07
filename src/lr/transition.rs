use std::fmt::{self, Display, Formatter};

use crate::symbol::Symbol;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Transition<T> {
    pub from: T,
    pub to: T,
    pub symbol: Symbol,
}

impl<T> Transition<T> {
    pub fn new(from: T, to: T, symbol: Symbol) -> Transition<T> {
        Transition { from, to, symbol }
    }
}

pub type StateTransition = Transition<usize>;

impl Display for StateTransition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} → {} {}", self.from, self.to, self.symbol)
    }
}

pub type ItemTransition = Transition<(usize, usize)>;

impl Display for ItemTransition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}) → ({}, {}) {}",
            self.from.0, self.from.1, self.to.0, self.to.1, self.symbol
        )
    }
}
