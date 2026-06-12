//! Symbol table for top-level Peps declarations.

use std::collections::HashMap;

use crate::types::Type;

/// Mapping from top-level Peps variable names to their inferred static types.
///
/// Local block bindings, such as loop variables, are tracked by the semantic
/// checker instead of being stored here.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Type>,
}

impl SymbolTable {
    /// Create an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether a top-level declaration exists for `name`.
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get the inferred type for a top-level declaration.
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.symbols.get(name)
    }

    /// Insert or replace a top-level declaration.
    pub fn insert(&mut self, name: String, ty: Type) {
        self.symbols.insert(name, ty);
    }
}
