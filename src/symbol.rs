use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Symbol {
    Epsilon,
    NonTerminal(String),
    Terminal(String),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Symbol::Epsilon => write!(f, "ϵ"),
            Symbol::NonTerminal(name) => write!(f, "{}", name),
            Symbol::Terminal(name) => {
                if name.contains('\'') {
                    write!(f, "\"{}\"", name)
                } else {
                    write!(f, "'{}'", name)
                }
            }
        }
    }
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        match self {
            Symbol::Terminal(_) | Symbol::Epsilon => true,
            _ => false,
        }
    }
}
