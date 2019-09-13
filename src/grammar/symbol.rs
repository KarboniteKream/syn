use std::fmt::{self, Display, Formatter};

/// The `Symbol` enum describes an element of a grammar rule.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    /// Start of the parse stack.
    Start,

    /// End of the parse stack.
    End,

    /// An empty symbol.
    Null,

    /// A nonterminal symbol, which is replaced with a set of terminal symbols.
    NonTerminal(usize, String),

    /// A terminal symbol, indicating a token in the input file.
    Terminal(usize, String),
}

impl Symbol {
    /// Returns a list of internal symbols.
    pub fn internal() -> Vec<Symbol> {
        vec![Self::Start, Self::End, Self::Null]
    }

    /// Returns the ID of the symbol.
    pub fn id(&self) -> usize {
        match self {
            Self::Start => 0,
            Self::End => 1,
            Self::Null => 2,
            Self::NonTerminal(id, _) | Self::Terminal(id, _) => *id,
        }
    }

    /// Returns the name of the symbol.
    pub fn name(&self) -> String {
        match self {
            Self::Start => "^".to_owned(),
            Self::End => "$".to_owned(),
            Self::Null => "ϵ".to_owned(),
            Self::NonTerminal(_, name) => name.clone(),
            Self::Terminal(_, name) => {
                if name.contains('\'') {
                    format!("\"{}\"", name)
                } else {
                    format!("'{}'", name)
                }
            }
        }
    }

    /// Returns `true` if the symbol is terminal.
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::NonTerminal(..) => false,
            _ => true,
        }
    }

    /// Returns `true` if the symbol is nonterminal.
    pub fn is_nonterminal(&self) -> bool {
        match self {
            Self::NonTerminal(..) => true,
            _ => false,
        }
    }

    /// Returns `true` if the symbol is internal.
    pub fn is_internal(&self) -> bool {
        match self {
            Self::NonTerminal(..) | Self::Terminal(..) => false,
            _ => true,
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
