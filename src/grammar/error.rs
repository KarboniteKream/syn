use std::fmt::{self, Display, Formatter};

use crate::symbol::Symbol;

#[derive(Debug)]
pub enum Error<'a> {
    Completeness(&'a Symbol),
    Unreachable(&'a Symbol),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Completeness(symbol) => write!(f, "Symbol '{}' is not complete", symbol),
            Error::Unreachable(symbol) => write!(f, "Symbol '{}' is unreachable", symbol),
        }
    }
}
