use std::collections::{HashMap, HashSet};

use crate::production::Production;
use crate::symbol::{Symbol, SymbolType};

#[derive(Debug)]
pub struct Grammar {
    pub start_symbol: Symbol,
    pub productions: HashMap<Symbol, Vec<Production>>,
}

impl Grammar {
    pub fn new(start_symbol: &str) -> Grammar {
        Grammar {
            start_symbol: Symbol::new(start_symbol, SymbolType::NonTerminal),
            productions: HashMap::new(),
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        let mut nonterminals: HashSet<&Symbol> = self
            .productions
            .values()
            .flat_map(|productions| productions.iter().flat_map(Production::nonterminals))
            .collect();

        let start_symbol = Symbol::new("S", SymbolType::NonTerminal);
        nonterminals.insert(&start_symbol);

        for symbol in self.productions.keys() {
            if !nonterminals.contains(symbol) {
                return Err(format!("Symbol '{}' is unreachable", symbol));
            }
        }

        // TODO: Detect left recursion.

        let mut completeness: HashMap<&Symbol, bool> = self
            .productions
            .iter()
            .map(|(symbol, productions)| {
                (
                    symbol,
                    productions
                        .iter()
                        .any(|production| production.symbols.iter().all(Symbol::is_terminal)),
                )
            })
            .collect();

        loop {
            let changes: HashMap<&Symbol, bool> = completeness
                .iter()
                .filter(|(_, &complete)| !complete)
                .map(|(&symbol, _)| {
                    let nonterminals: HashSet<&Symbol> = self
                        .productions
                        .get(symbol)
                        .unwrap()
                        .iter()
                        .flat_map(Production::nonterminals)
                        .collect();

                    (
                        symbol,
                        nonterminals
                            .iter()
                            .any(|symbol| *completeness.get(symbol).unwrap()),
                    )
                })
                .filter(|(_, complete)| *complete)
                .collect();

            if changes.is_empty() {
                break;
            }

            completeness.extend(changes);
        }

        if let Some((&symbol, _)) = completeness.iter().find(|(_, &complete)| !complete) {
            return Err(format!("Symbol '{}' is not complete", symbol));
        }

        Ok(())
    }
}
