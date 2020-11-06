use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::time::SystemTime;

use crate::utils::pretty_error;
use clap::{App, Arg};
use colored::*;

mod lexer;
mod parser;
mod printer;
mod utils;
use printer::{JsonPrinter};

fn main() {
    let matches = App::new("opam-file-format-rs")
        .version("0.1.0")
        .author("LightQuantum <self@lightquantum.me>")
        .about("Parser for the opam file syntax written in rust")
        .arg(Arg::with_name("INPUT")
            .help("Sets the opam file to be parsed")
            .required(true))
        .arg(Arg::with_name("benchmark")
            .short("b")
            .help("Benchmark mode. Takes in a list file, parse all opam files given, and report elapsed time"))
        .arg(Arg::with_name("json")
            .long("json")
            .short("j")
            .help("serialize to json instead of internal ast"))
        .get_matches();
    if matches.is_present("benchmark") {
        benchmark(
            matches.value_of("INPUT").unwrap(),
            matches.is_present("json"),
        )
    } else {
        single_file(
            matches.value_of("INPUT").unwrap(),
            matches.is_present("json"),
        )
    }
}

fn benchmark(filename: &str, json: bool) {
    let mut list_file = File::open(filename).unwrap();
    let mut list_buffer = String::new();
    list_file.read_to_string(&mut list_buffer).unwrap();

    println!("{}", "reading files into memory...".blue().bold());
    let files: Vec<(&str, String)> = list_buffer
        .lines()
        .map(|filename| {
            let mut file = File::open(filename).unwrap();
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            (filename, buffer)
        })
        .collect();

    println!("{}", "parsing files...".blue().bold());
    let now = SystemTime::now();
    files
        .iter()
        .map(|(filename, buffer)| {
            let tokens = lexer::lex(&buffer);
            if let Err(span) = tokens {
                pretty_error(&filename, &buffer, &span, "unexpected character");
                exit(1);
            }
            let output = parser::parse(tokens.unwrap().into_iter());
            match output {
                Err((e, msg)) => {
                    if let Some((_, span)) = e {
                        pretty_error(&filename, &buffer, &span, msg);
                    } else {
                        eprintln!("ERR");
                    }
                    exit(1);
                }
                Ok(ast) => ast,
            }
        })
        .for_each(if json {
            |ast| {
                JsonPrinter::new(&ast).to_string();
                drop(ast)
            }
        } else {
            drop
        });

    let elapsed_time = now.elapsed().unwrap().as_millis();
    println!(
        "{}",
        format!(
            "parsed {} files. elapsed {:.2} secs. speed: {:.2} files/sec",
            &files.len(),
            elapsed_time as f64 / 1000.0,
            (&files.len() * 1000) as f64 / elapsed_time as f64
        )
        .green()
        .bold()
    );
}

fn single_file(filename: &str, json: bool) {
    let mut file = File::open(filename).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    let tokens = lexer::lex(&buffer);
    if let Err(span) = tokens {
        pretty_error(&filename, &buffer, &span, "unexpected character");
        exit(1);
    }
    let output = parser::parse(tokens.unwrap().into_iter());
    match output {
        Err((e, msg)) => {
            if let Some((_, span)) = e {
                pretty_error(&filename, &buffer, &span, msg);
            } else {
                eprintln!("ERR");
            }
            exit(1);
        }
        Ok(ast) => {
            if json {
                println!("{}", JsonPrinter::new(&ast))
            } else {
                println!("{:#?}", ast)
            }
        }
    }
}
