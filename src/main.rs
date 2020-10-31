mod lexer;
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
    println!("{:#?}", tokens)
}
