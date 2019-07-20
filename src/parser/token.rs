use std::fmt::{self, Display, Formatter};

use super::span::Span;

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
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}] \"{}\" @ {}", self.symbol, self.lexeme, self.span)
    }
}
