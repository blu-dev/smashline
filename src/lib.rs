#![feature(proc_macro_hygiene)]

pub use smashline_macro::*;

#[macro_export]
macro_rules! install_hooks {
    ($($fn:ident),* $(,)?) => {
        $(
            smashline::install_hook!($fn);
        )*
    }
}

pub enum StaticSymbol {
    Resolved(usize),
    Unresolved(&'static str)
}

extern "Rust" {
    pub fn replace_symbol(module: &str, symbol: &str, replace: *const extern "C" fn(), original: Option<&'static mut *const extern "C" fn()>);
    pub fn replace_static_symbol(symbol: StaticSymbol, replace: *const extern "C" fn(), original: Option<&'static mut *const extern "C" fn()>);
}