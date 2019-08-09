use std::fmt::{self, Display, Formatter};

use crate::symbol::Symbol;

use super::span::Span;

/// The `Token` struct describes an element in the input file.
#[derive(Clone, Debug)]
pub struct Token {
    pub symbol: usize,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(symbol: usize, lexeme: String, span: Span) -> Token {
        Token {
            symbol,
            lexeme,
            span,
        }
    }

    pub fn end() -> Token {
        Token {
            symbol: Symbol::End.id(),
            lexeme: Symbol::End.name(),
            span: Span::default(),
        }
    }

    pub fn null() -> Token {
        Token {
            symbol: Symbol::Null.id(),
            lexeme: Symbol::Null.name(),
            span: Span::default(),
        }
    }

    /// Returns the last character of the lexeme.
    pub fn last(&self) -> Option<char> {
        self.lexeme.chars().last()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\"{}\" @ {} [{}]", self.lexeme, self.span, self.symbol)
    }
}
