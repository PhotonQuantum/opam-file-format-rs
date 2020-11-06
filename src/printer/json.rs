use std::fmt;
use std::fmt::{Display, Formatter};

use indexmap::{self, IndexMap};
use serde_json::{json, Map, Number, Value as JsonValue};

use crate::lexer::{Envop, Logop, Pfxop, Relop};
use crate::parser::{Item, OpamFile, Value};

pub struct JsonPrinter<'a> {
    ast: &'a OpamFile,
}

impl Display for JsonPrinter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let items: JsonValue = JsonValue::Object(Self::serialize_items(&self.ast.items));
        write!(f, "{}", items.to_string())
    }
}

impl JsonPrinter<'_> {
    pub fn new(ast: &OpamFile) -> JsonPrinter {
        JsonPrinter { ast }
    }
    fn relop_literal(op: &Relop) -> &'static str {
        match op {
            Relop::Eq => "eq",
            Relop::Neq => "neq",
            Relop::Geq => "geq",
            Relop::Gt => "gt",
            Relop::Leq => "leq",
            Relop::Lt => "lt",
        }
    }

    fn pfxop_literal(op: &Pfxop) -> &'static str {
        match op {
            Pfxop::Not => "not",
            Pfxop::Defined => "defined",
        }
    }

    fn envop_literal(op: &Envop) -> &'static str {
        match op {
            Envop::Eq => "eq",
            Envop::PlusEq => "plus_eq",
            Envop::EqPlus => "eq_plus",
            Envop::EqPlusEq => "eq_plus_eq",
            Envop::ColonEq => "colon_eq",
            Envop::EqColon => "eq_colon",
        }
    }

    fn logop_literal(op: &Logop) -> &'static str {
        match op {
            Logop::And => "and",
            Logop::Or => "or",
        }
    }

    fn serialize_items(items: &IndexMap<String, Box<Item>>) -> Map<String, JsonValue> {
        items
            .into_iter()
            .map(|(key, item)| Self::serialize_item(key.clone(), item))
            .collect()
    }

    fn serialize_item(key: String, item: &Item) -> (String, JsonValue) {
        match item {
            Item::Section { name, items } => {
                let mut items = Self::serialize_items(items);
                if let Some(name) = name {
                    items.insert("__name__".to_string(), JsonValue::String(name.clone()));
                }
                (key, JsonValue::Object(items))
            }
            Item::Variable(value) => (key, Self::serialize_value(value)),
        }
    }

    fn serialize_value(value: &Value) -> JsonValue {
        match value {
            Value::Bool(b) => JsonValue::Bool(b.clone()),
            Value::Int(i) => JsonValue::Number(Number::from(i.clone())),
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Relop(op, v1, v2) => {
                json!({Self::relop_literal(op): [Self::serialize_value(v1), Self::serialize_value(v2)]})
            }
            Value::PrefixRelop(op, v) => {
                json!({ Self::relop_literal(op): Self::serialize_value(v) })
            }
            Value::Logop(op, v1, v2) => {
                json!({Self::logop_literal(op): [Self::serialize_value(v1), Self::serialize_value(v2)]})
            }
            Value::Pfxop(op, v) => json!({ Self::pfxop_literal(op): Self::serialize_value(v) }),
            Value::Ident(id) => json!({ "__id__": id }),
            Value::List(l) => JsonValue::Array(
                l.into_iter()
                    .map(|item| Self::serialize_value(item))
                    .collect(),
            ),
            Value::Group(l) => JsonValue::Array(
                l.into_iter()
                    .map(|item| Self::serialize_value(item))
                    .collect(),
            ),
            Value::Option(v, l) => {
                json!({"__value__": Self::serialize_value(v), "__options__": l.into_iter().map(|item|Self::serialize_value(item)).collect::<Vec<JsonValue>>()})
            }
            Value::EnvBinding(v1, op, v2) => {
                json!({Self::envop_literal(op): [Self::serialize_value(v1), Self::serialize_value(v2)]})
            }
        }
    }
}
