#![feature(proc_macro_hygiene)]
mod lexer;
mod parser;

use std::fs::File;
use std::io::Read;
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let mut file = File::open(args.next().unwrap()).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    let tokens = lexer::lex(&buffer);
    let output = parser::parse(tokens.into_iter());
    match output {
        Err((E, msg)) => {
            if let Some((token, span)) = E {
                println!("ERR\n{}\nToken: {:#?} Char: {}\n{}", &buffer[span.start-10 .. span.end + 10], token, &buffer[span.start .. span.end], msg);
            } else {
                println!("ERR");
            }
        },
        Ok(file) => println!("{:#?}", file)
    }
}
