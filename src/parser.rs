use crate::lexer;
use crate::lexer::Token::*;
use plex::parser;
use indexmap::{indexmap, IndexMap};

#[derive(Debug)]
pub struct OpamFile {
    pub items: IndexMap<String, Box<Item>>,
}

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    String(String),
    Relop(lexer::Relop, Box<Value>, Box<Value>),
    PrefixRelop(lexer::Relop, Box<Value>),
    Logop(lexer::Logop, Box<Value>, Box<Value>),
    Pfxop(lexer::Pfxop, Box<Value>),
    Ident(String),
    List(Vec<Box<Value>>),
    Group(Vec<Box<Value>>),
    Option(Box<Value>, Vec<Box<Value>>),
    EnvBinding(Box<Value>, lexer::Envop, Box<Value>),
}

#[derive(Debug)]
pub enum Item {
    Section {
        name: Option<String>,
        items: IndexMap<String, Box<Item>>,
    },
    Variable(Value),
}

parser! {
    fn parse_(lexer::Token, lexer::Span);

    (a, b){
        lexer::Span{start: a.start, end: b.end}
    }

    main: OpamFile {
        items[itms] => OpamFile { items: itms }
    }

    items: IndexMap<String, Box<Item>> {
        => indexmap![],
        items[mut itms] item[itm] => {
            let (id, value) = itm;
            itms.insert(id, Box::new(value));
            itms
        }
    }

    item: (String, Item) {
        IDENT(id) COLON value[v] => {
            (id, Item::Variable(v))
        },
        IDENT(id) LBRACE items[v] RBRACE => {
            (id, Item::Section{name: None, items: v})
        },
        IDENT(id) STRING(str) LBRACE items[v] RBRACE => {
            (id, Item::Section{name: Some(str), items: v})
        }
    }

    value: Value {
        #[no_reduce(ENVOP, RELOP)]
        atom[a] => {
            a
        },
        LPAR values[v] RPAR => {
            Value::Group(v)
        },
        LBRACKET values[v] RBRACKET => {
            Value::List(v)
        },
        value[v] LBRACE values[vs] RBRACE => {
            Value::Option(Box::new(v), vs)
        },
        #[no_reduce(LBRACE, LOGOP)]
        value[v1] LOGOP(op) value[v2] => {
            Value::Logop(op, Box::new(v1), Box::new(v2))
        },
        atom[a1] RELOP(op) atom[a2]=> {
            Value::Relop(op, Box::new(a1), Box::new(a2))
        },
        atom[a1] ENVOP(op) atom[a2]=> {
            Value::EnvBinding(Box::new(a1), op, Box::new(a2))
        },
        #[no_reduce(LBRACE, LOGOP)]
        PFXOP(op) value[v] => {
            Value::Pfxop(op, Box::new(v))
        },
        RELOP(op) atom[a] => {
            Value::PrefixRelop(op, Box::new(a))
        }
    }

    values: Vec<Box<Value>> {
        => vec![],
        values[mut vs] value[v] => {
            vs.push(Box::new(v));
            vs
        }
    }

    atom: Value {
        IDENT(id) => {
            Value::Ident(id)
        },
        BOOL(b) => {
            Value::Bool(b)
        },
        INT(i) => {
            Value::Int(i)
        },
        STRING(str) => {
            Value::String(str)
        }
    }
}
pub fn parse<I: Iterator<Item = (lexer::Token, lexer::Span)>>(
    i: I,
) -> Result<OpamFile, (Option<(lexer::Token, lexer::Span)>, &'static str)> {
    parse_(i)
}
