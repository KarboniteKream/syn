use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    NonTerminal(String),
    Terminal(String),
    Null,
    Delimiter,
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::NonTerminal(name) | Symbol::Terminal(name) => name.as_str(),
            Symbol::Null => "ϵ",
            Symbol::Delimiter => "$",
        }
    }

    pub fn is_nonterminal(&self) -> bool {
        match self {
            Symbol::NonTerminal(_) => true,
            _ => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        !self.is_nonterminal()
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
            Symbol::Delimiter => write!(f, "$"),
        }
    }
}
