use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
    Start,
    End,
    Null,
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        match self {
            Symbol::NonTerminal(_) => false,
            _ => true,
        }
    }

    pub fn is_nonterminal(&self) -> bool {
        !self.is_terminal()
    }

    pub fn is_builtin(&self) -> bool {
        match self {
            Symbol::NonTerminal(_) | Symbol::Terminal(_) => false,
            _ => true,
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
            Symbol::Start => write!(f, "^"),
            Symbol::End => write!(f, "$"),
            Symbol::Null => write!(f, "ϵ"),
        }
    }
}
