use std::{char, u32};

use logos::{Lexer, Logos};
use regex::Regex;

#[derive(Logos, Debug, PartialEq)]
enum EscapeToken {
    #[regex(r"(\r?\n) *", |lex|lex.slice().parse())]
    EOL(String),
    #[regex(r#"["'\\n\r\t ]"#, | lex | lex.slice().parse())]
    #[regex(r"\d\d\d", char_from_dec)]
    #[regex(r"x[0-9a-fA-F][0-9a-fA-F]", char_from_hex)]
    CHAR(char),
    #[error]
    Error,
}

#[derive(Logos, Debug, PartialEq)]
enum StringToken {
    #[token("\"")]
    EOS,
    #[token("\\", parse_escape_string)]
    #[regex(r"\r?\n", | _ | '\n')]
    #[regex(r"[\s\S]", | lex | lex.slice().parse())]
    CHAR(char),
    #[error]
    Error,
}

#[derive(Logos, Debug, PartialEq)]
enum StringTripleToken {
    #[token("\"\"\"")]
    EOS,
    #[token("\\", parse_escape_string_triple)]
    #[regex(r"\r?\n", | _ | '\n')]
    #[regex(r"[\s\S]", | lex | lex.slice().parse())]
    CHAR(char),
    #[error]
    Error,
}

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

fn char_from_dec(lex: &mut Lexer<EscapeToken>) -> char {
    char::from_u32(lex.slice().to_string().parse::<u32>().unwrap()).unwrap()
}

fn char_from_hex(lex: &mut Lexer<EscapeToken>) -> char {
    char::from_u32(u32::from_str_radix(&lex.slice()[1..], 16).unwrap()).unwrap()
}

fn parse_escape_string(lex: &mut Lexer<StringToken>) -> char {
    let remainder = lex.remainder();
    let mut escape_lexer: Lexer<EscapeToken> = EscapeToken::lexer(remainder);
    let token = escape_lexer.next();
    match token.expect("illegal escape sequence") {
        EscapeToken::EOL(str) => {
            lex.bump(str.len());
            '\n'
        },
        EscapeToken::CHAR(char) => {
            lex.bump(1);
            char
        },
        EscapeToken::Error => panic!("illegal escape sequence")
    }
}

fn parse_escape_string_triple(lex: &mut Lexer<StringTripleToken>) -> char {
    let remainder = lex.remainder();
    let mut escape_lexer: Lexer<EscapeToken> = EscapeToken::lexer(remainder);
    let token = escape_lexer.next();
    match token.expect("illegal escape sequence") {
        EscapeToken::EOL(str) => {
            lex.bump(str.len());
            '\n'
        },
        EscapeToken::CHAR(char) => {
            lex.bump(1);
            char
        },
        EscapeToken::Error => panic!("illegal escape sequence")
    }
}

fn parse_relop(lex: &mut Lexer<Token>) -> Relop {
    match lex.slice() {
        "=" => Relop::Eq,
        "!=" => Relop::Neq,
        ">=" => Relop::Geq,
        ">" => Relop::Gt,
        "<=" => Relop::Leq,
        "<" => Relop::Lt,
        "~" => Relop::Geq,
        x => panic!("{} is not a valid comparison operator", x)
    }
}

fn parse_pfxop(lex: &mut Lexer<Token>) -> Pfxop {
    match lex.slice() {
        "!" => Pfxop::Not,
        "?" => Pfxop::Defined,
        x => panic!("{} is not a valid prefix operator", x)
    }
}

fn parse_envop(lex: &mut Lexer<Token>) -> Envop {
    match lex.slice() {
        "=" => Envop::Eq,
        "+=" => Envop::PlusEq,
        "=+" => Envop::EqPlus,
        "=+=" => Envop::EqPlusEq,
        ":=" => Envop::ColonEq,
        "=:" => Envop::EqColon,
        x => panic!("{} is not a valid environment update operator", x)
    }
}

fn parse_string(lex: &mut Lexer<Token>) -> String {
    let remainder = lex.remainder();
    let mut string_lexer: Lexer<StringToken> = StringToken::lexer(remainder);
    let mut result = String::new();
    loop {
        let token = string_lexer.next();
        match token.expect("unterminated string") {
            StringToken::EOS => break,
            StringToken::CHAR(char) => result.push(char),
            StringToken::Error => panic!("error when parsing string")
        };
    }
    lex.bump(string_lexer.span().end);
    String::from(result)
}

fn parse_string_triple(lex: &mut Lexer<Token>) -> String {
    let remainder = lex.remainder();
    let mut string_lexer: Lexer<StringTripleToken> = StringTripleToken::lexer(remainder);
    let mut result = String::new();
    loop {
        let token = string_lexer.next();
        match token.expect("unterminated string") {
            StringTripleToken::EOS => break,
            StringTripleToken::CHAR(char) => {
                result.push(char)
            },
            StringTripleToken::Error => panic!("error when parsing string")
        };
    }
    lex.bump(string_lexer.span().end);
    String::from(result)
}

fn parse_comment(lex: &mut Lexer<Token>) {
    let remainder = lex.remainder();
    let mut comment_lex: Lexer<CommentToken> = CommentToken::lexer(remainder);
    let mut counter = 1;
    loop {
        let token = comment_lex.next();
        match token.expect("unterminated comment") {
            CommentToken::LPAR => counter += 1,
            CommentToken::RPAR =>
                if counter > 1 {
                    counter -= 1;
                } else {
                    lex.bump(comment_lex.span().end);
                    break;
                },
            CommentToken::SKIP => (),
            CommentToken::Error => panic!("error when parsing comment"),
        }
    }
}

fn fix_ident_suffix(lex: &mut Lexer<Token>) -> String {
    let remainder = lex.remainder();
    let re = Regex::new(r":([a-zA-Z]|\d|[_-])*[a-zA-Z]([a-zA-Z]|\d|[_-])*").unwrap();
    if let Some(pos) = re.find(remainder) {
        if pos.start() == 0 {
            lex.bump(pos.end());
        }
    };
    String::from(lex.slice())
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
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
    IDENT(String),
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
