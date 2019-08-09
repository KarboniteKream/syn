use std::fmt::{self, Display, Formatter};

/// The `Action` enum in the ACTION table.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    /// Shift the automaton to the next state.
    Shift(usize),

    /// Reduce a grammar rule from the parse stack.
    Reduce(usize),

    /// Reduce a grammar rule and accept the parse stack.
    Accept(usize),
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Action::Shift(state) => write!(f, "s{}", state),
            Action::Reduce(rule) => write!(f, "r{}", rule),
            Action::Accept(rule) => write!(f, "acc{}", rule),
        }
    }
}
