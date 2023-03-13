use std::num::NonZeroU32;

use once_cell::sync::Lazy;
use symbol_table::{Symbol, SymbolTable};

static STRINGS: Lazy<SymbolTable> = Lazy::new(SymbolTable::new);

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

impl From<NonZeroU32> for IString {
    fn from(value: NonZeroU32) -> Self {
        let sym = value.into();
        IString(sym)
    }
}

impl From<IString> for String {
    fn from(value: IString) -> Self {
        let IString(sym) = value;
        STRINGS.resolve(sym).into()
    }
}

impl From<IString> for &'static str {
    fn from(value: IString) -> Self {
        let IString(sym) = value;
        STRINGS.resolve(sym)
    }
}

impl From<&IString> for &'static str {
    fn from(value: &IString) -> Self {
        let IString(sym) = value;
        STRINGS.resolve(*sym)
    }
}
