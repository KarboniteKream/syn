use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    typ: SymbolType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolType {
    Epsilon,
    NonTerminal,
    Terminal,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.name == other.name
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Symbol {
    pub fn new(name: String, typ: SymbolType) -> Symbol {
        Symbol { name, typ }
    }

    pub fn is_terminal(&self) -> bool {
        self.typ == SymbolType::Terminal || self.typ == SymbolType::Epsilon
    }

    pub fn is_epsilon(&self) -> bool {
        self.typ == SymbolType::Epsilon
    }
}
