use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Symbol {
    NonTerminal(String),
    Null,
    Terminal(String),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Symbol::NonTerminal(name) => write!(f, "{}", name),
            Symbol::Null => write!(f, "ϵ"),
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
            Symbol::Null => true,
            Symbol::Terminal(_) => true,
            _ => false,
        }
    }
}
