use std::collections::HashMap;

use crate::types::Type;

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Type>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.symbols.get(name)
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.symbols.insert(name, ty);
    }
}
