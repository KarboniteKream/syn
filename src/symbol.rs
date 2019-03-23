use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
    Null,
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::NonTerminal(name) | Symbol::Terminal(name) => name.as_str(),
            Symbol::Null => "ϵ",
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            Symbol::Terminal(_) | Symbol::Null => true,
            _ => false,
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Symbol::NonTerminal(name) => write!(f, "{}", name),
            Symbol::Terminal(name) => {
                if name.contains('\'') {
                    write!(f, "\"{}\"", name)
                } else {
                    write!(f, "'{}'", name)
                }
            }
            Symbol::Null => write!(f, "ϵ"),
        }
    }
}
