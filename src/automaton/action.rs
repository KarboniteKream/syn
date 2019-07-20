use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Action::Shift(state) => write!(f, "s{}", state),
            Action::Reduce(rule) => write!(f, "r{}", rule),
            Action::Accept => write!(f, "acc"),
        }
    }
}
