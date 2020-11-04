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

#[derive(Debug, PartialEq)]
pub enum Logop {
    And,
    Or,
}

fn parse_relop(lex: &mut Lexer<Token>) -> Option<Relop> {
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

fn parse_pfxop(lex: &mut Lexer<Token>) -> Option<Pfxop> {
    match lex.slice() {
        "!" => Some(Pfxop::Not),
        "?" => Some(Pfxop::Defined),
        _ => None,
    }
}

fn parse_envop(lex: &mut Lexer<Token>) -> Option<Envop> {
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

fn parse_logop(lex: &mut Lexer<Token>) -> Option<Logop> {
    match lex.slice() {
        "&" => Some(Logop::And),
        "|" => Some(Logop::Or),
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
        fn $func_name(lex: &mut Lexer<Token>) -> Option<String> {
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

fn parse_comment(lex: &mut Lexer<Token>) -> bool {
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

macro_rules! generate_regex {
    ($token: ident, $fn_name: ident, $regex: expr) => {
        #[derive(Logos, Debug, PartialEq)]
        enum $token {
            #[regex($regex)]
            MATCH,
            #[error]
            MISS,
        }
        fn $fn_name(input: &str) -> Option<usize> {
            let mut lexer: Lexer<$token> = $token::lexer(input);
            let token = lexer.next()?;
            match token {
                $token::MATCH => Some(lexer.span().end),
                $token::MISS => None,
            }
        }
    };
}

generate_regex!(ReColonToken, re_colon, r":");
generate_regex!(ReLodashToken, re_lodash, r"_");
generate_regex!(RePlusToken, re_plus, r"\+");
generate_regex!(ReAlphaToken, re_alpha, r"[a-zA-Z]*");
generate_regex!(ReIdExAlphaToken, re_id_ex_alpha, r"[\d_-]*");

fn parse_id(input: &str) -> Option<usize> {
    let mut contains_alpha = false;
    let mut remaining_pos = 0;
    loop {
        let mut matched = false;
        if let Some(pos) = re_alpha(&input[remaining_pos..]) {
            remaining_pos += pos;
            contains_alpha = true;
            matched = true;
        }
        if let Some(pos) = re_id_ex_alpha(&input[remaining_pos..]) {
            remaining_pos += pos;
            matched = true;
        }
        if !matched {
            break;
        }
    }
    if contains_alpha {
        Some(remaining_pos)
    } else {
        None
    }
}

fn parse_id_lodash(input: &str) -> Option<usize> {
    if let Some(pos) = parse_id(input) {
        Some(pos)
    } else if let Some(pos) = re_lodash(input) {
        Some(pos)
    } else {
        None
    }
}

fn match_ident(lex: &mut Lexer<Token>) -> Option<String> {
    // group1: (id|_)
    if let Some(pos) = parse_id(lex.remainder()) {
        lex.bump(pos);
    } else {
        return None;
    }

    // group2: (+ (id|_))*
    let remainder = lex.remainder();
    let mut remaining_pos = 0;
    let mut is_final = true;
    loop {
        if let Some(pos) = re_plus(&remainder[remaining_pos..]) {
            remaining_pos += pos;
            is_final = false;
        } else {
            break;
        };
        if let Some(pos) = parse_id_lodash(&remainder[remaining_pos..]) {
            remaining_pos += pos;
            is_final = true;
        } else {
            break;
        }
    }

    if is_final {
        lex.bump(remaining_pos);
    }

    let mut remaining_pos = 0;
    let remainder = lex.remainder();
    // :
    if let Some(pos) = re_colon(&remainder[remaining_pos..]) {
        remaining_pos += pos;
    } else {
        return Some(String::from(lex.slice()));
    };

    // id
    if let Some(pos) = parse_id(&remainder[remaining_pos..]) {
        remaining_pos += pos;
    } else {
        return Some(String::from(lex.slice()));
    }
    lex.bump(remaining_pos);
    Some(String::from(lex.slice()))
}

fn match_ident_suffix(lex: &mut Lexer<Token>) -> Option<String> {
    // group2: (+ (id|_))*
    let remainder = lex.remainder();
    let mut remaining_pos = 0;
    let mut is_final = true;
    loop {
        if let Some(pos) = re_plus(&remainder[remaining_pos..]) {
            remaining_pos += pos;
            is_final = false;
        } else {
            break;
        };
        if let Some(pos) = parse_id_lodash(&remainder[remaining_pos..]) {
            remaining_pos += pos;
            is_final = true;
        } else {
            break;
        }
    }

    let mut remaining_pos = 0;
    let remainder = lex.remainder();
    // :
    if let Some(pos) = re_colon(&remainder[remaining_pos..]) {
        remaining_pos += pos;
    } else {
        return Some(String::from(lex.slice()));
    };

    // id
    if let Some(pos) = parse_id(&remainder[remaining_pos..]) {
        remaining_pos += pos;
    } else {
        return Some(String::from(lex.slice()));
    }
    lex.bump(remaining_pos);
    Some(String::from(lex.slice()))
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
    #[regex(r"#[^\n]*")]
    COMMENT,
    #[token("true", | lex | lex.slice().parse())]
    #[token("false", | lex | lex.slice().parse())]
    BOOL(bool),
    #[regex(r"-?[0-9_]+", | lex | lex.slice().parse(), priority = 2)]
    INT(i64),
    // Some dirty hacks to work around logos bug
    #[regex(r"[\s\S]", match_ident, priority = 0)]
    #[token("_", match_ident_suffix, priority = 3)]
    IDENT(String),
    #[regex(r"(!?=|[<>]=?|~)", parse_relop)]
    RELOP(Relop),
    #[regex(r"&|\|", parse_logop)]
    LOGOP(Logop),
    #[regex(r"!|\?", parse_pfxop)]
    PFXOP(Pfxop),
    #[regex(r"[\+?]=|=[\+?]=?", parse_envop)]
    ENVOP(Envop),
    #[regex(r"[ \t\r\n]", logos::skip)]
    SKIP,
    #[error]
    Error,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub fn lex(input: &str) -> Result<Vec<(Token, Span)>, Span> {
    let lexer: Lexer<Token> = Token::lexer(input);
    let mut result: Vec<(Token, Span)> = vec![];
    for (token, span) in lexer.spanned() {
        let span = Span {
            start: span.start,
            end: span.end,
        };
        match token {
            Token::Error => return Err(span),
            Token::COMMENT => (),
            _ => result.push((token, span)),
        }
    }
    Ok(result)
}
