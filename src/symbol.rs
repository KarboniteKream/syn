use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    Start,
    End,
    Null,
    NonTerminal(usize, String),
    Terminal(usize, String),
}

impl Symbol {
    pub fn builtin() -> Vec<Symbol> {
        vec![Symbol::Start, Symbol::End, Symbol::Null]
    }

    pub fn id(&self) -> usize {
        match self {
            Symbol::Start => 0,
            Symbol::End => 1,
            Symbol::Null => 2,
            Symbol::NonTerminal(id, _) | Symbol::Terminal(id, _) => *id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Symbol::Start => "^".to_owned(),
            Symbol::End => "$".to_owned(),
            Symbol::Null => "ϵ".to_owned(),
            Symbol::NonTerminal(_, name) => name.clone(),
            Symbol::Terminal(_, name) => {
                if name.contains('\'') {
                    format!("\"{}\"", name)
                } else {
                    format!("'{}'", name)
                }
            }
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            Symbol::NonTerminal(..) => false,
            _ => true,
        }
    }

    pub fn is_nonterminal(&self) -> bool {
        !self.is_terminal()
    }

    pub fn is_builtin(&self) -> bool {
        match self {
            Symbol::NonTerminal(..) | Symbol::Terminal(..) => false,
            _ => true,
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
