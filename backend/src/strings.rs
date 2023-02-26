use once_cell::sync::Lazy;
use symbol_table::{Symbol, SymbolTable};

static STRINGS: Lazy<SymbolTable> = Lazy::new(|| SymbolTable::new());

#[derive(Hash, Eq, PartialEq, Clone, Copy, Ord, PartialOrd)]
pub struct IString(Symbol);

impl std::fmt::Debug for IString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: &str = self.into();
        f.write_str(str)
    }
}

impl From<&str> for IString {
    fn from(value: &str) -> Self {
        let sym = STRINGS.intern(value);
        IString(sym)
    }
}

impl From<&String> for IString {
    fn from(value: &String) -> Self {
        let sym = STRINGS.intern(value);
        IString(sym)
    }
}

impl Into<&'static str> for &IString {
    fn into(self) -> &'static str {
        let IString(sym) = self;
        STRINGS.resolve(*sym)
    }
}

impl Into<&'static str> for IString {
    fn into(self) -> &'static str {
        let IString(sym) = self;
        STRINGS.resolve(sym)
    }
}

impl Into<String> for IString {
    fn into(self) -> String {
        let IString(sym) = self;
        STRINGS.resolve(sym).into()
    }
}
