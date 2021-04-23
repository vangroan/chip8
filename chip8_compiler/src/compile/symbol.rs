use std::collections::BTreeMap;

pub struct Symbol {
    pub name: String,
    pub realm: SymbolRealm,
    pub ty: SymbolType,
}

pub enum SymbolType {
    U8,
    Bool,
}

pub enum SymbolRealm {
    /// Constants have a value fixed at compile time.
    Const,
    /// Variables can change value during runtime.
    Var,
}

pub enum SymbolScope {
    Global,
    Local,
    Parameter,
}

#[derive(Default)]
pub struct SymbolTable {
    symbols: BTreeMap<String, Symbol>,
}

impl SymbolTable {
    #[inline]
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    #[inline]
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    #[inline]
    pub fn contains_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}
