use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub terminal: bool,
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
