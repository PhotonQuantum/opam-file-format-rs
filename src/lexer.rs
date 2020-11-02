use std::{char, u32};

use lazy_static::lazy_static;
use logos::{Lexer, Logos};
use regex::Regex;

#[derive(Logos, Debug, PartialEq)]
enum EscapeToken {
    #[regex(r"(\r?\n) *", | lex | lex.slice().len())]
    EOL(usize),
    #[regex(r#"["'\\n\r\t ]"#, | lex | lex.slice().parse())]
    #[regex(r"\d\d\d", char_from_dec)]
    #[regex(r"x[0-9a-fA-F][0-9a-fA-F]", char_from_hex)]
    CHAR(char),
    #[error]
    Error,
}

macro_rules! enum_string_token {
    ($enum_name:ident, $eos_token:expr) => {
        #[derive(Logos, Debug, PartialEq)]
        enum $enum_name {
            #[token($eos_token)]
            EOS,
            #[token("\\", parse_escape)]
            #[regex(r"\r?\n", | _ | '\n')]
            #[regex(r"[\s\S]", | lex | lex.slice().parse())]
            CHAR(char),
            #[error]
            Error,
        }
    };
}

enum_string_token!(StringToken, "\"");
enum_string_token!(StringTripleToken, "\"\"\"");

#[derive(Logos, Debug, PartialEq)]
enum CommentToken {
    #[token("(*")]
    LPAR,
    #[token("*)")]
    RPAR,
    #[regex(r"[\s\S]")]
    SKIP,
    #[error]
    Error,
}

#[derive(Debug, PartialEq)]
pub enum Relop {
    Eq,
    Neq,
    Geq,
    Gt,
    Leq,
    Lt,
}

#[derive(Debug, PartialEq)]
pub enum Pfxop {
    Not,
    Defined,
}

#[derive(Debug, PartialEq)]
pub enum Envop {
    Eq,
    PlusEq,
    EqPlus,
    EqPlusEq,
    ColonEq,
    EqColon,
}

fn parse_relop<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<Relop> {
    match lex.slice() {
        "=" => Some(Relop::Eq),
        "!=" => Some(Relop::Neq),
        ">=" => Some(Relop::Geq),
        ">" => Some(Relop::Gt),
        "<=" => Some(Relop::Leq),
        "<" => Some(Relop::Lt),
        "~" => Some(Relop::Geq),
        _ => None,
    }
}

fn parse_pfxop<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<Pfxop> {
    match lex.slice() {
        "!" => Some(Pfxop::Not),
        "?" => Some(Pfxop::Defined),
        _ => None,
    }
}

fn parse_envop<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<Envop> {
    match lex.slice() {
        "=" => Some(Envop::Eq),
        "+=" => Some(Envop::PlusEq),
        "=+" => Some(Envop::EqPlus),
        "=+=" => Some(Envop::EqPlusEq),
        ":=" => Some(Envop::ColonEq),
        "=:" => Some(Envop::EqColon),
        _ => None,
    }
}

fn char_from_dec(lex: &mut Lexer<EscapeToken>) -> char {
    char::from_u32(lex.slice().to_string().parse::<u32>().unwrap()).unwrap()
}

fn char_from_hex(lex: &mut Lexer<EscapeToken>) -> char {
    char::from_u32(u32::from_str_radix(&lex.slice()[1..], 16).unwrap()).unwrap()
}

fn parse_escape<'a, T>(lex: &mut Lexer<'a, T>) -> Option<char>
where
    T: Logos<'a, Source = str>,
{
    let remainder = lex.remainder();
    let mut escape_lexer: Lexer<EscapeToken> = EscapeToken::lexer(remainder);
    let token = escape_lexer.next();
    if let None = token {
        return None;
    };
    match token.unwrap() {
        EscapeToken::EOL(len) => {
            lex.bump(len);
            Some('\n')
        }
        EscapeToken::CHAR(char) => {
            lex.bump(1);
            Some(char)
        }
        EscapeToken::Error => None,
    }
}

macro_rules! fn_parse_string {
    ($func_name:ident, $token_type: ident) => {
        fn $func_name<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<String> {
            let remainder = lex.remainder();
            let mut string_lexer: Lexer<$token_type> = $token_type::lexer(remainder);
            let mut result = String::new();
            loop {
                let token = string_lexer.next();
                if let None = token {
                    return None;
                };
                match token.unwrap() {
                    $token_type::EOS => break,
                    $token_type::CHAR(char) => result.push(char),
                    $token_type::Error => return None,
                };
            }
            lex.bump(string_lexer.span().end);
            Some(String::from(result))
        }
    };
}

fn_parse_string!(parse_string, StringToken);
fn_parse_string!(parse_string_triple, StringTripleToken);

fn parse_comment<'a>(lex: &mut Lexer<'a, Token<'a>>) -> bool {
    let remainder = lex.remainder();
    let mut comment_lex: Lexer<CommentToken> = CommentToken::lexer(remainder);
    let mut counter = 1;
    loop {
        let token = comment_lex.next();
        if let None = token {
            break false;
        }
        match token.unwrap() {
            CommentToken::LPAR => counter += 1,
            CommentToken::RPAR => {
                if counter > 1 {
                    counter -= 1;
                } else {
                    lex.bump(comment_lex.span().end);
                    break true;
                }
            }
            CommentToken::SKIP => (),
            CommentToken::Error => break false,
        }
    }
}

fn fix_ident_suffix<'a>(lex: &mut Lexer<'a, Token<'a>>) -> &'a str {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r":([a-zA-Z]|\d|[_-])*[a-zA-Z]([a-zA-Z]|\d|[_-])*").unwrap();
    }
    let remainder = lex.remainder();
    if let Some(pos) = RE.find(remainder) {
        if pos.start() == 0 {
            lex.bump(pos.end());
        }
    };
    lex.slice()
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'a> {
    #[token(":")]
    COLON,
    #[token("{")]
    LBRACE,
    #[token("}")]
    RBRACE,
    #[token("[")]
    LBRACKET,
    #[token("]")]
    RBRACKET,
    #[token("(")]
    LPAR,
    #[token(")")]
    RPAR,
    #[token("\"", parse_string)]
    #[token("\"\"\"", parse_string_triple)]
    STRING(String),
    #[token("(*", parse_comment)]
    #[regex(r"# [^\n]*")]
    COMMENT,
    #[token("true", | lex | lex.slice().parse())]
    #[token("false", | lex | lex.slice().parse())]
    BOOL(bool),
    #[regex(r"-?[0-9_]+", | lex | lex.slice().parse(), priority = 2)]
    INT(i64),
    #[regex(r"(([a-zA-Z]|\d|[_-])*[a-zA-Z]([a-zA-Z]|\d|[_-])*|_)(\+(([a-zA-Z]|\d|[_-])*[a-zA-Z]([a-zA-Z]|\d|[_-])*|_))*", fix_ident_suffix)]
    IDENT(&'a str),
    #[regex(r"(!?=|[<>]=?|~)", parse_relop)]
    RELOP(Relop),
    #[token("&")]
    AND,
    #[token("|")]
    OR,
    #[regex(r"!|\?", parse_pfxop)]
    PFXOP(Pfxop),
    #[regex(r"[\+?]=|=[\+?]=?", parse_envop)]
    ENVOP(Envop),
    #[regex(r"[ \t\r\n]", logos::skip)]
    SKIP,
    #[error]
    Error,
}

pub fn lex(input: &str) -> Vec<Token> {
    let lexer: Lexer<Token> = Token::lexer(input);
    lexer.spanned().map(|(token, _)| token).collect()
}
