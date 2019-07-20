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
        vec![Symbol::Start, Symbol::End, Symbol::Null]
    }

    /// Returns the ID of the symbol.
    pub fn id(&self) -> usize {
        match self {
            Symbol::Start => 0,
            Symbol::End => 1,
            Symbol::Null => 2,
            Symbol::NonTerminal(id, _) | Symbol::Terminal(id, _) => *id,
        }
    }

    /// Returns the name of the symbol.
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

    /// Returns `true` if the symbol is terminal.
    pub fn is_terminal(&self) -> bool {
        match self {
            Symbol::NonTerminal(..) => false,
            _ => true,
        }
    }

    /// Returns `true` if the symbol is nonterminal.
    pub fn is_nonterminal(&self) -> bool {
        !self.is_terminal()
    }

    /// Returns `true` if the symbol is internal.
    pub fn is_internal(&self) -> bool {
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
