#![deny(unsafe_code)]
#![allow(unused_braces)]
pub mod lexer;
pub mod parser;
pub mod printer;

pub use lexer::lex;
pub use parser::{OpamAST, parse};
pub use printer::JsonPrinter;